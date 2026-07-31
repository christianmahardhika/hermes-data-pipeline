//! Main collector orchestration logic

use crate::{
    circuit_breaker::{CircuitBreaker, CircuitState},
    feed_source::{FeedSource, FeedCategory},
    rss_fetcher::{RssFetcher, HttpClient, RssFeedResult},
    CollectionStats,
};
use anyhow::Result;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, warn, error, debug};

/// Main Hermes collector for RSS feeds with circuit breaker resilience
pub struct HermesCollector<T: HttpClient> {
    pub sources: Vec<FeedSource>,
    pub circuit_breakers: HashMap<String, CircuitBreaker>,
    pub rss_fetcher: RssFetcher<T>,
    pub max_concurrent_feeds: usize,
    pub collection_timeout_seconds: u64,
}

impl<T: HttpClient> HermesCollector<T> {
    /// Create new Hermes collector
    pub fn new(http_client: T) -> Self {
        Self {
            sources: Vec::new(),
            circuit_breakers: HashMap::new(),
            rss_fetcher: RssFetcher::new(http_client),
            max_concurrent_feeds: 10,
            collection_timeout_seconds: 30,
        }
    }

    /// Configure collector with sources and settings
    pub fn with_sources(mut self, sources: Vec<FeedSource>) -> Self {
        for source in &sources {
            // Create circuit breaker for each source
            let cb = CircuitBreaker::new(
                source.name.clone(),
                5, // failure_threshold
                60, // timeout_seconds
            );
            self.circuit_breakers.insert(source.name.clone(), cb);
        }
        self.sources = sources;
        self
    }

    /// Configure max concurrent feeds
    pub fn with_max_concurrent(mut self, max_concurrent: usize) -> Self {
        self.max_concurrent_feeds = max_concurrent;
        self
    }

    /// Collect RSS feeds from all configured sources
    pub fn collect_all_feeds(&mut self) -> Result<CollectionStats> {
        let start_time = Instant::now();
        let mut stats = CollectionStats::new();
        stats.total_sources = self.sources.len();

        info!(
            sources = stats.total_sources,
            max_concurrent = self.max_concurrent_feeds,
            "🚀 Starting RSS collection cycle"
        );

        // Collect from all sources (clone to avoid borrow conflict)
        let sources = self.sources.clone();
        for source in &sources {
            let feed_result = self.collect_single_feed(source, &mut stats);
            match feed_result {
                Ok(_) => {
                    debug!(
                        source = %source.name,
                        "✅ Successfully processed RSS feed"
                    );
                }
                Err(e) => {
                    error!(
                        source = %source.name,
                        error = %e,
                        "❌ Failed to process RSS feed"
                    );
                }
            }
        }

        stats.collection_duration_ms = start_time.elapsed().as_millis() as u64;

        info!(
            duration_ms = stats.collection_duration_ms,
            success_rate = %format!("{:.1}%", stats.success_rate()),
            articles_collected = stats.articles_collected,
            indonesian_articles = stats.indonesian_articles,
            international_articles = stats.international_articles,
            circuit_breaker_trips = stats.circuit_breaker_trips,
            "📊 RSS collection cycle completed"
        );

        Ok(stats)
    }

    /// Collect from a single RSS feed source
    fn collect_single_feed(&mut self, source: &FeedSource, stats: &mut CollectionStats) -> Result<()> {
        // Check circuit breaker
        let can_execute = {
            if let Some(cb) = self.circuit_breakers.get_mut(&source.name) {
                cb.can_execute()
            } else {
                warn!(
                    source = %source.name,
                    "⚠️ No circuit breaker found for source"
                );
                true
            }
        };

        if !can_execute {
            stats.circuit_breaker_trips += 1;
            warn!(
                source = %source.name,
                "🚫 Circuit breaker blocking RSS feed collection"
            );
            return Ok(()); // Not an error, just circuit breaker protection
        }

        // This is a placeholder since we can't actually fetch in this simplified implementation
        // In real implementation, this would call self.rss_fetcher.fetch_feed(source).await
        let mock_result = self.simulate_feed_collection(source);
        
        // Update circuit breaker and stats based on result
        if mock_result.is_success() {
            if let Some(cb) = self.circuit_breakers.get_mut(&source.name) {
                cb.record_success();
            }
            stats.successful_collections += 1;
            stats.articles_collected += mock_result.items.len();
            
            // Categorize articles
            if source.is_indonesian_market_related() {
                stats.indonesian_articles += mock_result.items.len();
            } else {
                stats.international_articles += mock_result.items.len();
            }
        } else {
            if let Some(cb) = self.circuit_breakers.get_mut(&source.name) {
                cb.record_failure();
            }
            stats.failed_collections += 1;
            
            warn!(
                source = %source.name,
                error = %mock_result.error,
                "⚠️ RSS feed collection failed"
            );
        }

        Ok(())
    }

