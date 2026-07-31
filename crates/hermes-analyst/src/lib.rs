/// hermes-analyst: Advanced Analyst Intelligence Service
/// 
/// Prof Jiang Xueqin's Predictive History Framework integration for Indonesian markets.
/// Combines geopolitical analysis, investment signal generation, and pattern matching.

pub mod prof_jiang;
pub mod geopolitical;
pub mod signals;
pub mod patterns;
pub mod decision;

pub use prof_jiang::{ProfJiangAnalyzer, PredictiveHistoryFramework, GeopoliticalEvent};
pub use geopolitical::{GeopoliticalEngine, GlobalTension, RegionalStability};
pub use signals::{InvestmentSignals, SignalStrength, ActionRecommendation};
pub use patterns::{PatternMatcher, MarketPattern, HistoricalAnalog};
pub use decision::{DecisionFramework, AnalystRecommendation, ConfidenceLevel};

use anyhow::Result;
use hermes_common::types::IndonesianStock;
use hermes_economic::{EconomicSnapshot, CommodityType};
use hermes_social::{SocialPost, SocialSource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// Advanced analyst intelligence configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalystConfig {
    pub prof_jiang_weights: HashMap<String, f64>,
    pub geopolitical_sources: Vec<String>,
    pub signal_thresholds: SignalThresholds,
    pub pattern_lookback_days: u32,
    pub confidence_requirements: ConfidenceRequirements,
    pub indonesian_focus_weight: f64,
}

impl Default for AnalystConfig {
    fn default() -> Self {
        let mut prof_jiang_weights = HashMap::new();
        prof_jiang_weights.insert("geostrategy".to_string(), 0.35);
        prof_jiang_weights.insert("game_theory".to_string(), 0.30);
        prof_jiang_weights.insert("secret_history".to_string(), 0.25);
        prof_jiang_weights.insert("economic_cycles".to_string(), 0.10);
        
        Self {
            prof_jiang_weights,
            geopolitical_sources: vec![
                "Reuters Geopolitics".to_string(),
                "Foreign Affairs".to_string(),
                "Asia Times".to_string(),
                "Jakarta Post International".to_string(),
            ],
            signal_thresholds: SignalThresholds::default(),
            pattern_lookback_days: 90,
            confidence_requirements: ConfidenceRequirements::default(),
            indonesian_focus_weight: 0.75,
        }
    }
}

/// Signal strength thresholds for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalThresholds {
    pub strong_buy: f64,
    pub buy: f64,
    pub neutral_upper: f64,
    pub neutral_lower: f64,
    pub sell: f64,
    pub strong_sell: f64,
}

impl Default for SignalThresholds {
    fn default() -> Self {
        Self {
            strong_buy: 0.8,
            buy: 0.6,
            neutral_upper: 0.2,
            neutral_lower: -0.2,
            sell: -0.6,
            strong_sell: -0.8,
        }
    }
}

/// Confidence level requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceRequirements {
    pub minimum_data_points: u32,
    pub minimum_pattern_matches: u32,
    pub minimum_geopolitical_correlation: f64,
    pub minimum_economic_correlation: f64,
}

impl Default for ConfidenceRequirements {
    fn default() -> Self {
        Self {
            minimum_data_points: 10,
            minimum_pattern_matches: 3,
            minimum_geopolitical_correlation: 0.3,
            minimum_economic_correlation: 0.4,
        }
    }
}

/// Comprehensive analyst intelligence results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalystIntelligence {
    pub timestamp: DateTime<Utc>,
    pub prof_jiang_analysis: ProfJiangAnalysis,
    pub geopolitical_assessment: GeopoliticalAssessment,
    pub investment_signals: HashMap<IndonesianStock, InvestmentSignal>,
    pub pattern_matches: Vec<PatternMatch>,
    pub decision_recommendations: Vec<DecisionRecommendation>,
    pub confidence_score: f64,
    pub key_insights: Vec<String>,
}

/// Prof Jiang framework analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfJiangAnalysis {
    pub geostrategy_score: f64,
    pub game_theory_implications: Vec<String>,
    pub secret_history_patterns: Vec<String>,
    pub predictive_confidence: f64,
    pub timeline_projections: Vec<TimelineProjection>,
    pub indonesian_context: IndonesianGeopoliticalContext,
}

/// Timeline projection for events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineProjection {
    pub event_type: String,
    pub probability: f64,
    pub timeframe: String,
    pub impact_level: ImpactLevel,
    pub affected_sectors: Vec<String>,
}

