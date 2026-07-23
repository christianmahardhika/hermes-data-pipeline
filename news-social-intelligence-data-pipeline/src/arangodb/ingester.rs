//! ArangoDB Article Ingester
//!
//! Replaces the Qdrant ingestion path (opt-in fallback via STORAGE_BACKEND env var).
//! Inserts articles with embeddings, extracts actors/events into the graph,
//! performs near-duplicate detection via ArangoSearch APPROX_NEAR (0.92 threshold),
//! and deduplicates by content_hash as _key.

use anyhow::{Context, Result};
use chrono::Utc;
use tracing::{info, warn};

use super::ArangoClient;
use crate::storage::{CleanedArticle, LabeledArticle};

/// Default similarity threshold for near-duplicate detection
const DEFAULT_SIMILARITY_THRESHOLD: f32 = 0.92;

/// ArangoDB article ingester with graph extraction
pub struct ArangoIngester<'a> {
    client: &'a ArangoClient,
    similarity_threshold: f32,
}

impl<'a> ArangoIngester<'a> {
    /// Create a new ArangoIngester with custom similarity threshold
    pub fn new(client: &'a ArangoClient, similarity_threshold: f32) -> Self {
        Self {
            client,
            similarity_threshold,
        }
    }

    /// Create a new ArangoIngester with default threshold (0.92)
    pub fn with_defaults(client: &'a ArangoClient) -> Self {
        Self::new(client, DEFAULT_SIMILARITY_THRESHOLD)
    }

    /// Ingest a labeled article with its embedding into ArangoDB.
    ///
    /// Returns the article _key if inserted, None if duplicate.
    ///
    /// Flow:
    /// 1. Dedup by content_hash (_key collision check)
    /// 2. Near-dup via ArangoSearch APPROX_NEAR (0.92 threshold)
    /// 3. Insert article document with embedding
    /// 4. Extract actors → insert into actors collection
    /// 5. Create graph edges (mentions, triggers)
    pub async fn ingest_article(
        &self,
        cleaned: &CleanedArticle,
        labeled: &LabeledArticle,
        embedding: &[f32],
    ) -> Result<Option<String>> {
        let article_key = &cleaned.content_hash;

        // 1. Dedup: check if content_hash already exists as _key
        if self.document_exists("articles", article_key).await? {
            info!("⏭️ Skipping duplicate article: _key={}", article_key);
            return Ok(None);
        }

        // 2. Near-dup: vector similarity check
        if self.is_near_duplicate(embedding).await? {
            info!("⏭️ Skipping near-duplicate article: {}", cleaned.title);
            return Ok(None);
        }

        // 3. Insert article document with embedding
        let article_doc = serde_json::json!({
            "_key": article_key,
            "title": cleaned.title,
            "content": cleaned.content,
            "url": cleaned.url,
            "source": cleaned.source,
            "published_at": cleaned.published_at.map(|dt| dt.to_rfc3339()),
            "content_hash": cleaned.content_hash,
            "cleaned_at": cleaned.cleaned_at.to_rfc3339(),
            "embedding": embedding,
            "sentiment": labeled.sentiment,
            "sentiment_score": labeled.sentiment_score,
            "news_type": labeled.news_type,
            "news_subtype": labeled.news_subtype,
            "events": labeled.events,
            "actors": labeled.actors,
            "relations": labeled.relations,
            "context": labeled.context,
            "pattern_match": labeled.pattern_match,
            "investment_signal": labeled.investment_signal,
            "labeled_at": labeled.labeled_at.to_rfc3339(),
            "labeled_by": labeled.labeled_by,
            "ingested_at": Utc::now().to_rfc3339(),
        });

        self.client
            .insert_document("articles", &article_doc)
            .await
            .context("inserting article into ArangoDB")?;

        // 4. Extract actors and create graph edges
        let actor_count = self
            .extract_and_link_actors(article_key, &labeled.actors)
            .await
            .unwrap_or_else(|e| {
                warn!("⚠️ Failed to extract actors for {}: {}", article_key, e);
                0
            });

        // 5. Extract events and create trigger edges
        let event_count = self
            .extract_and_link_events(article_key, &labeled.events)
            .await
            .unwrap_or_else(|e| {
                warn!("⚠️ Failed to extract events for {}: {}", article_key, e);
                0
            });

        info!(
            "✅ Ingested article: _key={}, actors={}, events={}",
            article_key, actor_count, event_count
        );

        Ok(Some(article_key.clone()))
    }

    /// Check if a document exists by _key in a collection
    async fn document_exists(&self, collection: &str, key: &str) -> Result<bool> {
        let query = "FOR doc IN @@collection FILTER doc._key == @key LIMIT 1 RETURN doc._key";
        let results: Vec<String> = self
            .client
            .query_aql(
                query,
                serde_json::json!({
                    "@collection": collection,
                    "key": key,
                }),
            )
            .await?;
        Ok(!results.is_empty())
    }

