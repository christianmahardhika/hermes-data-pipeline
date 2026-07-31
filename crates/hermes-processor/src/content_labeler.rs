//! Content labeling for Prof Jiang analysis and Indonesian market intelligence

use hermes_common::types::IndonesianStock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Content labeler for Prof Jiang predictive history framework
#[derive(Debug, Clone)]
pub struct ContentLabeler {
    prof_jiang_keywords: HashMap<String, f64>,
    geopolitical_patterns: Vec<GeopoliticalPattern>,
    indonesian_market_indicators: Vec<String>,
}

impl ContentLabeler {
    /// Create new content labeler with Prof Jiang framework
    pub fn new() -> Self {
        Self {
            prof_jiang_keywords: Self::build_prof_jiang_keywords(),
            geopolitical_patterns: Self::build_geopolitical_patterns(),
            indonesian_market_indicators: Self::build_indonesian_indicators(),
        }
    }

    /// Label content with comprehensive analysis
    pub fn label_content(&self, title: &str, content: &str) -> ContentLabels {
        let start_time = std::time::Instant::now();
        
        debug!("🏷️ Starting content labeling for: {}", title);
        
        let full_text = format!("{} {}", title, content);
        
        // Prof Jiang relevance analysis
        let prof_jiang_relevance = self.analyze_prof_jiang_relevance(&full_text);
        
        // Indonesian market analysis
        let is_indonesian_market = self.is_indonesian_market_content(&full_text);
        let portfolio_stocks = self.detect_portfolio_stocks(&full_text);
        
        // Geopolitical sensitivity
        let geopolitical_sensitivity = self.analyze_geopolitical_sensitivity(&full_text);
        
        // Content categorization
        let primary_category = self.categorize_content(&full_text);
        let secondary_categories = self.get_secondary_categories(&full_text);
        
        // Sentiment analysis (basic)
        let sentiment_score = self.analyze_sentiment(&full_text);
        
        // Market impact assessment
        let market_impact = self.assess_market_impact(&full_text, &portfolio_stocks);
        
        let processing_duration = start_time.elapsed();
        
        info!(
            title = title,
            prof_jiang_score = prof_jiang_relevance.score,
            indonesian_market = is_indonesian_market,
            portfolio_mentions = portfolio_stocks.len(),
            geopolitical_score = geopolitical_sensitivity,
            market_impact = ?market_impact,
            duration_ms = processing_duration.as_millis(),
            "✅ Content labeling completed"
        );

        ContentLabels {
            prof_jiang_relevance,
            is_indonesian_market,
            portfolio_stocks_mentioned: portfolio_stocks,
            geopolitical_sensitivity_score: geopolitical_sensitivity,
            primary_category,
            secondary_categories,
            sentiment_score,
            market_impact_level: market_impact,
            confidence_score: self.calculate_confidence_score(&full_text),
            processing_duration_ms: processing_duration.as_millis() as u64,
        }
    }
    
    /// Analyze Prof Jiang predictive history relevance
    fn analyze_prof_jiang_relevance(&self, text: &str) -> ProfJiangRelevance {
        let text_lower = text.to_lowercase();
        let mut total_score = 0.0;
        let mut matched_concepts = Vec::new();
        
        for (keyword, weight) in &self.prof_jiang_keywords {
            if text_lower.contains(keyword) {
                total_score += weight;
                matched_concepts.push(keyword.clone());
            }
        }
        
        // Geopolitical pattern matching
        for pattern in &self.geopolitical_patterns {
            if pattern.matches(&text_lower) {
                total_score += pattern.weight;
                matched_concepts.push(pattern.name.clone());
            }
        }
        
        // Normalize score to 0-1 range
        let normalized_score = (total_score / 10.0).min(1.0);
        
        ProfJiangRelevance {
            score: normalized_score,
            confidence: if matched_concepts.len() >= 3 { 0.9 } else { 0.6 },
            matched_concepts,
            geostrategy_elements: self.extract_geostrategy_elements(&text_lower),
            game_theory_aspects: self.extract_game_theory_aspects(&text_lower),
        }
    }
    
    /// Check if content is Indonesian market related
    fn is_indonesian_market_content(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        
        self.indonesian_market_indicators.iter().any(|indicator| {
            text_lower.contains(indicator)
        })
    }
    