/// Impact severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for ImpactLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImpactLevel::Low => write!(f, "🟢 Low Impact"),
            ImpactLevel::Medium => write!(f, "🟡 Medium Impact"),
            ImpactLevel::High => write!(f, "🟠 High Impact"),
            ImpactLevel::Critical => write!(f, "🔴 Critical Impact"),
        }
    }
}

/// Indonesian geopolitical context analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndonesianGeopoliticalContext {
    pub asean_dynamics: String,
    pub china_relations: RelationshipStatus,
    pub us_relations: RelationshipStatus,
    pub domestic_stability: StabilityLevel,
    pub economic_sovereignty: f64, // 0.0 to 1.0
    pub resource_diplomacy_strength: f64,
}

/// Relationship status indicators
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RelationshipStatus {
    Cooperative,
    Neutral,
    Tense,
    Conflicted,
}

/// Political stability levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum StabilityLevel {
    Stable,
    MildConcerns,
    Volatile,
    Unstable,
}

/// Geopolitical assessment results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeopoliticalAssessment {
    pub global_tension_level: f64, // 0.0 to 1.0
    pub regional_stability: HashMap<String, f64>,
    pub trade_route_security: f64,
    pub commodity_geopolitics: HashMap<CommodityType, f64>,
    pub currency_pressure: f64,
    pub sanctions_risk: f64,
}

/// Investment signal for specific stock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentSignal {
    pub stock: IndonesianStock,
    pub signal_strength: f64, // -1.0 to +1.0
    pub action: ActionRecommendation,
    pub rationale: Vec<String>,
    pub risk_factors: Vec<String>,
    pub time_horizon: TimeHorizon,
    pub confidence: f64,
}

/// Investment time horizons
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TimeHorizon {
    ShortTerm,  // 1-3 months
    MediumTerm, // 3-12 months  
    LongTerm,   // 1+ years
}

/// Pattern matching results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub pattern_name: String,
    pub similarity_score: f64,
    pub historical_period: String,
    pub current_indicators: Vec<String>,
    pub expected_outcomes: Vec<String>,
    pub success_probability: f64,
}

/// Decision recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecommendation {
    pub category: DecisionCategory,
    pub recommendation: String,
    pub priority: Priority,
    pub implementation_timeline: String,
    pub success_metrics: Vec<String>,
    pub risk_mitigation: Vec<String>,
}

/// Decision categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DecisionCategory {
    Portfolio,
    Economic,
    Geopolitical,
    Risk,
}

/// Priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Low => write!(f, "🔵 Low Priority"),
            Priority::Medium => write!(f, "🟡 Medium Priority"), 
            Priority::High => write!(f, "🟠 High Priority"),
            Priority::Critical => write!(f, "🔴 Critical Priority"),
        }
    }
}

/// Main analyst intelligence orchestrator
pub struct AnalystIntelligenceEngine {
    config: AnalystConfig,
    prof_jiang_analyzer: ProfJiangAnalyzer,
    geopolitical_engine: GeopoliticalEngine,
    signal_generator: InvestmentSignals,
    pattern_matcher: PatternMatcher,
    decision_framework: DecisionFramework,
}

impl AnalystIntelligenceEngine {
    /// Create new analyst intelligence engine
    pub fn new(config: AnalystConfig) -> Self {
        Self {
            prof_jiang_analyzer: ProfJiangAnalyzer::new(&config.prof_jiang_weights),
            geopolitical_engine: GeopoliticalEngine::new(&config.geopolitical_sources),
            signal_generator: InvestmentSignals::new(&config.signal_thresholds),
            pattern_matcher: PatternMatcher::new(config.pattern_lookback_days),
            decision_framework: DecisionFramework::new(&config.confidence_requirements),
            config,
        }
    }
    
    /// Create with Indonesian market focus
    pub fn new_indonesian_markets() -> Self {
        Self::new(AnalystConfig::default())
    }
    
