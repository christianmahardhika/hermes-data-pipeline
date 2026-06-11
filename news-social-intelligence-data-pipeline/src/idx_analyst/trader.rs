//! Trader Executor — Transaction Proposal Generation

use crate::idx_analyst::config::ExecutionConfig;
use crate::idx_analyst::models::{Signal, Confidence, StockData};

/// Transaction direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraderAction {
    Buy,
    Hold,
    Sell,
}

impl std::fmt::Display for TraderAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraderAction::Buy => write!(f, "BUY"),
            TraderAction::Hold => write!(f, "HOLD"),
            TraderAction::Sell => write!(f, "SELL"),
        }
    }
}

/// Structured transaction proposal
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

/// Generates trade proposals from debate signals
pub struct TraderExecutor<'a> {
    ticker: String,
    config: &'a ExecutionConfig,
}

impl<'a> TraderExecutor<'a> {
    pub fn new(ticker: &str, config: &'a ExecutionConfig) -> Self {
        Self { ticker: ticker.to_string(), config }
    }

    pub fn execute(&self, signal: &Signal, stock_data: &StockData) -> TraderProposal {
        let price = stock_data.current_price;

        match signal {
            Signal::StrongBuy | Signal::Buy => {
                let entry = price * (1.0 - self.config.entry_price_offset_pct);
                let stop = entry * (1.0 - self.config.stop_loss_pct);
                let target = entry * (1.0 + self.config.take_profit_pct);

                TraderProposal {
                    ticker: self.ticker.clone(),
                    action: TraderAction::Buy,
                    reasoning: format!(
                        "Debate reached {} consensus. Entry {:.0}, stop {:.0}, target {:.0}.",
                        signal, entry, stop, target
                    ),
                    entry_price: Some(entry),
                    stop_loss: Some(stop),
                    take_profit: Some(target),
                    position_size_pct: Some(self.config.default_position_size_pct),
                    holding_days: self.config.holding_days,
                    confidence: if *signal == Signal::StrongBuy { Confidence::High } else { Confidence::Medium },
                }
            }
            Signal::Hold => TraderProposal {
                ticker: self.ticker.clone(),
                action: TraderAction::Hold,
                reasoning: "Debate inconclusive. HOLD existing; avoid new entry.".into(),
                entry_price: None, stop_loss: None, take_profit: None,
                position_size_pct: None, holding_days: self.config.holding_days,
                confidence: Confidence::Medium,
            },
            Signal::Pass | Signal::Avoid => TraderProposal {
                ticker: self.ticker.clone(),
                action: TraderAction::Hold,
                reasoning: format!("Bear consensus ({}). PASS on new entry.", signal),
                entry_price: None, stop_loss: None, take_profit: None,
                position_size_pct: None, holding_days: self.config.holding_days,
                confidence: Confidence::Medium,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stock() -> StockData {
        StockData { ticker: "BMRI".into(), current_price: 6000.0, ..Default::default() }
    }

    #[test]
    fn test_buy_generates_levels() {
        let cfg = ExecutionConfig::default();
        let p = TraderExecutor::new("BMRI", &cfg).execute(&Signal::Buy, &stock());
        assert_eq!(p.action, TraderAction::Buy);
        assert!(p.entry_price.unwrap() < 6000.0);
        assert!(p.stop_loss.unwrap() < p.entry_price.unwrap());
        assert!(p.take_profit.unwrap() > p.entry_price.unwrap());
    }

    #[test]
    fn test_hold_no_levels() {
        let cfg = ExecutionConfig::default();
        let p = TraderExecutor::new("KLBF", &cfg).execute(&Signal::Hold, &stock());
        assert_eq!(p.action, TraderAction::Hold);
        assert!(p.entry_price.is_none());
    }

    #[test]
    fn test_pass_no_entry() {
        let cfg = ExecutionConfig::default();
        let p = TraderExecutor::new("MDKA", &cfg).execute(&Signal::Pass, &stock());
        assert!(p.entry_price.is_none());
    }
}
