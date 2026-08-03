//! Hermes Collector Service
//!
//! RSS feed collection service with circuit breaker resilience patterns.
//! Handles Indonesian news sources and international feeds.

pub mod feed_source;
pub mod circuit_breaker;
pub mod rss_fetcher;
pub mod collector;

use anyhow::Result;
use tracing::info;

/// Re-export core types for convenience
pub use feed_source::{FeedSource, FeedCategory};
pub use circuit_breaker::CircuitBreaker;
pub use rss_fetcher::RssFetcher;
pub use collector::HermesCollector;

/// Collection statistics for monitoring
#[derive(Debug, Clone)]
pub struct CollectionStats {
    pub total_sources: usize,
    pub successful_collections: usize,
    pub failed_collections: usize,
    pub articles_collected: usize,
    pub indonesian_articles: usize,
    pub international_articles: usize,
    pub collection_duration_ms: u64,
    pub circuit_breaker_trips: usize,
}

impl CollectionStats {
    pub fn new() -> Self {
        Self {
            total_sources: 0,
            successful_collections: 0,
            failed_collections: 0,
            articles_collected: 0,
            indonesian_articles: 0,
            international_articles: 0,
            collection_duration_ms: 0,
            circuit_breaker_trips: 0,
        }
    }
    
    /// Calculate success rate percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_sources == 0 {
            return 0.0;
        }
        (self.successful_collections as f64 / self.total_sources as f64) * 100.0
    }
    
    /// Calculate articles per second throughput
    pub fn articles_per_second(&self) -> f64 {
        if self.collection_duration_ms == 0 {
            return 0.0;
        }
        (self.articles_collected as f64 / self.collection_duration_ms as f64) * 1000.0
    }
    
    /// Check if collection meets minimum quality thresholds
    pub fn meets_quality_threshold(&self) -> bool {
        self.success_rate() >= 70.0 && self.articles_collected > 0
    }
}

/// Default Indonesian news sources for bootstrap
pub fn default_indonesian_sources() -> Vec<FeedSource> {
    vec![
        FeedSource::new(
            "kompas".to_string(),
            "https://www.kompas.com/rss".to_string(),
            FeedCategory::IndonesianNews,
            true, // high_priority
        ),
        FeedSource::new(
            "detik".to_string(),
            "https://rss.detik.com/index.php/detikcom".to_string(),
            FeedCategory::IndonesianNews,
            true,
        ),
        FeedSource::new(
            "tempo".to_string(),
            "https://www.tempo.co/rss".to_string(),
            FeedCategory::IndonesianNews,
            true,
        ),
        FeedSource::new(
            "cnn_indonesia".to_string(),
            "https://www.cnnindonesia.com/rss".to_string(),
            FeedCategory::IndonesianNews,
            true,
        ),
        FeedSource::new(
            "antara".to_string(),
            "https://www.antaranews.com/rss/terkini".to_string(),
            FeedCategory::IndonesianNews,
            false,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_stats() {
        let mut stats = CollectionStats::new();
        assert_eq!(stats.success_rate(), 0.0);
        assert_eq!(stats.articles_per_second(), 0.0);
        assert!(!stats.meets_quality_threshold());
        
        stats.total_sources = 10;
        stats.successful_collections = 8;
        stats.articles_collected = 150;
        stats.collection_duration_ms = 5000;
        
        assert_eq!(stats.success_rate(), 80.0);
        assert_eq!(stats.articles_per_second(), 30.0);
        assert!(stats.meets_quality_threshold());
    }

    #[test]
    fn test_default_indonesian_sources() {
        let sources = default_indonesian_sources();
        assert_eq!(sources.len(), 5);
        
        // Verify high-priority sources
        let high_priority_count = sources.iter().filter(|s| s.high_priority).count();
        assert_eq!(high_priority_count, 4);
        
        // Verify all are Indonesian news category
        assert!(sources.iter().all(|s| s.category == FeedCategory::IndonesianNews));
        
        // Verify specific sources exist
        assert!(sources.iter().any(|s| s.name == "kompas"));
        assert!(sources.iter().any(|s| s.name == "detik"));
        assert!(sources.iter().any(|s| s.name == "tempo"));
        assert!(sources.iter().any(|s| s.name == "cnn_indonesia"));
    }
}