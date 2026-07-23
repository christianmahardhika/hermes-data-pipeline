//! RSS Feed Collector
//! 
//! Fetches RSS feeds with retry logic and stores raw XML in SQLite

use anyhow::Result;
use backoff::{ExponentialBackoff, future::retry};
use chrono::{DateTime, Utc};
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, warn, error};

use crate::storage::{Database, RawFeed};

/// Feed category for per-category stats and observability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedCategory {
    Indonesian,
    InternationalBusiness,
    InternationalGeneral,
    AsiaPacific,
    Market,
    Tech,
}

impl std::fmt::Display for FeedCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Indonesian => write!(f, "Indonesian"),
            Self::InternationalBusiness => write!(f, "Intl Business"),
            Self::InternationalGeneral => write!(f, "Intl General"),
            Self::AsiaPacific => write!(f, "Asia Pacific"),
            Self::Market => write!(f, "Market"),
            Self::Tech => write!(f, "Tech"),
        }
    }
}

/// RSS Feed configuration
#[derive(Debug, Clone)]
pub struct FeedConfig {
    pub name: String,
    pub url: String,
    /// Fallback URLs to try if primary fails
    pub fallback_urls: Vec<String>,
    /// Category for observability and stats
    pub category: FeedCategory,
}

/// Circuit breaker state for a feed
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Feed is healthy, proceed normally
    Closed,
    /// Feed is unhealthy, skip until specified time
    Open { until: DateTime<Utc>, backoff_secs: i64 },
    /// Skip period expired, try one probe request
    HalfOpen,
}

/// Maximum backoff duration: 6 hours
const MAX_BACKOFF_SECS: i64 = 21600;
/// Default initial backoff: 1 hour
const DEFAULT_BACKOFF_SECS: i64 = 3600;
/// Failure threshold to open circuit
const CIRCUIT_FAILURE_THRESHOLD: i32 = 5;

/// Determine circuit state for a feed based on database health records
pub fn determine_circuit_state(feed_name: &str, db: &Database) -> Result<CircuitState> {
    let state = db.get_circuit_state(feed_name)?;

    match state {
        None => Ok(CircuitState::Closed), // No health record = new feed
        Some((failures, circuit_open_until, backoff_secs)) => {
            if failures < CIRCUIT_FAILURE_THRESHOLD {
                return Ok(CircuitState::Closed);
            }

            match circuit_open_until {
                None => {
                    // First time hitting threshold — open circuit
                    let open_until = Utc::now() + chrono::Duration::seconds(DEFAULT_BACKOFF_SECS);
                    db.set_circuit_open(feed_name, &open_until.to_rfc3339(), DEFAULT_BACKOFF_SECS)?;
                    Ok(CircuitState::Open { until: open_until, backoff_secs: DEFAULT_BACKOFF_SECS })
                }
                Some(ref until_str) => {
                    let until = DateTime::parse_from_rfc3339(until_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now());

                    if Utc::now() >= until {
                        Ok(CircuitState::HalfOpen)
                    } else {
                        Ok(CircuitState::Open { until, backoff_secs })
                    }
                }
            }
        }
    }
}

