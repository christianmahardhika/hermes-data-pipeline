//! RSS fetcher with trait-based HTTP client abstraction

use crate::feed_source::FeedSource;
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use rss::{Channel, Item};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, warn};

/// Trait for HTTP client abstraction (avoid OpenSSL dependency)
#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn get(&self, url: &str) -> Result<String>;
}

/// Mock HTTP client for testing
#[derive(Debug, Clone)]
pub struct MockHttpClient {
    pub responses: HashMap<String, Result<String, String>>,
}

/// Real HTTP client using reqwest with retry-with-backoff for transient errors
#[derive(Debug, Clone)]
pub struct ReqwestHttpClient {
    client: reqwest::Client,
    max_retries: u32,
    base_backoff_ms: u64,
}

impl ReqwestHttpClient {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("hermes-collector/0.1 (news intelligence pipeline; +https://hermes.local)")
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .build()?;
        Ok(Self { client, max_retries: 3, base_backoff_ms: 500 })
    }
}

/// Classify an HTTP error as transient (worth retry) or permanent (fail fast).
fn is_transient(err: &anyhow::Error) -> bool {
    if let Some(req_err) = err.downcast_ref::<reqwest::Error>() {
        // 5xx, connection reset, timeout, request error → retry
        if req_err.is_timeout() || req_err.is_connect() || req_err.is_request() {
            return true;
        }
        if let Some(status) = req_err.status() {
            return status.is_server_error();
        }
    }
    let msg = format!("{:#}", err).to_lowercase();
    msg.contains("connection reset")
        || msg.contains("connection refused")
        || msg.contains("os error 104")
        || msg.contains("timed out")
        || msg.contains("broken pipe")
}

#[async_trait]
impl HttpClient for ReqwestHttpClient {
    async fn get(&self, url: &str) -> Result<String> {
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            match self.try_get(url).await {
                Ok(body) => return Ok(body),
                Err(e) if attempt < self.max_retries && is_transient(&e) => {
                    // Exponential backoff + jitter: 500ms, 1s, 2s... ±30%
                    let backoff = self.base_backoff_ms * (1u64 << (attempt - 1));
                    let jitter = (backoff / 3) as f64 * (rand_simple() - 0.5) * 2.0;
                    let sleep_ms = (backoff as f64 + jitter).max(50.0) as u64;
                    // Silent retry: log only at debug level unless all retries fail
                    if attempt == 1 {
                        debug!(
                            attempt = attempt,
                            sleep_ms = sleep_ms,
                            url = url,
                            "transient error, retrying"
                        );
                    }
                    tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

impl ReqwestHttpClient {
    async fn try_get(&self, url: &str) -> Result<String> {
        let resp = self.client.get(url).send().await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(anyhow!("HTTP {} from {}", status, url));
        }
        Ok(resp.text().await?)
    }
}

/// Tiny PRNG for jitter — no extra dep, deterministic per-call.
fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    (nanos % 10_000) as f64 / 10_000.0
}

impl MockHttpClient {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }
    
    pub fn add_response(&mut self, url: String, response: Result<String, String>) {
        self.responses.insert(url, response);
    }
}

#[async_trait]
impl HttpClient for MockHttpClient {
    async fn get(&self, url: &str) -> Result<String> {
        match self.responses.get(url) {
            Some(Ok(content)) => Ok(content.clone()),
            Some(Err(error)) => Err(anyhow!("Mock HTTP error: {}", error)),
            None => Err(anyhow!("Mock HTTP client: no response configured for {}", url)),
        }
    }
}

/// RSS fetcher for collecting feed data
pub struct RssFetcher<T: HttpClient> {
    http_client: T,
}

impl<T: HttpClient> RssFetcher<T> {
    /// Create new RSS fetcher with HTTP client
    pub fn new(http_client: T) -> Self {
        Self { http_client }
    }

