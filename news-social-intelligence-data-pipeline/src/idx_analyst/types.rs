//! Shared types for the IDX Analyst module

use std::fmt;

/// Trading signal output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    StrongBuy,
    Buy,
    Hold,
    Pass,
    Avoid,
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Signal::StrongBuy => write!(f, "STRONG BUY"),
            Signal::Buy => write!(f, "BUY"),
            Signal::Hold => write!(f, "HOLD"),
            Signal::Pass => write!(f, "PASS"),
            Signal::Avoid => write!(f, "AVOID"),
        }
    }
}

/// Trader action recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraderAction {
    Buy,
    Sell,
    Hold,
}

impl fmt::Display for TraderAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TraderAction::Buy => write!(f, "BUY"),
            TraderAction::Sell => write!(f, "SELL"),
            TraderAction::Hold => write!(f, "HOLD"),
        }
    }
}

/// Confidence level for signals and debate arguments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Confidence::High => write!(f, "HIGH"),
            Confidence::Medium => write!(f, "MEDIUM"),
            Confidence::Low => write!(f, "LOW"),
        }
    }
}

/// Stock fundamental metrics used across all personas
#[derive(Debug, Clone, Default)]
pub struct StockMetrics {
    pub ticker: String,
    pub current_price: f64,
    pub per: f64,
    pub pbv: f64,
    pub roe: f64,
    pub der: f64,
    pub dy: f64,
    pub sentiment_score: f64,
}

/// Trader proposal — output of the trade executor
#[derive(Debug, Clone)]
pub struct TraderProposal {
    pub ticker: String,
    pub action: TraderAction,
    pub reasoning: String,
    pub entry_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub position_size_pct: Option<f64>,
    pub holding_days: u32,
    pub confidence: Confidence,
}

/// Risk assessment result
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub is_approved: bool,
    pub approval_reason: String,
    pub suggested_position_size: Option<f64>,
    pub risk_score: f64,
    pub warnings: Vec<String>,
}

/// A single debate round between bull and bear personas
#[derive(Debug, Clone)]
pub struct DebateRound {
    pub round_num: usize,
    pub bull_persona: String,
    pub bear_persona: String,
    pub bull_argument: String,
    pub bear_argument: String,
    pub bull_confidence: Confidence,
    pub bear_confidence: Confidence,
    pub bull_reasoning: String,
    pub bear_reasoning: String,
}

/// Final result of a full debate cycle
#[derive(Debug, Clone)]
pub struct DebateResult {
    pub ticker: String,
    pub rounds: usize,
    pub final_signal: Signal,
    pub confidence: Confidence,
    pub bull_win_rate: f64,
    pub debate_rounds: Vec<DebateRound>,
    pub consensus_summary: String,
}
