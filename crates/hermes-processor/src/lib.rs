//! Hermes Processor Service
//!
//! Text processing pipeline: Clean → Label → Embed
//! Specialized for Indonesian market intelligence and Prof Jiang analysis.

pub mod text_cleaner;
pub mod content_labeler;
pub mod embedding_generator;
pub mod processor;

use anyhow::Result;
use hermes_common::types::{Article, ProcessingStatus, IndonesianStock};

/// Re-export core types for convenience
pub use text_cleaner::TextCleaner;
pub use content_labeler::{ContentLabeler, ContentLabels, ProfJiangRelevance};
pub use embedding_generator::{EmbeddingGenerator, EmbeddingVector};
pub use processor::HermesProcessor;

/// Processing pipeline statistics
#[derive(Debug, Clone)]
pub struct ProcessingStats {
    pub articles_processed: usize,
    pub articles_cleaned: usize,
    pub articles_labeled: usize,
    pub articles_embedded: usize,
    pub processing_errors: usize,
    pub indonesian_articles: usize,
    pub prof_jiang_relevant: usize,
    pub portfolio_mentions: usize,
    pub processing_duration_ms: u64,
}

impl ProcessingStats {
    pub fn new() -> Self {
        Self {
            articles_processed: 0,
            articles_cleaned: 0,
            articles_labeled: 0,
            articles_embedded: 0,
            processing_errors: 0,
            indonesian_articles: 0,
            prof_jiang_relevant: 0,
            portfolio_mentions: 0,
            processing_duration_ms: 0,
        }
    }
    
    /// Calculate success rate percentage
    pub fn success_rate(&self) -> f64 {
        if self.articles_processed == 0 {
            return 0.0;
        }
        ((self.articles_processed - self.processing_errors) as f64 / self.articles_processed as f64) * 100.0
    }
    
    /// Calculate articles per second throughput
    pub fn articles_per_second(&self) -> f64 {
        if self.processing_duration_ms == 0 {
            return 0.0;
        }
        (self.articles_processed as f64 / self.processing_duration_ms as f64) * 1000.0
    }
    
    /// Check if processing meets quality thresholds
    pub fn meets_quality_threshold(&self) -> bool {
        self.success_rate() >= 90.0 && self.articles_embedded > 0
    }
}

/// Article processing pipeline result
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub article_id: String,
    pub original_content: String,
    pub cleaned_content: Option<String>,
    pub labels: Option<ContentLabels>,
    pub embedding: Option<EmbeddingVector>,
    pub processing_status: ProcessingStatus,
    pub error_message: Option<String>,
    pub processing_duration_ms: u64,
}

impl ProcessingResult {
    pub fn new(article_id: String, original_content: String) -> Self {
        Self {
            article_id,
            original_content,
            cleaned_content: None,
            labels: None,
            embedding: None,
            processing_status: ProcessingStatus::Pending,
            error_message: None,
            processing_duration_ms: 0,
        }
    }
    
    /// Check if processing completed successfully
    pub fn is_success(&self) -> bool {
        matches!(self.processing_status, ProcessingStatus::Processed) && self.error_message.is_none()
    }
    
    /// Check if article is Indonesian market relevant
    pub fn is_indonesian_market_relevant(&self) -> bool {
        if let Some(labels) = &self.labels {
            labels.is_indonesian_market || labels.portfolio_stocks_mentioned.len() > 0
        } else {
            false
        }
    }
    
    /// Check if article is Prof Jiang relevant
    pub fn is_prof_jiang_relevant(&self) -> bool {
        if let Some(labels) = &self.labels {
            labels.prof_jiang_relevance.score > 0.5
        } else {
            false
        }
    }
}

/// Default Indonesian stock symbols for processing pipeline
pub fn default_indonesian_stocks() -> Vec<IndonesianStock> {
    vec![
        IndonesianStock::BMRI,
        IndonesianStock::BBRI,
        IndonesianStock::INCO,
        IndonesianStock::ANTM,
        IndonesianStock::PTBA,
        IndonesianStock::TAPG,
        IndonesianStock::TLKM,
        IndonesianStock::ASII,
        IndonesianStock::KLBF,
        IndonesianStock::TSPC,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processing_stats() {
        let mut stats = ProcessingStats::new();
        assert_eq!(stats.success_rate(), 0.0);
        assert_eq!(stats.articles_per_second(), 0.0);
        assert!(!stats.meets_quality_threshold());
        
        stats.articles_processed = 100;
        stats.processing_errors = 5;
        stats.articles_embedded = 95;
        stats.processing_duration_ms = 5000;
        
        assert_eq!(stats.success_rate(), 95.0);
        assert_eq!(stats.articles_per_second(), 20.0);
        assert!(stats.meets_quality_threshold());
    }

    #[test]
    fn test_processing_result() {
        let mut result = ProcessingResult::new(
            "article-123".to_string(),
            "Sample article content".to_string(),
        );
        
        assert!(!result.is_success());
        assert!(!result.is_indonesian_market_relevant());
        assert!(!result.is_prof_jiang_relevant());
        
        result.processing_status = ProcessingStatus::Processed;
        assert!(result.is_success());
    }

    #[test]
    fn test_default_indonesian_stocks() {
        let stocks = default_indonesian_stocks();
        assert_eq!(stocks.len(), 10);
        
        // Verify key stocks are present
        assert!(stocks.contains(&IndonesianStock::BMRI));
        assert!(stocks.contains(&IndonesianStock::BBRI));
        assert!(stocks.contains(&IndonesianStock::INCO));
        assert!(stocks.contains(&IndonesianStock::ANTM));
        assert!(stocks.contains(&IndonesianStock::PTBA));
    }
}