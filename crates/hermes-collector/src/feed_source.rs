//! Feed source definitions for RSS collection

use serde::{Deserialize, Serialize};

/// Category of RSS feed source
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedCategory {
    IndonesianNews,
    InternationalNews,
    Financial,
    Technology,
    Commodity,
}

/// RSS feed source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedSource {
    pub name: String,
    pub url: String,
    pub category: FeedCategory,
    pub high_priority: bool,
    pub enabled: bool,
}

impl FeedSource {
    /// Create a new feed source
    pub fn new(name: String, url: String, category: FeedCategory, high_priority: bool) -> Self {
        Self {
            name,
            url,
            category,
            high_priority,
            enabled: true,
        }
    }
    
    /// Check if source is Indonesian market related
    pub fn is_indonesian_market_related(&self) -> bool {
        matches!(
            self.category,
            FeedCategory::IndonesianNews | FeedCategory::Financial | FeedCategory::Commodity
        ) || self.name.contains("indonesia") || self.url.contains("indonesia")
    }
    
    /// Check if source is high priority for Christian's intelligence system
    pub fn is_high_priority(&self) -> bool {
        self.high_priority && self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feed_source_creation() {
        let source = FeedSource::new(
            "kompas".to_string(),
            "https://www.kompas.com/rss".to_string(),
            FeedCategory::IndonesianNews,
            true,
        );
        
        assert_eq!(source.name, "kompas");
        assert_eq!(source.category, FeedCategory::IndonesianNews);
        assert!(source.high_priority);
        assert!(source.enabled);
    }

    #[test]
    fn test_indonesian_market_related() {
        let indonesian_source = FeedSource::new(
            "kompas".to_string(),
            "https://www.kompas.com/rss".to_string(),
            FeedCategory::IndonesianNews,
            true,
        );
        assert!(indonesian_source.is_indonesian_market_related());
        
        let tech_source = FeedSource::new(
            "hacker_news".to_string(),
            "https://hnrss.org/frontpage".to_string(),
            FeedCategory::Technology,
            false,
        );
        assert!(!tech_source.is_indonesian_market_related());
    }

    #[test]
    fn test_high_priority_check() {
        let high_priority = FeedSource::new(
            "detik".to_string(),
            "https://rss.detik.com/index.php/detikcom".to_string(),
            FeedCategory::IndonesianNews,
            true,
        );
        assert!(high_priority.is_high_priority());
        
        let low_priority = FeedSource::new(
            "antara".to_string(),
            "https://www.antaranews.com/rss/terkini".to_string(),
            FeedCategory::IndonesianNews,
            false,
        );
        assert!(!low_priority.is_high_priority());
        
        let mut disabled = FeedSource::new(
            "tempo".to_string(),
            "https://www.tempo.co/rss".to_string(),
            FeedCategory::IndonesianNews,
            true,
        );
        disabled.enabled = false;
        assert!(!disabled.is_high_priority()); // Disabled sources are not high priority
    }
}