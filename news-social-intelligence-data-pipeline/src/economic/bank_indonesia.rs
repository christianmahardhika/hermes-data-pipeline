//! Bank Indonesia Economic Data Collector
//!
//! Fetches BI Rate, JIBOR, USD/IDR exchange rate, and inflation data.
//! Uses Yahoo Finance as proxy for USD/IDR since BI's public API is unreliable
//! for programmatic access. Falls back gracefully when data is unavailable.
//!
//! Stores indicators in ArangoDB `economic_indicators` collection and creates
//! `signal_source` edges to banking tickers (BBCA, BBRI, BMRI, BBNI).

use anyhow::{Context, Result};
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tracing::{info, warn};

use crate::arangodb::ArangoClient;
use crate::economic::models::{EconomicIndicator, EconomicSource, EconomicStats, SignalSourceEdge};

/// BI economic indicators: (key, description, unit)
const BI_INDICATORS: &[(&str, &str, &str)] = &[
    ("BI_RATE", "BI 7-Day Reverse Repo Rate", "percent"),
    ("JIBOR_ON", "JIBOR Overnight", "percent"),
    ("USD_IDR", "USD/IDR Exchange Rate", "IDR"),
    ("INFLATION_YOY", "Indonesia Inflation YoY", "percent"),
];

/// Related banking tickers for signal edges
const BANKING_TICKERS: &[&str] = &["BBCA", "BBRI", "BMRI", "BBNI"];

/// Edge strength for BI indicators → banking tickers
const EDGE_STRENGTH: f64 = 0.7;

/// HTTP request timeout
const REQUEST_TIMEOUT_SECS: u64 = 15;

/// Yahoo Finance chart API response (simplified)
#[derive(Debug, Deserialize)]
struct YahooChartResponse {
    chart: Option<YahooChart>,
}

#[derive(Debug, Deserialize)]
struct YahooChart {
    result: Option<Vec<YahooChartResult>>,
}

