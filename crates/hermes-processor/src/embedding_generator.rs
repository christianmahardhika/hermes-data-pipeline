//! Embedding generation for Indonesian market intelligence

use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};
use crate::content_labeler::{ContentLabels, MarketImpactLevel};

/// Trait for embedding generation (abstraction for different ML backends)
#[async_trait]
pub trait EmbeddingGenerator: Send + Sync {
    async fn generate_embedding(&self, text: &str) -> Result<EmbeddingVector>;
    async fn generate_batch_embeddings(&self, texts: &[String]) -> Result<Vec<EmbeddingVector>>;
    fn embedding_dimension(&self) -> usize;
    fn model_name(&self) -> &str;
}

/// Mock embedding generator for testing and development
#[derive(Debug, Clone)]
pub struct MockEmbeddingGenerator {
    pub dimension: usize,
    pub model_name: String,
    pub indonesian_boost: f32,
    pub prof_jiang_boost: f32,
}

impl MockEmbeddingGenerator {
    /// Create new mock embedding generator
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            model_name: "mock-embedding-model".to_string(),
            indonesian_boost: 1.2,
            prof_jiang_boost: 1.5,
        }
    }
    
    /// Create with Indonesian market optimization
    pub fn new_indonesian_optimized() -> Self {
        Self {
            dimension: 384, // Common embedding dimension
            model_name: "mock-indonesian-model".to_string(),
            indonesian_boost: 2.0,
            prof_jiang_boost: 1.8,
        }
    }
}

#[async_trait]
impl EmbeddingGenerator for MockEmbeddingGenerator {
    async fn generate_embedding(&self, text: &str) -> Result<EmbeddingVector> {
        debug!("🧠 Generating mock embedding for {} chars", text.len());
        
        // Simulate processing delay
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        // Create deterministic mock embedding based on text content
        let mut vector = Vec::with_capacity(self.dimension);
        let text_lower = text.to_lowercase();
        
        // Base embedding from text hash
        let text_hash = self.simple_hash(text);
        for i in 0..self.dimension {
            let base_value = ((text_hash + i) % 1000) as f32 / 1000.0;
            vector.push(base_value);
        }
        
        // Apply Indonesian market boosting
        if self.is_indonesian_content(&text_lower) {
            self.apply_indonesian_boost(&mut vector);
        }
        
        // Apply Prof Jiang relevance boosting
        if self.is_prof_jiang_relevant(&text_lower) {
            self.apply_prof_jiang_boost(&mut vector);
        }
        
        // Apply portfolio stock boosting
        if self.mentions_portfolio_stocks(&text_lower) {
            self.apply_portfolio_boost(&mut vector);
        }
        
        // Normalize vector
        self.normalize_vector(&mut vector);
        
        Ok(EmbeddingVector {
            vector,
            dimension: self.dimension,
            model: self.model_name.clone(),
            metadata: EmbeddingMetadata {
                text_length: text.len(),
                indonesian_content: self.is_indonesian_content(&text_lower),
                prof_jiang_relevant: self.is_prof_jiang_relevant(&text_lower),
                portfolio_mentions: self.count_portfolio_mentions(&text_lower),
                processing_duration_ms: 10, // Mock processing time
            },
        })
    }
    
    async fn generate_batch_embeddings(&self, texts: &[String]) -> Result<Vec<EmbeddingVector>> {
        let mut embeddings = Vec::new();
        
        info!("🧠 Generating {} mock embeddings in batch", texts.len());
        
        for text in texts {
            let embedding = self.generate_embedding(text).await?;
            embeddings.push(embedding);
        }
        
        Ok(embeddings)
    }
    
    fn embedding_dimension(&self) -> usize {
        self.dimension
    }
    
    fn model_name(&self) -> &str {
        &self.model_name
    }
}

impl MockEmbeddingGenerator {
    /// Simple hash function for deterministic mock embeddings
    fn simple_hash(&self, text: &str) -> usize {
        text.chars().map(|c| c as usize).sum()
    }
    
