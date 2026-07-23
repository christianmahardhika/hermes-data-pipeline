//! ArangoDB Schema Initialization
//!
//! Idempotent schema manager: creates document collections, edge collections,
//! graph, and ArangoSearch views for the Hermes intelligence platform.
//!
//! Requirements:
//! - ArangoDB >= 3.10 (for APPROX_NEAR vector search)
//! - 7 document collections, 6 edge collections
//! - 1 named graph (intelligence_graph)
//! - 2 ArangoSearch views (articles_vector_view, fulltext_view)

use anyhow::{Context, Result};
use serde_json::json;
use tracing::info;

use super::ArangoClient;

/// Minimum required ArangoDB version for vector search support
const MIN_ARANGO_VERSION: &str = "3.10";

/// Document collections for the intelligence platform
const DOCUMENT_COLLECTIONS: &[&str] = &[
    "articles",            // News articles (cleaned + labeled)
    "social_posts",        // Social media intelligence
    "economic_indicators", // Time-series numerical data
    "actors",             // Prof Jiang extracted actors
    "events",            // Prof Jiang extracted events
    "feed_health",       // Feed monitoring
    "analysis_results",  // IDX Analyst debate results
];

/// Edge collections for relationships
const EDGE_COLLECTIONS: &[&str] = &[
    "mentions",         // article → actor
    "triggers",         // event → article
    "impacts",          // event → ticker
    "correlates_with",  // article ↔ article
    "actor_relations",  // actor → actor
    "signal_source",    // economic_indicator → ticker
];

/// Schema manager for idempotent ArangoDB schema initialization
pub struct SchemaManager<'a> {
    client: &'a ArangoClient,
}

impl<'a> SchemaManager<'a> {
    /// Create a new SchemaManager with a reference to an ArangoClient
    pub fn new(client: &'a ArangoClient) -> Self {
        Self { client }
    }

    /// Initialize the complete schema (idempotent).
    ///
    /// Order: check version → document collections → edge collections → graph → views
    pub async fn ensure_all(&self) -> Result<()> {
        info!("🗄️ Starting schema initialization...");

        self.check_version().await?;
        self.ensure_document_collections().await?;
        self.ensure_edge_collections().await?;
        self.ensure_graph().await?;
        self.ensure_views().await?;

        info!("✅ Schema initialization complete");
        Ok(())
    }

    /// Verify ArangoDB version >= 3.10 (required for APPROX_NEAR vector search)
    async fn check_version(&self) -> Result<()> {
        let url = self.client.server_url("/_api/version");

        let resp = self.client.http_get(&url).await
            .context("checking ArangoDB version")?;

        if !resp.status().is_success() {
            anyhow::bail!("Failed to get ArangoDB version: status={}", resp.status());
        }

        let body: serde_json::Value = resp.json().await
            .context("parsing ArangoDB version response")?;

        let version_str = body.get("version")
            .and_then(|v| v.as_str())
            .context("missing 'version' in ArangoDB response")?;

        // Compare major.minor
        let version_parts: Vec<&str> = version_str.split('.').collect();
        let min_parts: Vec<&str> = MIN_ARANGO_VERSION.split('.').collect();

        if version_parts.len() < 2 || min_parts.len() < 2 {
            anyhow::bail!("Cannot parse ArangoDB version: {}", version_str);
        }

        let major: u32 = version_parts[0].parse()
            .context("parsing ArangoDB major version")?;
        let minor: u32 = version_parts[1].parse()
            .context("parsing ArangoDB minor version")?;

        let min_major: u32 = min_parts[0].parse()
            .context("parsing minimum major version")?;
        let min_minor: u32 = min_parts[1].parse()
            .context("parsing minimum minor version")?;

        if major < min_major || (major == min_major && minor < min_minor) {
            anyhow::bail!(
                "ArangoDB version {} is below minimum required {}. \
                 APPROX_NEAR vector search requires >= {}",
                version_str, MIN_ARANGO_VERSION, MIN_ARANGO_VERSION
            );
        }

        info!("🗄️ ArangoDB version {} (>= {} required) ✓", version_str, MIN_ARANGO_VERSION);
        Ok(())
    }

