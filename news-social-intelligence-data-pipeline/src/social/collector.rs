//! Unified social intelligence collector.
//!
//! Fetches from all sources, generates TEI embeddings, and stores in Qdrant.

use anyhow::Result;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, UpsertPointsBuilder, VectorParamsBuilder,
};
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, warn};
use uuid::Uuid;

use super::dedup;
use super::{hackernews, reddit, youtube};
use super::{Depth, SocialArticle, SocialStats};

const COLLECTION_NAME: &str = "social_intelligence";
const VECTOR_DIM: u64 = 768;
const TEI_TIMEOUT_SECS: u64 = 60;

/// TeiResponse from the TEI embedding service
#[derive(Debug, serde::Deserialize)]
struct TeiResponse(Vec<Vec<f32>>);

/// Unified social intelligence collector with Qdrant storage.
pub struct SocialCollector {
    http_client: Client,
    qdrant_client: Qdrant,
    tei_url: String,
    collection: String,
    similarity_threshold: f32,
}

impl SocialCollector {
    /// Create new collector, ensuring Qdrant collection exists.
    pub async fn new(qdrant_url: &str, tei_url: &str) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(TEI_TIMEOUT_SECS))
            .build()?;

        let qdrant_client = Qdrant::from_url(qdrant_url).build()?;

        let collector = Self {
            http_client,
            qdrant_client,
            tei_url: tei_url.to_string(),
            collection: COLLECTION_NAME.to_string(),
            similarity_threshold: dedup::NEAR_DUPLICATE_THRESHOLD,
        };

        collector.ensure_collection().await?;
        Ok(collector)
    }

    /// Ensure Qdrant collection exists with correct dimensions.
    async fn ensure_collection(&self) -> Result<()> {
        let exists = self.qdrant_client.collection_exists(&self.collection).await?;
        if !exists {
            info!("Creating collection: {}", self.collection);
            self.qdrant_client
                .create_collection(
                    CreateCollectionBuilder::new(&self.collection)
                        .vectors_config(VectorParamsBuilder::new(VECTOR_DIM, Distance::Cosine)),
                )
                .await?;
        }
        Ok(())
    }

    /// Generate embedding via TEI service.
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let response = self
            .http_client
            .post(format!("{}/embed", self.tei_url))
            .json(&serde_json::json!({ "inputs": [text] }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("TEI error: {}", response.status()));
        }

        let tei_response: TeiResponse = response.json().await?;
        tei_response
            .0
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Empty TEI response"))
    }

    /// Compute URL hash for dedup (point ID derivation).
    fn url_hash(url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        hex::encode(hasher.finalize())[..16].to_string()
    }

    /// Store a single article in Qdrant with dedup check.
    pub async fn store_article(
        &self,
        article: &SocialArticle,
        stats: &mut SocialStats,
    ) -> Result<()> {
        // Generate embedding from title + description
        let text = format!("{}. {}", article.title, article.description);
        let embedding = match self.embed(&text).await {
            Ok(e) => e,
            Err(e) => {
                warn!("⚠️ Embed failed for '{}': {}", article.title, e);
                stats.errors += 1;
                return Ok(());
            }
        };

        // Check near-duplicate
        if dedup::is_near_duplicate(
            &self.qdrant_client,
            &self.collection,
            &embedding,
            self.similarity_threshold,
        )
        .await?
        {
            stats.duplicates_skipped += 1;
            return Ok(());
        }

        // Build payload
        let payload: HashMap<String, qdrant_client::qdrant::Value> = {
            let mut p = HashMap::new();
            let str_val = |s: String| qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(s)),
            };
            let int_val = |i: i64| qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::IntegerValue(i)),
            };
            let f64_val = |f: f64| qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::DoubleValue(f)),
            };

            p.insert("title".into(), str_val(article.title.clone()));
            p.insert("description".into(), str_val(article.description.clone()));
            p.insert("url".into(), str_val(article.url.clone()));
            p.insert("source".into(), str_val(article.source.clone()));
            p.insert("author".into(), str_val(article.author.clone()));
            p.insert("content_type".into(), str_val(article.content_type.clone()));
            p.insert("collected_at".into(), str_val(article.collected_at.clone()));
            p.insert("score".into(), int_val(article.score));
            p.insert("relevance".into(), f64_val(article.relevance as f64));

            if let Some(ref date) = article.date {
                p.insert("date".into(), str_val(date.clone()));
            }

            p
        };

        // Upsert point
        let point = PointStruct::new(Uuid::new_v4().to_string(), embedding, payload);

        self.qdrant_client
            .upsert_points(UpsertPointsBuilder::new(&self.collection, vec![point]))
            .await?;

        stats.stored += 1;
        info!("✅ Stored: {}", article.title);
        Ok(())
    }

    /// Collect from HackerNews.
    pub async fn collect_hackernews(
        &self,
        query: Option<&str>,
        depth: Depth,
        store: bool,
    ) -> Result<(Vec<SocialArticle>, SocialStats)> {
        let mut stats = SocialStats::default();
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;

        let items = if let Some(q) = query {
            hackernews::search_stories(&client, q, depth.limit()).await
        } else {
            hackernews::get_front_page(&client, depth.limit()).await
        };

        stats.total_fetched = items.len();

        if store {
            for article in &items {
                self.store_article(article, &mut stats).await?;
            }
        }

        Ok((items, stats))
    }

    /// Collect from Reddit.
    pub async fn collect_reddit(
        &self,
        query: &str,
        depth: Depth,
        subreddits: Option<&[&str]>,
        store: bool,
    ) -> Result<(Vec<SocialArticle>, SocialStats)> {
        let mut stats = SocialStats::default();
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;

        let items = reddit::search_reddit(&client, query, depth.limit(), subreddits).await;
        stats.total_fetched = items.len();

        if store {
            for article in &items {
                self.store_article(article, &mut stats).await?;
            }
        }

        Ok((items, stats))
    }

    /// Collect from YouTube (blocking subprocess, wrapped in spawn_blocking).
    pub async fn collect_youtube(
        &self,
        query: &str,
        depth: Depth,
        store: bool,
    ) -> Result<(Vec<SocialArticle>, SocialStats)> {
        let mut stats = SocialStats::default();
        let q = query.to_string();
        let limit = depth.limit();

        let items = tokio::task::spawn_blocking(move || {
            youtube::search_youtube(&q, limit)
        })
        .await?;

        stats.total_fetched = items.len();

        if store {
            for article in &items {
                self.store_article(article, &mut stats).await?;
            }
        }

        Ok((items, stats))
    }

    /// Collect from all sources.
    pub async fn collect_all(
        &self,
        query: &str,
        depth: Depth,
        subreddits: Option<&[&str]>,
        store: bool,
    ) -> Result<SocialStats> {
        let mut total_stats = SocialStats::default();

        info!("📥 Collecting HackerNews...");
        let (_, hn_stats) = self.collect_hackernews(Some(query), depth, store).await?;

        info!("📥 Collecting Reddit...");
        let (_, reddit_stats) = self.collect_reddit(query, depth, subreddits, store).await?;

        info!("📥 Collecting YouTube...");
        let (_, yt_stats) = self.collect_youtube(query, depth, store).await?;

        total_stats.total_fetched =
            hn_stats.total_fetched + reddit_stats.total_fetched + yt_stats.total_fetched;
        total_stats.stored = hn_stats.stored + reddit_stats.stored + yt_stats.stored;
        total_stats.duplicates_skipped = hn_stats.duplicates_skipped
            + reddit_stats.duplicates_skipped
            + yt_stats.duplicates_skipped;
        total_stats.errors = hn_stats.errors + reddit_stats.errors + yt_stats.errors;

        info!("📊 Social collection complete: {}", total_stats);
        Ok(total_stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_hash_deterministic() {
        let url = "https://example.com/article/123";
        let hash1 = SocialCollector::url_hash(url);
        let hash2 = SocialCollector::url_hash(url);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 16);
    }

    #[test]
    fn test_url_hash_different_urls() {
        let hash1 = SocialCollector::url_hash("https://example.com/a");
        let hash2 = SocialCollector::url_hash("https://example.com/b");
        assert_ne!(hash1, hash2);
    }
}