    /// Check if content is Indonesian market related
    fn is_indonesian_content(&self, text: &str) -> bool {
        text.contains("indonesia") || text.contains("rupiah") || text.contains("jakarta") ||
        text.contains("bmri") || text.contains("bbri") || text.contains("inco")
    }
    
    /// Check if content is Prof Jiang relevant
    fn is_prof_jiang_relevant(&self, text: &str) -> bool {
        text.contains("geopolitical") || text.contains("strategic") || text.contains("trade war") ||
        text.contains("china") || text.contains("competition") || text.contains("economic corridor")
    }
    
    /// Check if text mentions Christian's portfolio stocks
    fn mentions_portfolio_stocks(&self, text: &str) -> bool {
        let portfolio_stocks = ["bmri", "bbri", "inco", "antm", "ptba", "tapg", "tlkm", "asii", "klbf", "tspc"];
        portfolio_stocks.iter().any(|stock| text.contains(stock))
    }
    
    /// Count portfolio stock mentions
    fn count_portfolio_mentions(&self, text: &str) -> usize {
        let portfolio_stocks = ["bmri", "bbri", "inco", "antm", "ptba", "tapg", "tlkm", "asii", "klbf", "tspc"];
        portfolio_stocks.iter().filter(|stock| text.contains(*stock)).count()
    }
    
    /// Apply Indonesian market content boosting
    fn apply_indonesian_boost(&self, vector: &mut [f32]) {
        for val in vector.iter_mut() {
            *val *= self.indonesian_boost;
        }
    }
    
    /// Apply Prof Jiang relevance boosting
    fn apply_prof_jiang_boost(&self, vector: &mut [f32]) {
        for val in vector.iter_mut() {
            *val *= self.prof_jiang_boost;
        }
    }
    
    /// Apply portfolio stock mention boosting
    fn apply_portfolio_boost(&self, vector: &mut [f32]) {
        for val in vector.iter_mut() {
            *val *= 1.3; // Portfolio boost factor
        }
    }
    
    /// Normalize vector to unit length
    fn normalize_vector(&self, vector: &mut [f32]) {
        let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in vector.iter_mut() {
                *val /= magnitude;
            }
        }
    }
}

/// Enhanced embedding generator with content-aware optimization
#[derive(Debug, Clone)]
pub struct ContentAwareEmbeddingGenerator<T: EmbeddingGenerator> {
    base_generator: T,
    indonesian_weight: f32,
    prof_jiang_weight: f32,
    portfolio_weight: f32,
}

impl<T: EmbeddingGenerator> ContentAwareEmbeddingGenerator<T> {
    /// Create new content-aware embedding generator
    pub fn new(base_generator: T) -> Self {
        Self {
            base_generator,
            indonesian_weight: 1.2,
            prof_jiang_weight: 1.5,
            portfolio_weight: 1.3,
        }
    }
    
    /// Generate embedding with content labels for optimization
    pub async fn generate_labeled_embedding(
        &self,
        text: &str,
        labels: &ContentLabels,
    ) -> Result<EmbeddingVector> {
        let mut embedding = self.base_generator.generate_embedding(text).await?;
        
        // Apply content-specific optimizations
        self.optimize_embedding_for_content(&mut embedding.vector, labels);
        
        // Update metadata
        embedding.metadata.indonesian_content = labels.is_indonesian_market;
        embedding.metadata.prof_jiang_relevant = labels.prof_jiang_relevance.score > 0.5;
        embedding.metadata.portfolio_mentions = labels.portfolio_stocks_mentioned.len();
        
        Ok(embedding)
    }
    
