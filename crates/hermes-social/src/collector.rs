/// Core social media collector types and orchestration
use anyhow::Result;
use hermes_common::types::IndonesianStock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Social media post unified representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SocialPost {
    pub id: String,
    pub source: SocialSource,
    pub title: String,
    pub content: String,
    pub author: String,
    pub url: String,
    pub published_at: DateTime<Utc>,
    pub score: Option<i32>,
    pub comments_count: Option<i32>,
    pub metadata: HashMap<String, String>,
    pub indonesian_stocks: Vec<IndonesianStock>,
    pub embedding_id: Option<String>,
}

/// Supported social media platforms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SocialSource {
    HackerNews,
    Reddit,
    YouTube,
}

/// Collection depth configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollectionDepth {
    Quick,      // Top posts only
    Medium,     // Multiple pages
    Deep,       // Comprehensive scan
}

/// Social media collector trait
#[async_trait]
pub trait SocialCollector {
    /// Collect posts by topics with specified depth
    async fn collect_by_topics(&self, topics: &[String], depth: CollectionDepth) -> Result<Vec<SocialPost>>;
    
    /// Get platform-specific source identifier
    fn source(&self) -> SocialSource;
    
    /// Health check for API availability
    async fn health_check(&self) -> Result<bool>;
}

/// Main social collection orchestrator
pub struct SocialCollectorOrchestrator {
    collectors: Vec<Box<dyn SocialCollector + Send + Sync>>,
    indonesian_stocks: Vec<IndonesianStock>,
}

impl SocialCollectorOrchestrator {
    /// Create new orchestrator with all collectors
    pub fn new() -> Self {
        Self {
            collectors: Vec::new(),
            indonesian_stocks: vec![
                IndonesianStock::BMRI,
                IndonesianStock::BBRI,
                IndonesianStock::INCO,
                IndonesianStock::ANTM,
                IndonesianStock::PTBA,
                IndonesianStock::TAPG,
            ],
        }
    }
    
    /// Add collector to orchestrator
    pub fn add_collector(&mut self, collector: Box<dyn SocialCollector + Send + Sync>) {
        self.collectors.push(collector);
    }
    
    /// Collect from all registered collectors in parallel
    pub async fn collect_all(&self, topics: &[String], depth: CollectionDepth) -> Result<Vec<SocialPost>> {
        info!("🚀 Starting parallel collection from {} sources", self.collectors.len());
        
        let mut all_posts = Vec::new();
        let mut health_checks = Vec::new();
        
        // Parallel health checks first
        for collector in &self.collectors {
            health_checks.push(async move {
                match collector.health_check().await {
                    Ok(healthy) if healthy => Some(collector),
                    Ok(_) => {
                        warn!("❌ Source {:?} failed health check", collector.source());
                        None
                    },
                    Err(e) => {
                        error!("💥 Health check error for {:?}: {}", collector.source(), e);
                        None
                    }
                }
            });
        }
        
        // Execute parallel health checks
        let healthy_collectors: Vec<_> = futures::future::join_all(health_checks)
            .await
            .into_iter()
            .filter_map(|x| x)
            .collect();
        
        info!("✅ {} of {} collectors are healthy", healthy_collectors.len(), self.collectors.len());
        
        // Parallel data collection from healthy collectors
        let collection_futures: Vec<_> = healthy_collectors
            .into_iter()
            .map(|collector| async move {
                match collector.collect_by_topics(topics, depth).await {
                    Ok(posts) => {
                        info!("📊 Collected {} posts from {:?}", posts.len(), collector.source());
                        posts
                    },
                    Err(e) => {
                        error!("💥 Collection failed for {:?}: {}", collector.source(), e);
                        Vec::new()
                    }
                }
            })
            .collect();
        
        // Execute parallel collections
        let collection_results = futures::future::join_all(collection_futures).await;
        
        // Aggregate results
        for posts in collection_results {
            all_posts.extend(posts);
        }
        
        // Enrich with Indonesian stock detection
        self.enrich_with_indonesian_stocks(&mut all_posts);
        
        info!("🎯 Total collected: {} posts across all platforms", all_posts.len());
        Ok(all_posts)
    }
    
