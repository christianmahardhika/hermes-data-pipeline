/// HackerNews collector implementation
/// Collects tech discussions, startup news, and Indonesian tech mentions
use crate::collector::{SocialCollector, SocialPost, SocialSource, CollectionDepth};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use chrono::{DateTime, Utc, TimeZone};
use hermes_common::types::IndonesianStock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// HackerNews API response structures
#[derive(Debug, Deserialize)]
struct HNItem {
    id: u32,
    #[serde(rename = "type")]
    item_type: Option<String>,
    title: Option<String>,
    text: Option<String>,
    url: Option<String>,
    by: Option<String>,
    time: Option<u64>,
    score: Option<i32>,
    descendants: Option<i32>,
    kids: Option<Vec<u32>>,
}

/// Mock HTTP client trait for testing (dyn compatible)
#[async_trait]
pub trait HttpClient {
    async fn get_json_string(&self, url: &str) -> Result<String>;
}

/// Mock HTTP client implementation
pub struct MockHttpClient {
    responses: HashMap<String, String>,
}

impl MockHttpClient {
    pub fn new() -> Self {
        let mut responses = HashMap::new();
        
        // Mock HN top stories
        responses.insert(
            "https://hacker-news.firebaseio.com/v0/topstories.json".to_string(),
            "[1, 2, 3, 4, 5]".to_string(),
        );
        
        // Mock story items
        responses.insert(
            "https://hacker-news.firebaseio.com/v0/item/1.json".to_string(),
            r#"{"id":1,"type":"story","title":"Indonesian Fintech Revolution","url":"https://example.com","by":"techuser","time":1640995200,"score":150,"descendants":25}"#.to_string(),
        );
        
        responses.insert(
            "https://hacker-news.firebaseio.com/v0/item/2.json".to_string(),
            r#"{"id":2,"type":"story","title":"AI in Southeast Asia","text":"Discussion about AI adoption in Indonesia","by":"aiuser","time":1640995300,"score":80,"descendants":12}"#.to_string(),
        );
        
        Self { responses }
    }
}

#[async_trait]
impl HttpClient for MockHttpClient {
    async fn get_json_string(&self, url: &str) -> Result<String> {
        match self.responses.get(url) {
            Some(response) => Ok(response.clone()),
            None => Err(anyhow!("URL not found in mock responses: {}", url)),
        }
    }
}

/// HackerNews collector
pub struct HackerNewsCollector {
    client: Box<dyn HttpClient + Send + Sync>,
    base_url: String,
}

impl HackerNewsCollector {
    /// Create new HackerNews collector with mock client
    pub fn new() -> Self {
        Self {
            client: Box::new(MockHttpClient::new()),
            base_url: "https://hacker-news.firebaseio.com/v0".to_string(),
        }
    }
    
    /// Create with custom HTTP client for production
    pub fn with_client(client: Box<dyn HttpClient + Send + Sync>) -> Self {
        Self {
            client,
            base_url: "https://hacker-news.firebaseio.com/v0".to_string(),
        }
    }
    
    /// Get top story IDs
    async fn get_top_stories(&self, limit: usize) -> Result<Vec<u32>> {
        let url = format!("{}/topstories.json", self.base_url);
        let story_ids: Vec<u32> = self.client.get_json(&url).await?;
        
        Ok(story_ids.into_iter().take(limit).collect())
    }
    
    /// Get story item by ID
    async fn get_item(&self, id: u32) -> Result<Option<HNItem>> {
        let url = format!("{}/item/{}.json", self.base_url, id);
        
        match self.client.get_json::<HNItem>(&url).await {
            Ok(item) => Ok(Some(item)),
            Err(_) => {
                debug!("Failed to fetch HN item {}", id);
                Ok(None)
            }
        }
    }
    
    /// Convert HNItem to SocialPost
    fn item_to_post(&self, item: HNItem) -> Option<SocialPost> {
        // Filter for stories only
        if item.item_type.as_deref() != Some("story") {
            return None;
        }
        
        let title = item.title.unwrap_or_else(|| "No title".to_string());
        let content = item.text.unwrap_or_else(String::new);
        let author = item.by.unwrap_or_else(|| "anonymous".to_string());
        let url = item.url.unwrap_or_else(|| format!("https://news.ycombinator.com/item?id={}", item.id));
        
        let published_at = item.time
            .map(|t| Utc.timestamp_opt(t as i64, 0).single())
            .flatten()
            .unwrap_or_else(Utc::now);
        
        let mut metadata = HashMap::new();
        metadata.insert("item_id".to_string(), item.id.to_string());
        metadata.insert("platform".to_string(), "hackernews".to_string());
        
        Some(SocialPost {
            id: format!("hn_{}", item.id),
            source: SocialSource::HackerNews,
            title,
            content,
            author,
            url,
            published_at,
            score: item.score,
            comments_count: item.descendants,
            metadata,
            indonesian_stocks: Vec::new(), // Will be enriched later
            embedding_id: None,
        })
    }
    