    /// Optimize embedding vector based on content labels
    fn optimize_embedding_for_content(&self, vector: &mut [f32], labels: &ContentLabels) {
        // Indonesian market boosting
        if labels.is_indonesian_market {
            for val in vector.iter_mut() {
                *val *= self.indonesian_weight;
            }
        }
        
        // Prof Jiang relevance boosting
        if labels.prof_jiang_relevance.score > 0.5 {
            let boost = 1.0 + (labels.prof_jiang_relevance.score * self.prof_jiang_weight as f64);
            for val in vector.iter_mut() {
                *val *= boost as f32;
            }
        }
        
        // Portfolio stock boosting
        if !labels.portfolio_stocks_mentioned.is_empty() {
            let portfolio_boost = 1.0 + (labels.portfolio_stocks_mentioned.len() as f32 * 0.1);
            for val in vector.iter_mut() {
                *val *= portfolio_boost;
            }
        }
        
        // Market impact boosting
        let market_boost = match labels.market_impact_level {
            MarketImpactLevel::High => 1.4,
            MarketImpactLevel::Medium => 1.2,
            MarketImpactLevel::Low => 1.0,
        };
        
        for val in vector.iter_mut() {
            *val *= market_boost;
        }
        
        // Renormalize after boosting
        let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in vector.iter_mut() {
                *val /= magnitude;
            }
        }
    }
}

#[async_trait]
impl<T: EmbeddingGenerator> EmbeddingGenerator for ContentAwareEmbeddingGenerator<T> {
    async fn generate_embedding(&self, text: &str) -> Result<EmbeddingVector> {
        self.base_generator.generate_embedding(text).await
    }
    
    async fn generate_batch_embeddings(&self, texts: &[String]) -> Result<Vec<EmbeddingVector>> {
        self.base_generator.generate_batch_embeddings(texts).await
    }
    
    fn embedding_dimension(&self) -> usize {
        self.base_generator.embedding_dimension()
    }
    
    fn model_name(&self) -> &str {
        self.base_generator.model_name()
    }
}

/// Embedding vector with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingVector {
    pub vector: Vec<f32>,
    pub dimension: usize,
    pub model: String,
    pub metadata: EmbeddingMetadata,
}

impl EmbeddingVector {
    /// Calculate cosine similarity with another embedding
    pub fn cosine_similarity(&self, other: &EmbeddingVector) -> f32 {
        if self.dimension != other.dimension {
            return 0.0;
        }
        
        let dot_product: f32 = self.vector.iter()
            .zip(other.vector.iter())
            .map(|(a, b)| a * b)
            .sum();
        
        let magnitude_a: f32 = self.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = other.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if magnitude_a > 0.0 && magnitude_b > 0.0 {
            dot_product / (magnitude_a * magnitude_b)
        } else {
            0.0
        }
    }
    
    /// Check if this embedding is similar to another (threshold-based)
    pub fn is_similar(&self, other: &EmbeddingVector, threshold: f32) -> bool {
        self.cosine_similarity(other) >= threshold
    }
    
    /// Get embedding quality score based on metadata
    pub fn quality_score(&self) -> f32 {
        let mut score = 0.5; // Base score
        
        if self.metadata.indonesian_content {
            score += 0.2;
        }
        if self.metadata.prof_jiang_relevant {
            score += 0.2;
        }
        if self.metadata.portfolio_mentions > 0 {
            score += 0.1 * self.metadata.portfolio_mentions as f32;
        }
        
        score.min(1.0)
    }
}

/// Embedding metadata for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    pub text_length: usize,
    pub indonesian_content: bool,
    pub prof_jiang_relevant: bool,
    pub portfolio_mentions: usize,
    pub processing_duration_ms: u64,
}

/// Embedding similarity search result
#[derive(Debug, Clone)]
pub struct SimilarityResult {
    pub embedding_id: String,
    pub similarity_score: f32,
    pub metadata: EmbeddingMetadata,
}

/// Embedding database operations (trait for future implementation)
#[async_trait]
pub trait EmbeddingStore: Send + Sync {
    async fn store_embedding(&self, id: &str, embedding: &EmbeddingVector) -> Result<()>;
    async fn get_embedding(&self, id: &str) -> Result<Option<EmbeddingVector>>;
    async fn similarity_search(
        &self,
        query_embedding: &EmbeddingVector,
        top_k: usize,
        threshold: f32,
    ) -> Result<Vec<SimilarityResult>>;
    async fn delete_embedding(&self, id: &str) -> Result<()>;
}

