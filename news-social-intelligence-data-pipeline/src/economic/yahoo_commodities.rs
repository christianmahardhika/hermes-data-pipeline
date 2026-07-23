//! Yahoo Finance Commodity Collector
//!
//! Fetches 9 Indonesian-market-relevant commodity prices from Yahoo Finance
//! v8 chart endpoint (no API key needed), converts to EconomicIndicator,
//! and stores in ArangoDB with signal_source edges to affected tickers.

use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Deserialize;
use tracing::{info, warn};

use crate::arangodb::ArangoClient;
use crate::economic::models::{EconomicIndicator, EconomicSource, EconomicStats, SignalSourceEdge};

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Stagger delay between requests to respect Yahoo Finance rate limits
const STAGGER_MS: u64 = 150;

/// Yahoo Finance v8 chart endpoint base URL
const YAHOO_BASE_URL: &str = "https://query1.finance.yahoo.com/v8/finance/chart";

/// A commodity symbol with metadata
struct CommoditySymbol {
    symbol: &'static str,
    name: &'static str,
    unit: &'static str,
    related_tickers: &'static [&'static str],
}

/// All 11 commodity symbols relevant to Indonesian market & Pagupon portfolio
/// Includes coffee futures for Pondo Ngopi cost intelligence (raw material + PET packaging via oil)
const COMMODITY_SYMBOLS: &[CommoditySymbol] = &[
    CommoditySymbol {
        symbol: "GC=F",
        name: "Gold",
        unit: "USD/oz",
        related_tickers: &["ANTM", "MDKA"],
    },
    CommoditySymbol {
        symbol: "CL=F",
        name: "Crude Oil WTI",
        unit: "USD/bbl",
        related_tickers: &["MEDC", "ELSA"],
    },
    CommoditySymbol {
        symbol: "BZ=F",
        name: "Brent Crude",
        unit: "USD/bbl",
        related_tickers: &["MEDC", "ELSA"],
    },
    CommoditySymbol {
        symbol: "FCPO=F",
        name: "Palm Oil",
        unit: "MYR/ton",
        related_tickers: &["AALI", "LSIP", "SIMP"],
    },
    CommoditySymbol {
        symbol: "SI=F",
        name: "Silver",
        unit: "USD/oz",
        related_tickers: &["ANTM"],
    },
    CommoditySymbol {
        symbol: "HG=F",
        name: "Copper",
        unit: "USD/lb",
        related_tickers: &["TINS"],
    },
    CommoditySymbol {
        symbol: "NG=F",
        name: "Natural Gas",
        unit: "USD/MMBtu",
        related_tickers: &["PGAS"],
    },
    CommoditySymbol {
        symbol: "NI=F",
        name: "Nickel",
        unit: "USD/ton",
        related_tickers: &["INCO", "ANTM"],
    },
    CommoditySymbol {
        symbol: "ALI=F",
        name: "Aluminum",
        unit: "USD/ton",
        related_tickers: &[],
    },
    // Coffee futures — Pondo Ngopi COGS intelligence
    // Arabica: specialty/single-origin grade
    // Robusta: mass-market/blend/RTD/espresso grade
    CommoditySymbol {
        symbol: "KC=F",
        name: "Coffee Arabica",
        unit: "USc/lb",
        related_tickers: &[],
    },
    CommoditySymbol {
        symbol: "RC=F",
        name: "Coffee Robusta",
        unit: "USD/ton",
        related_tickers: &[],
    },
];

/// Yahoo Finance v8 chart API response (partial)
#[derive(Debug, Deserialize)]
struct YahooChartResponse {
    chart: ChartData,
}

#[derive(Debug, Deserialize)]
struct ChartData {
    result: Option<Vec<ChartResult>>,
}

#[derive(Debug, Deserialize)]
struct ChartResult {
    meta: ChartMeta,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChartMeta {
    regular_market_price: Option<f64>,
    previous_close: Option<f64>,
}

/// Yahoo Finance commodity collector
pub struct YahooCommodityCollector {
    client: reqwest::Client,
}

impl YahooCommodityCollector {
    /// Create a new collector with configured HTTP client
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent(USER_AGENT)
            .build()
            .context("building Yahoo Finance HTTP client")?;

