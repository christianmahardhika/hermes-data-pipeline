//! Enhanced IDX Analyst — Rust Port
//!
//! 5-persona debate system for Indonesian stock analysis.
//! Pipeline: Data Gathering → Debate Engine → Trade Proposal → Risk Assessment → Memory Log

pub mod config;
pub mod models;
pub mod debate;
pub mod trader;
pub mod risk;
pub mod memory;
pub mod formatter;
pub mod data_source;

pub use config::{IdxConfig, PORTFOLIO_STOCKS, WATCHLIST_STOCKS, CRITERIA};
pub use models::{StockData, Signal, Confidence};
pub use debate::{PersonaDebateEngine, DebateResult};
pub use trader::{TraderExecutor, TraderProposal, TraderAction};
pub use risk::{RiskManager, RiskAssessment};
pub use memory::MemoryLogger;
pub use formatter::RTIFormatter;

use anyhow::Result;
use tracing::info;

/// Main orchestrator for IDX analysis
pub struct IdxAnalyst {
    config: IdxConfig,
    memory: MemoryLogger,
}

impl IdxAnalyst {
    pub fn new(config: IdxConfig) -> Result<Self> {
        let memory = MemoryLogger::new(&config.memory_dir)?;
        Ok(Self { config, memory })
    }

    /// Analyze a single stock through the full pipeline
    pub async fn analyze_stock(&self, ticker: &str, stock_data: &StockData) -> Result<AnalysisResult> {
        info!("📊 Analyzing {} through debate pipeline...", ticker);

        // Phase 1: Run debate
        let debate_engine = PersonaDebateEngine::new(ticker, self.config.debate_max_rounds);
        let debate_result = debate_engine.run_debate(&stock_data.to_metrics());

        // Phase 2: Generate trade proposal
        let trader = TraderExecutor::new(ticker, &self.config.execution);
        let proposal = trader.execute(&debate_result.final_signal, stock_data);

        // Phase 3: Risk assessment
        let risk_mgr = RiskManager::new(self.config.portfolio_value, &self.config.risk);
        let risk = risk_mgr.assess(&proposal, stock_data);

        // Phase 4: Log decision
        self.memory.log_decision(ticker, &debate_result, &proposal)?;

        Ok(AnalysisResult {
            ticker: ticker.to_string(),
            stock_data: stock_data.clone(),
            debate: debate_result,
            proposal,
            risk,
        })
    }

    /// Analyze multiple stocks (can be parallelized with tokio::spawn)
    pub async fn analyze_batch(&self, stocks: &[StockData]) -> Vec<Result<AnalysisResult>> {
        let mut results = Vec::with_capacity(stocks.len());
        for data in stocks {
            results.push(self.analyze_stock(&data.ticker, data).await);
        }
        results
    }
}

/// Complete analysis result for one stock
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub ticker: String,
    pub stock_data: StockData,
    pub debate: DebateResult,
    pub proposal: TraderProposal,
    pub risk: RiskAssessment,
}
