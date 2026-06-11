//! Configuration for Enhanced IDX Analyst

/// Portfolio tickers (Christian's holdings)
pub const PORTFOLIO_STOCKS: &[&str] = &[
    "KLBF", "TLKM", "BBRI", "PTBA", "BJTM", "ADMF", "TAPG", "JPFA", "TSPC", "BMRI", "ASII",
];

/// Watchlist tickers
pub const WATCHLIST_STOCKS: &[&str] = &["INCO", "ANTM", "MDKA"];

/// Investment screening criteria
pub struct Criteria {
    pub per_max: f64,
    pub pbv_max: f64,
    pub roe_min: f64,
    pub der_max: f64,
    pub dy_min: f64,
}

pub const CRITERIA: Criteria = Criteria {
    per_max: 15.0,
    pbv_max: 2.0,
    roe_min: 10.0,
    der_max: 1.0,
    dy_min: 3.0,
};

/// Risk management configuration
#[derive(Debug, Clone)]
pub struct RiskConfig {
    pub max_position_size_pct: f64,
    pub max_sector_concentration: f64,
    pub max_portfolio_drawdown: f64,
    pub min_liquidity_score: f64,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_position_size_pct: 0.05,
            max_sector_concentration: 0.25,
            max_portfolio_drawdown: 0.10,
            min_liquidity_score: 0.3,
        }
    }
}

/// Execution configuration
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub default_position_size_pct: f64,
    pub entry_price_offset_pct: f64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub holding_days: u32,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            default_position_size_pct: 0.03,
            entry_price_offset_pct: 0.02,
            stop_loss_pct: 0.08,
            take_profit_pct: 0.15,
            holding_days: 5,
        }
    }
}

/// Full IDX analyst configuration
#[derive(Debug, Clone)]
pub struct IdxConfig {
    pub risk: RiskConfig,
    pub execution: ExecutionConfig,
    pub debate_max_rounds: usize,
    pub portfolio_value: f64,
    pub memory_dir: String,
}

impl Default for IdxConfig {
    fn default() -> Self {
        let memory_dir = std::env::var("HERMES_MEMORY_DIR")
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                format!("{}/.hermes/profiles/pagupon-finance/memory", home)
            });

        Self {
            risk: RiskConfig::default(),
            execution: ExecutionConfig::default(),
            debate_max_rounds: 2,
            portfolio_value: std::env::var("IDX_PORTFOLIO_VALUE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100_000_000.0),
            memory_dir,
        }
    }
}
