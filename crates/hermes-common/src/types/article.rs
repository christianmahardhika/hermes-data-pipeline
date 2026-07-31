//! Article domain model for Hermes intelligence pipeline

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
    pub source: Option<String>,
    pub category: Option<String>,
}

impl Article {
    /// Create a new article with raw status
    pub fn new(title: String, content: String, url: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            content,
            url,
            status: ArticleStatus::Raw,
            created_at: now,
            updated_at: now,
            source: None,
            category: None,
        }
    }

    /// Update article status and timestamp
    pub fn update_status(mut self, status: ArticleStatus) -> Self {
        self.status = status;
        self.updated_at = Utc::now();
        self
    }

    /// Set article source and category
    pub fn with_metadata(mut self, source: Option<String>, category: Option<String>) -> Self {
        self.source = source;
        self.category = category;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_article_creation() {
        let article = Article::new(
            "Test Article".to_string(),
            "Test content".to_string(),
            "https://example.com".to_string(),
        );

        assert_eq!(article.title, "Test Article");
        assert_eq!(article.status, ArticleStatus::Raw);
        assert!(article.id.to_string().len() > 0);
    }

    #[test]
    fn test_article_status_update() {
        let article = Article::new(
            "Test".to_string(),
            "Content".to_string(),
            "https://example.com".to_string(),
        )
        .update_status(ArticleStatus::Processed);

        assert_eq!(article.status, ArticleStatus::Processed);
    }

    #[test]
    fn test_article_with_metadata() {
        let article = Article::new(
            "Test".to_string(),
            "Content".to_string(),
            "https://example.com".to_string(),
        )
        .with_metadata(Some("Reuters".to_string()), Some("Politics".to_string()));

        assert_eq!(article.source, Some("Reuters".to_string()));
        assert_eq!(article.category, Some("Politics".to_string()));
    }
}