/// YouTube collector implementation
/// Collects video metadata, tech channel content, and Indonesian market discussions
use crate::collector::{SocialCollector, SocialPost, SocialSource, CollectionDepth};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, debug};

/// YouTube video metadata
#[derive(Debug, Deserialize)]
struct YouTubeVideo {
    id: String,
    title: String,
    description: String,
    channel_title: String,
    published_at: String,
    view_count: Option<u64>,
    like_count: Option<u32>,
    comment_count: Option<u32>,
    duration: Option<String>,
}

/// YouTube search response
#[derive(Debug, Deserialize)]
struct YouTubeSearchResponse {
    items: Vec<YouTubeSearchItem>,
}

#[derive(Debug, Deserialize)]
struct YouTubeSearchItem {
    id: YouTubeVideoId,
    snippet: YouTubeSnippet,
}

#[derive(Debug, Deserialize)]
struct YouTubeVideoId {
    #[serde(rename = "videoId")]
    video_id: String,
}

#[derive(Debug, Deserialize)]
struct YouTubeSnippet {
    title: String,
    description: String,
    #[serde(rename = "channelTitle")]
    channel_title: String,
    #[serde(rename = "publishedAt")]
    published_at: String,
}

/// YouTube collector with mock implementation
pub struct YouTubeCollector {
    tech_channels: Vec<String>,
    indonesian_channels: Vec<String>,
}

impl YouTubeCollector {
    /// Create new YouTube collector
    pub fn new() -> Self {
        Self {
            tech_channels: vec![
                "TechCrunch".to_string(),
                "Lex Fridman".to_string(),
                "Two Minute Papers".to_string(),
                "Y Combinator".to_string(),
            ],
            indonesian_channels: vec![
                "Tech in Asia Indonesia".to_string(),
                "DailySocial".to_string(),
                "StartupPedia".to_string(),
                "Indonesian Tech Review".to_string(),
            ],
        }
    }
    
    /// Generate mock YouTube posts for testing
    async fn get_mock_videos(&self, query: &str, limit: usize) -> Result<Vec<SocialPost>> {
        debug!("🎥 Fetching mock videos for query: {}", query);
        
        let mock_videos = vec![
            SocialPost {
                id: format!("youtube_{}", 1),
                source: SocialSource::YouTube,
                title: "Indonesian FinTech Revolution: BMRI Digital Banking".to_string(),
                content: "Deep dive into Bank Mandiri's digital transformation and the impact on Indonesian banking sector. Analysis of mobile banking adoption and fintech partnerships.".to_string(),
                author: "TechCrunch".to_string(),
                url: "https://youtube.com/watch?v=example1".to_string(),
                published_at: Utc::now(),
                score: Some(1250), // View count in thousands
                comments_count: Some(89),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("duration".to_string(), "12:34".to_string());
                    map.insert("platform".to_string(), "youtube".to_string());
                    map.insert("video_type".to_string(), "analysis".to_string());
                    map
                },
                indonesian_stocks: Vec::new(),
                embedding_id: None,
            },
            SocialPost {
                id: format!("youtube_{}", 2),
                source: SocialSource::YouTube,
                title: "Mining Tech Innovation: ANTM & INCO Sustainability".to_string(),
                content: "Exploring how Indonesian mining companies like Antam and Vale Indonesia are adopting green technology and sustainable mining practices. Impact on global nickel supply chain.".to_string(),
                author: "Tech in Asia Indonesia".to_string(),
                url: "https://youtube.com/watch?v=example2".to_string(),
                published_at: Utc::now(),
                score: Some(567), // View count
                comments_count: Some(34),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("duration".to_string(), "8:45".to_string());
                    map.insert("platform".to_string(), "youtube".to_string());
                    map.insert("video_type".to_string(), "documentary".to_string());
                    map
                },
                indonesian_stocks: Vec::new(),
                embedding_id: None,
            },
            SocialPost {
                id: format!("youtube_{}", 3),
                source: SocialSource::YouTube,
                title: "AI Startups in Southeast Asia: Indonesian Tech Boom".to_string(),
                content: "Interview with Indonesian AI startup founders discussing machine learning applications in agriculture (TAPG sector) and telecommunications (TLKM partnerships).".to_string(),
                author: "Y Combinator".to_string(),
                url: "https://youtube.com/watch?v=example3".to_string(),
                published_at: Utc::now(),
                score: Some(2100),
                comments_count: Some(156),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("duration".to_string(), "25:12".to_string());
                    map.insert("platform".to_string(), "youtube".to_string());
                    map.insert("video_type".to_string(), "interview".to_string());
                    map
                },
                indonesian_stocks: Vec::new(),
                embedding_id: None,
            },
        ];
        
        Ok(mock_videos.into_iter()
            .filter(|video| {
                let combined = format!("{} {}", video.title, video.content).to_lowercase();
                combined.contains(&query.to_lowercase())
            })
            .take(limit)
            .collect())
    }
    
    /// Filter videos by topics
    fn filter_by_topics(&self, posts: Vec<SocialPost>, topics: &[String]) -> Vec<SocialPost> {
        if topics.is_empty() {
            return posts;
        }
        
        posts.into_iter()
            .filter(|post| {
                let combined_text = format!("{} {} {}", post.title, post.content, post.author).to_lowercase();
                topics.iter().any(|topic| combined_text.contains(&topic.to_lowercase()))
            })
            .collect()
    }
    
    /// Collect metadata-only (no video download)
    pub async fn collect_metadata_by_topics(&self, topics: &[String], depth: CollectionDepth) -> Result<Vec<SocialPost>> {
        self.collect_by_topics(topics, depth).await
    }
}