/// Handle the result of a half-open probe
pub fn handle_half_open_result(feed_name: &str, success: bool, db: &Database) -> Result<()> {
    if success {
        db.reset_circuit(feed_name)?;
        info!("🔌 Circuit CLOSED for {}: probe succeeded", feed_name);
    } else {
        // Get current backoff and double it (capped)
        let current_backoff = db.get_circuit_state(feed_name)?
            .map(|(_, _, b)| b)
            .unwrap_or(DEFAULT_BACKOFF_SECS);
        let new_backoff = (current_backoff * 2).min(MAX_BACKOFF_SECS);
        let open_until = Utc::now() + chrono::Duration::seconds(new_backoff);
        db.set_circuit_open(feed_name, &open_until.to_rfc3339(), new_backoff)?;
        info!("🔌 Circuit OPEN for {}: probe failed, next retry in {}s", feed_name, new_backoff);
    }
    Ok(())
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
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .expect("Failed to create HTTP client");

        let feeds = vec![
            // === INDONESIAN NATIONAL ===
            FeedConfig { name: "Tempo".into(), url: "https://rss.tempo.co/".into(), fallback_urls: vec![], category: FeedCategory::Indonesian },
            FeedConfig { name: "CNN Indonesia".into(), url: "https://www.cnnindonesia.com/rss".into(), fallback_urls: vec![], category: FeedCategory::Indonesian },
            FeedConfig { name: "Antara News".into(), url: "https://www.antaranews.com/rss/terkini.xml".into(), fallback_urls: vec![], category: FeedCategory::Indonesian },
            FeedConfig { name: "Republika".into(), url: "https://www.republika.co.id/rss".into(), fallback_urls: vec![], category: FeedCategory::Indonesian },
            FeedConfig { name: "Detik News".into(), url: "https://news.detik.com/rss".into(), fallback_urls: vec!["https://finance.detik.com/rss".into()], category: FeedCategory::Indonesian },
            FeedConfig { name: "Detik Finance".into(), url: "https://finance.detik.com/rss".into(), fallback_urls: vec!["https://news.detik.com/rss".into()], category: FeedCategory::Indonesian },
            FeedConfig { name: "Kompas".into(), url: "https://rss.kompas.com/".into(), fallback_urls: vec![], category: FeedCategory::Indonesian },
            FeedConfig { name: "Okezone".into(), url: "https://sindikasi.okezone.com/index.php/rss/0/RSS2.0".into(), fallback_urls: vec![], category: FeedCategory::Indonesian },
            FeedConfig { name: "Sindonews".into(), url: "https://www.sindonews.com/rss".into(), fallback_urls: vec![], category: FeedCategory::Indonesian },
            FeedConfig { name: "Kontan".into(), url: "https://www.kontan.co.id/rss".into(), fallback_urls: vec![], category: FeedCategory::Indonesian },
            FeedConfig { name: "CNBC Indonesia".into(), url: "https://www.cnbcindonesia.com/rss".into(), fallback_urls: vec![], category: FeedCategory::Indonesian },
            FeedConfig { name: "Katadata".into(), url: "https://katadata.co.id/rss".into(), fallback_urls: vec![], category: FeedCategory::Indonesian },
            FeedConfig { name: "CNBC ID News".into(), url: "https://www.cnbcindonesia.com/news/rss".into(), fallback_urls: vec!["https://www.cnbcindonesia.com/rss".into()], category: FeedCategory::Indonesian },
            FeedConfig { name: "CNBC ID Entrepreneur".into(), url: "https://www.cnbcindonesia.com/entrepreneur/rss".into(), fallback_urls: vec!["https://www.cnbcindonesia.com/rss".into()], category: FeedCategory::Indonesian },
            
            // === MARKET ===
            FeedConfig { name: "CNBC ID Market".into(), url: "https://www.cnbcindonesia.com/market/rss".into(), fallback_urls: vec!["https://www.cnbcindonesia.com/rss".into()], category: FeedCategory::Market },
            FeedConfig { name: "Investing.com".into(), url: "https://www.investing.com/rss/news.rss".into(), fallback_urls: vec![], category: FeedCategory::Market },
            
            // === TECH ===
            FeedConfig { name: "CNBC ID Tech".into(), url: "https://www.cnbcindonesia.com/tech/rss".into(), fallback_urls: vec!["https://www.cnbcindonesia.com/rss".into()], category: FeedCategory::Tech },
            
            // === INTERNATIONAL - BUSINESS/FINANCE ===
            FeedConfig { name: "BBC Business".into(), url: "http://feeds.bbci.co.uk/news/business/rss.xml".into(), fallback_urls: vec![], category: FeedCategory::InternationalBusiness },
            FeedConfig { name: "BBC World".into(), url: "http://feeds.bbci.co.uk/news/world/rss.xml".into(), fallback_urls: vec![], category: FeedCategory::InternationalGeneral },
            FeedConfig { name: "CNBC".into(), url: "https://search.cnbc.com/rs/search/combinedcms/view.xml?partnerId=wrss01&id=100003114".into(), fallback_urls: vec![], category: FeedCategory::InternationalBusiness },
            FeedConfig { name: "Bloomberg".into(), url: "https://feeds.bloomberg.com/markets/news.rss".into(), fallback_urls: vec![], category: FeedCategory::InternationalBusiness },
            FeedConfig { name: "Financial Times".into(), url: "https://www.ft.com/rss/home".into(), fallback_urls: vec![], category: FeedCategory::InternationalBusiness },
            FeedConfig { name: "MarketWatch".into(), url: "https://feeds.marketwatch.com/marketwatch/topstories/".into(), fallback_urls: vec![], category: FeedCategory::InternationalBusiness },
            
            // === INTERNATIONAL - GENERAL ===
            FeedConfig { name: "Al Jazeera".into(), url: "https://www.aljazeera.com/xml/rss/all.xml".into(), fallback_urls: vec![], category: FeedCategory::InternationalGeneral },
            FeedConfig { name: "The Guardian".into(), url: "https://www.theguardian.com/world/rss".into(), fallback_urls: vec![], category: FeedCategory::InternationalGeneral },
            FeedConfig { name: "NPR".into(), url: "https://feeds.npr.org/1001/rss.xml".into(), fallback_urls: vec![], category: FeedCategory::InternationalGeneral },
            FeedConfig { name: "AP News".into(), url: "https://feedx.net/rss/ap.xml".into(), fallback_urls: vec![], category: FeedCategory::InternationalGeneral },
            
            // === ASIA PACIFIC ===
            FeedConfig { name: "Channel News Asia".into(), url: "https://www.channelnewsasia.com/api/v1/rss-outbound-feed?_format=xml".into(), fallback_urls: vec![], category: FeedCategory::AsiaPacific },
            FeedConfig { name: "Nikkei Asia".into(), url: "https://asia.nikkei.com/rss/feed/nar".into(), fallback_urls: vec![], category: FeedCategory::AsiaPacific },
            FeedConfig { name: "South China Morning Post".into(), url: "https://www.scmp.com/rss/91/feed".into(), fallback_urls: vec![], category: FeedCategory::AsiaPacific },
            FeedConfig { name: "Straits Times".into(), url: "https://www.straitstimes.com/news/asia/rss.xml".into(), fallback_urls: vec![], category: FeedCategory::AsiaPacific },
        ];

        Self { client, feeds }
    }

    /// Create with custom feeds
    pub fn with_feeds(feeds: Vec<FeedConfig>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .expect("Failed to create HTTP client");

        Self { client, feeds }
    }

    /// Fetch single feed with retry and failover to alternate URLs
    async fn fetch_feed(&self, feed: &FeedConfig) -> Result<Vec<u8>> {
        // Try primary URL first
        match self.fetch_url(&feed.url).await {
            Ok(bytes) => return Ok(bytes),
            Err(e) => {
                if feed.fallback_urls.is_empty() {
                    return Err(e);
                }
                warn!("⚠️ Primary URL failed for {}, trying fallbacks: {}", feed.name, e);
            }
        }

        // Try fallback URLs in order
        for fallback_url in &feed.fallback_urls {
            match self.fetch_url(fallback_url).await {
                Ok(bytes) => {
                    info!("✅ Fallback succeeded for {} via {}", feed.name, fallback_url);
                    return Ok(bytes);
                }
                Err(e) => {
                    warn!("⚠️ Fallback also failed for {}: {}", feed.name, e);
                }
            }
        }

        Err(anyhow::anyhow!("All URLs failed for feed: {}", feed.name))
    }

    /// Fetch a single URL with exponential backoff retry
    async fn fetch_url(&self, url: &str) -> Result<Vec<u8>> {
        let backoff = ExponentialBackoff {
            max_elapsed_time: Some(Duration::from_secs(60)),
            ..Default::default()
        };

        let url = url.to_string();
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
            let cat_stats = stats.per_category.entry(feed.category).or_default();

            // Circuit breaker check
            let circuit = determine_circuit_state(&feed.name, db)?;
            match &circuit {
                CircuitState::Open { until, .. } => {
                    stats.skipped += 1;
                    cat_stats.skipped += 1;
                    info!("⏭️ Skipping {} (circuit OPEN until {})", feed.name, until.format("%H:%M:%S"));
                    continue;
                }
                CircuitState::HalfOpen => {
                    info!("🔌 Circuit HALF-OPEN for {}: probing...", feed.name);
                    // Fall through to fetch — single attempt
                }
                CircuitState::Closed => {
                    // Normal fetch path
                }
            }

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
                            if circuit == CircuitState::HalfOpen {
                                handle_half_open_result(&feed.name, true, db)?;
                            }
                            stats.success += 1;
                            cat_stats.success += 1;
                            info!("✅ Fetched: {}", feed.name);
                        }
                        Err(e) => {
                            stats.errors += 1;
                            cat_stats.errors += 1;
                            error!("❌ DB insert failed for {}: {}", feed.name, e);
                        }
                    }
                }
                Err(e) => {
                    db.record_feed_failure(&feed.name, &e.to_string())?;

                    if circuit == CircuitState::HalfOpen {
                        handle_half_open_result(&feed.name, false, db)?;
                    }

                    stats.errors += 1;
                    cat_stats.errors += 1;
                    warn!("⚠️ Failed to fetch {}: {}", feed.name, e);
                }
            }
        }

        Ok(stats)
    }
}