    /// Detect Indonesian stocks mentioned in posts
    fn enrich_with_indonesian_stocks(&self, posts: &mut [SocialPost]) {
        for post in posts {
            let combined_text = format!("{} {}", post.title, post.content).to_lowercase();
            
            let mut detected_stocks = Vec::new();
            for stock in &self.indonesian_stocks {
                let stock_codes = match stock {
                    IndonesianStock::BMRI => vec!["bmri", "bank mandiri", "mandiri"],
                    IndonesianStock::BBRI => vec!["bbri", "bri", "bank rakyat"],
                    IndonesianStock::INCO => vec!["inco", "vale indonesia", "nickel"],
                    IndonesianStock::ANTM => vec!["antm", "aneka tambang", "antam"],
                    IndonesianStock::PTBA => vec!["ptba", "bukit asam", "coal"],
                    IndonesianStock::TLKM => vec!["tlkm", "telkom"],
                    IndonesianStock::ASII => vec!["asii", "astra"],
                    IndonesianStock::KLBF => vec!["klbf", "kalbe"],
                    IndonesianStock::TSPC => vec!["tspc", "tempo scan"],
                    IndonesianStock::BSDE => vec!["bsde", "bumi serpong"],
                    IndonesianStock::TAPG => vec!["tapg", "triputra agro"],
                };
                
                if stock_codes.iter().any(|code| combined_text.contains(code)) {
                    detected_stocks.push(*stock);
                }
            }
            
            post.indonesian_stocks = detected_stocks;
        }
    }
}

/// Collection statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionStats {
    pub source: SocialSource,
    pub posts_collected: usize,
    pub indonesian_mentions: usize,
    pub collection_time_ms: u64,
    pub error_count: usize,
}

impl CollectionStats {
    pub fn new(source: SocialSource) -> Self {
        Self {
            source,
            posts_collected: 0,
            indonesian_mentions: 0,
            collection_time_ms: 0,
            error_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = SocialCollectorOrchestrator::new();
        assert_eq!(orchestrator.collectors.len(), 0);
        assert_eq!(orchestrator.indonesian_stocks.len(), 6);
    }
    
    #[tokio::test]
    async fn test_indonesian_stock_detection() {
        let orchestrator = SocialCollectorOrchestrator::new();
        let mut posts = vec![
            SocialPost {
                id: "test1".to_string(),
                source: SocialSource::HackerNews,
                title: "BMRI quarterly report".to_string(),
                content: "Bank Mandiri shows strong growth".to_string(),
                author: "analyst".to_string(),
                url: "https://test.com".to_string(),
                published_at: Utc::now(),
                score: Some(100),
                comments_count: Some(20),
                metadata: HashMap::new(),
                indonesian_stocks: Vec::new(),
                embedding_id: None,
            }
        ];
        
        orchestrator.enrich_with_indonesian_stocks(&mut posts);
        
        assert!(!posts[0].indonesian_stocks.is_empty());
        assert!(posts[0].indonesian_stocks.contains(&IndonesianStock::BMRI));
    }
    
    #[tokio::test]
    async fn test_social_post_serialization() {
        let post = SocialPost {
            id: "test".to_string(),
            source: SocialSource::Reddit,
            title: "Test Post".to_string(),
            content: "Test content".to_string(),
            author: "testuser".to_string(),
            url: "https://reddit.com/test".to_string(),
            published_at: Utc::now(),
            score: Some(50),
            comments_count: Some(10),
            metadata: HashMap::new(),
            indonesian_stocks: vec![IndonesianStock::INCO],
            embedding_id: Some("embed123".to_string()),
        };
        
        let json = serde_json::to_string(&post).unwrap();
        let deserialized: SocialPost = serde_json::from_str(&json).unwrap();
        
        assert_eq!(post, deserialized);
    }
}