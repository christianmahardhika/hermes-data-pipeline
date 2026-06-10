//! TEI Embedder and Qdrant Storage
//! 
//! Phase 4: Generate embeddings via TEI, store in Qdrant with Prof Jiang payload

use anyhow::Result;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, VectorParamsBuilder,
    PointStruct, SearchPointsBuilder, UpsertPointsBuilder,
};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, warn};
use uuid::Uuid;

use crate::storage::{CleanedArticle, LabeledArticle};

/// TEI embedding response
#[derive(Debug, Deserialize)]
struct TeiResponse(Vec<Vec<f32>>);

/// TEI Embedder + Qdrant storage
pub struct TeiEmbedder {
    http_client: Client,
    qdrant_client: Qdrant,
    tei_url: String,
    collection_name: String,
    similarity_threshold: f32,
}

impl TeiEmbedder {
    /// Create new embedder
    pub async fn new(
        tei_url: &str,
        qdrant_url: &str,
        collection_name: &str,
        similarity_threshold: f32,
    ) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;

        let qdrant_client = Qdrant::from_url(qdrant_url).build()?;

        let embedder = Self {
            http_client,
            qdrant_client,
            tei_url: tei_url.to_string(),
            collection_name: collection_name.to_string(),
            similarity_threshold,
        };

        // Ensure collection exists
        embedder.ensure_collection().await?;

        Ok(embedder)
    }

    /// Ensure Qdrant collection exists
    async fn ensure_collection(&self) -> Result<()> {
        let exists = self.qdrant_client
            .collection_exists(&self.collection_name)
            .await?;
        
        if !exists {
            info!("Creating collection: {}", self.collection_name);
            
            self.qdrant_client
                .create_collection(
                    CreateCollectionBuilder::new(&self.collection_name)
                        .vectors_config(VectorParamsBuilder::new(768, Distance::Cosine))
                )
                .await?;
        }

        Ok(())
    }

    /// Generate embedding via TEI
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let response = self.http_client
            .post(format!("{}/embed", self.tei_url))
            .json(&serde_json::json!({
                "inputs": [text]
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("TEI error: {}", response.status()));
        }

        let tei_response: TeiResponse = response.json().await?;
        tei_response.0.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("Empty TEI response"))
    }

    /// Check for near-duplicates using vector similarity
    async fn is_near_duplicate(&self, embedding: &[f32]) -> Result<bool> {
        let results = self.qdrant_client
            .search_points(
                SearchPointsBuilder::new(&self.collection_name, embedding.to_vec(), 1)
                    .score_threshold(self.similarity_threshold)
            )
            .await?;

        Ok(!results.result.is_empty())
    }

    /// Process labeled articles and store in Qdrant
    pub async fn process_batch(
        &self,
        articles: Vec<(CleanedArticle, LabeledArticle)>,
    ) -> Result<EmbedStats> {
        let mut stats = EmbedStats::default();

        for (cleaned, labeled) in articles {
            // Combine title + content for embedding
            let text = format!("{} {}", cleaned.title, cleaned.content);
            
            match self.embed(&text).await {
                Ok(embedding) => {
                    // Check near-duplicate
                    if self.is_near_duplicate(&embedding).await? {
                        stats.near_duplicates += 1;
                        warn!("⚠️ Near-duplicate: {}", cleaned.title);
                        continue;
                    }

                    // Build Qdrant payload with Prof Jiang data
                    let payload = self.build_payload(&cleaned, &labeled);

                    // Insert into Qdrant
                    let point = PointStruct::new(
                        Uuid::new_v4().to_string(),
                        embedding,
                        payload,
                    );

                    self.qdrant_client
                        .upsert_points(UpsertPointsBuilder::new(&self.collection_name, vec![point]))
                        .await?;

                    stats.ingested += 1;
                    if let Some(labeled_id) = labeled.id {
                        stats.ingested_ids.push(labeled_id);
                    }
                    info!("✅ Ingested: {}", cleaned.title);
                }
                Err(e) => {
                    stats.errors += 1;
                    warn!("⚠️ Embed failed for '{}': {}", cleaned.title, e);
                }
            }
        }

        Ok(stats)
    }

    /// Build Qdrant payload from cleaned + labeled article
    fn build_payload(
        &self,
        cleaned: &CleanedArticle,
        labeled: &LabeledArticle,
    ) -> HashMap<String, qdrant_client::qdrant::Value> {
        use qdrant_client::qdrant::Value;
        
        let mut payload = HashMap::new();

        let str_val = |s: String| Value {
            kind: Some(qdrant_client::qdrant::value::Kind::StringValue(s)),
        };
        let f64_val = |f: f64| Value {
            kind: Some(qdrant_client::qdrant::value::Kind::DoubleValue(f)),
        };

        // Basic fields
        payload.insert("title".to_string(), str_val(cleaned.title.clone()));
        payload.insert("content".to_string(), str_val(cleaned.content.clone()));
        payload.insert("url".to_string(), str_val(cleaned.url.clone()));
        payload.insert("source".to_string(), str_val(cleaned.source.clone()));
        payload.insert("content_hash".to_string(), str_val(cleaned.content_hash.clone()));
        
        if let Some(dt) = cleaned.published_at {
            payload.insert("published_at".to_string(), str_val(dt.to_rfc3339()));
        }

        // Prof Jiang labels
        payload.insert("sentiment".to_string(), str_val(labeled.sentiment.clone()));
        payload.insert("sentiment_score".to_string(), f64_val(labeled.sentiment_score as f64));
        payload.insert("news_type".to_string(), str_val(labeled.news_type.clone()));
        
        if let Some(ref subtype) = labeled.news_subtype {
            payload.insert("news_subtype".to_string(), str_val(subtype.clone()));
        }

        // JSON fields (stored as strings)
        payload.insert("events".to_string(), str_val(labeled.events.to_string()));
        payload.insert("actors".to_string(), str_val(labeled.actors.to_string()));
        payload.insert("relations".to_string(), str_val(labeled.relations.to_string()));
        payload.insert("context".to_string(), str_val(labeled.context.to_string()));
        payload.insert("pattern_match".to_string(), str_val(labeled.pattern_match.to_string()));
        payload.insert("investment_signal".to_string(), str_val(labeled.investment_signal.to_string()));

        // Meta
        payload.insert("labeled_at".to_string(), str_val(labeled.labeled_at.to_rfc3339()));
        payload.insert("labeled_by".to_string(), str_val(labeled.labeled_by.clone()));

        payload
    }
}

/// Embedding statistics
#[derive(Debug, Default)]
pub struct EmbedStats {
    pub ingested: usize,
    pub ingested_ids: Vec<i64>,
    pub near_duplicates: usize,
    pub errors: usize,
}

impl std::fmt::Display for EmbedStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Embedded: {} ingested, {} near-dups, {} errors",
            self.ingested, self.near_duplicates, self.errors
        )
    }
}