    /// Filter posts by topics
    fn filter_by_topics(&self, posts: Vec<SocialPost>, topics: &[String]) -> Vec<SocialPost> {
        if topics.is_empty() {
            return posts;
        }
        
        posts.into_iter()
            .filter(|post| {
                let combined_text = format!("{} {}", post.title, post.content).to_lowercase();
                topics.iter().any(|topic| combined_text.contains(&topic.to_lowercase()))
            })
            .collect()
    }
}

#[async_trait]
impl SocialCollector for HackerNewsCollector {
    async fn collect_by_topics(&self, topics: &[String], depth: CollectionDepth) -> Result<Vec<SocialPost>> {
        info!("📰 Collecting HackerNews posts for topics: {:?} (depth: {:?})", topics, depth);
        
        let limit = match depth {
            CollectionDepth::Quick => 10,
            CollectionDepth::Medium => 50,
            CollectionDepth::Deep => 100,
        };
        
        let story_ids = self.get_top_stories(limit).await?;
        info!("🔍 Found {} top stories to process", story_ids.len());
        
        let mut posts = Vec::new();
        
        // Collect items in batches to avoid overwhelming the API
        for chunk in story_ids.chunks(10) {
            let item_futures: Vec<_> = chunk.iter()
                .map(|&id| self.get_item(id))
                .collect();
            
            let items = futures::future::join_all(item_futures).await;
            
            for item_result in items {
                if let Ok(Some(item)) = item_result {
                    if let Some(post) = self.item_to_post(item) {
                        posts.push(post);
                    }
                }
            }
            
            // Rate limiting - be nice to HN API
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        let filtered_posts = self.filter_by_topics(posts, topics);
        
        info!("✅ HackerNews collection complete: {} posts", filtered_posts.len());
        Ok(filtered_posts)
    }
    
    fn source(&self) -> SocialSource {
        SocialSource::HackerNews
    }
    
    async fn health_check(&self) -> Result<bool> {
        // Try to fetch top stories to verify API availability
        match self.get_top_stories(1).await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("❌ HackerNews health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hackernews_collector_creation() {
        let collector = HackerNewsCollector::new();
        assert_eq!(collector.source(), SocialSource::HackerNews);
    }
    
    #[tokio::test]
    async fn test_health_check() {
        let collector = HackerNewsCollector::new();
        let result = collector.health_check().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_collect_indonesia_topics() {
        let collector = HackerNewsCollector::new();
        let topics = vec!["Indonesian".to_string(), "fintech".to_string()];
        
        let result = collector.collect_by_topics(&topics, CollectionDepth::Quick).await;
        assert!(result.is_ok());
        
        let posts = result.unwrap();
        // Should collect some posts (mock data includes Indonesian fintech)
        assert!(!posts.is_empty());
        
        // Verify post structure
        if let Some(post) = posts.first() {
            assert_eq!(post.source, SocialSource::HackerNews);
            assert!(post.id.starts_with("hn_"));
        }
    }
    
    #[tokio::test]
    async fn test_topic_filtering() {
        let collector = HackerNewsCollector::new();
        let posts = vec![
            SocialPost {
                id: "hn_1".to_string(),
                source: SocialSource::HackerNews,
                title: "Indonesian Fintech Revolution".to_string(),
                content: "Discussion about fintech in Indonesia".to_string(),
                author: "user1".to_string(),
                url: "https://example.com".to_string(),
                published_at: Utc::now(),
                score: Some(100),
                comments_count: Some(20),
                metadata: HashMap::new(),
                indonesian_stocks: Vec::new(),
                embedding_id: None,
            },
            SocialPost {
                id: "hn_2".to_string(),
                source: SocialSource::HackerNews,
                title: "Random Tech News".to_string(),
                content: "Some other tech discussion".to_string(),
                author: "user2".to_string(),
                url: "https://example2.com".to_string(),
                published_at: Utc::now(),
                score: Some(50),
                comments_count: Some(5),
                metadata: HashMap::new(),
                indonesian_stocks: Vec::new(),
                embedding_id: None,
            },
        ];
        
        let topics = vec!["Indonesian".to_string()];
        let filtered = collector.filter_by_topics(posts, &topics);
        
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].title.contains("Indonesian"));
    }
}