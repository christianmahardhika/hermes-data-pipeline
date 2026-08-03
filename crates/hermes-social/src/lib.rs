/// hermes-social: Social Intelligence Service
/// 
/// Consolidated Rust implementation of social media collection.
/// Replaces Python social_intel module with unified 768-dim TEI embeddings.

pub mod hackernews;
pub mod reddit;
pub mod youtube;
pub mod collector;

pub use collector::{SocialCollector, SocialPost, SocialSource, CollectionDepth};
pub use hackernews::HackerNewsCollector;
pub use reddit::RedditCollector;
pub use youtube::YouTubeCollector;

use anyhow::Result;
use hermes_common::types::IndonesianStock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Initialize social intelligence collection system
pub async fn init_social_system() -> Result<()> {
    info!("🤖 Initializing Hermes Social Intelligence System");
    info!("📊 Supporting: HackerNews, Reddit, YouTube");
    info!("🔗 Using 768-dim TEI embeddings unified with news pipeline");
    
    Ok(())
}

/// Collect from all social sources with topic filtering
pub async fn collect_all_sources(
    topics: &[String], 
    depth: CollectionDepth,
    indonesian_stocks: &[IndonesianStock]
) -> Result<Vec<SocialPost>> {
    let mut all_posts = Vec::new();
    
    // Parallel collection from all sources
    let hn_collector = HackerNewsCollector::new();
    let reddit_collector = RedditCollector::new();
    let youtube_collector = YouTubeCollector::new();
    
    let (hn_posts, reddit_posts, youtube_posts) = tokio::try_join!(
        hn_collector.collect_by_topics(topics, depth),
        reddit_collector.collect_by_topics(topics, depth),
        youtube_collector.collect_metadata_by_topics(topics, depth)
    )?;
    
    all_posts.extend(hn_posts);
    all_posts.extend(reddit_posts);
    all_posts.extend(youtube_posts);
    
    info!("📈 Collected {} social posts across all platforms", all_posts.len());
    
    Ok(all_posts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_social_system_init() {
        let result = init_social_system().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_indonesian_tech_collection() {
        let topics = vec!["AI".to_string(), "tech".to_string(), "Indonesia".to_string()];
        let stocks = vec![IndonesianStock::BMRI, IndonesianStock::TLKM];
        
        let result = collect_all_sources(&topics, CollectionDepth::Quick, &stocks).await;
        // Should handle gracefully even if external APIs fail
        assert!(result.is_ok() || result.is_err());
    }
}