    /// Check if an article is a near-duplicate via vector similarity.
    ///
    /// Uses ArangoSearch APPROX_NEAR on the articles_vector_view.
    async fn is_near_duplicate(&self, embedding: &[f32]) -> Result<bool> {
        let query = r#"
            FOR doc IN articles_vector_view
              SEARCH ANALYZER(APPROX_NEAR(doc.embedding, @vector, @threshold), "identity")
              LIMIT 1
              RETURN doc._key
        "#;

        let results: Vec<String> = self
            .client
            .query_aql(
                query,
                serde_json::json!({
                    "vector": embedding,
                    "threshold": self.similarity_threshold,
                }),
            )
            .await?;

        Ok(!results.is_empty())
    }

    /// Extract actors from labeled JSON and insert as graph nodes + edges.
    ///
    /// Creates:
    /// - Actor documents in the "actors" collection
    /// - "mentions" edges: articles/{article_key} → actors/{actor_key}
    async fn extract_and_link_actors(
        &self,
        article_key: &str,
        actors_json: &serde_json::Value,
    ) -> Result<usize> {
        let mut count = 0;

        let actors = match actors_json.as_array() {
            Some(arr) => arr,
            None => return Ok(0),
        };

        for actor in actors {
            let actor_name = actor
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("unknown");

            let actor_type = actor
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown");

            let actor_role = actor
                .get("role")
                .and_then(|r| r.as_str())
                .unwrap_or("");

            // Sanitize actor name for use as _key (alphanumeric + underscore only)
            let sanitized_name = actor_name
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
                .collect::<String>()
                .to_lowercase();

            let key_suffix_len = 8.min(article_key.len());
            let actor_key = format!("{}_{}", sanitized_name, &article_key[..key_suffix_len]);

            let actor_doc = serde_json::json!({
                "_key": actor_key,
                "name": actor_name,
                "type": actor_type,
                "role": actor_role,
                "source_article": article_key,
                "created_at": Utc::now().to_rfc3339(),
            });

            // Insert actor (ignore duplicate key error)
            let _ = self.client.insert_document("actors", &actor_doc).await;

            // Create mentions edge: articles/{article_key} → actors/{actor_key}
            let edge_data = serde_json::json!({
                "context": actor_role,
                "actor_type": actor_type,
                "created_at": Utc::now().to_rfc3339(),
            });

            let _ = self
                .client
                .insert_edge(
                    "mentions",
                    &format!("articles/{}", article_key),
                    &format!("actors/{}", actor_key),
                    &edge_data,
                )
                .await;

            count += 1;
        }

        Ok(count)
    }

    /// Extract events from labeled JSON and insert as graph nodes + edges.
    ///
    /// Creates:
    /// - Event documents in the "events" collection
    /// - "triggers" edges: events/{event_key} → articles/{article_key}
    async fn extract_and_link_events(
        &self,
        article_key: &str,
        events_json: &serde_json::Value,
    ) -> Result<usize> {
        let mut count = 0;

        let events = match events_json.as_array() {
            Some(arr) => arr,
            None => return Ok(0),
        };

        for event in events {
            let event_name = event
                .get("name")
                .or_else(|| event.get("event"))
                .and_then(|n| n.as_str())
                .unwrap_or("unknown_event");

            let event_type = event
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown");

            let significance = event
                .get("significance")
                .and_then(|s| s.as_str())
                .unwrap_or("");

            // Sanitize event name for _key
            let sanitized_name = event_name
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
                .collect::<String>()
                .to_lowercase();

            let key_suffix_len = 8.min(article_key.len());
            let event_key = format!("{}_{}", sanitized_name, &article_key[..key_suffix_len]);

            let event_doc = serde_json::json!({
                "_key": event_key,
                "name": event_name,
                "type": event_type,
                "significance": significance,
                "tense": event.get("tense").and_then(|t| t.as_str()).unwrap_or("past"),
                "source_article": article_key,
                "created_at": Utc::now().to_rfc3339(),
            });

            // Insert event (ignore duplicate key error)
            let _ = self.client.insert_document("events", &event_doc).await;

            // Create triggers edge: events/{event_key} → articles/{article_key}
            let edge_data = serde_json::json!({
                "significance": significance,
                "event_type": event_type,
                "created_at": Utc::now().to_rfc3339(),
            });

            let _ = self
                .client
                .insert_edge(
                    "triggers",
                    &format!("events/{}", event_key),
                    &format!("articles/{}", article_key),
                    &edge_data,
                )
                .await;

            count += 1;
        }

        Ok(count)
    }
}