    /// Detect Christian's portfolio stock mentions
    fn detect_portfolio_stocks(&self, text: &str) -> Vec<IndonesianStock> {
        let mut detected_stocks = Vec::new();
        let text_upper = text.to_uppercase();
        
        let stock_mappings = [
            ("BMRI", IndonesianStock::BMRI),
            ("BANK MANDIRI", IndonesianStock::BMRI),
            ("BBRI", IndonesianStock::BBRI),
            ("BANK BRI", IndonesianStock::BBRI),
            ("INCO", IndonesianStock::INCO),
            ("VALE INDONESIA", IndonesianStock::INCO),
            ("ANTM", IndonesianStock::ANTM),
            ("ANEKA TAMBANG", IndonesianStock::ANTM),
            ("PTBA", IndonesianStock::PTBA),
            ("BUKIT ASAM", IndonesianStock::PTBA),
            ("TAPG", IndonesianStock::TAPG),
            ("TRIPUTRA AGRO", IndonesianStock::TAPG),
            ("TLKM", IndonesianStock::TLKM),
            ("TELKOM", IndonesianStock::TLKM),
            ("ASII", IndonesianStock::ASII),
            ("ASTRA", IndonesianStock::ASII),
            ("KLBF", IndonesianStock::KLBF),
            ("KALBE", IndonesianStock::KLBF),
            ("TSPC", IndonesianStock::TSPC),
            ("TEMPO SCAN", IndonesianStock::TSPC),
        ];
        
        for (pattern, stock) in &stock_mappings {
            if text_upper.contains(pattern) {
                if !detected_stocks.contains(stock) {
                    detected_stocks.push(*stock);
                }
            }
        }
        
        detected_stocks
    }
    
    /// Analyze geopolitical sensitivity (Prof Jiang framework)
    fn analyze_geopolitical_sensitivity(&self, text: &str) -> f64 {
        let text_lower = text.to_lowercase();
        let mut sensitivity_score: f64 = 0.0;
        
        // High sensitivity keywords
        let high_sensitivity = ["war", "conflict", "sanctions", "trade war", "diplomatic", "military"];
        for keyword in &high_sensitivity {
            if text_lower.contains(keyword) {
                sensitivity_score += 0.3;
            }
        }
        
        // Medium sensitivity keywords
        let medium_sensitivity = ["tariff", "embargo", "alliance", "treaty", "negotiation"];
        for keyword in &medium_sensitivity {
            if text_lower.contains(keyword) {
                sensitivity_score += 0.2;
            }
        }
        
        // Indonesian specific geopolitical markers
        let indonesian_geopolitical = ["asean", "china belt road", "south china sea", "jokowi", "prabowo"];
        for keyword in &indonesian_geopolitical {
            if text_lower.contains(keyword) {
                sensitivity_score += 0.25;
            }
        }
        
        sensitivity_score.min(1.0)
    }
    
    /// Categorize content type
    fn categorize_content(&self, text: &str) -> ContentCategory {
        let text_lower = text.to_lowercase();
        
        if text_lower.contains("earnings") || text_lower.contains("profit") || text_lower.contains("laba") {
            ContentCategory::FinancialReport
        } else if text_lower.contains("policy") || text_lower.contains("regulation") || text_lower.contains("kebijakan") {
            ContentCategory::PolicyUpdate
        } else if text_lower.contains("market") || text_lower.contains("trading") || text_lower.contains("pasar") {
            ContentCategory::MarketAnalysis
        } else if text_lower.contains("merger") || text_lower.contains("acquisition") || text_lower.contains("akuisisi") {
            ContentCategory::CorporateAction
        } else {
            ContentCategory::GeneralNews
        }
    }
    
    /// Get secondary categories
    fn get_secondary_categories(&self, text: &str) -> Vec<ContentCategory> {
        let mut categories = Vec::new();
        let text_lower = text.to_lowercase();
        
        if text_lower.contains("commodity") || text_lower.contains("komoditas") {
            categories.push(ContentCategory::CommodityNews);
        }
        if text_lower.contains("technology") || text_lower.contains("teknologi") {
            categories.push(ContentCategory::TechnologyNews);
        }
        if text_lower.contains("infrastructure") || text_lower.contains("infrastruktur") {
            categories.push(ContentCategory::InfrastructureNews);
        }
        
        categories
    }
    
    /// Basic sentiment analysis
    fn analyze_sentiment(&self, text: &str) -> f64 {
        let text_lower = text.to_lowercase();
        let mut sentiment_score: f64 = 0.0;
        
        // Positive indicators
        let positive_words = ["growth", "profit", "increase", "positive", "strong", "naik", "untung", "positif"];
        for word in &positive_words {
            if text_lower.contains(word) {
                sentiment_score += 0.1;
            }
        }
        
        // Negative indicators
        let negative_words = ["loss", "decline", "negative", "weak", "fall", "turun", "rugi", "negatif"];
        for word in &negative_words {
            if text_lower.contains(word) {
                sentiment_score -= 0.1;
            }
        }
        
        sentiment_score.max(-1.0).min(1.0)
    }
    
