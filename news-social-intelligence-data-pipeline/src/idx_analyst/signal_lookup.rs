//! Graph-based signal lookup for IDX Analyst
//!
//! Queries ArangoDB for economic indicators and news sentiment
//! linked to a ticker via signal_source and mentions edges.
//! Applies temporal decay to signal strength.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use tracing::{info, warn};

use crate::arangodb::ArangoClient;
use crate::idx_analyst::models::ExternalSignal;

/// Half-life values (in hours) for different event types
const COMMODITY_HALF_LIFE_HOURS: f64 = 48.0; // Commodity prices decay over 2 days
const NEWS_HALF_LIFE_HOURS: f64 = 24.0; // News sentiment decays over 1 day
const MACRO_HALF_LIFE_HOURS: f64 = 168.0; // Macro indicators (rates) decay over 1 week
const DEFAULT_HALF_LIFE_HOURS: f64 = 72.0; // Default: 3 days

/// Temporal decay function
/// strength = base_confidence * exp(-0.693 * hours_elapsed / half_life)
pub fn temporal_decay(base_confidence: f64, hours_elapsed: f64, half_life_hours: f64) -> f64 {
    base_confidence * (-0.693 * hours_elapsed / half_life_hours).exp()
}

/// Get the half-life for a given signal source type
fn get_half_life(source: &str) -> f64 {
    if source.contains("commodity")
        || source.contains("coal")
        || source.contains("oil")
        || source.contains("gold")
    {
        COMMODITY_HALF_LIFE_HOURS
    } else if source.contains("news") || source.contains("sentiment") {
        NEWS_HALF_LIFE_HOURS
    } else if source.contains("rate") || source.contains("fred") || source.contains("bi_") {
        MACRO_HALF_LIFE_HOURS
    } else {
        DEFAULT_HALF_LIFE_HOURS
    }
}

/// Look up external signals for a ticker from ArangoDB graph
pub async fn lookup_signals_for_ticker(
    arango: &ArangoClient,
    ticker: &str,
    lookback_days: i64,
) -> Result<Vec<ExternalSignal>> {
    let mut signals = Vec::new();

    // 1. Query signal_source edges: economic_indicators → ticker
    let economic_signals = query_economic_signals(arango, ticker, lookback_days)
        .await
        .unwrap_or_else(|e| {
            warn!(
                "⚠️ Failed to query economic signals for {}: {}",
                ticker, e
            );
            Vec::new()
        });
    signals.extend(economic_signals);

    // 2. Query mentions edges: articles mentioning actors related to ticker
    let sentiment_signals = query_sentiment_signals(arango, ticker, lookback_days)
        .await
        .unwrap_or_else(|e| {
            warn!(
                "⚠️ Failed to query sentiment signals for {}: {}",
                ticker, e
            );
            Vec::new()
        });
    signals.extend(sentiment_signals);

    info!(
        "📊 Signal lookup for {}: {} signals found",
        ticker,
        signals.len()
    );
    Ok(signals)
}

/// Query economic indicators linked to ticker via signal_source edges
async fn query_economic_signals(
    arango: &ArangoClient,
    ticker: &str,
    lookback_days: i64,
) -> Result<Vec<ExternalSignal>> {
    let query = r#"
        FOR edge IN signal_source
          FILTER edge._to == @ticker_id
          LET indicator = DOCUMENT(edge._from)
          FILTER indicator != null
          FILTER indicator.timestamp > DATE_SUBTRACT(DATE_NOW(), @days, "days")
          RETURN {
            source: indicator.source,
            indicator: indicator.indicator,
            change_pct: indicator.change_pct,
            strength: edge.strength,
            direction: edge.direction,
            timestamp: indicator.timestamp
          }
    "#;

    let ticker_id = format!("tickers/{}", ticker);
    let results: Vec<serde_json::Value> = arango
        .query_aql(
            query,
            serde_json::json!({
                "ticker_id": ticker_id,
                "days": lookback_days,
            }),
        )
        .await
        .context("querying economic signals")?;

    let now = Utc::now();
    let mut signals = Vec::new();

    for result in results {
        let source = result
            .get("source")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string();
        let direction = result
            .get("direction")
            .and_then(|d| d.as_str())
            .unwrap_or("neutral")
            .to_string();
        let base_strength = result
            .get("strength")
            .and_then(|s| s.as_f64())
            .unwrap_or(0.5);
        let timestamp_str = result
            .get("timestamp")
            .and_then(|t| t.as_str())
            .unwrap_or("");

        let hours_elapsed = if let Ok(ts) = DateTime::parse_from_rfc3339(timestamp_str) {
            (now - ts.with_timezone(&Utc)).num_hours() as f64
        } else {
            DEFAULT_HALF_LIFE_HOURS // Assume old if can't parse
        };

        let half_life = get_half_life(&source);
        let decayed_confidence = temporal_decay(base_strength, hours_elapsed, half_life);

        // Only include signals with meaningful strength (> 0.1)
        if decayed_confidence > 0.1 {
            signals.push(ExternalSignal {
                source: format!(
                    "{}_{}",
                    source,
                    result
                        .get("indicator")
                        .and_then(|i| i.as_str())
                        .unwrap_or("")
                ),
                direction,
                confidence: decayed_confidence,
            });
        }
    }

    Ok(signals)
}