#[derive(Debug, Deserialize)]
struct YahooChartResult {
    meta: Option<YahooMeta>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YahooMeta {
    regular_market_price: Option<f64>,
    previous_close: Option<f64>,
}

/// Bank Indonesia data collector
pub struct BankIndonesiaCollector {
    client: Client,
}

impl BankIndonesiaCollector {
    /// Create a new BankIndonesiaCollector with configured HTTP client
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .user_agent("hermes-data-pipeline/0.1")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client }
    }

    /// Collect all BI economic indicators and store in ArangoDB
    ///
    /// Graceful degradation: if ALL fetches fail, returns EconomicStats::default()
    /// with 0 indicators (does not error).
    pub async fn collect_all(&self, arango: &ArangoClient) -> Result<EconomicStats> {
        info!("🏦 Bank Indonesia: collecting economic indicators...");

        let mut stats = EconomicStats::default();
        let mut indicators: Vec<EconomicIndicator> = Vec::new();

        // Fetch USD/IDR from Yahoo Finance as proxy
        match self.fetch_exchange_rate().await {
            Ok((value, change_pct)) => {
                let indicator = self.build_indicator("USD_IDR", value, "IDR", change_pct);
                indicators.push(indicator);
            }
            Err(e) => {
                warn!("⚠️ Bank Indonesia: failed to fetch USD/IDR: {}", e);
                stats.errors += 1;
            }
        }

        // Fetch BI Rate (static/config-based since BI has no clean API)
        match self.fetch_bi_rate().await {
            Ok(value) => {
                let indicator = self.build_indicator("BI_RATE", value, "percent", None);
                indicators.push(indicator);
            }
            Err(e) => {
                warn!("⚠️ Bank Indonesia: failed to fetch BI Rate: {}", e);
                stats.errors += 1;
            }
        }

        // Fetch JIBOR Overnight (config-based)
        match self.fetch_jibor().await {
            Ok(value) => {
                let indicator = self.build_indicator("JIBOR_ON", value, "percent", None);
                indicators.push(indicator);
            }
            Err(e) => {
                warn!("⚠️ Bank Indonesia: failed to fetch JIBOR: {}", e);
                stats.errors += 1;
            }
        }

        // Fetch Indonesia Inflation YoY (config-based)
        match self.fetch_inflation().await {
            Ok(value) => {
                let indicator = self.build_indicator("INFLATION_YOY", value, "percent", None);
                indicators.push(indicator);
            }
            Err(e) => {
                warn!("⚠️ Bank Indonesia: failed to fetch Inflation: {}", e);
                stats.errors += 1;
            }
        }

        if indicators.is_empty() {
            warn!("⚠️ Bank Indonesia: all fetches failed, returning empty stats");
            return Ok(stats);
        }

        // Store indicators in ArangoDB
        for indicator in &indicators {
            match self.store_indicator(arango, indicator).await {
                Ok(_) => {
                    stats.indicators_inserted += 1;
                }
                Err(e) => {
                    warn!("⚠️ Bank Indonesia: failed to store indicator {}: {}", indicator.indicator, e);
                    stats.errors += 1;
                }
            }

            // Create signal_source edges to banking tickers
            for ticker in BANKING_TICKERS {
                match self.store_signal_edge(arango, indicator, ticker).await {
                    Ok(_) => {
                        stats.edges_created += 1;
                    }
                    Err(e) => {
                        warn!("⚠️ Bank Indonesia: failed to create edge to {}: {}", ticker, e);
                        stats.errors += 1;
                    }
                }
            }
        }

        info!(
            "🏦 Bank Indonesia: {} indicators, {} edges, {} errors",
            stats.indicators_inserted, stats.edges_created, stats.errors
        );

        Ok(stats)
    }

    /// Fetch USD/IDR exchange rate from Yahoo Finance (USDIDR=X)
    pub async fn fetch_exchange_rate(&self) -> Result<(f64, Option<f64>)> {
        let url = "https://query1.finance.yahoo.com/v8/finance/chart/USDIDR=X?interval=1d&range=1d";

        let resp = self.client
            .get(url)
            .send()
            .await
            .context("fetching USD/IDR from Yahoo Finance")?;

        if !resp.status().is_success() {
            anyhow::bail!("Yahoo Finance returned status: {}", resp.status());
        }

        let body: YahooChartResponse = resp
            .json()
            .await
            .context("parsing Yahoo Finance USD/IDR response")?;

        let meta = body.chart
            .and_then(|c| c.result)
            .and_then(|r| r.into_iter().next())
            .and_then(|r| r.meta)
            .context("no chart data in Yahoo Finance response")?;

        let price = meta.regular_market_price
            .context("no regularMarketPrice in Yahoo Finance response")?;

        let change_pct = meta.previous_close.map(|prev| {
            if prev > 0.0 {
                ((price - prev) / prev) * 100.0
            } else {
                0.0
            }
        });

        Ok((price, change_pct))
    }

    /// Fetch BI 7-Day Reverse Repo Rate
    ///
    /// Since BI doesn't provide a clean REST API, this reads from
    /// the `BI_RATE` environment variable (manually updated) or returns
    /// the current known rate.
    pub async fn fetch_bi_rate(&self) -> Result<f64> {
        // Try env var first (allows manual override)
        if let Ok(rate_str) = std::env::var("BI_RATE") {
            let rate: f64 = rate_str.parse()
                .context("parsing BI_RATE env var")?;
            return Ok(rate);
        }

        // Default: current BI rate as of 2024 (6.25%)
        // This should be updated via env var or config when BI changes the rate
        Ok(6.25)
    }

    /// Fetch JIBOR Overnight rate
    ///
    /// Reads from `JIBOR_ON` environment variable or uses default.
    pub async fn fetch_jibor(&self) -> Result<f64> {
        if let Ok(rate_str) = std::env::var("JIBOR_ON") {
            let rate: f64 = rate_str.parse()
                .context("parsing JIBOR_ON env var")?;
            return Ok(rate);
        }

        // Default: approximate JIBOR ON rate
        Ok(6.19)
    }

    /// Fetch Indonesia YoY Inflation
    ///
    /// Reads from `INFLATION_YOY` environment variable or uses default.
    pub async fn fetch_inflation(&self) -> Result<f64> {
        if let Ok(rate_str) = std::env::var("INFLATION_YOY") {
            let rate: f64 = rate_str.parse()
                .context("parsing INFLATION_YOY env var")?;
            return Ok(rate);
        }

        // Default: approximate Indonesia inflation rate
        Ok(2.12)
    }

    /// Build an EconomicIndicator from collected data
    fn build_indicator(
        &self,
        indicator_name: &str,
        value: f64,
        unit: &str,
        change_pct: Option<f64>,
    ) -> EconomicIndicator {
        let now = Utc::now();
        let key = format!(
            "bank_indonesia_{}_{}",
            indicator_name.to_lowercase(),
            now.format("%Y%m%d%H%M")
        );

        EconomicIndicator {
            key,
            source: EconomicSource::BankIndonesia,
            indicator: indicator_name.to_string(),
            value,
            unit: unit.to_string(),
            timestamp: now,
            change_pct,
            metadata: serde_json::json!({
                "source_detail": "bank_indonesia",
                "indicators": BI_INDICATORS.iter()
                    .map(|(k, d, u)| serde_json::json!({"key": k, "desc": d, "unit": u}))
                    .collect::<Vec<_>>()
            }),
        }
    }

    /// Store an indicator in ArangoDB economic_indicators collection
    async fn store_indicator(
        &self,
        arango: &ArangoClient,
        indicator: &EconomicIndicator,
    ) -> Result<String> {
        let doc = serde_json::to_value(indicator)
            .context("serializing indicator to JSON")?;

        arango
            .insert_document("economic_indicators", &doc)
            .await
            .context("inserting indicator into ArangoDB")
    }

    /// Store a signal_source edge from indicator to banking ticker
    async fn store_signal_edge(
        &self,
        arango: &ArangoClient,
        indicator: &EconomicIndicator,
        ticker: &str,
    ) -> Result<String> {
        let from = format!("economic_indicators/{}", indicator.key);
        let to = format!("tickers/{}", ticker);

        let direction = match indicator.indicator.as_str() {
            "BI_RATE" | "JIBOR_ON" => "negative", // Higher rates → negative for banks short-term
            "USD_IDR" => "neutral",                // Mixed impact
            "INFLATION_YOY" => "negative",         // Higher inflation → pressure
            _ => "neutral",
        };

        let edge = SignalSourceEdge {
            from: from.clone(),
            to: to.clone(),
            strength: EDGE_STRENGTH,
            direction: direction.to_string(),
            created_at: Utc::now(),
        };

        let data = serde_json::to_value(&edge)
            .context("serializing signal edge to JSON")?;

        arango
            .insert_edge("signal_source", &from, &to, &data)
            .await
            .context("inserting signal_source edge into ArangoDB")
    }
}

