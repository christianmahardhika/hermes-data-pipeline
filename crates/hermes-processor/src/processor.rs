//! Main processor orchestrator for Clean → Label → Embed pipeline

use crate::{
    text_cleaner::{TextCleaner, CleaningResult, FinancialEntities},
    content_labeler::{ContentLabeler, ContentLabels},
    embedding_generator::{EmbeddingGenerator, EmbeddingVector, ContentAwareEmbeddingGenerator},
    ProcessingStats, ProcessingResult, default_indonesian_stocks,
};
use anyhow::{Result, anyhow};
use hermes_common::types::{Article, ProcessingStatus, IndonesianStock};
use std::time::Instant;
use tracing::{info, warn, error, debug};

/// Main Hermes processor for Indonesian market intelligence
pub struct HermesProcessor<T: EmbeddingGenerator> {
    text_cleaner: TextCleaner,
    content_labeler: ContentLabeler,
    embedding_generator: ContentAwareEmbeddingGenerator<T>,
    max_concurrent_processing: usize,
    processing_timeout_seconds: u64,
}

impl<T: EmbeddingGenerator> HermesProcessor<T> {
    /// Create new Hermes processor
    pub fn new(embedding_generator: T) -> Result<Self> {
        Ok(Self {
            text_cleaner: TextCleaner::new()?,
            content_labeler: ContentLabeler::new(),
            embedding_generator: ContentAwareEmbeddingGenerator::new(embedding_generator),
            max_concurrent_processing: 10,
            processing_timeout_seconds: 120,
        })
    }
    
    /// Configure max concurrent processing
    pub fn with_max_concurrent(mut self, max_concurrent: usize) -> Self {
        self.max_concurrent_processing = max_concurrent;
        self
    }
    