    /// Check if a collection exists
    async fn collection_exists(&self, name: &str) -> Result<bool> {
        let url = self.client.db_api_url(&format!("/_api/collection/{}", name));

        let resp = self.client.http_get(&url).await
            .context(format!("checking collection existence: {}", name))?;

        Ok(resp.status().is_success())
    }

    /// Create a collection if it doesn't exist (idempotent)
    ///
    /// - type 2 = document collection
    /// - type 3 = edge collection
    async fn ensure_collection(&self, name: &str, is_edge: bool) -> Result<()> {
        if self.collection_exists(name).await? {
            info!("  ├─ Collection '{}' already exists, skipping", name);
            return Ok(());
        }

        let collection_type = if is_edge { 3 } else { 2 };
        let url = self.client.db_api_url("/_api/collection");

        let payload = json!({
            "name": name,
            "type": collection_type,
        });

        let resp = self.client.http_post(&url, &payload).await
            .context(format!("creating collection: {}", name))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create collection '{}': status={}, body={}", name, status, body);
        }

        let kind = if is_edge { "edge" } else { "document" };
        info!("  ├─ Created {} collection: '{}'", kind, name);
        Ok(())
    }

    /// Ensure all 7 document collections exist
    async fn ensure_document_collections(&self) -> Result<()> {
        info!("📦 Ensuring document collections ({})...", DOCUMENT_COLLECTIONS.len());
        for name in DOCUMENT_COLLECTIONS {
            self.ensure_collection(name, false).await?;
        }
        Ok(())
    }

    /// Ensure all 6 edge collections exist
    async fn ensure_edge_collections(&self) -> Result<()> {
        info!("🔗 Ensuring edge collections ({})...", EDGE_COLLECTIONS.len());
        for name in EDGE_COLLECTIONS {
            self.ensure_collection(name, true).await?;
        }
        Ok(())
    }

    /// Ensure the intelligence_graph exists (idempotent)
    async fn ensure_graph(&self) -> Result<()> {
        info!("🕸️ Ensuring intelligence_graph...");

        let graph_name = "intelligence_graph";

        // Check if graph exists
        let url = self.client.db_api_url(&format!("/_api/gharial/{}", graph_name));
        let resp = self.client.http_get(&url).await
            .context("checking graph existence")?;

        if resp.status().is_success() {
            info!("  ├─ Graph '{}' already exists, skipping", graph_name);
            return Ok(());
        }

        // Define edge definitions connecting vertex collections
        let edge_definitions = json!([
            {
                "collection": "mentions",
                "from": ["articles"],
                "to": ["actors"]
            },
            {
                "collection": "triggers",
                "from": ["events"],
                "to": ["articles"]
            },
            {
                "collection": "impacts",
                "from": ["events"],
                "to": ["economic_indicators"]
            },
            {
                "collection": "correlates_with",
                "from": ["articles"],
                "to": ["articles"]
            },
            {
                "collection": "actor_relations",
                "from": ["actors"],
                "to": ["actors"]
            },
            {
                "collection": "signal_source",
                "from": ["economic_indicators"],
                "to": ["economic_indicators"]
            }
        ]);

        let create_url = self.client.db_api_url("/_api/gharial");
        let payload = json!({
            "name": graph_name,
            "edgeDefinitions": edge_definitions,
        });

        let resp = self.client.http_post(&create_url, &payload).await
            .context("creating intelligence_graph")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create graph '{}': status={}, body={}", graph_name, status, body);
        }

        info!("  ├─ Created graph: '{}'", graph_name);
        Ok(())
    }

    /// Check if a view exists
    async fn view_exists(&self, name: &str) -> Result<bool> {
        let url = self.client.db_api_url(&format!("/_api/view/{}", name));

        let resp = self.client.http_get(&url).await
            .context(format!("checking view existence: {}", name))?;

        Ok(resp.status().is_success())
    }

    /// Ensure ArangoSearch views exist (idempotent)
    async fn ensure_views(&self) -> Result<()> {
        info!("🔍 Ensuring ArangoSearch views...");

        self.ensure_articles_vector_view().await?;
        self.ensure_fulltext_view().await?;

        Ok(())
    }

    /// Create articles_vector_view — ArangoSearch view with embedding field (768 dimensions)
    async fn ensure_articles_vector_view(&self) -> Result<()> {
        let view_name = "articles_vector_view";

        if self.view_exists(view_name).await? {
            info!("  ├─ View '{}' already exists, skipping", view_name);
            return Ok(());
        }

        let url = self.client.db_api_url("/_api/view");

        // ArangoSearch view with vector index configuration
        // The embedding field uses 768 dimensions (multilingual-e5-base)
        let payload = json!({
            "name": view_name,
            "type": "arangosearch",
            "links": {
                "articles": {
                    "fields": {
                        "embedding": {
                            "analyzers": ["identity"],
                            "features": []
                        },
                        "title": {
                            "analyzers": ["text_en", "identity"],
                            "features": ["frequency", "position"]
                        },
                        "content": {
                            "analyzers": ["text_en", "identity"],
                            "features": ["frequency", "position"]
                        },
                        "published_at": {
                            "analyzers": ["identity"],
                            "features": []
                        },
                        "source": {
                            "analyzers": ["identity"],
                            "features": []
                        }
                    },
                    "includeAllFields": false,
                    "storeValues": "id"
                }
            },
            "primarySort": [
                { "field": "published_at", "direction": "desc" }
            ]
        });

        let resp = self.client.http_post(&url, &payload).await
            .context("creating articles_vector_view")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create view '{}': status={}, body={}", view_name, status, body);
        }

        info!("  ├─ Created view: '{}' (768-dim embedding, text_en analyzers)", view_name);
        Ok(())
    }

    /// Create fulltext_view — Full-text search on articles + social_posts
    async fn ensure_fulltext_view(&self) -> Result<()> {
        let view_name = "fulltext_view";

        if self.view_exists(view_name).await? {
            info!("  ├─ View '{}' already exists, skipping", view_name);
            return Ok(());
        }

        let url = self.client.db_api_url("/_api/view");

        let payload = json!({
            "name": view_name,
            "type": "arangosearch",
            "links": {
                "articles": {
                    "fields": {
                        "title": {
                            "analyzers": ["text_en", "identity"],
                            "features": ["frequency", "position", "norm"]
                        },
                        "content": {
                            "analyzers": ["text_en", "identity"],
                            "features": ["frequency", "position", "norm"]
                        }
                    },
                    "includeAllFields": false,
                    "storeValues": "id"
                },
                "social_posts": {
                    "fields": {
                        "title": {
                            "analyzers": ["text_en", "identity"],
                            "features": ["frequency", "position", "norm"]
                        },
                        "content": {
                            "analyzers": ["text_en", "identity"],
                            "features": ["frequency", "position", "norm"]
                        }
                    },
                    "includeAllFields": false,
                    "storeValues": "id"
                }
            }
        });

        let resp = self.client.http_post(&url, &payload).await
            .context("creating fulltext_view")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create view '{}': status={}, body={}", view_name, status, body);
        }

        info!("  ├─ Created view: '{}' (articles + social_posts full-text)", view_name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running ArangoDB at localhost:8529
    async fn test_schema_init_fresh_database() {
        // This test requires a running ArangoDB instance
        // Run with: cargo test test_schema_init_fresh_database -- --ignored --nocapture
        let client = ArangoClient::new().expect("failed to create ArangoClient");
        let schema = SchemaManager::new(&client);

        // Should be idempotent — safe to run multiple times
        schema.ensure_all().await.expect("schema initialization failed");

        // Run again to verify idempotency
        schema.ensure_all().await.expect("schema re-initialization should be idempotent");
    }

    #[test]
    fn test_document_collections_count() {
        assert_eq!(DOCUMENT_COLLECTIONS.len(), 7, "Expected 7 document collections");
    }

    #[test]
    fn test_edge_collections_count() {
        assert_eq!(EDGE_COLLECTIONS.len(), 6, "Expected 6 edge collections");
    }

    #[test]
    fn test_min_version_format() {
        let parts: Vec<&str> = MIN_ARANGO_VERSION.split('.').collect();
        assert_eq!(parts.len(), 2, "MIN_ARANGO_VERSION should be major.minor format");
        assert!(parts[0].parse::<u32>().is_ok());
        assert!(parts[1].parse::<u32>().is_ok());
    }
}