        Ok(Self { client })
    }

    /// Collect all commodity prices with 150ms stagger between requests.
    /// Stores indicators and signal_source edges in ArangoDB.
    /// Continues on individual symbol failures.
    pub async fn collect_all(&self, arango: &ArangoClient) -> Result<EconomicStats> {
        info!("📊 Collecting {} commodity prices from Yahoo Finance", COMMODITY_SYMBOLS.len());

        let mut stats = EconomicStats::default();

        for (i, commodity) in COMMODITY_SYMBOLS.iter().enumerate() {
            // Stagger between requests (skip delay for first)
            if i > 0 {
                tokio::time::sleep(Duration::from_millis(STAGGER_MS)).await;
            }

            match self.fetch_and_store(commodity, arango).await {
                Ok((indicators, edges)) => {
                    stats.indicators_inserted += indicators;
                    stats.edges_created += edges;
                }
                Err(e) => {
                    warn!("⚠️ Failed to fetch {}: {}", commodity.symbol, e);
                    stats.errors += 1;
                }
            }
        }

        info!("✅ Commodity collection complete: {}", stats);
        Ok(stats)
    }

    /// Fetch price for a single commodity and store in ArangoDB
    async fn fetch_and_store(
        &self,
        commodity: &CommoditySymbol,
        arango: &ArangoClient,
    ) -> Result<(usize, usize)> {
        let (current_price, previous_close) = self.fetch_price(commodity.symbol).await?;
        let change_pct = if previous_close > 0.0 {
            Some((current_price - previous_close) / previous_close * 100.0)
        } else {
            None
        };

        let now = Utc::now();
        let indicator_key = format!(
            "yahoo_{}_{}", 
            commodity.symbol.replace('=', "_").to_lowercase(),
            now.format("%Y%m%d_%H%M")
        );

        let indicator = EconomicIndicator {
            key: indicator_key.clone(),
            source: EconomicSource::YahooFinance,
            indicator: commodity.name.to_uppercase().replace(' ', "_"),
            value: current_price,
            unit: commodity.unit.to_string(),
            timestamp: now,
            change_pct,
            metadata: serde_json::json!({
                "symbol": commodity.symbol,
                "previous_close": previous_close,
                "related_tickers": commodity.related_tickers,
            }),
        };

        // Store indicator in ArangoDB
        let doc = serde_json::to_value(&indicator)
            .context("serializing indicator")?;
        arango
            .insert_document("economic_indicators", &doc)
            .await
            .context("inserting economic indicator")?;

        // Create signal_source edges to related tickers
        let mut edges_created = 0;
        for ticker in commodity.related_tickers {
            let _edge = SignalSourceEdge {
                from: format!("economic_indicators/{}", indicator_key),
                to: format!("tickers/{}", ticker),
                strength: compute_signal_strength(change_pct),
                direction: compute_direction(change_pct),
                created_at: now,
            };

            let edge_data = serde_json::json!({
                "strength": compute_signal_strength(change_pct),
                "direction": compute_direction(change_pct),
                "created_at": now.to_rfc3339(),
                "commodity": commodity.name,
                "change_pct": change_pct,
            });

            arango
                .insert_edge(
                    "signal_source",
                    &format!("economic_indicators/{}", indicator_key),
                    &format!("tickers/{}", ticker),
                    &edge_data,
                )
                .await
                .context("inserting signal_source edge")?;

            edges_created += 1;
        }

        info!(
            "📈 {} ({}) = {:.2} {} (Δ {:.2}%)",
            commodity.name,
            commodity.symbol,
            current_price,
            commodity.unit,
            change_pct.unwrap_or(0.0)
        );

        Ok((1, edges_created))
    }

