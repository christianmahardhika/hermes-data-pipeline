//! Data Source — Yahoo Finance + mock data

use anyhow::{Result, Context};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tracing::warn;

use crate::idx_analyst::models::StockData;

/// Yahoo Finance data fetcher
pub struct YahooFinanceSource {
    client: Client,
}

impl YahooFinanceSource {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
            .build()
            .context("building HTTP client")?;
        Ok(Self { client })
    }

    /// Fetch via chart API (no auth required)
    pub async fn fetch(&self, ticker: &str) -> Result<StockData> {
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}.JK?interval=1d&range=1d",
            ticker
        );

        let resp = self.client.get(&url).send().await.context("fetching chart")?;
        if !resp.status().is_success() {
            anyhow::bail!("Yahoo returned {}", resp.status());
        }

        let data: YahooChartResponse = resp.json().await?;
        let meta = &data.chart.result.first().context("no result")?.meta;

        Ok(StockData {
            ticker: ticker.to_string(),
            current_price: meta.regular_market_price.unwrap_or(0.0),
            high_52w: meta.fifty_two_week_high.unwrap_or(0.0),
            low_52w: meta.fifty_two_week_low.unwrap_or(0.0),
            ..Default::default()
        })
    }

    /// Fetch full fundamentals via quoteSummary
    pub async fn fetch_fundamentals(&self, ticker: &str) -> Result<StockData> {
        let url = format!(
            "https://query2.finance.yahoo.com/v10/finance/quoteSummary/{}.JK?modules=defaultKeyStatistics,financialData,summaryDetail",
            ticker
        );

        let resp = self.client.get(&url).send().await.context("fetching quoteSummary")?;
        if !resp.status().is_success() {
            warn!("quoteSummary failed for {}, fallback to chart", ticker);
            return self.fetch(ticker).await;
        }

        let body: serde_json::Value = resp.json().await?;
        let result = &body["quoteSummary"]["result"][0];
        let fin = &result["financialData"];
        let stats = &result["defaultKeyStatistics"];
        let summary = &result["summaryDetail"];

        Ok(StockData {
            ticker: ticker.to_string(),
            current_price: raw(fin, "currentPrice"),
            roe: raw(fin, "returnOnEquity") * 100.0,
            roa: raw(fin, "returnOnAssets") * 100.0,
            npm: raw(fin, "profitMargins") * 100.0,
            der: raw(fin, "debtToEquity") / 100.0,
            per: raw(summary, "trailingPE"),
            pbv: raw(stats, "priceToBook"),
            dy: raw(summary, "dividendYield") * 100.0,
            eps: raw(summary, "trailingEps"),
            high_52w: raw(summary, "fiftyTwoWeekHigh"),
            low_52w: raw(summary, "fiftyTwoWeekLow"),
            ..Default::default()
        })
    }
}

fn raw(obj: &serde_json::Value, field: &str) -> f64 {
    obj[field]["raw"].as_f64().unwrap_or(0.0)
}

#[derive(Deserialize)]
struct YahooChartResponse { chart: ChartData }

#[derive(Deserialize)]
struct ChartData { result: Vec<ChartResult> }

#[derive(Deserialize)]
struct ChartResult { meta: ChartMeta }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChartMeta {
    regular_market_price: Option<f64>,
    fifty_two_week_high: Option<f64>,
    fifty_two_week_low: Option<f64>,
}

/// Mock data for testing
pub fn mock_stock_data(ticker: &str) -> StockData {
    match ticker {
        "BMRI" => StockData {
            ticker: "BMRI".into(), current_price: 6250.0,
            per: 6.8, pbv: 1.3, roe: 21.0, roa: 2.6, der: 0.5, dy: 11.2,
            eps: 626.65, bv_per_share: 3270.59, npm: 40.2,
            revenue_trn: 145.3, net_income_trn: 58.5, market_cap_trn: 582.0,
            high_52w: 7500.0, low_52w: 5100.0, sentiment_score: 0.3,
        },
        "BBRI" => StockData {
            ticker: "BBRI".into(), current_price: 4260.0,
            per: 7.2, pbv: 1.8, roe: 19.5, roa: 2.3, der: 0.6, dy: 8.5,
            eps: 420.0, bv_per_share: 2367.0, npm: 35.0,
            revenue_trn: 180.0, net_income_trn: 63.0, market_cap_trn: 640.0,
            high_52w: 5375.0, low_52w: 3650.0, sentiment_score: 0.2,
        },
        "PTBA" => StockData {
            ticker: "PTBA".into(), current_price: 2800.0,
            per: 5.5, pbv: 1.1, roe: 25.0, roa: 18.0, der: 0.3, dy: 12.0,
            eps: 509.0, bv_per_share: 2545.0, npm: 22.0,
            revenue_trn: 35.0, net_income_trn: 7.7, market_cap_trn: 32.0,
            high_52w: 3500.0, low_52w: 2200.0, sentiment_score: 0.4,
        },
        _ => StockData {
            ticker: ticker.into(), current_price: 1000.0,
            per: 12.0, pbv: 1.5, roe: 12.0, roa: 8.0, der: 0.7, dy: 4.0,
            eps: 83.0, bv_per_share: 667.0, npm: 10.0,
            revenue_trn: 10.0, net_income_trn: 1.0, market_cap_trn: 20.0,
            high_52w: 1300.0, low_52w: 800.0, sentiment_score: 0.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_known_ticker() {
        let d = mock_stock_data("BMRI");
        assert_eq!(d.ticker, "BMRI");
        assert!(d.current_price > 0.0);
        assert!(d.roe > 0.0);
    }

    #[test]
    fn test_mock_unknown_ticker() {
        let d = mock_stock_data("ZZZZ");
        assert_eq!(d.current_price, 1000.0);
    }
}