    /// Simulate feed collection for testing (placeholder)
    fn simulate_feed_collection(&self, source: &FeedSource) -> RssFeedResult {
        // This is a mock implementation for TDD purposes
        // In real implementation, this would be an async call to rss_fetcher
        
        if source.enabled && source.high_priority {
            // Simulate successful collection for high-priority enabled sources
            RssFeedResult::new(source.clone(), vec![], String::new())
        } else if !source.enabled {
            RssFeedResult::new(source.clone(), vec![], "Source disabled".to_string())
        } else {
            // Simulate occasional failures for low-priority sources
            RssFeedResult::new(source.clone(), vec![], "Simulated network error".to_string())
        }
    }

    /// Get circuit breaker status for monitoring
    pub fn get_circuit_breaker_status(&self) -> HashMap<String, CircuitState> {
        self.circuit_breakers
            .iter()
            .map(|(name, cb)| (name.clone(), cb.state.clone()))
            .collect()
    }

    /// Reset all circuit breakers (for testing/maintenance)
    pub fn reset_circuit_breakers(&mut self) {
        info!("🔄 Resetting all circuit breakers");
        for (name, cb) in &mut self.circuit_breakers {
            *cb = CircuitBreaker::new(name.clone(), 5, 60);
        }
    }

    /// Get collection statistics summary
    pub fn get_stats_summary(&self) -> String {
        let total_sources = self.sources.len();
        let high_priority_sources = self.sources.iter().filter(|s| s.high_priority).count();
        let indonesian_sources = self.sources.iter().filter(|s| s.is_indonesian_market_related()).count();
        let open_circuit_breakers = self.circuit_breakers.values()
            .filter(|cb| cb.state == CircuitState::Open)
            .count();

        format!(
            "Hermes Collector: {} sources ({} high-priority, {} Indonesian), {} circuit breakers open",
            total_sources, high_priority_sources, indonesian_sources, open_circuit_breakers
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rss_fetcher::MockHttpClient;

    #[test]
    fn test_hermes_collector_creation() {
        let client = MockHttpClient::new();
        let collector = HermesCollector::new(client);
        
        assert_eq!(collector.sources.len(), 0);
        assert_eq!(collector.circuit_breakers.len(), 0);
        assert_eq!(collector.max_concurrent_feeds, 10);
        assert_eq!(collector.collection_timeout_seconds, 30);
    }

    #[test]
    fn test_hermes_collector_with_sources() {
        let client = MockHttpClient::new();
        let sources = vec![
            FeedSource::new(
                "kompas".to_string(),
                "https://kompas.com/rss".to_string(),
                FeedCategory::IndonesianNews,
                true,
            ),
            FeedSource::new(
                "detik".to_string(),
                "https://detik.com/rss".to_string(),
                FeedCategory::IndonesianNews,
                true,
            ),
        ];
        
        let collector = HermesCollector::new(client).with_sources(sources);
        
        assert_eq!(collector.sources.len(), 2);
        assert_eq!(collector.circuit_breakers.len(), 2);
        assert!(collector.circuit_breakers.contains_key("kompas"));
        assert!(collector.circuit_breakers.contains_key("detik"));
    }

    #[test]
    fn test_collection_stats_calculation() {
        let client = MockHttpClient::new();
        let sources = vec![
            FeedSource::new(
                "kompas".to_string(),
                "https://kompas.com/rss".to_string(),
                FeedCategory::IndonesianNews,
                true,
            ),
            FeedSource::new(
                "hacker_news".to_string(),
                "https://hnrss.org/frontpage".to_string(),
                FeedCategory::Technology,
                false,
            ),
        ];
        
        let mut collector = HermesCollector::new(client).with_sources(sources);
        let stats = collector.collect_all_feeds().unwrap();
        
        assert_eq!(stats.total_sources, 2);
        // Collection duration could be 0 in fast test environment
        assert!(stats.collection_duration_ms >= 0);
        
        // High-priority sources should succeed, low-priority might fail
        assert!(stats.successful_collections >= 1);
        assert_eq!(stats.successful_collections + stats.failed_collections, stats.total_sources);
    }

    #[test]
    fn test_circuit_breaker_status() {
        let client = MockHttpClient::new();
        let sources = vec![
            FeedSource::new(
                "test_source".to_string(),
                "https://example.com/rss".to_string(),
                FeedCategory::IndonesianNews,
                true,
            ),
        ];
        
        let collector = HermesCollector::new(client).with_sources(sources);
        let status = collector.get_circuit_breaker_status();
        
        assert_eq!(status.len(), 1);
        assert_eq!(status.get("test_source"), Some(&CircuitState::Closed));
    }

    #[test]
    fn test_stats_summary() {
        let client = MockHttpClient::new();
        let sources = vec![
            FeedSource::new(
                "kompas".to_string(),
                "https://kompas.com/rss".to_string(),
                FeedCategory::IndonesianNews,
                true,
            ),
            FeedSource::new(
                "hacker_news".to_string(),
                "https://hnrss.org/frontpage".to_string(),
                FeedCategory::Technology,
                false,
            ),
        ];
        
        let collector = HermesCollector::new(client).with_sources(sources);
        let summary = collector.get_stats_summary();
        
        assert!(summary.contains("2 sources"));
        assert!(summary.contains("1 high-priority"));
        assert!(summary.contains("1 Indonesian"));
        assert!(summary.contains("0 circuit breakers open"));
    }
}