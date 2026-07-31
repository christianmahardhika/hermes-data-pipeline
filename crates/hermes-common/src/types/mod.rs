//! Domain types and models shared across Hermes services
//!
//! Common data structures used throughout the pipeline including:
//! - Article and news item models
//! - Processing status enums
//! - Correlation data structures

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Status of an article in the processing pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleStatus {
    Raw,
    Cleaned,
    Labeled,
    Embedded,
    Processed,
    Error,
}

/// Core article model shared across services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub url: String,
    pub status: ArticleStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Indonesian stock symbol for portfolio tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndonesianStock {
    pub symbol: String, // e.g., "BMRI", "BBRI", "INCO", "ANTM"
    pub name: String,
    pub price: Option<f64>,
    pub change_pct: Option<f64>,
    pub updated_at: DateTime<Utc>,
}