/// Mock embedding store for testing
#[derive(Debug, Clone)]
pub struct MockEmbeddingStore {
    pub embeddings: HashMap<String, EmbeddingVector>,
}

impl MockEmbeddingStore {
    pub fn new() -> Self {
        Self {
            embeddings: HashMap::new(),
        }
    }
}

#[async_trait]
impl EmbeddingStore for MockEmbeddingStore {
    async fn store_embedding(&self, id: &str, embedding: &EmbeddingVector) -> Result<()> {
        // In real implementation, this would be mutable
        debug!("📦 Mock storing embedding for ID: {}", id);
        Ok(())
    }
    
    async fn get_embedding(&self, id: &str) -> Result<Option<EmbeddingVector>> {
        debug!("🔍 Mock retrieving embedding for ID: {}", id);
        Ok(self.embeddings.get(id).cloned())
    }
    
    async fn similarity_search(
        &self,
        query_embedding: &EmbeddingVector,
        top_k: usize,
        threshold: f32,
    ) -> Result<Vec<SimilarityResult>> {
        debug!(
            "🔍 Mock similarity search: top_k={}, threshold={}",
            top_k, threshold
        );
        
        let mut results = Vec::new();
        
        for (id, embedding) in &self.embeddings {
            let similarity = query_embedding.cosine_similarity(embedding);
            if similarity >= threshold {
                results.push(SimilarityResult {
                    embedding_id: id.clone(),
                    similarity_score: similarity,
                    metadata: embedding.metadata.clone(),
                });
            }
        }
        
        // Sort by similarity (descending) and take top_k
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
        results.truncate(top_k);
        
        Ok(results)
    }
    