    /// Assess market impact potential
    fn assess_market_impact(&self, text: &str, portfolio_stocks: &[IndonesianStock]) -> MarketImpactLevel {
        if portfolio_stocks.len() > 2 {
            return MarketImpactLevel::High;
        }
        
        let text_lower = text.to_lowercase();
        
        if text_lower.contains("bank indonesia") || text_lower.contains("ojk") || text_lower.contains("government") {
            MarketImpactLevel::High
        } else if portfolio_stocks.len() > 0 || text_lower.contains("indonesia") {
            MarketImpactLevel::Medium
        } else {
            MarketImpactLevel::Low
        }
    }
    
    /// Calculate overall confidence score
    fn calculate_confidence_score(&self, text: &str) -> f64 {
        let text_length = text.len();
        
        if text_length > 1000 {
            0.9
        } else if text_length > 500 {
            0.7
        } else {
            0.5
        }
    }
    
    /// Extract geostrategy elements for Prof Jiang analysis
    fn extract_geostrategy_elements(&self, text: &str) -> Vec<String> {
        let mut elements = Vec::new();
        
        let geostrategy_indicators = [
            "economic corridor", "trade route", "supply chain", "energy security",
            "maritime", "territorial", "strategic partnership"
        ];
        
        for indicator in &geostrategy_indicators {
            if text.contains(indicator) {
                elements.push(indicator.to_string());
            }
        }
        
        elements
    }
    
    /// Extract game theory aspects
    fn extract_game_theory_aspects(&self, text: &str) -> Vec<String> {
        let mut aspects = Vec::new();
        
        let game_theory_markers = [
            "competition", "cooperation", "negotiation", "strategy",
            "alliance", "rivalry", "equilibrium"
        ];
        
        for marker in &game_theory_markers {
            if text.contains(marker) {
                aspects.push(marker.to_string());
            }
        }
        
        aspects
    }
    
    /// Build Prof Jiang keyword weights
    fn build_prof_jiang_keywords() -> HashMap<String, f64> {
        let mut keywords = HashMap::new();
        
        // High weight keywords (geostrategy core)
        keywords.insert("geopolitical".to_string(), 2.0);
        keywords.insert("strategic".to_string(), 1.5);
        keywords.insert("economic corridor".to_string(), 2.5);
        keywords.insert("belt and road".to_string(), 2.0);
        keywords.insert("trade war".to_string(), 2.0);
        
        // Medium weight keywords
        keywords.insert("china".to_string(), 1.0);
        keywords.insert("usa".to_string(), 1.0);
        keywords.insert("russia".to_string(), 1.0);
        keywords.insert("indonesia".to_string(), 1.2);
        keywords.insert("asean".to_string(), 1.3);
        
        // Game theory aspects
        keywords.insert("competition".to_string(), 1.0);
        keywords.insert("cooperation".to_string(), 1.0);
        keywords.insert("alliance".to_string(), 1.2);
        keywords.insert("negotiation".to_string(), 0.8);
        
        keywords
    }
    
    /// Build geopolitical patterns
    fn build_geopolitical_patterns() -> Vec<GeopoliticalPattern> {
        vec![
            GeopoliticalPattern {
                name: "energy_security".to_string(),
                pattern: "energy.*security|oil.*supply|gas.*pipeline".to_string(),
                weight: 1.5,
            },
            GeopoliticalPattern {
                name: "maritime_dispute".to_string(),
                pattern: "south china sea|maritime.*dispute|territorial.*waters".to_string(),
                weight: 2.0,
            },
            GeopoliticalPattern {
                name: "economic_warfare".to_string(),
                pattern: "trade.*war|economic.*sanctions|tariff.*war".to_string(),
                weight: 2.2,
            },
        ]
    }
    
    /// Build Indonesian market indicators
    fn build_indonesian_indicators() -> Vec<String> {
        vec![
            "indonesia".to_string(),
            "jakarta".to_string(),
            "rupiah".to_string(),
            "idr".to_string(),
            "bei".to_string(),
            "ihsg".to_string(),
            "jokowi".to_string(),
            "prabowo".to_string(),
            "bank indonesia".to_string(),
            "ojk".to_string(),
            "kemenkeu".to_string(),
            "pertamina".to_string(),
            "pln".to_string(),
            "garuda".to_string(),
        ]
    }
}

/// Prof Jiang relevance analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfJiangRelevance {
    pub score: f64,
    pub confidence: f64,
    pub matched_concepts: Vec<String>,
    pub geostrategy_elements: Vec<String>,
    pub game_theory_aspects: Vec<String>,
}

/// Comprehensive content labels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentLabels {
    pub prof_jiang_relevance: ProfJiangRelevance,
    pub is_indonesian_market: bool,
    pub portfolio_stocks_mentioned: Vec<IndonesianStock>,
    pub geopolitical_sensitivity_score: f64,
    pub primary_category: ContentCategory,
    pub secondary_categories: Vec<ContentCategory>,
    pub sentiment_score: f64,
    pub market_impact_level: MarketImpactLevel,
    pub confidence_score: f64,
    pub processing_duration_ms: u64,
}

