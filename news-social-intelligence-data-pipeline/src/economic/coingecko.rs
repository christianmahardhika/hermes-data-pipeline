//! CoinGecko cryptocurrency price collector
//!
//! Fetches BTC, ETH, USDT, BNB, XRP prices from CoinGecko free API.
//! Single API call fetches all coins at once (no internal rate limit stagger needed).
//! Free tier rate limit: 10-30 calls/min.

use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Deserialize;
use tracing::{info, warn};

use crate::arangodb::ArangoClient;
use crate::economic::models::{EconomicIndicator, EconomicSource, EconomicStats};

/// CoinGecko API base URL
const COINGECKO_API_URL: &str =
    "https://api.coingecko.com/api/v3/simple/price";

/// Coin IDs to fetch
const COIN_IDS: &[(&str, &str)] = &[
    ("bitcoin", "BTC_USD"),
    ("ethereum", "ETH_USD"),
    ("tether", "USDT_USD"),
    ("binancecoin", "BNB_USD"),
    ("ripple", "XRP_USD"),
];

/// HTTP request timeout
const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Price data from CoinGecko API response
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoPrice {
    pub usd: f64,
    pub usd_24h_change: Option<f64>,
}

/// CoinGecko cryptocurrency price collector
pub struct CoinGeckoCollector {
    client: reqwest::Client,
}

impl CoinGeckoCollector {
    /// Create a new CoinGeckoCollector with default HTTP client
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .user_agent("hermes-data-pipeline/0.1")
            .build()
            .context("building CoinGecko HTTP client")?;

