pub mod aggregator;
pub mod sentiment;
pub mod jiang_correlation;

pub use aggregator::{NewsAggregator, TimeframeEngine, Timeframe, NewsArticle, TimeframeAnalysis};
pub use sentiment::{IndonesianSentimentAnalyzer, SentimentCategory, BilingualProcessor};
pub use jiang_correlation::{JiangCorrelationEngine, HistoricalPattern, JiangCategory};

use anyhow::Result;

/// News Intelligence Correlation System
/// 
/// Multi-timeframe news analysis with Indonesian bilingual sentiment processing
/// and Prof Jiang Predictive History Framework correlations for portfolio intelligence.
pub struct NewsIntelligenceSystem {
    pub aggregator: NewsAggregator,
    pub sentiment_analyzer: IndonesianSentimentAnalyzer,
    pub jiang_engine: JiangCorrelationEngine,
}

impl NewsIntelligenceSystem {
    pub fn new() -> Self {
        Self {
            aggregator: NewsAggregator::new(),
            sentiment_analyzer: IndonesianSentimentAnalyzer::new(),
            jiang_engine: JiangCorrelationEngine::new(),
        }
    }
    
    /// Execute comprehensive multi-timeframe analysis
    pub async fn execute_comprehensive_analysis(&self) -> Result<ComprehensiveReport> {
        let timeframes = vec![
            Timeframe::Week,
            Timeframe::Month, 
            Timeframe::Quarter,
            Timeframe::HalfYear,
        ];
        
        let timeframe_analyses = self.aggregator
            .timeframe_engine
            .process_concurrent_timeframes(&self.aggregator, timeframes)
            .await?;
        
        Ok(ComprehensiveReport {
            timeframe_analyses,
            generated_at: chrono::Utc::now(),
            system_performance: SystemPerformance::default(),
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct ComprehensiveReport {
    pub timeframe_analyses: std::collections::HashMap<Timeframe, TimeframeAnalysis>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub system_performance: SystemPerformance,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct SystemPerformance {
    pub processing_time_seconds: f64,
    pub articles_processed: usize,
    pub sentiment_accuracy: Option<f64>,
    pub correlation_precision: Option<f64>,
}