    /// Configure processing timeout
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.processing_timeout_seconds = timeout_seconds;
        self
    }
    
    /// Process single article through complete pipeline
    pub async fn process_article(&mut self, article: &Article) -> Result<ProcessingResult> {
        let start_time = Instant::now();
        let article_id = article.id.clone();
        let full_content = format!("{} {}", article.title, article.content);
        
        info!(
            article_id = %article_id,
            title = %article.title,
            content_length = full_content.len(),
            "🔄 Starting article processing pipeline"
        );
        
        let mut result = ProcessingResult::new(article_id.to_string(), full_content.clone());
        
        // Step 1: Text Cleaning
        match self.clean_article_content(&full_content) {
            Ok(cleaning_result) => {
                result.cleaned_content = Some(cleaning_result.cleaned_text.clone());
                debug!(
                    article_id = %article_id,
                    original_length = cleaning_result.original_length,
                    cleaned_length = cleaning_result.final_length,
                    reduction_pct = %format!("{:.1}%", cleaning_result.reduction_percentage()),
                    "✅ Text cleaning completed"
                );
            }
            Err(e) => {
                error!(article_id = %article_id, error = %e, "❌ Text cleaning failed");
                result.processing_status = ProcessingStatus::Failed;
                result.error_message = Some(e.to_string());
                result.processing_duration_ms = start_time.elapsed().as_millis() as u64;
                return Ok(result);
            }
        }
        
        // Step 2: Content Labeling
        let cleaned_text = result.cleaned_content.as_ref().unwrap();
        match self.label_article_content(&article.title, cleaned_text) {
            Ok(labels) => {
                result.labels = Some(labels.clone());
                debug!(
                    article_id = %article_id,
                    prof_jiang_score = labels.prof_jiang_relevance.score,
                    indonesian_market = labels.is_indonesian_market,
                    portfolio_stocks = labels.portfolio_stocks_mentioned.len(),
                    market_impact = ?labels.market_impact_level,
                    "✅ Content labeling completed"
                );
            }
            Err(e) => {
                error!(article_id = %article_id, error = %e, "❌ Content labeling failed");
                result.processing_status = ProcessingStatus::Failed;
                result.error_message = Some(e.to_string());
                result.processing_duration_ms = start_time.elapsed().as_millis() as u64;
                return Ok(result);
            }
        }
        
        // Step 3: Embedding Generation
        let labels = result.labels.as_ref().unwrap();
        match self.generate_article_embedding(cleaned_text, labels).await {
            Ok(embedding) => {
                debug!(
                    article_id = %article_id,
                    embedding_dimension = embedding.dimension,
                    quality_score = %format!("{:.2}", embedding.quality_score()),
                    "✅ Embedding generation completed"
                );
                result.embedding = Some(embedding);
            }
            Err(e) => {
                error!(article_id = %article_id, error = %e, "❌ Embedding generation failed");
                result.processing_status = ProcessingStatus::Failed;
                result.error_message = Some(e.to_string());
                result.processing_duration_ms = start_time.elapsed().as_millis() as u64;
                return Ok(result);
            }
        }
        
        // Mark as successfully processed
        result.processing_status = ProcessingStatus::Processed;
        result.processing_duration_ms = start_time.elapsed().as_millis() as u64;
        
        info!(
            article_id = %article_id,
            processing_duration_ms = result.processing_duration_ms,
            prof_jiang_relevant = result.is_prof_jiang_relevant(),
            indonesian_market = result.is_indonesian_market_relevant(),
            "🎯 Article processing pipeline completed successfully"
        );
        
        Ok(result)
    }
    
    /// Process multiple articles in batch
    pub async fn process_articles_batch(&mut self, articles: &[Article]) -> Result<ProcessingStats> {
        let start_time = Instant::now();
        let mut stats = ProcessingStats::new();
        stats.articles_processed = articles.len();
        
        info!(
            batch_size = articles.len(),
            max_concurrent = self.max_concurrent_processing,
            "🚀 Starting batch article processing"
        );
        
        // Process articles sequentially for now (can be made concurrent later)
        for article in articles {
            match self.process_article(article).await {
                Ok(result) => {
                    self.update_stats_from_result(&mut stats, &result);
                }
                Err(e) => {
                    stats.processing_errors += 1;
                    error!(
                        article_id = %article.id,
                        error = %e,
                        "❌ Critical error processing article"
                    );
                }
            }
        }
        
        stats.processing_duration_ms = start_time.elapsed().as_millis() as u64;
        
        info!(
            batch_size = stats.articles_processed,
            success_rate = %format!("{:.1}%", stats.success_rate()),
            articles_per_second = %format!("{:.1}", stats.articles_per_second()),
            indonesian_articles = stats.indonesian_articles,
            prof_jiang_relevant = stats.prof_jiang_relevant,
            portfolio_mentions = stats.portfolio_mentions,
            duration_ms = stats.processing_duration_ms,
            "📊 Batch processing completed"
        );
        
        Ok(stats)
    }
    
    /// Clean article content
    fn clean_article_content(&self, content: &str) -> Result<CleaningResult> {
        if content.trim().is_empty() {
            return Err(anyhow!("Article content is empty"));
        }
        
        let result = self.text_cleaner.clean_text(content);
        
        if !result.is_effective() {
            return Err(anyhow!("Text cleaning was not effective - content too reduced"));
        }
        
        Ok(result)
    }
    
    /// Label article content
    fn label_article_content(&self, title: &str, content: &str) -> Result<ContentLabels> {
        if content.trim().is_empty() {
            return Err(anyhow!("Content for labeling is empty"));
        }
        
        let labels = self.content_labeler.label_content(title, content);
        
        // Validate labels quality
        if labels.confidence_score < 0.3 {
            return Err(anyhow!("Content labeling confidence too low: {}", labels.confidence_score));
        }
        
        Ok(labels)
    }
    
    /// Generate article embedding
    async fn generate_article_embedding(
        &self,
        content: &str,
        labels: &ContentLabels,
    ) -> Result<EmbeddingVector> {
        if content.trim().is_empty() {
            return Err(anyhow!("Content for embedding is empty"));
        }
        
        let embedding = self.embedding_generator.generate_labeled_embedding(content, labels).await?;
        
        // Validate embedding quality
        if embedding.quality_score() < 0.3 {
            return Err(anyhow!("Embedding quality too low: {}", embedding.quality_score()));
        }
        
        Ok(embedding)
    }
    
    /// Update processing statistics from result
    fn update_stats_from_result(&self, stats: &mut ProcessingStats, result: &ProcessingResult) {
        match result.processing_status {
            ProcessingStatus::Processed => {
                stats.articles_cleaned += if result.cleaned_content.is_some() { 1 } else { 0 };
                stats.articles_labeled += if result.labels.is_some() { 1 } else { 0 };
                stats.articles_embedded += if result.embedding.is_some() { 1 } else { 0 };
                
                if result.is_indonesian_market_relevant() {
                    stats.indonesian_articles += 1;
                }
                
                if result.is_prof_jiang_relevant() {
                    stats.prof_jiang_relevant += 1;
                }
                
                if let Some(labels) = &result.labels {
                    stats.portfolio_mentions += labels.portfolio_stocks_mentioned.len();
                }
            }
            ProcessingStatus::Failed => {
                stats.processing_errors += 1;
            }
            _ => {}
        }
    }
    
    /// Extract financial entities from processed articles
    pub fn extract_batch_financial_entities(&self, results: &[ProcessingResult]) -> BatchFinancialAnalysis {
        let mut analysis = BatchFinancialAnalysis::new();
        
        for result in results {
            if let Some(cleaned_content) = &result.cleaned_content {
                let entities = self.text_cleaner.extract_financial_entities(cleaned_content);
                
                // Aggregate stock mentions
                for stock_symbol in &entities.stock_symbols {
                    *analysis.stock_mentions.entry(stock_symbol.clone()).or_insert(0) += 1;
                }
                
                // Aggregate institution mentions
                for institution in &entities.institutions {
                    *analysis.institution_mentions.entry(institution.clone()).or_insert(0) += 1;
                }
                
                analysis.total_currency_amounts += entities.currency_amounts.len();
                
                if entities.contains_portfolio_stocks() {
                    analysis.portfolio_relevant_articles += 1;
                }
            }
        }
        
        analysis.total_articles = results.len();
        analysis
    }
    
    /// Get processing pipeline health status
    pub fn get_pipeline_health(&self) -> PipelineHealth {
        PipelineHealth {
            text_cleaner_ready: true, // Always ready for mock
            content_labeler_ready: true, // Always ready for mock
            embedding_generator_ready: true, // Always ready for mock
            embedding_dimension: self.embedding_generator.embedding_dimension(),
            model_name: self.embedding_generator.model_name().to_string(),
            max_concurrent_processing: self.max_concurrent_processing,
            timeout_seconds: self.processing_timeout_seconds,
        }
    }
}

