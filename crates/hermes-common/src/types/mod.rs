//! Domain types and models shared across Hermes services
//!
//! Common data structures used throughout the pipeline including:
//! - Article and news item models
//! - Processing status enums
//! - Actor tracking for intelligence analysis
//! - Topic categorization and sentiment analysis
//! - Signal detection and alert systems
//! - Indonesian stock market specific types

use serde::{Deserialize, Serialize};

// Re-export all domain modules
pub mod article;
pub mod actor;
pub mod topic;
pub mod signal;

// Re-export core types for easy access
pub use article::{Article, ArticleStatus};
pub use actor::{Actor, ActorType};
pub use topic::{Topic, TopicCategory, TopicSentiment};
pub use signal::{Signal, ExternalSignal, SignalCategory, SignalStrength};

/// Article processing status for pipeline
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessingStatus {
    Pending,
    InProgress,
    Processed,
    Failed,
    Skipped,
}

/// Indonesian stock symbols for portfolio tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndonesianStock {
    /// Bank Mandiri (Persero) Tbk
    BMRI,
    /// Bank Rakyat Indonesia (Persero) Tbk
    BBRI,
    /// Vale Indonesia Tbk (formerly INCO)
    INCO,
    /// Aneka Tambang (Persero) Tbk
    ANTM,
    /// Bukit Asam (Persero) Tbk
    PTBA,
    /// Triputra Agro Persada Tbk
    TAPG,
    /// Telkom Indonesia (Persero) Tbk
    TLKM,
    /// Astra International Tbk
    ASII,
    /// Kalbe Farma Tbk
    KLBF,
    /// Tempo Scan Pacific Tbk
    TSPC,
    /// Bumi Serpong Damai Tbk
    BSDE,
}

impl IndonesianStock {
    /// Get stock symbol as string
    pub fn symbol(&self) -> &'static str {
        match self {
            IndonesianStock::BMRI => "BMRI",
            IndonesianStock::BBRI => "BBRI",
            IndonesianStock::INCO => "INCO",
            IndonesianStock::ANTM => "ANTM",
            IndonesianStock::PTBA => "PTBA",
            IndonesianStock::TAPG => "TAPG",
            IndonesianStock::TLKM => "TLKM",
            IndonesianStock::ASII => "ASII",
            IndonesianStock::KLBF => "KLBF",
            IndonesianStock::TSPC => "TSPC",
            IndonesianStock::BSDE => "BSDE",
        }
    }

    /// Get sector classification
    pub fn sector(&self) -> &'static str {
        match self {
            IndonesianStock::BMRI | IndonesianStock::BBRI => "Banking",
            IndonesianStock::INCO | IndonesianStock::ANTM | IndonesianStock::PTBA => "Mining",
            IndonesianStock::TAPG => "Agriculture",
            IndonesianStock::TLKM => "Telecommunications",
            IndonesianStock::ASII => "Automotive",
            IndonesianStock::KLBF | IndonesianStock::TSPC => "Healthcare",
            IndonesianStock::BSDE => "Real Estate",
        }
    }

    /// Check if stock is in user's portfolio focus (Christian's holdings)
    pub fn is_portfolio_focus(&self) -> bool {
        matches!(
            self,
            IndonesianStock::BMRI
                | IndonesianStock::BBRI
                | IndonesianStock::INCO
                | IndonesianStock::ANTM
                | IndonesianStock::PTBA
                | IndonesianStock::TAPG
                | IndonesianStock::TLKM
                | IndonesianStock::ASII
                | IndonesianStock::KLBF
                | IndonesianStock::TSPC
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indonesian_stock_symbols() {
        assert_eq!(IndonesianStock::BMRI.symbol(), "BMRI");
        assert_eq!(IndonesianStock::INCO.symbol(), "INCO");
    }

    #[test]
    fn test_indonesian_stock_sectors() {
        assert_eq!(IndonesianStock::BMRI.sector(), "Banking");
        assert_eq!(IndonesianStock::INCO.sector(), "Mining");
        assert_eq!(IndonesianStock::TLKM.sector(), "Telecommunications");
    }

    #[test]
    fn test_portfolio_focus() {
        assert!(IndonesianStock::BMRI.is_portfolio_focus());
        assert!(IndonesianStock::INCO.is_portfolio_focus());
        assert!(!IndonesianStock::BSDE.is_portfolio_focus()); // Not in Christian's portfolio
    }
}