/// Query recent news sentiment for articles mentioning actors related to ticker
async fn query_sentiment_signals(
    arango: &ArangoClient,
    ticker: &str,
    lookback_days: i64,
) -> Result<Vec<ExternalSignal>> {
    let query = r#"
        FOR edge IN mentions
          FILTER CONTAINS(edge._to, @ticker_lower)
          LET article = DOCUMENT(edge._from)
          FILTER article != null
          FILTER article.labeled_at > DATE_SUBTRACT(DATE_NOW(), @days, "days")
          FILTER article.sentiment != null
          RETURN {
            sentiment: article.sentiment,
            sentiment_score: article.sentiment_score,
            title: article.title,
            labeled_at: article.labeled_at
          }
    "#;

    let results: Vec<serde_json::Value> = arango
        .query_aql(
            query,
            serde_json::json!({
                "ticker_lower": ticker.to_lowercase(),
                "days": lookback_days,
            }),
        )
        .await
        .context("querying sentiment signals")?;

    if results.is_empty() {
        return Ok(Vec::new());
    }

    // Aggregate sentiment: average score → direction
    let total: f64 = results
        .iter()
        .filter_map(|r| r.get("sentiment_score").and_then(|s| s.as_f64()))
        .sum();
    let count = results.len() as f64;
    let avg_sentiment = total / count;

    let direction = if avg_sentiment > 0.3 {
        "positive"
    } else if avg_sentiment < -0.3 {
        "negative"
    } else {
        "neutral"
    };

    let confidence = avg_sentiment.abs().min(1.0);

    if confidence > 0.1 {
        Ok(vec![ExternalSignal {
            source: format!("news_sentiment_{}", ticker),
            direction: direction.to_string(),
            confidence,
        }])
    } else {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_decay_at_zero_hours() {
        let result = temporal_decay(1.0, 0.0, 48.0);
        assert!((result - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_temporal_decay_at_half_life() {
        let result = temporal_decay(1.0, 48.0, 48.0);
        assert!((result - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_temporal_decay_at_double_half_life() {
        let result = temporal_decay(1.0, 96.0, 48.0);
        assert!((result - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_get_half_life_commodity() {
        assert_eq!(get_half_life("coal_price"), COMMODITY_HALF_LIFE_HOURS);
        assert_eq!(get_half_life("gold_spot"), COMMODITY_HALF_LIFE_HOURS);
        assert_eq!(get_half_life("oil_wti"), COMMODITY_HALF_LIFE_HOURS);
    }

    #[test]
    fn test_get_half_life_news() {
        assert_eq!(get_half_life("news_sentiment"), NEWS_HALF_LIFE_HOURS);
    }

    #[test]
    fn test_get_half_life_macro() {
        assert_eq!(get_half_life("bi_rate"), MACRO_HALF_LIFE_HOURS);
        assert_eq!(get_half_life("fred_fedfunds"), MACRO_HALF_LIFE_HOURS);
    }

    #[test]
    fn test_get_half_life_default() {
        assert_eq!(get_half_life("unknown_source"), DEFAULT_HALF_LIFE_HOURS);
    }

    #[tokio::test]
    #[ignore] // Requires running ArangoDB
    async fn test_lookup_signals_integration() {
        let arango = ArangoClient::new().unwrap();
        let signals = lookup_signals_for_ticker(&arango, "PTBA", 7)
            .await
            .unwrap();
        println!("Signals for PTBA: {:?}", signals);
    }
}
