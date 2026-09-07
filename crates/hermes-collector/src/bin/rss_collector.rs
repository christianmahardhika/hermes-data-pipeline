//! RSS collector binary: fetch real feeds, store to ArangoDB via StorageClient.
//!
//! Usage:
//!   cargo run -p hermes-collector --bin rss_collector
//! Env: ARANGO_URL, ARANGO_DATABASE, ARANGO_USERNAME, ARANGO_PASSWORD

use hermes_collector::{HermesCollector, ReqwestHttpClient, FeedSource, FeedCategory};
use news_intelligence::StorageClient;
use std::env;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

fn default_sources() -> Vec<FeedSource> {
    vec![
        // Indonesian
        FeedSource::new("detik".to_string(), "https://rss.detik.com/index.php/detikcom".to_string(), FeedCategory::IndonesianNews, true),
        FeedSource::new("antara".to_string(), "https://www.antaranews.com/rss/terkini".to_string(), FeedCategory::IndonesianNews, true),
        FeedSource::new("tempo".to_string(), "https://rss.tempo.co/nasional".to_string(), FeedCategory::IndonesianNews, false),
        FeedSource::new("cnn_indonesia".to_string(), "https://www.cnnindonesia.com/rss".to_string(), FeedCategory::IndonesianNews, false),
        // International (verified live)
        FeedSource::new("bbc_world".to_string(), "https://feeds.bbci.co.uk/news/world/rss.xml".to_string(), FeedCategory::InternationalNews, true),
        FeedSource::new("aljazeera".to_string(), "https://www.aljazeera.com/xml/rss/all.xml".to_string(), FeedCategory::InternationalNews, true),
        FeedSource::new("nyt_world".to_string(), "https://rss.nytimes.com/services/xml/rss/nyt/HomePage.xml".to_string(), FeedCategory::InternationalNews, false),
        FeedSource::new("guardian_world".to_string(), "https://www.theguardian.com/world/rss".to_string(), FeedCategory::InternationalNews, false),
        // Tech
        FeedSource::new("hacker_news".to_string(), "https://hnrss.org/frontpage".to_string(), FeedCategory::Technology, true),
        FeedSource::new("hackernews_thn".to_string(), "https://feeds.feedburner.com/TheHackersNews".to_string(), FeedCategory::Technology, false),
        FeedSource::new("ars_technica".to_string(), "https://feeds.arstechnica.com/arstechnica/index".to_string(), FeedCategory::Technology, false),
    ]
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🚀 Starting Rust Native RSS Collector");

    let storage = StorageClient::from_env()?;
    storage.ensure_collection("articles").await?;

    let client = ReqwestHttpClient::new()?;
    let sources = default_sources();
    let mut collector = HermesCollector::new(client).with_sources(sources);

    let stats = collector.collect_all_feeds_async(&storage).await?;

    info!("✅ RSS Collection complete: {:?}", stats);
    Ok(())
}
