//! FRED (Federal Reserve Economic Data) API client
//!
//! Fetches key US economic indicators from the St. Louis Fed API.
//! Gracefully degrades if FRED_API_KEY is not set (logs warning, returns empty stats).
//!
//! Series collected:
//! - FEDFUNDS: Fed Funds Rate (percent)
//! - CPIAUCSL: Consumer Price Index (index)
//! - UNRATE: Unemployment Rate (percent)
//! - GDP: Gross Domestic Product (billions_usd)
//! - DGS10: 10-Year Treasury Yield (percent)
//! - DTWEXBGS: Trade-Weighted USD Index (index)

use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tracing::{info, warn, error};

use crate::arangodb::ArangoClient;
use crate::economic::models::{EconomicIndicator, EconomicSource, EconomicStats, SignalSourceEdge};

/// FRED API base URL
const FRED_API_BASE: &str = "https://api.stlouisfed.org/fred/series/observations";

/// Stagger between API requests (200ms)
const REQUEST_STAGGER_MS: u64 = 200;

/// Series definitions: (series_id, name, unit, description)
const FRED_SERIES: &[(&str, &str, &str, &str)] = &[
    ("FEDFUNDS", "Fed Funds Rate", "percent", "US benchmark interest rate"),
    ("CPIAUCSL", "CPI", "index", "US Consumer Price Index (inflation)"),
    ("UNRATE", "Unemployment", "percent", "US unemployment rate"),
    ("GDP", "GDP", "billions_usd", "US quarterly Gross Domestic Product"),
    ("DGS10", "10Y Treasury", "percent", "US 10-year Treasury yield"),
    ("DTWEXBGS", "USD Index", "index", "Trade-weighted US Dollar strength"),
];

/// FRED API observation response
#[derive(Debug, Deserialize)]
struct FredResponse {
    observations: Vec<FredObservation>,
}

/// A single FRED observation data point
#[derive(Debug, Deserialize)]
pub struct FredObservation {
    pub date: String,
    pub value: String,
}

/// FRED economic data collector
pub struct FredCollector {
    client: Client,
    api_key: Option<String>,
}

impl FredCollector {
    /// Create a new FredCollector, reading FRED_API_KEY from environment.
    /// If the key is not set, collection will be skipped gracefully.
    pub fn new() -> Self {
        let api_key = std::env::var("FRED_API_KEY").ok();
        if api_key.is_none() {
            warn!("⚠️ FRED_API_KEY not set — FRED data collection disabled");
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client for FRED");

        Self { client, api_key }
    }

    /// Collect all FRED series and store in ArangoDB.
    /// Returns empty stats if API key is not configured (no error).
    pub async fn collect_all(&self, arango: &ArangoClient) -> Result<EconomicStats> {
        let api_key = match &self.api_key {
            Some(key) => key,
            None => {
                info!("⏭️ Skipping FRED collection (no API key)");
                return Ok(EconomicStats::default());
            }
        };

        let mut stats = EconomicStats::default();

        info!("📊 FRED: Fetching {} economic series...", FRED_SERIES.len());

        for (i, (series_id, name, unit, description)) in FRED_SERIES.iter().enumerate() {
            // 200ms stagger between requests (skip first)
            if i > 0 {
                tokio::time::sleep(Duration::from_millis(REQUEST_STAGGER_MS)).await;
            }

            match self.fetch_series(api_key, series_id).await {
                Ok(Some(obs)) => {
                    match self.store_indicator(arango, series_id, name, unit, description, &obs).await {
                        Ok(_) => {
                            stats.indicators_inserted += 1;
                            info!("✅ FRED {}: {} = {} ({})", series_id, name, obs.value, obs.date);
                        }
                        Err(e) => {
                            stats.errors += 1;
                            error!("❌ FRED {} store error: {}", series_id, e);
                        }
                    }
                }
                Ok(None) => {
                    info!("⏭️ FRED {}: no valid data point available", series_id);
                }
                Err(e) => {
                    stats.errors += 1;
                    error!("❌ FRED {} fetch error: {}", series_id, e);
                }
            }
        }

        info!("📊 FRED collection complete: {}", stats);
        Ok(stats)
    }

    /// Fetch the latest observation for a single FRED series.
    /// Returns None if no valid data point exists (e.g., value is ".").
    async fn fetch_series(&self, api_key: &str, series_id: &str) -> Result<Option<FredObservation>> {
        let url = format!(
            "{}?series_id={}&api_key={}&file_type=json&sort_order=desc&limit=1",
            FRED_API_BASE, series_id, api_key
        );

        let resp = self.client
            .get(&url)
            .send()
            .await
            .context(format!("FRED API request for {}", series_id))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("FRED API error for {}: status={}, body={}", series_id, status, body);
        }

        let fred_resp: FredResponse = resp.json().await
            .context(format!("parsing FRED response for {}", series_id))?;

        // Find first observation with a valid numeric value
        // FRED uses "." for missing data
        let valid_obs = fred_resp.observations.into_iter().find(|obs| {
            obs.value != "." && obs.value.parse::<f64>().is_ok()
        });

        Ok(valid_obs)
    }

