/// Reddit collector implementation
/// Collects tech discussions, Indonesian mentions, and market sentiment from Reddit
use crate::collector::{SocialCollector, SocialPost, SocialSource, CollectionDepth};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, debug};

/// Reddit post structure
#[derive(Debug, Deserialize)]
struct RedditPost {
    title: String,
    selftext: Option<String>,
    author: String,
    url: String,
    permalink: String,
    created_utc: f64,
    score: i32,
    num_comments: i32,
    subreddit: String,
}

/// Reddit listing response
#[derive(Debug, Deserialize)]
struct RedditListing {
    data: RedditListingData,
}

#[derive(Debug, Deserialize)]
struct RedditListingData {
    children: Vec<RedditChild>,
}

#[derive(Debug, Deserialize)]
struct RedditChild {
    data: RedditPost,
}

/// Reddit collector with mock implementation
pub struct RedditCollector {
    subreddits: Vec<String>,
}

impl RedditCollector {
    /// Create new Reddit collector
    pub fn new() -> Self {
        Self {
            subreddits: vec![
                "technology".to_string(),
                "programming".to_string(),
                "investing".to_string(),
                "indonesia".to_string(),
                "startups".to_string(),
            ],
        }
    }
    
    /// Generate mock Reddit posts for testing
    async fn get_mock_posts(&self, subreddit: &str, limit: usize) -> Result<Vec<SocialPost>> {
        debug!("🔍 Fetching mock posts from r/{}", subreddit);
        
        let mock_posts = vec![
            SocialPost {
                id: format!("reddit_{}_{}", subreddit, 1),
                source: SocialSource::Reddit,
                title: format!("Indonesian Fintech Growth in {}", subreddit),
                content: "Discussion about Indonesian fintech companies like BMRI digital transformation".to_string(),
                author: "tech_analyst".to_string(),
                url: format!("https://reddit.com/r/{}/comments/example1", subreddit),
                published_at: Utc::now(),
                score: Some(150),
                comments_count: Some(45),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("subreddit".to_string(), subreddit.to_string());
                    map.insert("platform".to_string(), "reddit".to_string());
                    map
                },
                indonesian_stocks: Vec::new(),
                embedding_id: None,
            },
            SocialPost {
                id: format!("reddit_{}_{}", subreddit, 2),
                source: SocialSource::Reddit,
                title: format!("Tech Investment Trends in {}", subreddit),
                content: "Analysis of ANTM mining sector performance and market outlook".to_string(),
                author: "market_watcher".to_string(),
                url: format!("https://reddit.com/r/{}/comments/example2", subreddit),
                published_at: Utc::now(),
                score: Some(89),
                comments_count: Some(23),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("subreddit".to_string(), subreddit.to_string());
                    map.insert("platform".to_string(), "reddit".to_string());
                    map
                },
                indonesian_stocks: Vec::new(),
                embedding_id: None,
            },
        ];
        
        Ok(mock_posts.into_iter().take(limit).collect())
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
impl SocialCollector for RedditCollector {
    async fn collect_by_topics(&self, topics: &[String], depth: CollectionDepth) -> Result<Vec<SocialPost>> {
        info!("🤖 Collecting Reddit posts for topics: {:?} (depth: {:?})", topics, depth);
        
        let (limit_per_sub, subreddit_count) = match depth {
            CollectionDepth::Quick => (5, 2),
            CollectionDepth::Medium => (15, 3),
            CollectionDepth::Deep => (25, 5),
        };
        
        let mut all_posts = Vec::new();
        
        for subreddit in self.subreddits.iter().take(subreddit_count) {
            match self.get_mock_posts(subreddit, limit_per_sub).await {
                Ok(posts) => {
                    debug!("✅ Collected {} posts from r/{}", posts.len(), subreddit);
                    all_posts.extend(posts);
                },
                Err(e) => {
                    warn!("❌ Failed to collect from r/{}: {}", subreddit, e);
                }
            }
        }
        
        let filtered_posts = self.filter_by_topics(all_posts, topics);
        
        info!("✅ Reddit collection complete: {} posts", filtered_posts.len());
        Ok(filtered_posts)
    }
    
    fn source(&self) -> SocialSource {
        SocialSource::Reddit
    }
    
    async fn health_check(&self) -> Result<bool> {
        // Mock health check - always healthy
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_reddit_collector_creation() {
        let collector = RedditCollector::new();
        assert_eq!(collector.source(), SocialSource::Reddit);
        assert!(!collector.subreddits.is_empty());
    }
    
    #[tokio::test]
    async fn test_reddit_health_check() {
        let collector = RedditCollector::new();
        let result = collector.health_check().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
    
    #[tokio::test]
    async fn test_collect_indonesian_posts() {
        let collector = RedditCollector::new();
        let topics = vec!["Indonesian".to_string(), "BMRI".to_string()];
        
        let result = collector.collect_by_topics(&topics, CollectionDepth::Quick).await;
        assert!(result.is_ok());
        
        let posts = result.unwrap();
        assert!(!posts.is_empty());
        
        // Verify Indonesian content filtering
        for post in &posts {
            let combined = format!("{} {}", post.title, post.content).to_lowercase();
            assert!(combined.contains("indonesian") || combined.contains("bmri"));
        }
    }
}