/// Content category classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentCategory {
    FinancialReport,
    PolicyUpdate,
    MarketAnalysis,
    CorporateAction,
    CommodityNews,
    TechnologyNews,
    InfrastructureNews,
    GeneralNews,
}

/// Market impact assessment level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketImpactLevel {
    High,
    Medium,
    Low,
}

/// Geopolitical pattern for Prof Jiang analysis
#[derive(Debug, Clone)]
pub struct GeopoliticalPattern {
    pub name: String,
    pub pattern: String,
    pub weight: f64,
}

impl GeopoliticalPattern {
    /// Check if pattern matches text
    pub fn matches(&self, text: &str) -> bool {
        // Simple contains check for now - can be upgraded to regex
        self.pattern.split('|').any(|part| text.contains(part))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_labeler_creation() {
        let labeler = ContentLabeler::new();
        assert!(labeler.prof_jiang_keywords.len() > 0);
        assert!(labeler.geopolitical_patterns.len() > 0);
        assert!(labeler.indonesian_market_indicators.len() > 0);
    }

    #[test]
    fn test_prof_jiang_analysis() {
        let labeler = ContentLabeler::new();
        let content = "China's Belt and Road initiative creates geopolitical competition with USA in Indonesia";
        let labels = labeler.label_content("Geopolitical Analysis", content);
        
        assert!(labels.prof_jiang_relevance.score > 0.5);
        assert!(labels.prof_jiang_relevance.matched_concepts.len() > 0);
        assert!(labels.is_indonesian_market);
        assert!(labels.geopolitical_sensitivity_score > 0.0);
    }

    #[test]
    fn test_portfolio_stock_detection() {
        let labeler = ContentLabeler::new();
        let content = "BMRI and BBRI reported strong earnings, while INCO faces commodity price pressure";
        let labels = labeler.label_content("Banking Earnings", content);
        
        assert_eq!(labels.portfolio_stocks_mentioned.len(), 3);
        assert!(labels.portfolio_stocks_mentioned.contains(&IndonesianStock::BMRI));
        assert!(labels.portfolio_stocks_mentioned.contains(&IndonesianStock::BBRI));
        assert!(labels.portfolio_stocks_mentioned.contains(&IndonesianStock::INCO));
        assert_eq!(labels.market_impact_level, MarketImpactLevel::High);
    }

    #[test]
    fn test_indonesian_market_detection() {
        let labeler = ContentLabeler::new();
        let content = "Bank Indonesia raises interest rates to support rupiah amid global inflation";
        let labels = labeler.label_content("Monetary Policy", content);
        
        assert!(labels.is_indonesian_market);
        assert_eq!(labels.primary_category, ContentCategory::PolicyUpdate);
        assert_eq!(labels.market_impact_level, MarketImpactLevel::High);
    }

    #[test]
    fn test_sentiment_analysis() {
        let labeler = ContentLabeler::new();
        
        let positive_content = "Indonesia economy shows strong growth with positive outlook";
        let positive_labels = labeler.label_content("Economic Growth", positive_content);
        assert!(positive_labels.sentiment_score > 0.0);
        
        let negative_content = "Indonesia market faces decline with negative sentiment";
        let negative_labels = labeler.label_content("Market Decline", negative_content);
        assert!(negative_labels.sentiment_score < 0.0);
    }

    #[test]
    fn test_content_categorization() {
        let labeler = ContentLabeler::new();
        
        let earnings_content = "ANTM reports quarterly earnings with profit increase";
        let earnings_labels = labeler.label_content("Q4 Earnings", earnings_content);
        assert_eq!(earnings_labels.primary_category, ContentCategory::FinancialReport);
        
        let policy_content = "Government announces new regulation on mining sector";
        let policy_labels = labeler.label_content("Mining Policy", policy_content);
        assert_eq!(policy_labels.primary_category, ContentCategory::PolicyUpdate);
    }

    #[test]
    fn test_geopolitical_sensitivity() {
        let labeler = ContentLabeler::new();
        let content = "Trade war escalates with new sanctions affecting ASEAN economic corridor";
        let labels = labeler.label_content("Trade Conflict", content);
        
        assert!(labels.geopolitical_sensitivity_score > 0.5);
        assert!(labels.prof_jiang_relevance.score > 0.3);
    }

    #[test]
    fn test_confidence_scoring() {
        let labeler = ContentLabeler::new();
        
        let short_content = "BMRI up";
        let short_labels = labeler.label_content("Brief", short_content);
        assert!(short_labels.confidence_score < 0.7);
        
        let long_content = "A".repeat(1500); // Long content
        let long_labels = labeler.label_content("Detailed Analysis", &long_content);
        assert!(long_labels.confidence_score > 0.8);
    }
}