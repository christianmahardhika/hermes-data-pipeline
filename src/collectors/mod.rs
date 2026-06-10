//! RSS Feed Collector
//! 
//! Fetches RSS feeds with retry logic and stores raw XML in SQLite

use anyhow::Result;
use backoff::{ExponentialBackoff, future::retry};
use chrono::Utc;
use reqwest::Client;
use std::time::Duration;
use tracing::{info, warn, error};

use crate::storage::{Database, RawFeed};

/// RSS Feed configuration
#[derive(Debug, Clone)]
pub struct FeedConfig {
    pub name: String,
    pub url: String,
}

/// RSS Collector with retry logic
pub struct RssCollector {
    client: Client,
    feeds: Vec<FeedConfig>,
}

impl RssCollector {
    /// Create new collector with default Indonesian feeds
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; NewsCollector/1.0)")
            .build()
            .expect("Failed to create HTTP client");

        let feeds = vec![
            // === INDONESIAN NATIONAL ===
            FeedConfig { name: "Tempo".into(), url: "https://rss.tempo.co/".into() },
            FeedConfig { name: "CNN Indonesia".into(), url: "https://www.cnnindonesia.com/rss".into() },
            FeedConfig { name: "Antara News".into(), url: "https://www.antaranews.com/rss/terkini.xml".into() },
            FeedConfig { name: "Republika".into(), url: "https://www.republika.co.id/rss".into() },
            FeedConfig { name: "Merdeka".into(), url: "https://www.merdeka.com/feed/".into() },
            FeedConfig { name: "Tribunnews".into(), url: "https://www.tribunnews.com/rss".into() },
            FeedConfig { name: "Detik".into(), url: "https://rss.detik.com/index.php/detikcom".into() },
            FeedConfig { name: "Kompas".into(), url: "https://rss.kompas.com/".into() },
            FeedConfig { name: "Liputan6".into(), url: "https://www.liputan6.com/rss".into() },
            FeedConfig { name: "Okezone".into(), url: "https://sindikasi.okezone.com/index.php/rss/0/RSS2.0".into() },
            FeedConfig { name: "Sindonews".into(), url: "https://www.sindonews.com/rss".into() },
            FeedConfig { name: "Bisnis Indonesia".into(), url: "https://www.bisnis.com/rss".into() },
            FeedConfig { name: "Kontan".into(), url: "https://www.kontan.co.id/rss".into() },
            FeedConfig { name: "CNBC Indonesia".into(), url: "https://www.cnbcindonesia.com/rss".into() },
            FeedConfig { name: "IDN Times".into(), url: "https://www.idntimes.com/rss".into() },
            
            // === INTERNATIONAL - BUSINESS/FINANCE ===
            FeedConfig { name: "BBC Business".into(), url: "http://feeds.bbci.co.uk/news/business/rss.xml".into() },
            FeedConfig { name: "BBC World".into(), url: "http://feeds.bbci.co.uk/news/world/rss.xml".into() },
            FeedConfig { name: "CNBC".into(), url: "https://www.cnbc.com/id/100003114/device/rss/rss.html".into() },
            FeedConfig { name: "Bloomberg".into(), url: "https://feeds.bloomberg.com/markets/news.rss".into() },
            FeedConfig { name: "Financial Times".into(), url: "https://www.ft.com/rss/home".into() },
            FeedConfig { name: "MarketWatch".into(), url: "https://feeds.marketwatch.com/marketwatch/topstories/".into() },
            
            // === INTERNATIONAL - GENERAL ===
            FeedConfig { name: "Al Jazeera".into(), url: "https://www.aljazeera.com/xml/rss/all.xml".into() },
            FeedConfig { name: "The Guardian".into(), url: "https://www.theguardian.com/world/rss".into() },
            FeedConfig { name: "NPR".into(), url: "https://feeds.npr.org/1001/rss.xml".into() },
            FeedConfig { name: "AP News".into(), url: "https://rsshub.app/apnews/topics/apf-topnews".into() },
            
            // === ASIA PACIFIC ===
            FeedConfig { name: "Channel News Asia".into(), url: "https://www.channelnewsasia.com/rss/latest_news".into() },
            FeedConfig { name: "Nikkei Asia".into(), url: "https://asia.nikkei.com/rss/feed/nar".into() },
            FeedConfig { name: "South China Morning Post".into(), url: "https://www.scmp.com/rss/91/feed".into() },
            FeedConfig { name: "Straits Times".into(), url: "https://www.straitstimes.com/news/asia/rss.xml".into() },
        ];

        Self { client, feeds }
    }

    /// Create with custom feeds
    pub fn with_feeds(feeds: Vec<FeedConfig>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; NewsCollector/1.0)")
            .build()
            .expect("Failed to create HTTP client");

        Self { client, feeds }
    }

    /// Fetch single feed with retry
    async fn fetch_feed(&self, feed: &FeedConfig) -> Result<Vec<u8>> {
        let backoff = ExponentialBackoff {
            max_elapsed_time: Some(Duration::from_secs(60)),
            ..Default::default()
        };

        let url = feed.url.clone();
        let client = self.client.clone();

        let result = retry(backoff, || async {
            let response = client
                .get(&url)
                .send()
                .await
                .map_err(|e| backoff::Error::transient(anyhow::anyhow!(e)))?;

            if response.status().is_success() {
                let bytes = response
                    .bytes()
                    .await
                    .map_err(|e| backoff::Error::transient(anyhow::anyhow!(e)))?;
                Ok(bytes.to_vec())
            } else if response.status().is_server_error() {
                Err(backoff::Error::transient(anyhow::anyhow!(
                    "Server error: {}",
                    response.status()
                )))
            } else {
                Err(backoff::Error::permanent(anyhow::anyhow!(
                    "Client error: {}",
                    response.status()
                )))
            }
        })
        .await?;

        Ok(result)
    }

    /// Collect all feeds and store in database
    pub async fn collect_all(&self, db: &Database) -> Result<CollectStats> {
        let mut stats = CollectStats::default();

        for feed in &self.feeds {
            match self.fetch_feed(feed).await {
                Ok(raw_content) => {
                    let raw_feed = RawFeed {
                        id: None,
                        feed_name: feed.name.clone(),
                        raw_content,
                        content_type: "xml".to_string(),
                        fetched_at: Utc::now(),
                        status: "pending".to_string(),
                        retry_count: 0,
                    };

                    match db.insert_raw(&raw_feed) {
                        Ok(_) => {
                            db.record_feed_success(&feed.name)?;
                            stats.success += 1;
                            info!("✅ Fetched: {}", feed.name);
                        }
                        Err(e) => {
                            stats.errors += 1;
                            error!("❌ DB insert failed for {}: {}", feed.name, e);
                        }
                    }
                }
                Err(e) => {
                    db.record_feed_failure(&feed.name, &e.to_string())?;
                    stats.errors += 1;
                    warn!("⚠️ Failed to fetch {}: {}", feed.name, e);
                }
            }
        }

        Ok(stats)
    }
}

/// Collection statistics
#[derive(Debug, Default)]
pub struct CollectStats {
    pub success: usize,
    pub errors: usize,
}

impl std::fmt::Display for CollectStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Collected: {} success, {} errors", self.success, self.errors)
    }
}