    /// Generate comprehensive analyst intelligence
    pub async fn analyze_comprehensive(&mut self, 
                                     economic_data: &EconomicSnapshot,
                                     social_posts: &[SocialPost]) -> Result<AnalystIntelligence> {
        info!("🧠 Generating comprehensive analyst intelligence using Prof Jiang framework");
        
        let timestamp = Utc::now();
        
        // Prof Jiang framework analysis
        let prof_jiang_analysis = self.prof_jiang_analyzer
            .analyze_predictive_history(economic_data, social_posts).await?;
        
        // Geopolitical assessment
        let geopolitical_assessment = self.geopolitical_engine
            .assess_global_situation(social_posts).await?;
        
        // Investment signal generation
        let mut investment_signals = HashMap::new();
        for &stock in &self.config.signal_thresholds.get_indonesian_stocks() {
            let signal = self.signal_generator
                .generate_signal(stock, economic_data, &prof_jiang_analysis).await?;
            investment_signals.insert(stock, signal);
        }
        
        // Pattern matching
        let pattern_matches = self.pattern_matcher
            .find_historical_patterns(economic_data, &prof_jiang_analysis).await?;
        
        // Decision recommendations
        let decision_recommendations = self.decision_framework
            .generate_recommendations(&prof_jiang_analysis, &investment_signals).await?;
        
        // Calculate overall confidence
        let confidence_score = self.calculate_confidence_score(
            &prof_jiang_analysis, 
            &investment_signals,
            &pattern_matches
        );
        
        // Generate key insights
        let key_insights = self.generate_key_insights(
            &prof_jiang_analysis,
            &geopolitical_assessment,
            &investment_signals
        );
        
        let intelligence = AnalystIntelligence {
            timestamp,
            prof_jiang_analysis,
            geopolitical_assessment,
            investment_signals,
            pattern_matches,
            decision_recommendations,
            confidence_score,
            key_insights,
        };
        
        info!("✅ Analyst intelligence complete: {} signals, {} patterns, confidence: {:.1}%", 
              intelligence.investment_signals.len(), 
              intelligence.pattern_matches.len(),
              intelligence.confidence_score * 100.0);
        
        Ok(intelligence)
    }
    
    /// Generate focused analysis for specific stock
    pub async fn analyze_stock_focused(&mut self, 
                                     stock: IndonesianStock,
                                     economic_data: &EconomicSnapshot) -> Result<StockAnalysis> {
        info!("🎯 Generating focused analysis for {:?} using Prof Jiang framework", stock);
        
        // Prof Jiang analysis specific to stock
        let prof_jiang_score = self.prof_jiang_analyzer
            .calculate_stock_relevance(stock, economic_data).await?;
        
        // Generate investment signal
        let prof_jiang_analysis = ProfJiangAnalysis::default(); // Simplified for focused analysis
        let signal = self.signal_generator
            .generate_signal(stock, economic_data, &prof_jiang_analysis).await?;
        
        // Find relevant patterns
        let patterns = self.pattern_matcher
            .find_stock_patterns(stock, economic_data).await?;
        
        Ok(StockAnalysis {
            stock,
            prof_jiang_relevance: prof_jiang_score,
            investment_signal: signal,
            relevant_patterns: patterns,
            timestamp: Utc::now(),
        })
    }
    
    /// Monitor geopolitical developments
    pub async fn monitor_geopolitical(&mut self, social_posts: &[SocialPost]) -> Result<Vec<GeopoliticalAlert>> {
        info!("🌍 Monitoring geopolitical developments affecting Indonesian markets");
        
        self.geopolitical_engine.monitor_developments(social_posts).await
    }
    
    /// Calculate overall confidence score
    fn calculate_confidence_score(&self, 
                                prof_jiang: &ProfJiangAnalysis,
                                signals: &HashMap<IndonesianStock, InvestmentSignal>,
                                patterns: &[PatternMatch]) -> f64 {
        let mut factors = Vec::new();
        
        // Prof Jiang framework confidence
        factors.push(prof_jiang.predictive_confidence);
        
        // Signal consistency
        let signal_confidences: Vec<f64> = signals.values().map(|s| s.confidence).collect();
        if !signal_confidences.is_empty() {
            factors.push(signal_confidences.iter().sum::<f64>() / signal_confidences.len() as f64);
        }
        
        // Pattern match quality
        if !patterns.is_empty() {
            let pattern_scores: Vec<f64> = patterns.iter().map(|p| p.similarity_score).collect();
            factors.push(pattern_scores.iter().sum::<f64>() / pattern_scores.len() as f64);
        }
        
        // Data quality factor
        factors.push(0.85); // Assume good data quality for Indonesian markets
        
        // Weighted average
        factors.iter().sum::<f64>() / factors.len() as f64
    }
    
    /// Generate key insights from analysis
    fn generate_key_insights(&self,
                           prof_jiang: &ProfJiangAnalysis,
                           geopolitical: &GeopoliticalAssessment,
                           signals: &HashMap<IndonesianStock, InvestmentSignal>) -> Vec<String> {
        let mut insights = Vec::new();
        
        // Prof Jiang insights
        if prof_jiang.geostrategy_score > 0.7 {
            insights.push("High geopolitical relevance detected - monitor closely".to_string());
        }
        
        // Signal insights
        let bullish_signals = signals.values().filter(|s| s.signal_strength > 0.5).count();
        if bullish_signals > signals.len() / 2 {
            insights.push(format!("Positive sentiment across {} Indonesian stocks", bullish_signals));
        }
        
        // Geopolitical insights
        if geopolitical.global_tension_level > 0.6 {
            insights.push("Elevated global tensions - consider defensive positions".to_string());
        }
        
        // Indonesian specific insights
        if let Some(inco_signal) = signals.get(&IndonesianStock::INCO) {
            if inco_signal.signal_strength > 0.6 {
                insights.push("INCO showing strong momentum - nickel market dynamics favorable".to_string());
            }
        }
        
        insights
    }
}

