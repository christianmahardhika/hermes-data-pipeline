//! Stub storage module - types only
//! Real storage migrated to ArangoDB via hermes-common

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Cleaned article ready for labeling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanedArticle {
    pub id: Option<i64>,
    pub raw_id: i64,
    pub title: String,
    pub content: String,
    pub published_at: Option<DateTime<Utc>>,
    pub source: String,
    pub url: String,
    pub content_hash: String,
    pub cleaned_at: DateTime<Utc>,
}

/// Prof Jiang labeled article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledArticle {
    pub id: Option<i64>,
    pub cleaned_id: i64,
    pub sentiment: String,
    pub sentiment_score: f32,
    pub news_type: String,
    pub news_subtype: Option<String>,
    pub events: serde_json::Value,
    pub actors: serde_json::Value,
    pub relations: serde_json::Value,
    pub context: serde_json::Value,
    pub pattern_match: serde_json::Value,
    pub investment_signal: serde_json::Value,
    pub labeled_at: DateTime<Utc>,
    pub labeled_by: String,
}

/// Raw feed entry (stub)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFeed {
    pub id: Option<i64>,
    pub feed_name: String,
    pub raw_content: Vec<u8>,
    pub content_type: String,
    pub fetched_at: DateTime<Utc>,
    pub status: String,
    pub retry_count: i32,
}

/// Stub database (not used for idx-analyst)
pub struct Database;