/// Per-category collection statistics
#[derive(Debug, Default, Clone)]
pub struct CategoryStats {
    pub success: usize,
    pub errors: usize,
    pub skipped: usize,
}

/// Collection statistics
#[derive(Debug, Default)]
pub struct CollectStats {
    pub success: usize,
    pub errors: usize,
    pub skipped: usize,
    pub per_category: HashMap<FeedCategory, CategoryStats>,
}

impl std::fmt::Display for CollectStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Collected: {} success, {} errors, {} skipped", self.success, self.errors, self.skipped)?;
        if !self.per_category.is_empty() {
            write!(f, " [")?;
            let mut first = true;
            for (cat, stats) in &self.per_category {
                if !first { write!(f, " | ")?; }
                write!(f, "{}: {}/{}", cat, stats.success, stats.success + stats.errors + stats.skipped)?;
                first = false;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Database;

    fn setup_test_db() -> Database {
        Database::open(":memory:").expect("Failed to create in-memory DB")
    }

    #[test]
    fn test_circuit_state_closed_for_new_feed() {
        let db = setup_test_db();
        let state = determine_circuit_state("unknown_feed", &db).unwrap();
        assert_eq!(state, CircuitState::Closed);
    }

    #[test]
    fn test_circuit_state_closed_below_threshold() {
        let db = setup_test_db();
        // Simulate 4 failures (below threshold of 5)
        for _ in 0..4 {
            db.record_feed_failure("test_feed", "timeout").unwrap();
        }
        let state = determine_circuit_state("test_feed", &db).unwrap();
        assert_eq!(state, CircuitState::Closed);
    }

    #[test]
    fn test_circuit_state_opens_at_threshold() {
        let db = setup_test_db();
        // Simulate 5 failures (at threshold)
        for _ in 0..5 {
            db.record_feed_failure("test_feed", "timeout").unwrap();
        }
        let state = determine_circuit_state("test_feed", &db).unwrap();
        match state {
            CircuitState::Open { backoff_secs, .. } => {
                assert_eq!(backoff_secs, DEFAULT_BACKOFF_SECS);
            }
            _ => panic!("Expected CircuitState::Open, got {:?}", state),
        }
    }

    #[test]
    fn test_circuit_half_open_after_backoff_expires() {
        let db = setup_test_db();
        // Simulate failures
        for _ in 0..5 {
            db.record_feed_failure("test_feed", "timeout").unwrap();
        }
        // Set circuit open until the past (expired)
        let past = (Utc::now() - chrono::Duration::seconds(10)).to_rfc3339();
        db.set_circuit_open("test_feed", &past, 3600).unwrap();

        let state = determine_circuit_state("test_feed", &db).unwrap();
        assert_eq!(state, CircuitState::HalfOpen);
    }

    #[test]
    fn test_handle_half_open_success_resets_circuit() {
        let db = setup_test_db();
        for _ in 0..5 {
            db.record_feed_failure("test_feed", "timeout").unwrap();
        }
        let past = (Utc::now() - chrono::Duration::seconds(10)).to_rfc3339();
        db.set_circuit_open("test_feed", &past, 3600).unwrap();

        handle_half_open_result("test_feed", true, &db).unwrap();

        let state = determine_circuit_state("test_feed", &db).unwrap();
        assert_eq!(state, CircuitState::Closed);
    }

    #[test]
    fn test_handle_half_open_failure_doubles_backoff() {
        let db = setup_test_db();
        for _ in 0..5 {
            db.record_feed_failure("test_feed", "timeout").unwrap();
        }
        let past = (Utc::now() - chrono::Duration::seconds(10)).to_rfc3339();
        db.set_circuit_open("test_feed", &past, 3600).unwrap();

        handle_half_open_result("test_feed", false, &db).unwrap();

        let state = determine_circuit_state("test_feed", &db).unwrap();
        match state {
            CircuitState::Open { backoff_secs, .. } => {
                assert_eq!(backoff_secs, 7200); // doubled from 3600
            }
            _ => panic!("Expected CircuitState::Open, got {:?}", state),
        }
    }

    #[test]
    fn test_backoff_caps_at_max() {
        let db = setup_test_db();
        for _ in 0..5 {
            db.record_feed_failure("test_feed", "timeout").unwrap();
        }
        // Set backoff already at 14400 (just below max/2)
        let past = (Utc::now() - chrono::Duration::seconds(10)).to_rfc3339();
        db.set_circuit_open("test_feed", &past, 14400).unwrap();

        handle_half_open_result("test_feed", false, &db).unwrap();

        let state = determine_circuit_state("test_feed", &db).unwrap();
        match state {
            CircuitState::Open { backoff_secs, .. } => {
                assert_eq!(backoff_secs, MAX_BACKOFF_SECS); // capped at 21600
            }
            _ => panic!("Expected CircuitState::Open, got {:?}", state),
        }
    }

    #[test]
    fn test_feed_category_display() {
        assert_eq!(format!("{}", FeedCategory::Indonesian), "Indonesian");
        assert_eq!(format!("{}", FeedCategory::InternationalBusiness), "Intl Business");
        assert_eq!(format!("{}", FeedCategory::InternationalGeneral), "Intl General");
        assert_eq!(format!("{}", FeedCategory::AsiaPacific), "Asia Pacific");
        assert_eq!(format!("{}", FeedCategory::Market), "Market");
        assert_eq!(format!("{}", FeedCategory::Tech), "Tech");
    }

    #[test]
    fn test_collect_stats_display() {
        let stats = CollectStats {
            success: 20,
            errors: 3,
            skipped: 2,
            per_category: HashMap::new(),
        };
        let display = format!("{}", stats);
        assert!(display.contains("20 success"));
        assert!(display.contains("3 errors"));
        assert!(display.contains("2 skipped"));
    }
}