    /// Store an indicator in ArangoDB and create signal_source edge.
    async fn store_indicator(
        &self,
        arango: &ArangoClient,
        series_id: &str,
        name: &str,
        unit: &str,
        description: &str,
        obs: &FredObservation,
    ) -> Result<()> {
        let value: f64 = obs.value.parse()
            .context(format!("parsing value '{}' for {}", obs.value, series_id))?;

        let date = NaiveDate::parse_from_str(&obs.date, "%Y-%m-%d")
            .context(format!("parsing date '{}' for {}", obs.date, series_id))?;

        let timestamp = date.and_hms_opt(0, 0, 0)
            .context("invalid time component")?
            .and_utc();

        let key = format!("fred_{}_{}", series_id.to_lowercase(), obs.date);

        let indicator = EconomicIndicator {
            key: key.clone(),
            source: EconomicSource::Fred,
            indicator: series_id.to_string(),
            value,
            unit: unit.to_string(),
            timestamp,
            change_pct: None,
            metadata: serde_json::json!({
                "name": name,
                "description": description,
                "date": obs.date,
                "series_id": series_id,
            }),
        };

        // Insert indicator document
        let doc = serde_json::to_value(&indicator)
            .context("serializing FRED indicator")?;
        arango.insert_document("economic_indicators", &doc).await
            .context(format!("inserting FRED indicator {}", key))?;

        // Create signal_source edge linking indicator to market context
        let edge = SignalSourceEdge {
            from: format!("economic_indicators/{}", key),
            to: "market_context/us_economy".to_string(),
            strength: 0.8,
            direction: "neutral".to_string(),
            created_at: Utc::now(),
        };

        let edge_data = serde_json::to_value(&edge)
            .context("serializing signal_source edge")?;
        arango.insert_edge(
            "signal_source",
            &edge.from,
            &edge.to,
            &edge_data,
        ).await
            .context(format!("inserting signal_source edge for {}", key))?;

        Ok(())
    }
}

