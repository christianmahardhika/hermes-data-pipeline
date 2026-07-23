use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

// Core data structures matching Python FastAPI models

// Health and System Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
}

// Stock Data Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockSymbolsRequest {
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockData {
    pub symbol: String,
    pub price: f64,
    pub volume: i64,
    pub timestamp: String,
    pub change_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockDataResponse {
    pub status: String,
    pub data: Vec<StockData>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDataResponse {
    pub symbol: String,
    pub history: Vec<HistoricalData>,
    pub period: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalData {
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}

// News Analysis Models (Prof Jiang Framework)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsAnalysisRequest {
    pub text: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsAnalysis {
    pub elite_overproduction_score: f64,
    pub indicators: Vec<String>,
    pub detailed_scores: HashMap<String, f64>,
    pub confidence: f64,
    pub language_detected: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsBatchRequest {
    pub articles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsBatchResponse {
    pub results: Vec<NewsAnalysis>,
    pub summary: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsTrendsResponse {
    pub trends: HashMap<String, Vec<f64>>,
    pub period: String,
    pub indicators_count: HashMap<String, i32>,
}

// Correlation Analysis Models (Aladdin Engine)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationRequest {
    pub price_data: HashMap<String, Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationResponse {
    pub correlation_matrix: Vec<Vec<f64>>,
    pub asset_names: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeRiskRequest {
    pub price_data: HashMap<String, Vec<f64>>,
    pub shock_asset: String,
    pub shock_magnitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeRiskResponse {
    pub cascade_probability: f64,
    pub affected_assets: Vec<HashMap<String, serde_json::Value>>,
    pub total_system_impact: f64,
    pub propagation_path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemicRiskRequest {
    pub correlations: Vec<Vec<f64>>,
    pub volatilities: Vec<f64>,
    pub asset_names: Vec<String>,
    pub market_stress: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemicRiskResponse {
    pub overall_risk: f64,
    pub risk_components: HashMap<String, f64>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    LOW,
    MEDIUM,
    HIGH,
    CRITICAL,
}

// Portfolio Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetWeight {
    pub symbol: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRequest {
    pub assets: Vec<AssetWeight>,
    pub horizon_days: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub volatility: f64,
    pub var_95: f64,
    pub max_drawdown: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioPredictionResponse {
    pub predicted_return: f64,
    pub risk_metrics: RiskMetrics,
    pub confidence_interval: HashMap<String, f64>,
    pub scenario_analysis: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConstraints {
    pub max_weight: f64,
    pub min_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioOptimizationRequest {
    pub universe: Vec<String>,
    pub objective: OptimizationObjective,
    pub constraints: OptimizationConstraints,
    pub risk_tolerance: RiskTolerance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationObjective {
    #[serde(rename = "maximize_sharpe")]
    MaximizeSharpe,
    #[serde(rename = "minimize_risk")]
    MinimizeRisk,
    #[serde(rename = "maximize_return")]
    MaximizeReturn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskTolerance {
    #[serde(rename = "conservative")]
    Conservative,
    #[serde(rename = "moderate")]
    Moderate,
    #[serde(rename = "aggressive")]
    Aggressive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioOptimizationResponse {
    pub optimal_weights: HashMap<String, f64>,
    pub expected_return: f64,
    pub expected_risk: f64,
    pub sharpe_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAttributionResponse {
    pub asset_contributions: HashMap<String, f64>,
    pub factor_exposures: HashMap<String, f64>,
    pub diversification_ratio: f64,
}

// Intelligence Report Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceReportRequest {
    pub stock_symbols: Vec<String>,
    pub news_keywords: Vec<String>,
    pub analysis_period: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceReportResponse {
    pub stock_analysis: HashMap<String, serde_json::Value>,
    pub news_analysis: HashMap<String, serde_json::Value>,
    pub correlation_analysis: HashMap<String, serde_json::Value>,
    pub portfolio_recommendations: HashMap<String, serde_json::Value>,
    pub risk_assessment: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

// Dashboard Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStatus {
    pub is_open: bool,
    pub session: String,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub market_status: MarketStatus,
    pub top_stocks: Vec<HashMap<String, serde_json::Value>>,
    pub news_sentiment: HashMap<String, f64>,
    pub risk_alerts: Vec<HashMap<String, serde_json::Value>>,
    pub last_updated: DateTime<Utc>,
}

// Alert Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "critical")]
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    #[serde(rename = "type")]
    pub alert_type: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsResponse {
    pub alerts: Vec<Alert>,
    pub count: i32,
}

// Error Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub detail: Option<String>,
    pub timestamp: DateTime<Utc>,
}

// Commodity Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommodityData {
    pub commodity: String,
    pub price: f64,
    pub change: f64,
    pub currency: String,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
}

// Geopolitical Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeopoliticalSignal {
    pub signal: String,
    pub impact: String,
    pub region: String,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
    pub elite_overproduction_score: f64,
    pub game_theory_analysis: HashMap<String, f64>,
}

// Performance Models for High-Speed Operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub response_time_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub cache_hit_ratio: f64,
}

// WebSocket Real-time Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeUpdate {
    pub update_type: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

// Database Models for ArangoDB/Redis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub arangodb_url: String,
    pub arangodb_database: String,
    pub arangodb_username: String,
    pub redis_url: String,
}

// Portfolio Data Models for Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioData {
    #[serde(rename = "portfolioValue")]
    pub portfolio_value: f64,
    #[serde(rename = "totalGain")]
    pub total_gain: f64,
    #[serde(rename = "gainPercentage")]
    pub gain_percentage: f64,
    pub holdings: Vec<Holding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holding {
    pub symbol: String,
    pub shares: f64,
    pub value: f64,
    pub change: f64,
}

// Correlation Data Models for Dashboard  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationData {
    pub pair: String,
    pub correlation: f64,
    pub strength: String,
    pub timeframe: String,
}

// News Correlation Models for Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsCorrelation {
    pub title: String,
    pub sentiment: f64,
    pub impact: f64,
    pub language: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
}

// Prediction Data Models for Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionData {
    pub symbol: String,
    pub prediction: f64,
    pub confidence: f64,
    pub timeframe: String,
    pub model: String,
}

// Alert Data Models for Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertData {
    pub message: String,
    pub severity: String,
    pub timestamp: DateTime<Utc>,
    pub category: String,
}

// Complete Dashboard Response Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardResponse {
    #[serde(rename = "portfolioData")]
    pub portfolio_data: PortfolioData,
    #[serde(rename = "stockData")]
    pub stock_data: Vec<StockData>,
    #[serde(rename = "correlationData")]
    pub correlation_data: Vec<CorrelationData>,
    #[serde(rename = "geopoliticalSignals")]
    pub geopolitical_signals: Vec<GeopoliticalSignal>,
    #[serde(rename = "commodityData")]
    pub commodity_data: Vec<CommodityData>,
    #[serde(rename = "newsCorrelations")]
    pub news_correlations: Vec<NewsCorrelation>,
    pub predictions: Vec<PredictionData>,
    pub alerts: Vec<AlertData>,
    pub performance: PerformanceMetrics,
}