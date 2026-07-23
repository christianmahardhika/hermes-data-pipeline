//! GDELT (Global Database of Events, Language, and Tone) API v2 client
//!
//! Fetches Indonesian-related events from the GDELT Project API.
//! Free API — no API key required.
//!
//! Stores events in ArangoDB `events` collection and creates `impacts` edges.
//! Schedule: every 2 hours.

use anyhow::{Context, Result};
use chrono::Utc;
use reqwest::Client;
use std::time::Duration;
use tracing::{info, warn};

use crate::arangodb::ArangoClient;
use crate::economic::models::EconomicStats;

/// GDELT API v2 base URL
const GDELT_API_URL: &str = "https://api.gdeltproject.org/api/v2/doc/doc";

/// HTTP request timeout for GDELT API
const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Maximum records to fetch per request
const MAX_RECORDS: u32 = 25;

/// GDELT article data from API response
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GdeltArticle {
    pub url: String,
    pub title: String,
    pub seendate: String,
    pub domain: String,
    pub language: Option<String>,
    pub sourcecountry: Option<String>,
}

/// GDELT API response wrapper
#[derive(Debug, serde::Deserialize)]
pub struct GdeltResponse {
    pub articles: Option<Vec<GdeltArticle>>,
}

/// GDELT event collector for Indonesian events
pub struct GdeltCollector {
    client: Client,
    query: String,
}

impl GdeltCollector {
    /// Create a new GdeltCollector with default query "indonesia"
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .expect("Failed to build GDELT HTTP client");

        Self {
            client,
            query: "indonesia".to_string(),
        }
    }

    /// Override the search query
    pub fn with_query(mut self, query: &str) -> Self {
        self.query = query.to_string();
        self
    }

    /// Collect events from GDELT API and store in ArangoDB.
    /// Returns empty stats on API failure (graceful degradation).
    pub async fn collect_events(&self, arango: &ArangoClient) -> Result<EconomicStats> {
        let url = format!(
            "{}?query={}&mode=artlist&maxrecords={}&format=json",
            GDELT_API_URL, self.query, MAX_RECORDS
        );

        let resp = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!("⚠️ GDELT API request failed: {}", e);
                return Ok(EconomicStats::default());
            }
        };

        if !resp.status().is_success() {
            warn!("⚠️ GDELT API returned status: {}", resp.status());
            return Ok(EconomicStats::default());
        }

        let body: GdeltResponse = match resp.json().await {
            Ok(b) => b,
            Err(e) => {
                warn!("⚠️ GDELT response parse error: {}", e);
                return Ok(EconomicStats::default());
            }
        };

        let articles = body.articles.unwrap_or_default();
        let mut stats = EconomicStats::default();

        for article in &articles {
            // Generate deterministic key for dedup
            let event_key = format!(
                "gdelt_{}",
                sha2_hash(&format!("{}_{}", article.url, article.seendate))
            );

            let event_doc = serde_json::json!({
                "_key": event_key,
                "name": article.title,
                "type": "gdelt_event",
                "source_url": article.url,
                "domain": article.domain,
                "language": article.language,
                "source_country": article.sourcecountry,
                "seen_date": article.seendate,
                "collected_at": Utc::now().to_rfc3339(),
            });

            match arango.insert_document("events", &event_doc).await {
                Ok(_) => stats.indicators_inserted += 1,
                Err(e) => {
                    // Likely duplicate key — skip silently
                    if !e.to_string().contains("unique constraint") {
                        warn!("⚠️ GDELT insert error: {}", e);
                        stats.errors += 1;
                    }
                }
            }

            // Create impacts edge linking event to market context
            let edge_data = serde_json::json!({
                "_key": format!("impact_{}", event_key),
                "impact_type": "news_event",
                "query": self.query,
                "domain": article.domain,
                "created_at": Utc::now().to_rfc3339(),
            });

            let from = format!("events/{}", event_key);
            let to = "market_context/indonesia_economy".to_string();

            match arango.insert_edge("impacts", &from, &to, &edge_data).await {
                Ok(_) => stats.edges_created += 1,
                Err(e) => {
                    if !e.to_string().contains("unique constraint") {
                        warn!("⚠️ GDELT edge insert error: {}", e);
                    }
                }
            }
        }

        info!(
            "📰 GDELT: collected {} events, {} edges for query '{}'",
            stats.indicators_inserted, stats.edges_created, self.query
        );
        Ok(stats)
    }
}