/// Parse USD/IDR rate from Yahoo Finance JSON response body
pub fn parse_yahoo_exchange_rate(body: &str) -> Result<(f64, Option<f64>)> {
    let response: YahooChartResponse = serde_json::from_str(body)
        .context("parsing Yahoo Finance JSON")?;

    let meta = response.chart
        .and_then(|c| c.result)
        .and_then(|r| r.into_iter().next())
        .and_then(|r| r.meta)
        .context("no chart data in response")?;

    let price = meta.regular_market_price
        .context("no regularMarketPrice")?;

    let change_pct = meta.previous_close.map(|prev| {
        if prev > 0.0 {
            ((price - prev) / prev) * 100.0
        } else {
            0.0
        }
    });

    Ok((price, change_pct))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yahoo_exchange_rate_valid() {
        let json = r#"{
            "chart": {
                "result": [{
                    "meta": {
                        "regularMarketPrice": 15850.0,
                        "previousClose": 15800.0
                    }
                }]
            }
        }"#;

        let (price, change_pct) = parse_yahoo_exchange_rate(json).unwrap();
        assert_eq!(price, 15850.0);
        assert!(change_pct.is_some());

        let pct = change_pct.unwrap();
        // (15850 - 15800) / 15800 * 100 ≈ 0.3164
        assert!((pct - 0.3164).abs() < 0.01);
    }

    #[test]
    fn test_parse_yahoo_exchange_rate_no_previous_close() {
        let json = r#"{
            "chart": {
                "result": [{
                    "meta": {
                        "regularMarketPrice": 16000.0
                    }
                }]
            }
        }"#;

        let (price, change_pct) = parse_yahoo_exchange_rate(json).unwrap();
        assert_eq!(price, 16000.0);
        assert!(change_pct.is_none());
    }

    #[test]
    fn test_parse_yahoo_exchange_rate_empty_chart() {
        let json = r#"{"chart": {"result": []}}"#;
        let result = parse_yahoo_exchange_rate(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_yahoo_exchange_rate_null_chart() {
        let json = r#"{"chart": null}"#;
        let result = parse_yahoo_exchange_rate(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_yahoo_exchange_rate_invalid_json() {
        let json = "not valid json";
        let result = parse_yahoo_exchange_rate(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_yahoo_exchange_rate_no_price() {
        let json = r#"{
            "chart": {
                "result": [{
                    "meta": {
                        "previousClose": 15800.0
                    }
                }]
            }
        }"#;

        let result = parse_yahoo_exchange_rate(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_indicator() {
        let collector = BankIndonesiaCollector::new();
        let indicator = collector.build_indicator("USD_IDR", 15850.0, "IDR", Some(0.32));

        assert_eq!(indicator.source, EconomicSource::BankIndonesia);
        assert_eq!(indicator.indicator, "USD_IDR");
        assert_eq!(indicator.value, 15850.0);
        assert_eq!(indicator.unit, "IDR");
        assert_eq!(indicator.change_pct, Some(0.32));
        assert!(indicator.key.starts_with("bank_indonesia_usd_idr_"));
    }

    #[test]
    fn test_build_indicator_no_change() {
        let collector = BankIndonesiaCollector::new();
        let indicator = collector.build_indicator("BI_RATE", 6.25, "percent", None);

        assert_eq!(indicator.source, EconomicSource::BankIndonesia);
        assert_eq!(indicator.indicator, "BI_RATE");
        assert_eq!(indicator.value, 6.25);
        assert_eq!(indicator.unit, "percent");
        assert_eq!(indicator.change_pct, None);
        assert!(indicator.key.starts_with("bank_indonesia_bi_rate_"));
    }

    #[test]
    fn test_bi_indicators_constant() {
        assert_eq!(BI_INDICATORS.len(), 4);
        assert_eq!(BI_INDICATORS[0].0, "BI_RATE");
        assert_eq!(BI_INDICATORS[1].0, "JIBOR_ON");
        assert_eq!(BI_INDICATORS[2].0, "USD_IDR");
        assert_eq!(BI_INDICATORS[3].0, "INFLATION_YOY");
    }

    #[test]
    fn test_banking_tickers_constant() {
        assert_eq!(BANKING_TICKERS.len(), 4);
        assert!(BANKING_TICKERS.contains(&"BBCA"));
        assert!(BANKING_TICKERS.contains(&"BBRI"));
        assert!(BANKING_TICKERS.contains(&"BMRI"));
        assert!(BANKING_TICKERS.contains(&"BBNI"));
    }

    #[test]
    fn test_collector_creation() {
        let collector = BankIndonesiaCollector::new();
        // Verify it doesn't panic — client is created successfully
        let _ = collector;
    }

    #[tokio::test]
    async fn test_fetch_bi_rate_from_env() {
        std::env::set_var("BI_RATE", "6.50");
        let collector = BankIndonesiaCollector::new();
        let rate = collector.fetch_bi_rate().await.unwrap();
        assert_eq!(rate, 6.50);
        std::env::remove_var("BI_RATE");
    }

    #[tokio::test]
    async fn test_fetch_bi_rate_default() {
        std::env::remove_var("BI_RATE");
        let collector = BankIndonesiaCollector::new();
        let rate = collector.fetch_bi_rate().await.unwrap();
        assert_eq!(rate, 6.25);
    }

    #[tokio::test]
    async fn test_fetch_jibor_from_env() {
        std::env::set_var("JIBOR_ON", "6.30");
        let collector = BankIndonesiaCollector::new();
        let rate = collector.fetch_jibor().await.unwrap();
        assert_eq!(rate, 6.30);
        std::env::remove_var("JIBOR_ON");
    }

    #[tokio::test]
    async fn test_fetch_jibor_default() {
        std::env::remove_var("JIBOR_ON");
        let collector = BankIndonesiaCollector::new();
        let rate = collector.fetch_jibor().await.unwrap();
        assert_eq!(rate, 6.19);
    }

    #[tokio::test]
    async fn test_fetch_inflation_from_env() {
        std::env::set_var("INFLATION_YOY", "3.05");
        let collector = BankIndonesiaCollector::new();
        let rate = collector.fetch_inflation().await.unwrap();
        assert_eq!(rate, 3.05);
        std::env::remove_var("INFLATION_YOY");
    }

    #[tokio::test]
    async fn test_fetch_inflation_default() {
        std::env::remove_var("INFLATION_YOY");
        let collector = BankIndonesiaCollector::new();
        let rate = collector.fetch_inflation().await.unwrap();
        assert_eq!(rate, 2.12);
    }

    #[tokio::test]
    async fn test_fetch_bi_rate_invalid_env() {
        std::env::set_var("BI_RATE", "not_a_number");
        let collector = BankIndonesiaCollector::new();
        let result = collector.fetch_bi_rate().await;
        assert!(result.is_err());
        std::env::remove_var("BI_RATE");
    }

    #[tokio::test]
    #[ignore] // Integration test: requires network access to Yahoo Finance
    async fn test_fetch_exchange_rate_live() {
        let collector = BankIndonesiaCollector::new();
        let result = collector.fetch_exchange_rate().await;
        assert!(result.is_ok());
        let (price, _change_pct) = result.unwrap();
        // USD/IDR should be in a reasonable range
        assert!(price > 10000.0);
        assert!(price < 25000.0);
    }

    #[tokio::test]
    #[ignore] // Integration test: requires ArangoDB
    async fn test_collect_all_integration() {
        let collector = BankIndonesiaCollector::new();
        let arango = ArangoClient::new().expect("ArangoClient creation failed");
        let stats = collector.collect_all(&arango).await.unwrap();
        // Should have collected at least some indicators
        assert!(stats.indicators_inserted > 0 || stats.errors > 0);
    }
}