/// Storage backend selection based on STORAGE_BACKEND env var
#[derive(Debug, Clone, PartialEq)]
pub enum StorageBackend {
    /// ArangoDB graph + vector store (default)
    ArangoDB,
    /// Qdrant vector store (legacy fallback)
    Qdrant,
}

impl std::fmt::Display for StorageBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArangoDB => write!(f, "arangodb"),
            Self::Qdrant => write!(f, "qdrant"),
        }
    }
}

/// Determine which storage backend to use.
///
/// Reads STORAGE_BACKEND env var:
/// - "arangodb" (default) → ArangoDB graph + vector
/// - "qdrant" → Legacy Qdrant path (opt-in fallback)
pub fn get_storage_backend() -> StorageBackend {
    match std::env::var("STORAGE_BACKEND")
        .unwrap_or_else(|_| "arangodb".to_string())
        .to_lowercase()
        .as_str()
    {
        "qdrant" => StorageBackend::Qdrant,
        _ => StorageBackend::ArangoDB,
    }
}

/// Ingestion statistics
#[derive(Debug, Default)]
pub struct IngestStats {
    pub articles_inserted: usize,
    pub duplicates_skipped: usize,
    pub near_duplicates_skipped: usize,
    pub actors_created: usize,
    pub events_created: usize,
    pub edges_created: usize,
    pub errors: usize,
}

