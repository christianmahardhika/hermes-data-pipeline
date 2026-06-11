//! Memory Logger — Decision Log & Reflection

use std::path::PathBuf;
use std::fs;
use std::io::Write;
use anyhow::Result;
use chrono::Utc;

use crate::idx_analyst::debate::DebateResult;
use crate::idx_analyst::trader::TraderProposal;

/// Logs trading decisions to markdown file
pub struct MemoryLogger {
    log_file: PathBuf,
}

impl MemoryLogger {
    pub fn new(memory_dir: &str) -> Result<Self> {
        let expanded = if memory_dir.starts_with('~') {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            memory_dir.replacen('~', &home, 1)
        } else {
            memory_dir.to_string()
        };

        let dir = PathBuf::from(&expanded);
        fs::create_dir_all(&dir)?;
        let log_file = dir.join("trading_decisions.md");
        Ok(Self { log_file })
    }

    /// Log a trading decision
    pub fn log_decision(
        &self,
        ticker: &str,
        debate: &DebateResult,
        proposal: &TraderProposal,
    ) -> Result<()> {
        let timestamp = Utc::now().to_rfc3339();
        let date = Utc::now().format("%Y-%m-%d").to_string();

        let entry_str = proposal.entry_price.map_or("N/A".into(), |p| format!("Rp {:.0}", p));
        let stop_str = proposal.stop_loss.map_or("N/A".into(), |p| format!("Rp {:.0}", p));
        let tp_str = proposal.take_profit.map_or("N/A".into(), |p| format!("Rp {:.0}", p));
        let size_str = proposal.position_size_pct.map_or("N/A".into(), |s| format!("{:.1}%", s * 100.0));

        let entry = format!(
            "\n## {} — {}\n**Signal:** {} | **Confidence:** {}\n**Timestamp:** {}\n\n\
             ### Debate Summary\n{}\n\n\
             ### Trader Proposal\nEntry: {} | Stop: {} | Target: {} | Size: {}\n\
             Reasoning: {}\n\n\
             ### Status\n**Decision Status:** PENDING\n\n---\n",
            ticker, date, debate.final_signal, debate.confidence, timestamp,
            debate.consensus_summary,
            entry_str, stop_str, tp_str, size_str,
            proposal.reasoning,
        );

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)?;
        write!(file, "{}", entry)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn log_path(&self) -> &PathBuf {
        &self.log_file
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::idx_analyst::models::{Signal, Confidence};
    use crate::idx_analyst::trader::TraderAction;

    #[test]
    fn test_log_creates_file() {
        let tmp = std::env::temp_dir().join(format!("hermes_test_{}", std::process::id()));
        let logger = MemoryLogger::new(tmp.to_str().unwrap()).unwrap();

        let debate = DebateResult {
            ticker: "BMRI".into(), rounds: vec![],
            final_signal: Signal::Buy, confidence: Confidence::Medium,
            bull_win_rate: 0.5, consensus_summary: "Test".into(),
        };
        let proposal = TraderProposal {
            ticker: "BMRI".into(), action: TraderAction::Buy,
            reasoning: "Test".into(), entry_price: Some(5800.0),
            stop_loss: Some(5300.0), take_profit: Some(6670.0),
            position_size_pct: Some(0.03), holding_days: 5,
            confidence: Confidence::Medium,
        };

        logger.log_decision("BMRI", &debate, &proposal).unwrap();

        let content = fs::read_to_string(logger.log_path()).unwrap();
        assert!(content.contains("## BMRI"));
        assert!(content.contains("PENDING"));

        // Cleanup
        let _ = fs::remove_dir_all(&tmp);
    }
}
