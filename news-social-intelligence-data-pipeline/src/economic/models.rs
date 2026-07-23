use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Source of economic data
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EconomicSource {
    YahooFinance,
    CoinGecko,
    Fred,
    BankIndonesia,
    Gdelt,
}

impl std::fmt::Display for EconomicSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::YahooFinance => write!(f, "yahoo_finance"),
            Self::CoinGecko => write!(f, "coingecko"),
            Self::Fred => write!(f, "fred"),
            Self::BankIndonesia => write!(f, "bank_indonesia"),
            Self::Gdelt => write!(f, "gdelt"),
        }
    }
}

/// A single economic indicator data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicIndicator {
    /// Unique key (source_indicator_timestamp hash)
    pub key: String,
    /// Data source
    pub source: EconomicSource,
    /// Indicator name (e.g., "COAL_PRICE", "BTC_USD", "FEDFUNDS")
    pub indicator: String,
    /// Current value
    pub value: f64,
    /// Unit (e.g., "USD", "IDR", "percent")
    pub unit: String,
    /// Timestamp of the data point
    pub timestamp: DateTime<Utc>,
    /// Percentage change from previous value (if available)
    pub change_pct: Option<f64>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Signal source edge — links an economic indicator to an affected ticker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalSourceEdge {
    /// Source node: economic_indicators/{key}
    pub from: String,
    /// Target node: tickers/{ticker} or articles/{key}
    pub to: String,
    /// Relationship strength (0.0 - 1.0)
    pub strength: f64,
    /// Direction of impact: "positive", "negative", "neutral"
    pub direction: String,
    /// When this relationship was established
    pub created_at: DateTime<Utc>,
}

/// Collection stats for economic data ingestion
#[derive(Debug, Default)]
pub struct EconomicStats {
    pub indicators_inserted: usize,
    pub edges_created: usize,
    pub errors: usize,
}

impl std::fmt::Display for EconomicStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Economic: {} indicators, {} edges, {} errors",
            self.indicators_inserted, self.edges_created, self.errors
        )
    }
}

// TODO: Storage functions (wire in when arangodb module is available)
//
// pub async fn store_indicator(client: &ArangoClient, indicator: &EconomicIndicator) -> anyhow::Result<()> {
//     // Insert into economic_indicators collection
//     todo!()
// }
//
// pub async fn store_signal_edge(client: &ArangoClient, edge: &SignalSourceEdge) -> anyhow::Result<()> {
//     // Insert into signal_source edge collection
//     todo!()
// }

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_economic_source_display() {
        assert_eq!(EconomicSource::YahooFinance.to_string(), "yahoo_finance");
        assert_eq!(EconomicSource::CoinGecko.to_string(), "coingecko");
        assert_eq!(EconomicSource::Fred.to_string(), "fred");
        assert_eq!(EconomicSource::BankIndonesia.to_string(), "bank_indonesia");
        assert_eq!(EconomicSource::Gdelt.to_string(), "gdelt");
    }

    #[test]
    fn test_economic_indicator_serialization() {
        let indicator = EconomicIndicator {
            key: "yahoo_finance_coal_price_20240101".to_string(),
            source: EconomicSource::YahooFinance,
            indicator: "COAL_PRICE".to_string(),
            value: 135.50,
            unit: "USD".to_string(),
            timestamp: Utc::now(),
            change_pct: Some(-2.3),
            metadata: serde_json::json!({"exchange": "ICE"}),
        };

        let json = serde_json::to_string(&indicator).expect("serialize");
        let deserialized: EconomicIndicator =
            serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.key, indicator.key);
        assert_eq!(deserialized.source, EconomicSource::YahooFinance);
        assert_eq!(deserialized.indicator, "COAL_PRICE");
        assert_eq!(deserialized.value, 135.50);
        assert_eq!(deserialized.unit, "USD");
        assert_eq!(deserialized.change_pct, Some(-2.3));
    }

    #[test]
    fn test_economic_indicator_no_change_pct() {
        let indicator = EconomicIndicator {
            key: "fred_fedfunds_20240101".to_string(),
            source: EconomicSource::Fred,
            indicator: "FEDFUNDS".to_string(),
            value: 5.25,
            unit: "percent".to_string(),
            timestamp: Utc::now(),
            change_pct: None,
            metadata: serde_json::json!({}),
        };

        let json = serde_json::to_string(&indicator).expect("serialize");
        let deserialized: EconomicIndicator =
            serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.change_pct, None);
    }

    #[test]
    fn test_signal_source_edge_serialization() {
        let edge = SignalSourceEdge {
            from: "economic_indicators/coal_price_123".to_string(),
            to: "tickers/ADRO".to_string(),
            strength: 0.85,
            direction: "positive".to_string(),
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&edge).expect("serialize");
        let deserialized: SignalSourceEdge =
            serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.from, "economic_indicators/coal_price_123");
        assert_eq!(deserialized.to, "tickers/ADRO");
        assert_eq!(deserialized.strength, 0.85);
        assert_eq!(deserialized.direction, "positive");
    }

    #[test]
    fn test_economic_stats_display() {
        let stats = EconomicStats {
            indicators_inserted: 42,
            edges_created: 15,
            errors: 2,
        };
        assert_eq!(
            stats.to_string(),
            "Economic: 42 indicators, 15 edges, 2 errors"
        );
    }

    #[test]
    fn test_economic_stats_default() {
        let stats = EconomicStats::default();
        assert_eq!(stats.indicators_inserted, 0);
        assert_eq!(stats.edges_created, 0);
        assert_eq!(stats.errors, 0);
        assert_eq!(
            stats.to_string(),
            "Economic: 0 indicators, 0 edges, 0 errors"
        );
    }
}