/// SHA256 hash for deterministic dedup keys (first 16 hex chars)
fn sha2_hash(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(input.as_bytes());
    hex::encode(&hash[..8])
}

/// Parse GDELT API response JSON into articles (exported for testing)
pub fn parse_gdelt_response(json: &str) -> Result<Vec<GdeltArticle>> {
    let resp: GdeltResponse =
        serde_json::from_str(json).context("parsing GDELT JSON")?;
    Ok(resp.articles.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gdelt_response_valid() {
        let json = r#"{"articles": [{"url": "https://example.com/news", "title": "Indonesia Export Ban", "seendate": "20240101T120000Z", "domain": "reuters.com", "language": "English", "sourcecountry": "Indonesia"}]}"#;
        let articles = parse_gdelt_response(json).unwrap();
        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0].title, "Indonesia Export Ban");
        assert_eq!(articles[0].domain, "reuters.com");
        assert_eq!(articles[0].url, "https://example.com/news");
        assert_eq!(articles[0].seendate, "20240101T120000Z");
        assert_eq!(articles[0].language.as_deref(), Some("English"));
        assert_eq!(articles[0].sourcecountry.as_deref(), Some("Indonesia"));
    }

    #[test]
    fn test_parse_gdelt_response_empty() {
        let json = r#"{"articles": []}"#;
        let articles = parse_gdelt_response(json).unwrap();
        assert_eq!(articles.len(), 0);
    }

    #[test]
    fn test_parse_gdelt_response_null_articles() {
        let json = r#"{}"#;
        let articles = parse_gdelt_response(json).unwrap();
        assert_eq!(articles.len(), 0);
    }

    #[test]
    fn test_parse_gdelt_response_multiple_articles() {
        let json = r#"{"articles": [
            {"url": "https://a.com/1", "title": "Article One", "seendate": "20240101T100000Z", "domain": "a.com", "language": "English", "sourcecountry": "Indonesia"},
            {"url": "https://b.com/2", "title": "Article Two", "seendate": "20240101T110000Z", "domain": "b.com", "language": null, "sourcecountry": null}
        ]}"#;
        let articles = parse_gdelt_response(json).unwrap();
        assert_eq!(articles.len(), 2);
        assert_eq!(articles[0].title, "Article One");
        assert_eq!(articles[1].title, "Article Two");
        assert!(articles[1].language.is_none());
        assert!(articles[1].sourcecountry.is_none());
    }

    #[test]
    fn test_parse_gdelt_response_invalid_json() {
        let json = r#"{ not valid json }"#;
        let result = parse_gdelt_response(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_sha2_hash_deterministic() {
        let h1 = sha2_hash("test_input");
        let h2 = sha2_hash("test_input");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 16); // 8 bytes = 16 hex chars
    }

    #[test]
    fn test_sha2_hash_different_inputs() {
        let h1 = sha2_hash("input_a");
        let h2 = sha2_hash("input_b");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_sha2_hash_empty_input() {
        let h = sha2_hash("");
        assert_eq!(h.len(), 16);
        assert!(!h.is_empty());
    }

    #[test]
    fn test_gdelt_collector_default_query() {
        let collector = GdeltCollector::new();
        assert_eq!(collector.query, "indonesia");
    }

    #[test]
    fn test_gdelt_collector_custom_query() {
        let collector = GdeltCollector::new().with_query("trade policy");
        assert_eq!(collector.query, "trade policy");
    }

    #[test]
    fn test_event_key_format() {
        let url = "https://example.com/news";
        let seendate = "20240101T120000Z";
        let key = format!("gdelt_{}", sha2_hash(&format!("{}_{}", url, seendate)));
        assert!(key.starts_with("gdelt_"));
        assert_eq!(key.len(), 6 + 16); // "gdelt_" + 16 hex chars
    }

    #[tokio::test]
    #[ignore] // Integration test: requires running ArangoDB
    async fn test_gdelt_collect_events_integration() {
        let collector = GdeltCollector::new();
        let arango = crate::arangodb::ArangoClient::new().unwrap();
        let stats = collector.collect_events(&arango).await.unwrap();
        // Should not error even if GDELT API is unreachable
        println!("GDELT stats: {}", stats);
    }
}
