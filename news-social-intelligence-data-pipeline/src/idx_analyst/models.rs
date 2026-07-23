//! Shared data models for IDX Analyst

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Investment signal from debate consensus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signal {
    StrongBuy,
    Buy,
    Hold,
    Pass,
    Avoid,
}

impl Signal {
    pub fn as_str(&self) -> &'static str {
        match self {
            Signal::StrongBuy => "STRONG BUY",
            Signal::Buy => "BUY",
            Signal::Hold => "HOLD",
            Signal::Pass => "PASS",
            Signal::Avoid => "AVOID",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Signal::StrongBuy => "🚀",
            Signal::Buy => "📈",
            Signal::Hold => "⏸️",
            Signal::Pass => "⏭️",
            Signal::Avoid => "🛑",
        }
    }
}

impl std::fmt::Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Confidence level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl Confidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            Confidence::High => "HIGH",
            Confidence::Medium => "MEDIUM",
            Confidence::Low => "LOW",
        }
    }
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Stock fundamental data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StockData {
    pub ticker: String,
    pub current_price: f64,
    pub per: f64,
    pub pbv: f64,
    pub roe: f64,
    pub roa: f64,
    pub der: f64,
    pub dy: f64,
    pub eps: f64,
    pub bv_per_share: f64,
    pub npm: f64,
    pub revenue_trn: f64,
    pub net_income_trn: f64,
    pub market_cap_trn: f64,
    pub high_52w: f64,
    pub low_52w: f64,
    pub sentiment_score: f64,
}

impl StockData {
    /// Convert to metrics map for debate engine
    pub fn to_metrics(&self) -> HashMap<String, f64> {
        let mut m = HashMap::new();
        m.insert("per".into(), self.per);
        m.insert("pbv".into(), self.pbv);
        m.insert("roe".into(), self.roe);
        m.insert("roa".into(), self.roa);
        m.insert("der".into(), self.der);
        m.insert("dy".into(), self.dy);
        m.insert("eps".into(), self.eps);
        m.insert("current_price".into(), self.current_price);
        m
    }
}

/// External signal from economic/news/social data that modifies debate confidence
#[derive(Debug, Clone)]
pub struct ExternalSignal {
    /// Source identifier (e.g., "coal_price_spike", "bi_rate_hike", "news_sentiment")
    pub source: String,
    /// Direction of impact: "positive", "negative", "neutral"
    pub direction: String,
    /// Confidence strength (0.0 - 1.0)
    pub confidence: f64,
}

/// IDX profile (extra data from scraping)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdxProfile {
    pub name: String,
    pub free_float_pct: f64,
    pub ownership: Ownership,
    pub liquidity: Liquidity,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Ownership {
    pub institutional: f64,
    pub retail: f64,
    pub foreign: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Liquidity {
    pub bid_ask_spread_pct: f64,
    pub volume_30d_avg: f64,
    pub rating: String,
}