    /// Fetch current price and previous close from Yahoo Finance v8 chart API
    async fn fetch_price(&self, symbol: &str) -> Result<(f64, f64)> {
        let url = format!("{}?interval=1d&range=1d", symbol_url(symbol));

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Yahoo Finance request failed")?;

        if !resp.status().is_success() {
            anyhow::bail!(
                "Yahoo Finance returned HTTP {}: {}",
                resp.status(),
                symbol
            );
        }

        let body: YahooChartResponse = resp
            .json()
            .await
            .context("parsing Yahoo Finance JSON response")?;

        let result = body
            .chart
            .result
            .and_then(|r| r.into_iter().next())
            .context("no chart result in Yahoo Finance response")?;

        let current_price = result
            .meta
            .regular_market_price
            .context("missing regularMarketPrice")?;

        let previous_close = result.meta.previous_close.unwrap_or(current_price);

        Ok((current_price, previous_close))
    }
}

/// Build Yahoo Finance chart URL for a symbol
fn symbol_url(symbol: &str) -> String {
    format!("{}/{}", YAHOO_BASE_URL, urlencoding::encode(symbol))
}

/// Compute signal strength based on percentage change magnitude
fn compute_signal_strength(change_pct: Option<f64>) -> f64 {
    match change_pct {
        Some(pct) => {
            let abs_pct = pct.abs();
            if abs_pct >= 5.0 {
                0.9
            } else if abs_pct >= 3.0 {
                0.7
            } else if abs_pct >= 1.0 {
                0.5
            } else {
                0.3
            }
        }
        None => 0.1,
    }
}

/// Compute direction of impact from price change
fn compute_direction(change_pct: Option<f64>) -> String {
    match change_pct {
        Some(pct) if pct > 0.5 => "positive".to_string(),
        Some(pct) if pct < -0.5 => "negative".to_string(),
        _ => "neutral".to_string(),
    }
}