/// Batch financial analysis result
#[derive(Debug, Clone)]
pub struct BatchFinancialAnalysis {
    pub total_articles: usize,
    pub portfolio_relevant_articles: usize,
    pub stock_mentions: std::collections::HashMap<String, usize>,
    pub institution_mentions: std::collections::HashMap<String, usize>,
    pub total_currency_amounts: usize,
}

impl BatchFinancialAnalysis {
    fn new() -> Self {
        Self {
            total_articles: 0,
            portfolio_relevant_articles: 0,
            stock_mentions: std::collections::HashMap::new(),
            institution_mentions: std::collections::HashMap::new(),
            total_currency_amounts: 0,
        }
    }
    
    /// Get top mentioned stocks
    pub fn top_mentioned_stocks(&self, limit: usize) -> Vec<(String, usize)> {
        let mut stocks: Vec<_> = self.stock_mentions.iter()
            .map(|(stock, count)| (stock.clone(), *count))
            .collect();
        stocks.sort_by(|a, b| b.1.cmp(&a.1));
        stocks.truncate(limit);
        stocks
    }
    
    /// Get portfolio relevance percentage
    pub fn portfolio_relevance_percentage(&self) -> f64 {
        if self.total_articles == 0 {
            return 0.0;
        }
        (self.portfolio_relevant_articles as f64 / self.total_articles as f64) * 100.0
    }
}