/// Focused stock analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockAnalysis {
    pub stock: IndonesianStock,
    pub prof_jiang_relevance: f64,
    pub investment_signal: InvestmentSignal,
    pub relevant_patterns: Vec<PatternMatch>,
    pub timestamp: DateTime<Utc>,
}

/// Geopolitical alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeopoliticalAlert {
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub description: String,
    pub affected_regions: Vec<String>,
    pub market_implications: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Alert types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AlertType {
    TradeDispute,
    MilitaryTension,
    EconomicSanctions,
    PoliticalInstability,
    CommodityDisruption,
    CurrencyVolatility,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Initialize analyst intelligence system
pub async fn init_analyst_system() -> Result<AnalystIntelligenceEngine> {
    info!("🧠 Initializing Hermes Advanced Analyst Intelligence System");
    info!("📚 Prof Jiang Predictive History Framework: Geostrategy + Game Theory + Secret History");
    info!("🌍 Geopolitical Analysis: Global tensions, ASEAN dynamics, Trade routes");
    info!("📊 Investment Signals: Indonesian stocks with correlation analysis");
    info!("🔍 Pattern Matching: Historical analogs and predictive patterns");
    info!("⚖️  Decision Framework: Evidence-based recommendations with confidence scoring");
    
    Ok(AnalystIntelligenceEngine::new_indonesian_markets())
}

// Helper trait implementations
impl SignalThresholds {
    fn get_indonesian_stocks(&self) -> Vec<IndonesianStock> {
        vec![
            IndonesianStock::BMRI,
            IndonesianStock::BBRI,
            IndonesianStock::INCO,
            IndonesianStock::ANTM,
            IndonesianStock::PTBA,
            IndonesianStock::TAPG,
        ]
    }
}

impl Default for ProfJiangAnalysis {
    fn default() -> Self {
        Self {
            geostrategy_score: 0.5,
            game_theory_implications: Vec::new(),
            secret_history_patterns: Vec::new(),
            predictive_confidence: 0.6,
            timeline_projections: Vec::new(),
            indonesian_context: IndonesianGeopoliticalContext::default(),
        }
    }
}

impl Default for IndonesianGeopoliticalContext {
    fn default() -> Self {
        Self {
            asean_dynamics: "Stable cooperation with growing economic integration".to_string(),
            china_relations: RelationshipStatus::Cooperative,
            us_relations: RelationshipStatus::Cooperative,
            domestic_stability: StabilityLevel::Stable,
            economic_sovereignty: 0.75,
            resource_diplomacy_strength: 0.8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_analyst_intelligence_engine_creation() {
        let engine = AnalystIntelligenceEngine::new_indonesian_markets();
        assert_eq!(engine.config.indonesian_focus_weight, 0.75);
        assert!(engine.config.prof_jiang_weights.contains_key("geostrategy"));
    }
    
    #[test]
    fn test_analyst_config_default() {
        let config = AnalystConfig::default();
        assert!(!config.prof_jiang_weights.is_empty());
        assert!(!config.geopolitical_sources.is_empty());
        assert_eq!(config.pattern_lookback_days, 90);
    }
    
    #[test]
    fn test_signal_thresholds() {
        let thresholds = SignalThresholds::default();
        assert!(thresholds.strong_buy > thresholds.buy);
        assert!(thresholds.buy > thresholds.neutral_upper);
        assert!(thresholds.neutral_lower < thresholds.sell);
        assert!(thresholds.sell > thresholds.strong_sell);
    }
    
    #[test]
    fn test_impact_level_display() {
        assert!(ImpactLevel::Critical.to_string().contains("Critical"));
        assert!(ImpactLevel::Low.to_string().contains("Low"));
    }
    
    #[test]
    fn test_priority_display() {
        assert!(Priority::Critical.to_string().contains("Critical"));
        assert!(Priority::Medium.to_string().contains("Medium"));
    }
    
    #[tokio::test]
    async fn test_init_analyst_system() {
        let result = init_analyst_system().await;
        assert!(result.is_ok());
    }
}