#[async_trait]
impl SocialCollector for YouTubeCollector {
    async fn collect_by_topics(&self, topics: &[String], depth: CollectionDepth) -> Result<Vec<SocialPost>> {
        info!("📺 Collecting YouTube videos for topics: {:?} (depth: {:?})", topics, depth);
        
        let limit = match depth {
            CollectionDepth::Quick => 5,
            CollectionDepth::Medium => 15,
            CollectionDepth::Deep => 30,
        };
        
        let mut all_posts = Vec::new();
        
        // Search by topics
        for topic in topics {
            match self.get_mock_videos(topic, limit / topics.len().max(1)).await {
                Ok(videos) => {
                    debug!("✅ Collected {} videos for topic: {}", videos.len(), topic);
                    all_posts.extend(videos);
                },
                Err(e) => {
                    warn!("❌ Failed to collect videos for topic '{}': {}", topic, e);
                }
            }
        }
        
        // If no topics, search general Indonesian tech content
        if topics.is_empty() {
            let general_videos = self.get_mock_videos("indonesian tech", limit).await?;
            all_posts.extend(general_videos);
        }
        
        let filtered_posts = self.filter_by_topics(all_posts, topics);
        
        info!("✅ YouTube collection complete: {} videos", filtered_posts.len());
        Ok(filtered_posts)
    }
    
    fn source(&self) -> SocialSource {
        SocialSource::YouTube
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
    async fn test_youtube_collector_creation() {
        let collector = YouTubeCollector::new();
        assert_eq!(collector.source(), SocialSource::YouTube);
        assert!(!collector.tech_channels.is_empty());
        assert!(!collector.indonesian_channels.is_empty());
    }
    
    #[tokio::test]
    async fn test_youtube_health_check() {
        let collector = YouTubeCollector::new();
        let result = collector.health_check().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
    
    #[tokio::test]
    async fn test_collect_indonesian_tech_videos() {
        let collector = YouTubeCollector::new();
        let topics = vec!["Indonesian".to_string(), "fintech".to_string()];
        
        let result = collector.collect_by_topics(&topics, CollectionDepth::Quick).await;
        assert!(result.is_ok());
        
        let posts = result.unwrap();
        assert!(!posts.is_empty());
        
        // Verify all posts are YouTube source
        for post in &posts {
            assert_eq!(post.source, SocialSource::YouTube);
            assert!(post.url.contains("youtube.com"));
        }
    }
    
    #[tokio::test]
    async fn test_metadata_collection() {
        let collector = YouTubeCollector::new();
        let topics = vec!["AI".to_string()];
        
        let result = collector.collect_metadata_by_topics(&topics, CollectionDepth::Medium).await;
        assert!(result.is_ok());
        
        let posts = result.unwrap();
        for post in &posts {
            // Verify metadata exists
            assert!(post.metadata.contains_key("duration"));
            assert!(post.metadata.contains_key("platform"));
            assert_eq!(post.metadata.get("platform").unwrap(), "youtube");
        }
    }
}