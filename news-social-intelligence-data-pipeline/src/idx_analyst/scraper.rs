//! IDX Data Scraper — Fetch stock data from Yahoo Finance API
//!
//! Replaces Python's yfinance + curl_cffi.
//! Uses reqwest with browser-like headers to fetch Yahoo Finance data.

use anyhow::{Result, Context};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{info, warn};

use super::types::{StockMetrics, IdxProfile, OwnershipData, LiquidityData, DividendRecord, DebtStructure};

/// IDX Data Scraper using Yahoo Finance as data source
pub struct IDXDataScraper {
    client: Client,
}

#[derive(Deserialize, Debug)]
struct YahooQuoteResponse {
    #[serde(rename = "quoteSummary")]
    quote_summary: Option<QuoteSummary>,
}

#[derive(Deserialize, Debug)]
struct QuoteSummary {
    result: Option<Vec<QuoteResult>>,
}

#[derive(Deserialize, Debug)]
struct QuoteResult {
    #[serde(rename = "defaultKeyStatistics")]
    default_key_statistics: Option<serde_json::Value>,
    #[serde(rename = "financialData")]
    financial_data: Option<serde_json::Value>,
    #[serde(rename = "summaryDetail")]
    summary_detail: Option<serde_json::Value>,
}

impl IDXDataScraper {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("building HTTP client")?;

        Ok(Self { client })
    }

    /// Fetch stock metrics from Yahoo Finance for IDX ticker
    pub async fn fetch_stock_metrics(&self, ticker: &str) -> Result<StockMetrics> {
        let yahoo_ticker = format!("{}.JK", ticker);
        let url = format!(
            "https://query1.finance.yahoo.com/v10/finance/quoteSummary/{}?modules=defaultKeyStatistics,financialData,summaryDetail",
            yahoo_ticker
        );

        info!("📥 Fetching Yahoo Finance data for {}", ticker);

        let resp = self.client
            .get(&url)
            .send()
            .await
            .context("fetching Yahoo Finance")?;

        if !resp.status().is_success() {
            warn!("⚠️ Yahoo Finance returned {} for {}", resp.status(), ticker);
            return Ok(StockMetrics {
                ticker: ticker.to_string(),
                ..Default::default()
            });
        }

        let body: serde_json::Value = resp.json().await
            .context("parsing Yahoo Finance JSON")?;

        // Extract from nested JSON
        let result = body
            .pointer("/quoteSummary/result/0")
            .unwrap_or(&serde_json::Value::Null);

        let financial = result.get("financialData").unwrap_or(&serde_json::Value::Null);
        let summary = result.get("summaryDetail").unwrap_or(&serde_json::Value::Null);
        let stats = result.get("defaultKeyStatistics").unwrap_or(&serde_json::Value::Null);

        let extract_raw = |val: &serde_json::Value, key: &str| -> f64 {
            val.get(key)
                .and_then(|v| v.get("raw"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
        };

        Ok(StockMetrics {
            ticker: ticker.to_string(),
            current_price: extract_raw(financial, "currentPrice"),
            per: extract_raw(summary, "trailingPE"),
            pbv: extract_raw(summary, "priceToBook"),
            roe: extract_raw(financial, "returnOnEquity") * 100.0,
            roa: extract_raw(financial, "returnOnAssets") * 100.0,
            der: extract_raw(financial, "debtToEquity") / 100.0, // Yahoo returns as percentage
            dy: extract_raw(summary, "dividendYield") * 100.0,
            eps: extract_raw(summary, "trailingEps"),
            bv_per_share: extract_raw(stats, "bookValue"),
            npm: extract_raw(financial, "profitMargins") * 100.0,
            high_52w: extract_raw(summary, "fiftyTwoWeekHigh"),
            low_52w: extract_raw(summary, "fiftyTwoWeekLow"),
            market_cap_trn: extract_raw(summary, "marketCap") / 1_000_000_000_000.0,
            ..Default::default()
        })
    }

    /// Fetch IDX profile (enriched data)
    pub async fn fetch_idx_profile(&self, ticker: &str) -> IdxProfile {
        // For now, return defaults with estimated data
        // In production, this would scrape IDX or use Notion cache
        IdxProfile {
            name: ticker.to_string(),
            free_float_pct: 35.0, // Default assumption for IDX
            ownership: OwnershipData {
                institutional: 40.0,
                retail: 40.0,
                foreign: 20.0,
            },
            liquidity: LiquidityData {
                bid_ask_spread_pct: 0.3,
                volume_trend_30d_avg: 50_000_000.0,
                liquidity_rating: "Medium".to_string(),
            },
            dividend_history: vec![],
            debt_structure: DebtStructure::default(),
        }
    }

    /// Fetch complete profile (metrics + IDX extras)
    pub async fn fetch_complete(&self, ticker: &str) -> Result<(StockMetrics, IdxProfile)> {
        let metrics = self.fetch_stock_metrics(ticker).await?;
        let profile = self.fetch_idx_profile(ticker).await;
        Ok((metrics, profile))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scraper_creation() {
        let scraper = IDXDataScraper::new();
        assert!(scraper.is_ok());
    }

    // Integration test — requires network access
    // #[tokio::test]
    // async fn test_fetch_bbri() {
    //     let scraper = IDXDataScraper::new().unwrap();
    //     let metrics = scraper.fetch_stock_metrics("BBRI").await.unwrap();
    //     assert_eq!(metrics.ticker, "BBRI");
    //     assert!(metrics.current_price > 0.0);
    // }
}