/// Parse a Yahoo Finance chart response JSON into (current_price, previous_close).
/// Exposed for unit testing.
pub fn parse_yahoo_response(json: &str) -> Result<(f64, f64)> {
    let body: YahooChartResponse =
        serde_json::from_str(json).context("parsing Yahoo Finance JSON")?;

    let result = body
        .chart
        .result
        .and_then(|r| r.into_iter().next())
        .context("no chart result")?;

    let current_price = result
        .meta
        .regular_market_price
        .context("missing regularMarketPrice")?;

    let previous_close = result.meta.previous_close.unwrap_or(current_price);

    Ok((current_price, previous_close))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sample Yahoo Finance v8 chart API response for GC=F (Gold)
    const MOCK_GOLD_RESPONSE: &str = r#"{
        "chart": {
            "result": [
                {
                    "meta": {
                        "currency": "USD",
                        "symbol": "GC=F",
                        "exchangeName": "CMX",
                        "fullExchangeName": "COMEX",
                        "instrumentType": "FUTURE",
                        "regularMarketPrice": 2045.30,
                        "previousClose": 2038.50
                    }
                }
            ],
            "error": null
        }
    }"#;

    /// Response with no previousClose
    const MOCK_NO_PREVIOUS_CLOSE: &str = r#"{
        "chart": {
            "result": [
                {
                    "meta": {
                        "currency": "USD",
                        "symbol": "SI=F",
                        "regularMarketPrice": 23.45
                    }
                }
            ],
            "error": null
        }
    }"#;

    /// Response with null result
    const MOCK_NULL_RESULT: &str = r#"{
        "chart": {
            "result": null,
            "error": {
                "code": "Not Found",
                "description": "No data found"
            }
        }
    }"#;

    /// Empty result array
    const MOCK_EMPTY_RESULT: &str = r#"{
        "chart": {
            "result": [],
            "error": null
        }
    }"#;

    #[test]
    fn test_parse_yahoo_response_gold() {
        let (price, prev) = parse_yahoo_response(MOCK_GOLD_RESPONSE).unwrap();
        assert!((price - 2045.30).abs() < f64::EPSILON);
        assert!((prev - 2038.50).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_yahoo_response_no_previous_close() {
        let (price, prev) = parse_yahoo_response(MOCK_NO_PREVIOUS_CLOSE).unwrap();
        assert!((price - 23.45).abs() < f64::EPSILON);
        // Falls back to current price when previousClose is missing
        assert!((prev - 23.45).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_yahoo_response_null_result() {
        let result = parse_yahoo_response(MOCK_NULL_RESULT);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no chart result"));
    }

    #[test]
    fn test_parse_yahoo_response_empty_result() {
        let result = parse_yahoo_response(MOCK_EMPTY_RESULT);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no chart result"));
    }

    #[test]
    fn test_change_pct_calculation() {
        let current: f64 = 2045.30;
        let previous: f64 = 2038.50;
        let change_pct = (current - previous) / previous * 100.0;
        // Expected: (6.8 / 2038.5) * 100 ≈ 0.3336..%
        assert!((change_pct - 0.3336).abs() < 0.01);
    }

    #[test]
    fn test_compute_signal_strength() {
        assert!((compute_signal_strength(Some(6.0)) - 0.9).abs() < f64::EPSILON);
        assert!((compute_signal_strength(Some(4.0)) - 0.7).abs() < f64::EPSILON);
        assert!((compute_signal_strength(Some(2.0)) - 0.5).abs() < f64::EPSILON);
        assert!((compute_signal_strength(Some(0.5)) - 0.3).abs() < f64::EPSILON);
        assert!((compute_signal_strength(None) - 0.1).abs() < f64::EPSILON);
        // Negative percentages should use absolute value
        assert!((compute_signal_strength(Some(-5.5)) - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_direction() {
        assert_eq!(compute_direction(Some(2.0)), "positive");
        assert_eq!(compute_direction(Some(-2.0)), "negative");
        assert_eq!(compute_direction(Some(0.3)), "neutral");
        assert_eq!(compute_direction(Some(-0.3)), "neutral");
        assert_eq!(compute_direction(None), "neutral");
    }

    #[test]
    fn test_symbol_url() {
        assert_eq!(
            symbol_url("GC=F"),
            "https://query1.finance.yahoo.com/v8/finance/chart/GC%3DF"
        );
        assert_eq!(
            symbol_url("CL=F"),
            "https://query1.finance.yahoo.com/v8/finance/chart/CL%3DF"
        );
    }

    #[test]
    fn test_commodity_symbols_count() {
        assert_eq!(COMMODITY_SYMBOLS.len(), 11);
    }

    #[test]
    fn test_indicator_from_parsed_response() {
        let (price, prev) = parse_yahoo_response(MOCK_GOLD_RESPONSE).unwrap();
        let change_pct = (price - prev) / prev * 100.0;

        let indicator = EconomicIndicator {
            key: "yahoo_gc_f_20240101_1200".to_string(),
            source: EconomicSource::YahooFinance,
            indicator: "GOLD".to_string(),
            value: price,
            unit: "USD/oz".to_string(),
            timestamp: Utc::now(),
            change_pct: Some(change_pct),
            metadata: serde_json::json!({
                "symbol": "GC=F",
                "previous_close": prev,
                "related_tickers": ["ANTM", "MDKA"],
            }),
        };

        assert_eq!(indicator.source, EconomicSource::YahooFinance);
        assert!((indicator.value - 2045.30).abs() < f64::EPSILON);
        assert_eq!(indicator.unit, "USD/oz");
        assert!(indicator.change_pct.unwrap() > 0.0);
    }

    #[test]
    fn test_collector_creation() {
        let collector = YahooCommodityCollector::new();
        assert!(collector.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires network access to Yahoo Finance
    async fn test_fetch_gold_price_integration() {
        let collector = YahooCommodityCollector::new().unwrap();
        let result = collector.fetch_price("GC=F").await;
        assert!(result.is_ok(), "should fetch gold price: {:?}", result.err());
        let (price, prev) = result.unwrap();
        assert!(price > 0.0, "gold price should be positive");
        assert!(prev > 0.0, "previous close should be positive");
    }

    #[tokio::test]
    #[ignore] // Requires network + ArangoDB
    async fn test_collect_all_integration() {
        let collector = YahooCommodityCollector::new().unwrap();
        let arango = ArangoClient::new().unwrap();
        let stats = collector.collect_all(&arango).await.unwrap();
        // At least some should succeed
        assert!(stats.indicators_inserted > 0);
    }
}