impl std::fmt::Display for IngestStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ingested: {} articles, {} actors, {} events, {} edges | Skipped: {} dups, {} near-dups | Errors: {}",
            self.articles_inserted,
            self.actors_created,
            self.events_created,
            self.edges_created,
            self.duplicates_skipped,
            self.near_duplicates_skipped,
            self.errors,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_backend_default() {
        // Clear the env var to test default behavior
        std::env::remove_var("STORAGE_BACKEND");
        assert_eq!(get_storage_backend(), StorageBackend::ArangoDB);
    }

    #[test]
    fn test_storage_backend_qdrant() {
        std::env::set_var("STORAGE_BACKEND", "qdrant");
        assert_eq!(get_storage_backend(), StorageBackend::Qdrant);
        // Clean up
        std::env::remove_var("STORAGE_BACKEND");
    }

    #[test]
    fn test_storage_backend_arangodb_explicit() {
        std::env::set_var("STORAGE_BACKEND", "arangodb");
        assert_eq!(get_storage_backend(), StorageBackend::ArangoDB);
        // Clean up
        std::env::remove_var("STORAGE_BACKEND");
    }

    #[test]
    fn test_storage_backend_case_insensitive_qdrant_upper() {
        std::env::set_var("STORAGE_BACKEND", "QDRANT");
        assert_eq!(get_storage_backend(), StorageBackend::Qdrant);
        std::env::remove_var("STORAGE_BACKEND");
    }

    #[test]
    fn test_storage_backend_case_insensitive_mixed() {
        std::env::set_var("STORAGE_BACKEND", "Qdrant");
        assert_eq!(get_storage_backend(), StorageBackend::Qdrant);
        std::env::remove_var("STORAGE_BACKEND");
    }

    #[test]
    fn test_storage_backend_unknown_defaults_to_arangodb() {
        std::env::set_var("STORAGE_BACKEND", "unknown_backend");
        assert_eq!(get_storage_backend(), StorageBackend::ArangoDB);
        // Clean up
        std::env::remove_var("STORAGE_BACKEND");
    }

    #[test]
    fn test_storage_backend_display() {
        assert_eq!(StorageBackend::ArangoDB.to_string(), "arangodb");
        assert_eq!(StorageBackend::Qdrant.to_string(), "qdrant");
    }

    #[test]
    fn test_ingest_stats_default() {
        let stats = IngestStats::default();
        assert_eq!(stats.articles_inserted, 0);
        assert_eq!(stats.duplicates_skipped, 0);
        assert_eq!(stats.near_duplicates_skipped, 0);
        assert_eq!(stats.actors_created, 0);
        assert_eq!(stats.events_created, 0);
        assert_eq!(stats.edges_created, 0);
        assert_eq!(stats.errors, 0);
    }

    #[test]
    fn test_ingest_stats_display() {
        let stats = IngestStats {
            articles_inserted: 10,
            duplicates_skipped: 3,
            near_duplicates_skipped: 2,
            actors_created: 15,
            events_created: 8,
            edges_created: 23,
            errors: 1,
        };
        let display = stats.to_string();
        assert!(display.contains("10 articles"));
        assert!(display.contains("15 actors"));
        assert!(display.contains("8 events"));
        assert!(display.contains("23 edges"));
        assert!(display.contains("3 dups"));
        assert!(display.contains("2 near-dups"));
        assert!(display.contains("Errors: 1"));
    }

    #[test]
    fn test_default_similarity_threshold() {
        assert!((DEFAULT_SIMILARITY_THRESHOLD - 0.92).abs() < f32::EPSILON);
    }

    #[tokio::test]
    #[ignore] // Requires running ArangoDB at localhost:8529
    async fn test_integration_ingest_and_verify_graph() {
        use chrono::Utc;

        let client = ArangoClient::new().expect("failed to create ArangoClient");
        let ingester = ArangoIngester::with_defaults(&client);

        // Create test article
        let cleaned = CleanedArticle {
            id: Some(1),
            raw_id: 1,
            title: "Test Integration Article".to_string(),
            content: "This is a test article for integration testing.".to_string(),
            published_at: Some(Utc::now()),
            source: "test_source".to_string(),
            url: "https://example.com/test-article".to_string(),
            content_hash: format!("test_integration_{}", Utc::now().timestamp()),
            cleaned_at: Utc::now(),
        };

        let labeled = LabeledArticle {
            id: Some(1),
            cleaned_id: 1,
            sentiment: "neutral".to_string(),
            sentiment_score: 0.5,
            news_type: "business".to_string(),
            news_subtype: Some("technology".to_string()),
            events: serde_json::json!([
                {
                    "name": "product_launch",
                    "type": "corporate",
                    "significance": "medium",
                    "tense": "past"
                }
            ]),
            actors: serde_json::json!([
                {
                    "name": "Test Corporation",
                    "type": "company",
                    "role": "protagonist"
                },
                {
                    "name": "John Doe",
                    "type": "individual",
                    "role": "CEO"
                }
            ]),
            relations: serde_json::json!([]),
            context: serde_json::json!({"sector": "technology"}),
            pattern_match: serde_json::json!({}),
            investment_signal: serde_json::json!({"signal": "neutral", "confidence": 0.5}),
            labeled_at: Utc::now(),
            labeled_by: "test".to_string(),
        };

        // Generate a fake 768-dim embedding
        let embedding: Vec<f32> = (0..768).map(|i| (i as f32) * 0.001).collect();

        // Ingest the article
        let result = ingester
            .ingest_article(&cleaned, &labeled, &embedding)
            .await
            .expect("ingest_article failed");

        assert!(result.is_some(), "Article should be inserted (not duplicate)");
        let article_key = result.unwrap();
        assert_eq!(article_key, cleaned.content_hash);

        // Verify: article exists in collection
        let verify_query = "FOR doc IN articles FILTER doc._key == @key RETURN doc._key";
        let found: Vec<String> = client
            .query_aql(verify_query, serde_json::json!({"key": article_key}))
            .await
            .expect("verify query failed");
        assert_eq!(found.len(), 1, "Article should exist in collection");

        // Verify: graph edges exist (mentions)
        let edge_query = r#"
            FOR edge IN mentions
              FILTER edge._from == @from
              RETURN edge._to
        "#;
        let edges: Vec<String> = client
            .query_aql(
                edge_query,
                serde_json::json!({"from": format!("articles/{}", article_key)}),
            )
            .await
            .expect("edge query failed");
        assert!(edges.len() >= 2, "Should have at least 2 mention edges (2 actors)");

        // Verify: triggers edges exist
        let trigger_query = r#"
            FOR edge IN triggers
              FILTER edge._to == @to
              RETURN edge._from
        "#;
        let triggers: Vec<String> = client
            .query_aql(
                trigger_query,
                serde_json::json!({"to": format!("articles/{}", article_key)}),
            )
            .await
            .expect("trigger query failed");
        assert!(triggers.len() >= 1, "Should have at least 1 trigger edge (1 event)");

        // Verify: duplicate insertion returns None
        let dup_result = ingester
            .ingest_article(&cleaned, &labeled, &embedding)
            .await
            .expect("duplicate ingest should not error");
        assert!(dup_result.is_none(), "Duplicate article should return None");

        // Cleanup: remove test data
        let _ = client
            .query_aql::<serde_json::Value>(
                "FOR doc IN articles FILTER doc._key == @key REMOVE doc IN articles",
                serde_json::json!({"key": article_key}),
            )
            .await;
        let _ = client
            .query_aql::<serde_json::Value>(
                "FOR edge IN mentions FILTER edge._from == @from REMOVE edge IN mentions",
                serde_json::json!({"from": format!("articles/{}", article_key)}),
            )
            .await;
        let _ = client
            .query_aql::<serde_json::Value>(
                "FOR edge IN triggers FILTER edge._to == @to REMOVE edge IN triggers",
                serde_json::json!({"to": format!("articles/{}", article_key)}),
            )
            .await;

        info!("✅ Integration test passed: ingest + verify graph edges");
    }
}
