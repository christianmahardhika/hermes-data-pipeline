//! RSS collector binary: fetch real feeds, store to ArangoDB via StorageClient.
//!
//! Usage:
//!   cargo run -p hermes-collector --bin rss_collector
//! Env: ARANGO_URL, ARANGO_DATABASE, ARANGO_USERNAME, ARANGO_PASSWORD

use hermes_collector::{RssFetcher, ReqwestHttpClient, FeedSource, FeedCategory};
use news_intelligence::StorageClient;
use std::env;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

fn default_sources() -> Vec<FeedSource> {
    vec![
        // Indonesian
        FeedSource::new("detik".to_string(), "https://rss.detik.com/index.php/detikcom".to_string(), FeedCategory::IndonesianNews, true),
        FeedSource::new("antara".to_string(), "https://www.antaranews.com/rss/terkini".to_string(), FeedCategory::IndonesianNews, true),
        FeedSource::new("kompas".to_string(), "https://www.kompas.com/rss".to_string(), FeedCategory::IndonesianNews, true),
        FeedSource::new("tempo".to_string(), "https://www.tempo.co/rss".to_string(), FeedCategory::IndonesianNews, false),
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

    let url = env::var("ARANGO_URL").unwrap_or_else(|_| "http://localhost:8529".to_string());
    let db = env::var("ARANGO_DATABASE").unwrap_or_else(|_| "news_analysis".to_string());
    let user = env::var("ARANGO_USERNAME").unwrap_or_else(|_| "root".to_string());
    let pass = env::var("ARANGO_PASSWORD").unwrap_or_else(|_| "".to_string());

    let storage = StorageClient::new(&url, &db, &user, &pass)?;
    storage.ensure_collection("articles").await?;

    let client = ReqwestHttpClient::new();
    let mut fetcher = RssFetcher::new(client);
    let sources = default_sources();

    let stats = fetcher.collect_all_feeds_async(&sources, &storage).await?;

    info!("✅ RSS Collection complete: {:?}", stats);
    Ok(())
}