/// Pipeline health status
#[derive(Debug, Clone)]
pub struct PipelineHealth {
    pub text_cleaner_ready: bool,
    pub content_labeler_ready: bool,
    pub embedding_generator_ready: bool,
    pub embedding_dimension: usize,
    pub model_name: String,
    pub max_concurrent_processing: usize,
    pub timeout_seconds: u64,
}

impl PipelineHealth {
    /// Check if entire pipeline is healthy
    pub fn is_healthy(&self) -> bool {
        self.text_cleaner_ready && self.content_labeler_ready && self.embedding_generator_ready
    }
    
    /// Get health summary
    pub fn health_summary(&self) -> String {
        if self.is_healthy() {
            format!(
                "Pipeline Healthy: {} embeddings, {} concurrent, {}s timeout",
                self.embedding_dimension, self.max_concurrent_processing, self.timeout_seconds
            )
        } else {
            "Pipeline Unhealthy: Some components not ready".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding_generator::MockEmbeddingGenerator;
    use chrono::Utc;

    fn create_test_article(id: &str, title: &str, content: &str) -> Article {
        Article::new(
            title.to_string(),
            content.to_string(),
            "https://example.com/test".to_string(),
        )
    }

    #[tokio::test]
    async fn test_processor_creation() {
        let embedding_generator = MockEmbeddingGenerator::new(128);
        let processor = HermesProcessor::new(embedding_generator).unwrap();
        
        let health = processor.get_pipeline_health();
        assert!(health.is_healthy());
        assert_eq!(health.embedding_dimension, 128);
        assert!(health.health_summary().contains("Pipeline Healthy"));
    }

    #[tokio::test]
    async fn test_single_article_processing() {
        let embedding_generator = MockEmbeddingGenerator::new_indonesian_optimized();
        let mut processor = HermesProcessor::new(embedding_generator).unwrap();
        
        let article = create_test_article(
            "test-123",
            "BMRI Q4 Earnings Report",
            "Bank Mandiri (BMRI) reported strong quarterly earnings with Rp 15 triliun profit, supported by Indonesia economic growth"
        );
        
        let result = processor.process_article(&article).await.unwrap();
        
        assert!(result.is_success());
        assert!(result.cleaned_content.is_some());
        assert!(result.labels.is_some());
        assert!(result.embedding.is_some());
        assert!(result.is_indonesian_market_relevant());
        assert!(result.processing_duration_ms > 0);
        
        let labels = result.labels.as_ref().unwrap();
        assert!(labels.is_indonesian_market);
        assert!(labels.portfolio_stocks_mentioned.contains(&IndonesianStock::BMRI));
    }

    #[tokio::test]
    async fn test_batch_processing() {
        let embedding_generator = MockEmbeddingGenerator::new(64);
        let mut processor = HermesProcessor::new(embedding_generator).unwrap();
        
        let articles = vec![
            create_test_article("1", "BMRI Earnings", "Bank Mandiri reports profit in Indonesia"),
            create_test_article("2", "BBRI Update", "Bank BRI expands digital services"),
            create_test_article("3", "INCO Mining", "Vale Indonesia increases nickel production"),
            create_test_article("4", "Tech News", "Apple releases new iPhone globally"),
        ];
        
        let stats = processor.process_articles_batch(&articles).await.unwrap();
        
        assert_eq!(stats.articles_processed, 4);
        assert!(stats.success_rate() > 90.0);
        assert!(stats.indonesian_articles >= 3); // First 3 are Indonesian
        assert!(stats.portfolio_mentions >= 3); // BMRI, BBRI, INCO mentions
        assert!(stats.articles_per_second() > 0.0);
        assert!(stats.meets_quality_threshold());
    }

    #[tokio::test]
    async fn test_prof_jiang_relevance_detection() {
        let embedding_generator = MockEmbeddingGenerator::new(128);
        let mut processor = HermesProcessor::new(embedding_generator).unwrap();
        
        let article = create_test_article(
            "geopolitical-1",
            "China Belt Road Initiative Impact",
            "Geopolitical competition intensifies as China's economic corridor strategy affects Indonesia trade relations with strategic implications"
        );
        
        let result = processor.process_article(&article).await.unwrap();
        
        assert!(result.is_success());
        assert!(result.is_prof_jiang_relevant());
        
        let labels = result.labels.as_ref().unwrap();
        assert!(labels.prof_jiang_relevance.score > 0.5);
        assert!(!labels.prof_jiang_relevance.matched_concepts.is_empty());
        assert!(labels.geopolitical_sensitivity_score > 0.0);
    }

    #[tokio::test]
    async fn test_financial_entity_extraction() {
        let embedding_generator = MockEmbeddingGenerator::new(64);
        let processor = HermesProcessor::new(embedding_generator).unwrap();
        
        let results = vec![
            ProcessingResult {
                article_id: "1".to_string(),
                original_content: "content".to_string(),
                cleaned_content: Some("BMRI and BBRI reported earnings, Bank Indonesia policy update".to_string()),
                labels: None,
                embedding: None,
                processing_status: ProcessingStatus::Processed,
                error_message: None,
                processing_duration_ms: 100,
            }
        ];
        
        let analysis = processor.extract_batch_financial_entities(&results);
        
        assert_eq!(analysis.total_articles, 1);
        assert!(analysis.portfolio_relevant_articles > 0);
        assert!(analysis.stock_mentions.contains_key("BMRI"));
        assert!(analysis.stock_mentions.contains_key("BBRI"));
        assert!(analysis.institution_mentions.contains_key("BI"));
        
        let top_stocks = analysis.top_mentioned_stocks(2);
        assert!(!top_stocks.is_empty());
    }

    #[tokio::test]
    async fn test_error_handling() {
        let embedding_generator = MockEmbeddingGenerator::new(32);
        let mut processor = HermesProcessor::new(embedding_generator).unwrap();
        
        // Test empty content
        let empty_article = create_test_article("empty", "Empty", "");
        let result = processor.process_article(&empty_article).await.unwrap();
        assert!(!result.is_success());
        assert!(result.error_message.is_some());
        
        // Test very short content (might fail confidence threshold)
        let short_article = create_test_article("short", "X", "Y");
        let result = processor.process_article(&short_article).await.unwrap();
        // Result depends on implementation - either success or failure is valid
        assert!(result.processing_duration_ms > 0);
    }

    #[tokio::test]
    async fn test_pipeline_configuration() {
        let embedding_generator = MockEmbeddingGenerator::new(256);
        let processor = HermesProcessor::new(embedding_generator)
            .unwrap()
            .with_max_concurrent(20)
            .with_timeout(300);
        
        let health = processor.get_pipeline_health();
        assert_eq!(health.max_concurrent_processing, 20);
        assert_eq!(health.timeout_seconds, 300);
        assert_eq!(health.embedding_dimension, 256);
    }

    #[test]
    fn test_batch_financial_analysis() {
        let mut analysis = BatchFinancialAnalysis::new();
        analysis.total_articles = 10;
        analysis.portfolio_relevant_articles = 7;
        
        analysis.stock_mentions.insert("BMRI".to_string(), 5);
        analysis.stock_mentions.insert("BBRI".to_string(), 3);
        analysis.stock_mentions.insert("INCO".to_string(), 8);
        
        assert_eq!(analysis.portfolio_relevance_percentage(), 70.0);
        
        let top_stocks = analysis.top_mentioned_stocks(2);
        assert_eq!(top_stocks.len(), 2);
        assert_eq!(top_stocks[0], ("INCO".to_string(), 8));
        assert_eq!(top_stocks[1], ("BMRI".to_string(), 5));
    }
}