        Ok(Self { client })
    }

    /// Collect all cryptocurrency prices and store in ArangoDB
    pub async fn collect_all(&self, arango: &ArangoClient) -> Result<EconomicStats> {
        info!("📊 CoinGecko: fetching crypto prices...");

        let prices = self.fetch_prices().await?;
        let mut stats = EconomicStats::default();
        let now = Utc::now();
        let timestamp_secs = now.timestamp();

        for (coin_id, indicator_name) in COIN_IDS {
            match prices.get(*coin_id) {
                Some(price) => {
                    let indicator = EconomicIndicator {
                        key: format!("coingecko_{}_{}", coin_id, timestamp_secs),
                        source: EconomicSource::CoinGecko,
                        indicator: indicator_name.to_string(),
                        value: price.usd,
                        unit: "USD".to_string(),
                        timestamp: now,
                        change_pct: price.usd_24h_change,
                        metadata: serde_json::json!({
                            "coin_id": coin_id,
                            "symbol": indicator_name.replace("_USD", ""),
                        }),
                    };

                    let doc = serde_json::to_value(&indicator)
                        .context("serializing CoinGecko indicator")?;

                    match arango.insert_document("economic_indicators", &doc).await {
                        Ok(_key) => {
                            stats.indicators_inserted += 1;
                            info!(
                                "✅ CoinGecko: {} = ${:.2} (change: {:+.2}%)",
                                indicator_name,
                                price.usd,
                                price.usd_24h_change.unwrap_or(0.0)
                            );
                        }
                        Err(e) => {
                            stats.errors += 1;
                            warn!("⚠️ CoinGecko: failed to store {}: {}", indicator_name, e);
                        }
                    }
                }
                None => {
                    stats.errors += 1;
                    warn!("⚠️ CoinGecko: no price data for {}", coin_id);
                }
            }
        }

        info!("📊 CoinGecko: {}", stats);
        Ok(stats)
    }

    /// Fetch prices from CoinGecko API (single call for all coins)
    pub async fn fetch_prices(&self) -> Result<HashMap<String, CryptoPrice>> {
        let ids: Vec<&str> = COIN_IDS.iter().map(|(id, _)| *id).collect();
        let ids_param = ids.join(",");

        let url = format!(
            "{}?ids={}&vs_currencies=usd&include_24hr_change=true",
            COINGECKO_API_URL, ids_param
        );

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("CoinGecko API request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("CoinGecko API error: status={}, body={}", status, body);
        }

        let prices: HashMap<String, CryptoPrice> = resp
            .json()
            .await
            .context("parsing CoinGecko API response")?;

        Ok(prices)
    }

    /// Parse a raw JSON response into CryptoPrice map (useful for testing)
    pub fn parse_response(json: &str) -> Result<HashMap<String, CryptoPrice>> {
        let prices: HashMap<String, CryptoPrice> =
            serde_json::from_str(json).context("parsing CoinGecko JSON response")?;
        Ok(prices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOCK_RESPONSE: &str = r#"{
        "bitcoin": {"usd": 67000.50, "usd_24h_change": 2.5},
        "ethereum": {"usd": 3400.25, "usd_24h_change": -1.2},
        "tether": {"usd": 1.0001, "usd_24h_change": 0.01},
        "binancecoin": {"usd": 580.75, "usd_24h_change": 3.8},
        "ripple": {"usd": 0.52, "usd_24h_change": -0.5}
    }"#;

    #[test]
    fn test_parse_coingecko_response() {
        let prices = CoinGeckoCollector::parse_response(MOCK_RESPONSE)
            .expect("should parse mock response");

        assert_eq!(prices.len(), 5);

        let btc = prices.get("bitcoin").expect("bitcoin should exist");
        assert_eq!(btc.usd, 67000.50);
        assert_eq!(btc.usd_24h_change, Some(2.5));

        let eth = prices.get("ethereum").expect("ethereum should exist");
        assert_eq!(eth.usd, 3400.25);
        assert_eq!(eth.usd_24h_change, Some(-1.2));

        let usdt = prices.get("tether").expect("tether should exist");
        assert_eq!(usdt.usd, 1.0001);
        assert_eq!(usdt.usd_24h_change, Some(0.01));

        let bnb = prices.get("binancecoin").expect("binancecoin should exist");
        assert_eq!(bnb.usd, 580.75);
        assert_eq!(bnb.usd_24h_change, Some(3.8));

        let xrp = prices.get("ripple").expect("ripple should exist");
        assert_eq!(xrp.usd, 0.52);
        assert_eq!(xrp.usd_24h_change, Some(-0.5));
    }

    #[test]
    fn test_parse_response_with_null_change() {
        let json = r#"{
            "bitcoin": {"usd": 67000.50, "usd_24h_change": null}
        }"#;

        let prices =
            CoinGeckoCollector::parse_response(json).expect("should handle null change");

        let btc = prices.get("bitcoin").expect("bitcoin should exist");
        assert_eq!(btc.usd, 67000.50);
        assert_eq!(btc.usd_24h_change, None);
    }

    #[test]
    fn test_parse_response_missing_coin() {
        let json = r#"{
            "bitcoin": {"usd": 67000.50, "usd_24h_change": 2.5}
        }"#;

        let prices = CoinGeckoCollector::parse_response(json)
            .expect("should parse partial response");

        assert_eq!(prices.len(), 1);
        assert!(prices.get("ethereum").is_none());
    }

    #[test]
    fn test_indicator_creation_from_price() {
        let prices = CoinGeckoCollector::parse_response(MOCK_RESPONSE)
            .expect("should parse mock response");

        let now = Utc::now();
        let timestamp_secs = now.timestamp();

        let btc_price = prices.get("bitcoin").unwrap();
        let indicator = EconomicIndicator {
            key: format!("coingecko_bitcoin_{}", timestamp_secs),
            source: EconomicSource::CoinGecko,
            indicator: "BTC_USD".to_string(),
            value: btc_price.usd,
            unit: "USD".to_string(),
            timestamp: now,
            change_pct: btc_price.usd_24h_change,
            metadata: serde_json::json!({
                "coin_id": "bitcoin",
                "symbol": "BTC",
            }),
        };

        assert_eq!(indicator.source, EconomicSource::CoinGecko);
        assert_eq!(indicator.indicator, "BTC_USD");
        assert_eq!(indicator.value, 67000.50);
        assert_eq!(indicator.unit, "USD");
        assert_eq!(indicator.change_pct, Some(2.5));
        assert!(indicator.key.starts_with("coingecko_bitcoin_"));
    }

    #[test]
    fn test_coin_ids_configuration() {
        assert_eq!(COIN_IDS.len(), 5);
        assert_eq!(COIN_IDS[0], ("bitcoin", "BTC_USD"));
        assert_eq!(COIN_IDS[1], ("ethereum", "ETH_USD"));
        assert_eq!(COIN_IDS[2], ("tether", "USDT_USD"));
        assert_eq!(COIN_IDS[3], ("binancecoin", "BNB_USD"));
        assert_eq!(COIN_IDS[4], ("ripple", "XRP_USD"));
    }

    #[test]
    fn test_collector_creation() {
        let collector = CoinGeckoCollector::new();
        assert!(collector.is_ok(), "CoinGeckoCollector::new() should succeed");
    }

    #[test]
    fn test_parse_invalid_json() {
        let invalid = "not valid json";
        let result = CoinGeckoCollector::parse_response(invalid);
        assert!(result.is_err(), "should fail on invalid JSON");
    }

    #[tokio::test]
    #[ignore] // Integration test — requires real CoinGecko API
    async fn test_fetch_prices_live() {
        let collector = CoinGeckoCollector::new().expect("collector creation");
        let prices = collector.fetch_prices().await.expect("fetch should succeed");
        assert!(!prices.is_empty(), "should return at least one price");
        assert!(prices.contains_key("bitcoin"), "should contain bitcoin");
    }
}
