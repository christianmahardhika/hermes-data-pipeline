//! Risk Manager — Portfolio Constraint Validation

use crate::idx_analyst::config::RiskConfig;
use crate::idx_analyst::models::StockData;
use crate::idx_analyst::trader::TraderProposal;

/// Risk assessment result
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub is_approved: bool,
    pub reason: String,
    pub suggested_position_size: Option<f64>,
    pub risk_score: f64,
    pub warnings: Vec<String>,
}

/// Validates proposals against portfolio constraints
pub struct RiskManager {
    #[allow(dead_code)]
    portfolio_value: f64,
    config: RiskConfig,
}

impl RiskManager {
    pub fn new(portfolio_value: f64, config: &RiskConfig) -> Self {
        Self {
            portfolio_value,
            config: config.clone(),
        }
    }

    pub fn assess(&self, proposal: &TraderProposal, stock_data: &StockData) -> RiskAssessment {
        let position_size = match proposal.position_size_pct {
            Some(size) => size,
            None => {
                return RiskAssessment {
                    is_approved: true,
                    reason: "Action is HOLD, no position risk to assess".into(),
                    suggested_position_size: None,
                    risk_score: 0.0,
                    warnings: vec![],
                };
            }
        };

        let mut warnings = Vec::new();
        let mut risk_score: f64 = 0.0;
        let mut capped_size = position_size;

        if position_size > self.config.max_position_size_pct {
            warnings.push(format!(
                "Position size {:.1}% exceeds limit {:.1}%",
                position_size * 100.0,
                self.config.max_position_size_pct * 100.0
            ));
            capped_size = self.config.max_position_size_pct;
            risk_score += 0.3;
        }

        if stock_data.dy < 1.0 {
            warnings.push(format!("Low dividend yield ({:.1}%)", stock_data.dy));
            risk_score += 0.1;
        }

        if stock_data.roe < 5.0 {
            warnings.push(format!("Low ROE ({:.1}%)", stock_data.roe));
            risk_score += 0.2;
        }

        if stock_data.per > 30.0 {
            warnings.push(format!("High P/E ({:.1})", stock_data.per));
            risk_score += 0.2;
        }

        if stock_data.der > 1.5 {
            warnings.push(format!("High debt (D/E {:.2})", stock_data.der));
            risk_score += 0.15;
        }

        let is_approved = risk_score < 0.5;
        let reason = if is_approved {
            format!("Approved at {:.1}% with manageable risk", capped_size * 100.0)
        } else {
            format!("Risk too high (score {:.2}). Reduce or wait.", risk_score)
        };

        RiskAssessment {
            is_approved,
            reason,
            suggested_position_size: if is_approved { Some(capped_size) } else { None },
            risk_score,
            warnings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::idx_analyst::trader::TraderAction;
    use crate::idx_analyst::models::Confidence;

    fn buy_proposal() -> TraderProposal {
        TraderProposal {
            ticker: "BMRI".into(),
            action: TraderAction::Buy,
            reasoning: "test".into(),
            entry_price: Some(5800.0),
            stop_loss: Some(5300.0),
            take_profit: Some(6670.0),
            position_size_pct: Some(0.03),
            holding_days: 5,
            confidence: Confidence::Medium,
        }
    }

    #[test]
    fn test_healthy_stock_approved() {
        let mgr = RiskManager::new(100_000_000.0, &RiskConfig::default());
        let stock = StockData {
            ticker: "BMRI".into(),
            per: 8.0, roe: 20.0, der: 0.5, dy: 5.0,
            ..Default::default()
        };
        let result = mgr.assess(&buy_proposal(), &stock);
        assert!(result.is_approved);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_risky_stock_flagged() {
        let mgr = RiskManager::new(100_000_000.0, &RiskConfig::default());
        let stock = StockData {
            ticker: "XYZZ".into(),
            per: 35.0, roe: 3.0, der: 2.0, dy: 0.5,
            ..Default::default()
        };
        let result = mgr.assess(&buy_proposal(), &stock);
        assert!(!result.is_approved);
        assert!(result.risk_score >= 0.5);
    }

    #[test]
    fn test_hold_always_approved() {
        let mgr = RiskManager::new(100_000_000.0, &RiskConfig::default());
        let hold = TraderProposal {
            ticker: "TEST".into(),
            action: TraderAction::Hold,
            reasoning: "test".into(),
            entry_price: None, stop_loss: None, take_profit: None,
            position_size_pct: None, holding_days: 5,
            confidence: Confidence::Medium,
        };
        let result = mgr.assess(&hold, &StockData::default());
        assert!(result.is_approved);
    }
}