    /// Fetch RSS feed from source
    pub async fn fetch_feed(&self, source: &FeedSource) -> Result<RssFeedResult> {
        debug!(
            source = %source.name,
            url = %source.url,
            category = ?source.category,
            "📡 Fetching RSS feed"
        );

        if !source.enabled {
            warn!(
                source = %source.name,
                "⚠️ RSS feed is disabled, skipping"
            );
            return Ok(RssFeedResult::new(source.clone(), Vec::new(), "Feed disabled".to_string()));
        }

        match self.http_client.get(&source.url).await {
            Ok(content) => {
                match self.parse_rss_content(&content) {
                    Ok(items) => {
                        debug!(
                            source = %source.name,
                            items_count = items.len(),
                            "✅ Successfully fetched and parsed RSS feed"
                        );
                        Ok(RssFeedResult::new(source.clone(), items, String::new()))
                    }
                    Err(parse_error) => {
                        error!(
                            source = %source.name,
                            error = %parse_error,
                            "❌ Failed to parse RSS content"
                        );
                        Ok(RssFeedResult::new(source.clone(), Vec::new(), parse_error.to_string()))
                    }
                }
            }
            Err(http_error) => {
                error!(
                    source = %source.name,
                    error = %http_error,
                    "❌ Failed to fetch RSS feed"
                );
                Ok(RssFeedResult::new(source.clone(), Vec::new(), http_error.to_string()))
            }
        }
    }

    /// Parse RSS content into items
    fn parse_rss_content(&self, content: &str) -> Result<Vec<RssItem>> {
        let channel = Channel::read_from(content.as_bytes())?;
        
        let items: Vec<RssItem> = channel
            .items()
            .iter()
            .map(|item| RssItem::from_rss_item(item))
            .collect();

        Ok(items)
    }

    /// Fetch multiple feeds concurrently
    pub async fn fetch_multiple_feeds(&self, sources: &[FeedSource]) -> Vec<RssFeedResult> {
        let mut results = Vec::new();
        
        for source in sources {
            let result = self.fetch_feed(source).await.unwrap_or_else(|e| {
                error!(
                    source = %source.name,
                    error = %e,
                    "❌ Critical error fetching RSS feed"
                );
                RssFeedResult::new(source.clone(), Vec::new(), e.to_string())
            });
            results.push(result);
        }
        
        results
    }
}

/// Result from RSS feed fetching
#[derive(Debug, Clone)]
pub struct RssFeedResult {
    pub source: FeedSource,
    pub items: Vec<RssItem>,
    pub error: String,
    pub fetch_time: chrono::DateTime<chrono::Utc>,
}

impl RssFeedResult {
    pub fn new(source: FeedSource, items: Vec<RssItem>, error: String) -> Self {
        Self {
            source,
            items,
            error,
            fetch_time: chrono::Utc::now(),
        }
    }
    
    /// Check if fetch was successful
    pub fn is_success(&self) -> bool {
        self.error.is_empty()
    }
    
