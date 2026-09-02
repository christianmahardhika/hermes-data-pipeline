//! RSS collector binary: fetch real feeds, store to ArangoDB via StorageClient.
//!
//! Usage:
//!   cargo run -p hermes-collector --bin rss_collector
//! Env: ARANGO_URL, ARANGO_DATABASE, ARANGO_USERNAME, ARANGO_PASSWORD

use hermes_collector::{FeedSource, FeedCategory, HermesCollector, rss_fetcher::ReqwestHttpClient};

fn default_sources() -> Vec<FeedSource> {
    vec![
        FeedSource::new("detik".to_string(), "https://rss.detik.com/index.php/detikcom".to_string(), FeedCategory::IndonesianNews, true),
        FeedSource::new("antara".to_string(), "https://www.antaranews.com/rss/terkini".to_string(), FeedCategory::IndonesianNews, true),
        FeedSource::new("kompas".to_string(), "https://www.kompas.com/rss".to_string(), FeedCategory::IndonesianNews, true),
        FeedSource::new("tempo".to_string(), "https://www.tempo.co/rss".to_string(), FeedCategory::IndonesianNews, false),
        FeedSource::new("cnn_indonesia".to_string(), "https://www.cnnindonesia.com/rss".to_string(), FeedCategory::IndonesianNews, false),
        FeedSource::new("reuters_business".to_string(), "https://feeds.reuters.com/reuters/businessNews".to_string(), FeedCategory::InternationalNews, true),
        FeedSource::new("hacker_news".to_string(), "https://hnrss.org/frontpage".to_string(), FeedCategory::Technology, false),
    ]
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let storage = news_intelligence::StorageClient::from_env()?;
    let http = ReqwestHttpClient::new()?;

    let sources = default_sources();
    let mut collector = HermesCollector::new(http).with_sources(sources);

    let stats = collector.collect_all_feeds_async(&storage).await?;

    println!(
        "✅ Collection done: {} sources, {} articles, {} failed, {} ms",
        stats.total_sources,
        stats.articles_collected,
        stats.failed_collections,
        stats.collection_duration_ms
    );

    Ok(())
}