/// Parse a FRED API JSON response into observations.
/// Exported for testing purposes.
pub fn parse_fred_response(json: &str) -> Result<Vec<FredObservation>> {
    let resp: FredResponse = serde_json::from_str(json)
        .context("parsing FRED JSON response")?;
    Ok(resp.observations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_parse_fred_response_valid() {
        let json = r#"{
            "observations": [
                {"date": "2024-01-01", "value": "5.33"}
            ]
        }"#;

        let observations = parse_fred_response(json).unwrap();
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].date, "2024-01-01");
        assert_eq!(observations[0].value, "5.33");
    }

    #[test]
    fn test_parse_fred_response_multiple_observations() {
        let json = r#"{
            "observations": [
                {"date": "2024-03-01", "value": "5.33"},
                {"date": "2024-02-01", "value": "5.25"},
                {"date": "2024-01-01", "value": "5.50"}
            ]
        }"#;

        let observations = parse_fred_response(json).unwrap();
        assert_eq!(observations.len(), 3);
        assert_eq!(observations[0].value, "5.33");
        assert_eq!(observations[1].value, "5.25");
        assert_eq!(observations[2].value, "5.50");
    }

    #[test]
    fn test_parse_fred_response_missing_data_dot() {
        // FRED uses "." for missing data
        let json = r#"{
            "observations": [
                {"date": "2024-01-01", "value": "."}
            ]
        }"#;

        let observations = parse_fred_response(json).unwrap();
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].value, ".");
        // Verify "." is not parseable as f64
        assert!(observations[0].value.parse::<f64>().is_err());
    }

    #[test]
    fn test_parse_fred_response_empty_observations() {
        let json = r#"{
            "observations": []
        }"#;

        let observations = parse_fred_response(json).unwrap();
        assert!(observations.is_empty());
    }

    #[test]
    fn test_parse_fred_response_invalid_json() {
        let json = r#"{ invalid json }"#;
        let result = parse_fred_response(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_fred_key_format() {
        let series_id = "FEDFUNDS";
        let date = "2024-01-01";
        let key = format!("fred_{}_{}", series_id.to_lowercase(), date);
        assert_eq!(key, "fred_fedfunds_2024-01-01");
    }

    #[test]
    fn test_fred_key_format_all_series() {
        let expected_keys = vec![
            ("FEDFUNDS", "2024-01-01", "fred_fedfunds_2024-01-01"),
            ("CPIAUCSL", "2024-02-01", "fred_cpiaucsl_2024-02-01"),
            ("UNRATE", "2024-03-01", "fred_unrate_2024-03-01"),
            ("GDP", "2024-01-01", "fred_gdp_2024-01-01"),
            ("DGS10", "2024-03-15", "fred_dgs10_2024-03-15"),
            ("DTWEXBGS", "2024-03-01", "fred_dtwexbgs_2024-03-01"),
        ];

        for (series_id, date, expected) in expected_keys {
            let key = format!("fred_{}_{}", series_id.to_lowercase(), date);
            assert_eq!(key, expected);
        }
    }

    #[test]
    fn test_missing_value_filter() {
        // Simulate the filtering logic from fetch_series
        let observations = vec![
            FredObservation { date: "2024-03-01".to_string(), value: ".".to_string() },
            FredObservation { date: "2024-02-01".to_string(), value: "5.33".to_string() },
        ];

        let valid = observations.into_iter().find(|obs| {
            obs.value != "." && obs.value.parse::<f64>().is_ok()
        });

        assert!(valid.is_some());
        let obs = valid.unwrap();
        assert_eq!(obs.date, "2024-02-01");
        assert_eq!(obs.value, "5.33");
    }

    #[test]
    fn test_missing_value_all_dots() {
        let observations = vec![
            FredObservation { date: "2024-03-01".to_string(), value: ".".to_string() },
            FredObservation { date: "2024-02-01".to_string(), value: ".".to_string() },
        ];

        let valid = observations.into_iter().find(|obs| {
            obs.value != "." && obs.value.parse::<f64>().is_ok()
        });

        assert!(valid.is_none());
    }

    #[test]
    fn test_fred_series_count() {
        assert_eq!(FRED_SERIES.len(), 6);
    }

    #[test]
    fn test_fred_series_ids() {
        let ids: Vec<&str> = FRED_SERIES.iter().map(|(id, _, _, _)| *id).collect();
        assert!(ids.contains(&"FEDFUNDS"));
        assert!(ids.contains(&"CPIAUCSL"));
        assert!(ids.contains(&"UNRATE"));
        assert!(ids.contains(&"GDP"));
        assert!(ids.contains(&"DGS10"));
        assert!(ids.contains(&"DTWEXBGS"));
    }

    #[test]
    fn test_fred_collector_new_without_key() {
        // Ensure FRED_API_KEY is not set for this test
        std::env::remove_var("FRED_API_KEY");
        let collector = FredCollector::new();
        assert!(collector.api_key.is_none());
    }

    #[test]
    fn test_parse_fred_observation_value() {
        let obs = FredObservation {
            date: "2024-01-01".to_string(),
            value: "5.33".to_string(),
        };
        let parsed: f64 = obs.value.parse().unwrap();
        assert!((parsed - 5.33).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_fred_date() {
        let date_str = "2024-01-01";
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 1);
    }

    #[test]
    fn test_indicator_construction() {
        let obs = FredObservation {
            date: "2024-06-15".to_string(),
            value: "5.50".to_string(),
        };

        let value: f64 = obs.value.parse().unwrap();
        let date = NaiveDate::parse_from_str(&obs.date, "%Y-%m-%d").unwrap();
        let timestamp = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let key = format!("fred_{}_{}", "fedfunds", obs.date);

        let indicator = EconomicIndicator {
            key: key.clone(),
            source: EconomicSource::Fred,
            indicator: "FEDFUNDS".to_string(),
            value,
            unit: "percent".to_string(),
            timestamp,
            change_pct: None,
            metadata: serde_json::json!({
                "name": "Fed Funds Rate",
                "description": "US benchmark interest rate",
                "date": obs.date,
                "series_id": "FEDFUNDS",
            }),
        };

        assert_eq!(indicator.key, "fred_fedfunds_2024-06-15");
        assert_eq!(indicator.source, EconomicSource::Fred);
        assert_eq!(indicator.indicator, "FEDFUNDS");
        assert_eq!(indicator.value, 5.50);
        assert_eq!(indicator.unit, "percent");
        assert!(indicator.change_pct.is_none());
    }

    #[tokio::test]
    #[ignore] // Integration test: requires FRED_API_KEY and running ArangoDB
    async fn test_fred_collect_all_integration() {
        let collector = FredCollector::new();
        if collector.api_key.is_none() {
            println!("Skipping: FRED_API_KEY not set");
            return;
        }
        let arango = crate::arangodb::ArangoClient::new().unwrap();
        let stats = collector.collect_all(&arango).await.unwrap();
        assert!(stats.indicators_inserted > 0);
        assert_eq!(stats.errors, 0);
    }
}