    /// Check if result contains Indonesian market content
    pub fn has_indonesian_content(&self) -> bool {
        self.source.is_indonesian_market_related() ||
        self.items.iter().any(|item| item.is_indonesian_related())
    }
}

/// RSS item representation
#[derive(Debug, Clone)]
pub struct RssItem {
    pub title: String,
    pub link: String,
    pub description: String,
    pub pub_date: Option<chrono::DateTime<chrono::Utc>>,
    pub guid: Option<String>,
}

impl RssItem {
    /// Create RSS item from rss crate Item
    pub fn from_rss_item(item: &Item) -> Self {
        let pub_date = item.pub_date()
            .and_then(|date_str| {
                // Try parsing common RSS date formats
                chrono::DateTime::parse_from_rfc2822(date_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            });

        Self {
            title: item.title().unwrap_or("").to_string(),
            link: item.link().unwrap_or("").to_string(),
            description: item.description().unwrap_or("").to_string(),
            pub_date,
            guid: item.guid().map(|g| g.value().to_string()),
        }
    }
    
    /// Check if item is Indonesian market related
    pub fn is_indonesian_related(&self) -> bool {
        let content = format!("{} {} {}", self.title, self.description, self.link).to_lowercase();
        
        content.contains("indonesia") ||
        content.contains("rupiah") ||
        content.contains("jakarta") ||
        content.contains("bmri") ||
        content.contains("bbri") ||
        content.contains("inco") ||
        content.contains("antm") ||
        content.contains("idx") ||
        content.contains("bei") // Bursa Efek Indonesia
    }
    
    /// Check if item mentions Christian's portfolio stocks
    pub fn mentions_portfolio_stocks(&self) -> bool {
        let content = format!("{} {}", self.title, self.description).to_lowercase();
        
        content.contains("bmri") ||
        content.contains("bbri") ||
        content.contains("inco") ||
        content.contains("antm") ||
        content.contains("ptba") ||
        content.contains("tapg") ||
        content.contains("tlkm") ||
        content.contains("asii") ||
        content.contains("klbf") ||
        content.contains("tspc")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed_source::FeedCategory;

    #[test]
    fn test_mock_http_client() {
        let mut client = MockHttpClient::new();
        client.add_response(
            "https://example.com/rss".to_string(),
            Ok("<rss></rss>".to_string()),
        );
        
        assert!(client.responses.contains_key("https://example.com/rss"));
    }

    #[tokio::test]
    async fn test_rss_fetcher_disabled_feed() {
        let client = MockHttpClient::new();
        let fetcher = RssFetcher::new(client);
        
        let mut source = FeedSource::new(
            "test".to_string(),
            "https://example.com/rss".to_string(),
            FeedCategory::IndonesianNews,
            false,
        );
        source.enabled = false;
        
        let result = fetcher.fetch_feed(&source).await.unwrap();
        assert!(!result.is_success());
        assert!(result.items.is_empty());
        assert!(result.error.contains("disabled"));
    }

    #[tokio::test]
    async fn test_rss_fetcher_http_error() {
        let mut client = MockHttpClient::new();
        client.add_response(
            "https://example.com/rss".to_string(),
            Err("Network error".to_string()),
        );
        
        let fetcher = RssFetcher::new(client);
        let source = FeedSource::new(
            "test".to_string(),
            "https://example.com/rss".to_string(),
            FeedCategory::IndonesianNews,
            false,
        );
        
        let result = fetcher.fetch_feed(&source).await.unwrap();
        assert!(!result.is_success());
        assert!(result.items.is_empty());
        assert!(result.error.contains("Network error"));
    }

    #[tokio::test]
    async fn test_rss_fetcher_success() {
        let mut client = MockHttpClient::new();
        let rss_content = r#"<?xml version="1.0"?>
        <rss version="2.0">
            <channel>
                <title>Test Feed</title>
                <item>
                    <title>BMRI earnings report released</title>
                    <link>https://example.com/bmri-earnings</link>
                    <description>Bank Mandiri reports Indonesia quarterly results</description>
                </item>
            </channel>
        </rss>"#;
        
        client.add_response(
            "https://example.com/rss".to_string(),
            Ok(rss_content.to_string()),
        );
        
        let fetcher = RssFetcher::new(client);
        let source = FeedSource::new(
            "test".to_string(),
            "https://example.com/rss".to_string(),
            FeedCategory::IndonesianNews,
            true,
        );
        
        let result = fetcher.fetch_feed(&source).await.unwrap();
        assert!(result.is_success());
        assert_eq!(result.items.len(), 1);
        
        let item = &result.items[0];
        assert_eq!(item.title, "BMRI earnings report released");
        assert!(item.is_indonesian_related());
        assert!(item.mentions_portfolio_stocks());
    }

    #[test]
    fn test_rss_item_indonesian_detection() {
        let item = RssItem {
            title: "Indonesia economy grows".to_string(),
            link: "https://example.com".to_string(),
            description: "Jakarta stock market BMRI performance".to_string(),
            pub_date: None,
            guid: None,
        };
        
        assert!(item.is_indonesian_related());
        assert!(item.mentions_portfolio_stocks());
        
        let non_indonesian = RssItem {
            title: "US tech earnings".to_string(),
            link: "https://example.com".to_string(),
            description: "Apple reports quarterly results".to_string(),
            pub_date: None,
            guid: None,
        };
        
        assert!(!non_indonesian.is_indonesian_related());
        assert!(!non_indonesian.mentions_portfolio_stocks());
    }

    #[test]
    fn test_rss_feed_result() {
        let source = FeedSource::new(
            "kompas".to_string(),
            "https://kompas.com/rss".to_string(),
            FeedCategory::IndonesianNews,
            true,
        );
        
        let result = RssFeedResult::new(source, Vec::new(), String::new());
        assert!(result.is_success());
        assert!(result.has_indonesian_content()); // Source is Indonesian
        
        let error_result = RssFeedResult::new(
            FeedSource::new("test".to_string(), "url".to_string(), FeedCategory::Technology, false),
            Vec::new(),
            "Error occurred".to_string(),
        );
        assert!(!error_result.is_success());
    }
}