    async fn delete_embedding(&self, id: &str) -> Result<()> {
        debug!("🗑️ Mock deleting embedding for ID: {}", id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_embedding_generator() {
        let generator = MockEmbeddingGenerator::new(128);
        assert_eq!(generator.embedding_dimension(), 128);
        assert_eq!(generator.model_name(), "mock-embedding-model");
        
        let embedding = generator.generate_embedding("BMRI reports strong Q4 earnings in Indonesia").await.unwrap();
        assert_eq!(embedding.dimension, 128);
        assert_eq!(embedding.vector.len(), 128);
        assert!(embedding.metadata.indonesian_content);
        assert!(embedding.metadata.portfolio_mentions > 0);
    }

    #[tokio::test]
    async fn test_indonesian_optimized_generator() {
        let generator = MockEmbeddingGenerator::new_indonesian_optimized();
        assert_eq!(generator.embedding_dimension(), 384);
        assert_eq!(generator.model_name(), "mock-indonesian-model");
        
        let embedding = generator.generate_embedding("Bank Indonesia policy update").await.unwrap();
        assert!(embedding.metadata.indonesian_content);
    }

    #[tokio::test]
    async fn test_batch_embedding_generation() {
        let generator = MockEmbeddingGenerator::new(64);
        let texts = vec![
            "BMRI earnings report".to_string(),
            "BBRI market analysis".to_string(),
            "INCO commodity update".to_string(),
        ];
        
        let embeddings = generator.generate_batch_embeddings(&texts).await.unwrap();
        assert_eq!(embeddings.len(), 3);
        
        for embedding in &embeddings {
            assert_eq!(embedding.dimension, 64);
            assert!(embedding.metadata.portfolio_mentions > 0);
        }
    }

    #[test]
    fn test_cosine_similarity() {
        let embedding1 = EmbeddingVector {
            vector: vec![1.0, 0.0, 0.0],
            dimension: 3,
            model: "test".to_string(),
            metadata: EmbeddingMetadata {
                text_length: 10,
                indonesian_content: false,
                prof_jiang_relevant: false,
                portfolio_mentions: 0,
                processing_duration_ms: 5,
            },
        };
        
        let embedding2 = EmbeddingVector {
            vector: vec![0.0, 1.0, 0.0],
            dimension: 3,
            model: "test".to_string(),
            metadata: embedding1.metadata.clone(),
        };
        
        let embedding3 = EmbeddingVector {
            vector: vec![1.0, 0.0, 0.0],
            dimension: 3,
            model: "test".to_string(),
            metadata: embedding1.metadata.clone(),
        };
        
        assert!((embedding1.cosine_similarity(&embedding2) - 0.0).abs() < 0.001);
        assert!((embedding1.cosine_similarity(&embedding3) - 1.0).abs() < 0.001);
        assert!(embedding1.is_similar(&embedding3, 0.9));
        assert!(!embedding1.is_similar(&embedding2, 0.9));
    }

    #[test]
    fn test_embedding_quality_score() {
        let mut metadata = EmbeddingMetadata {
            text_length: 100,
            indonesian_content: true,
            prof_jiang_relevant: true,
            portfolio_mentions: 2,
            processing_duration_ms: 10,
        };
        
        let embedding = EmbeddingVector {
            vector: vec![0.5; 10],
            dimension: 10,
            model: "test".to_string(),
            metadata: metadata.clone(),
        };
        
        let quality = embedding.quality_score();
        assert!(quality > 0.8); // High quality due to all positive indicators
        
        metadata.indonesian_content = false;
        metadata.prof_jiang_relevant = false;
        metadata.portfolio_mentions = 0;
        
        let low_quality_embedding = EmbeddingVector {
            vector: vec![0.5; 10],
            dimension: 10,
            model: "test".to_string(),
            metadata,
        };
        
        assert!(low_quality_embedding.quality_score() < 0.6);
    }

    #[tokio::test]
    async fn test_mock_embedding_store() {
        let store = MockEmbeddingStore::new();
        
        let embedding = EmbeddingVector {
            vector: vec![0.1, 0.2, 0.3],
            dimension: 3,
            model: "test".to_string(),
            metadata: EmbeddingMetadata {
                text_length: 50,
                indonesian_content: true,
                prof_jiang_relevant: false,
                portfolio_mentions: 1,
                processing_duration_ms: 5,
            },
        };
        
        // Test store operation
        let result = store.store_embedding("test-id", &embedding).await;
        assert!(result.is_ok());
        
        // Test similarity search with empty store
        let results = store.similarity_search(&embedding, 5, 0.5).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_content_aware_generator() {
        use crate::content_labeler::{ContentLabeler, ProfJiangRelevance, ContentCategory, MarketImpactLevel};
        use hermes_common::types::IndonesianStock;
        
        let base_generator = MockEmbeddingGenerator::new(64);
        let content_aware = ContentAwareEmbeddingGenerator::new(base_generator);
        
        // Create mock labels
        let labels = crate::content_labeler::ContentLabels {
            prof_jiang_relevance: ProfJiangRelevance {
                score: 0.8,
                confidence: 0.9,
                matched_concepts: vec!["geopolitical".to_string()],
                geostrategy_elements: vec!["economic corridor".to_string()],
                game_theory_aspects: vec!["competition".to_string()],
            },
            is_indonesian_market: true,
            portfolio_stocks_mentioned: vec![IndonesianStock::BMRI, IndonesianStock::BBRI],
            geopolitical_sensitivity_score: 0.7,
            primary_category: ContentCategory::MarketAnalysis,
            secondary_categories: vec![],
            sentiment_score: 0.2,
            market_impact_level: MarketImpactLevel::High,
            confidence_score: 0.9,
            processing_duration_ms: 15,
        };
        
        let embedding = content_aware.generate_labeled_embedding(
            "Indonesia market analysis with BMRI focus",
            &labels
        ).await.unwrap();
        
        assert!(embedding.metadata.indonesian_content);
        assert!(embedding.metadata.prof_jiang_relevant);
        assert_eq!(embedding.metadata.portfolio_mentions, 2);
    }
}