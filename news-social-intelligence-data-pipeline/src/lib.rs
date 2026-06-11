//! News Collector - Rust ETL Pipeline
//! 
//! Pipeline: RSS → SQLite (staging) → Clean → Label (Kiromania) → Embed (TEI) → Qdrant
//! 
//! Prof Jiang Game Theory Framework:
//! Event = Actor (incentives, constraints) + Action + Target + Context

mod collectors;
mod cleaners;
mod labelers;
mod embedders;
pub mod storage;
pub mod health;
pub mod social;
pub mod unlimited;
pub mod idx_analyst;

pub use collectors::RssCollector;
pub use cleaners::ArticleCleaner;
pub use labelers::KiroLabeler;
pub use embedders::TeiEmbedder;
pub use storage::{Database, RawFeed, CleanedArticle, LabeledArticle};
pub use health::KiroHealth;
pub use unlimited::UnlimitedCollector;

/// Pipeline configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// SQLite database path
    pub db_path: String,
    /// Kiromania API URL
    pub kiro_url: String,
    /// Kiromania API key
    pub kiro_api_key: String,
    /// TEI embedding service URL
    pub tei_url: String,
    /// Qdrant URL
    pub qdrant_url: String,
    /// Qdrant collection name
    pub collection_name: String,
    /// Batch size for labeling
    pub label_batch_size: usize,
    /// Similarity threshold for near-duplicate detection
    pub similarity_threshold: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_path: std::env::var("DB_PATH")
                .unwrap_or_else(|_| "news_staging.db".to_string()),
            kiro_url: std::env::var("KIRO_URL")
                .unwrap_or_else(|_| "http://localhost:9000".to_string()),
            kiro_api_key: std::env::var("KIRO_API_KEY")
                .expect("KIRO_API_KEY environment variable must be set"),
            tei_url: std::env::var("TEI_URL")
                .unwrap_or_else(|_| "http://localhost:8082".to_string()),
            qdrant_url: std::env::var("QDRANT_URL")
                .unwrap_or_else(|_| "http://localhost:6334".to_string()),
            collection_name: std::env::var("QDRANT_COLLECTION")
                .unwrap_or_else(|_| "news_articles".to_string()),
            label_batch_size: 10,
            similarity_threshold: 0.95,
        }
    }
}
