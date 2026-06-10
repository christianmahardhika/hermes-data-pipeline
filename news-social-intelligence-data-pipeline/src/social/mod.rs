//! Social Intelligence module — normalized collection from HackerNews, Reddit, YouTube, X/Twitter.

pub mod relevance;
pub mod hackernews;
pub mod reddit;
pub mod youtube;
pub mod x_twitter;
pub mod collector;
pub mod dedup;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Normalized article type collected from any social source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialArticle {
    pub id: String,
    pub title: String,
    pub url: String,
    pub description: String,
    /// "HackerNews", "Reddit/r/sub", "YouTube", "X/Twitter"
    pub source: String,
    pub author: String,
    pub score: i64,
    pub num_comments: i64,
    /// Unix timestamp
    pub created_utc: Option<f64>,
    /// YYYY-MM-DD
    pub date: Option<String>,
    /// 0.0-1.0
    pub relevance: f32,
    /// ISO 8601
    pub collected_at: String,
    /// "hackernews", "reddit", "youtube", "x_twitter"
    pub content_type: String,
    /// Source-specific extra fields
    pub metadata: serde_json::Value,
}

/// Statistics for a social collection run.
#[derive(Debug, Default)]
pub struct SocialStats {
    pub total_fetched: usize,
    pub stored: usize,
    pub duplicates_skipped: usize,
    pub errors: usize,
}

impl fmt::Display for SocialStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "fetched={}, stored={}, duplicates={}, errors={}",
            self.total_fetched, self.stored, self.duplicates_skipped, self.errors
        )
    }
}

/// Collection depth controlling how many items to fetch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Depth {
    Quick,
    Default,
    Deep,
}

impl Depth {
    /// Maximum number of items to fetch for this depth level.
    pub fn limit(&self) -> usize {
        match self {
            Depth::Quick => 10,
            Depth::Default => 25,
            Depth::Deep => 50,
        }
    }
}

impl FromStr for Depth {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "quick" => Ok(Depth::Quick),
            "default" => Ok(Depth::Default),
            "deep" => Ok(Depth::Deep),
            other => Err(format!("unknown depth: '{}' (expected quick, default, deep)", other)),
        }
    }
}
