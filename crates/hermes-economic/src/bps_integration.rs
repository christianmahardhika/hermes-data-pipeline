use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use governor::{Quota, RateLimiter};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// BPS Government API configuration
#[derive(Debug, Clone)]
pub struct BPSConfig {
    pub base_url: String,
    pub app_id: String,
    pub rate_limit: f64, // requests per second
    pub timeout_secs: u64,
}

impl Default for BPSConfig {
    fn default() -> Self {
        Self {
            base_url: "https://webapi.bps.go.id".to_string(),
            app_id: "71eeba17f4f1e4b6ef253886c04ec49e".to_string(), // Christian's registered App ID
            rate_limit: 1.9, // Conservative rate limit compliance
            timeout_secs: 30,
        }
    }
}

/// BPS Economic Data Point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BPSDataPoint {
    pub id: Uuid,
    pub variable_id: u32,
    pub variable_name: String,
    pub value: f64,
    pub unit: String,
    pub period: String,
    pub region: String,
    pub timestamp: DateTime<Utc>,
    pub quality_score: f32, // 0.0 to 1.0
}

/// Portfolio Impact Analysis Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioImpactAnalysis {
    pub correlation_scores: HashMap<String, f64>, // stock_symbol -> correlation
    pub risk_level: RiskLevel,
    pub confidence: f32,
    pub analysis_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// BPS API Response Structure
#[derive(Debug, Deserialize)]
struct BPSApiResponse {
    #[serde(rename = "data-availability")]
    data_availability: String,
    #[serde(rename = "data")]
    data: Vec<BPSRawData>,
}

#[derive(Debug, Deserialize)]
struct BPSRawData {
    #[serde(rename = "var_id")]
    var_id: u32,
    #[serde(rename = "var")]
    var_name: String,
    #[serde(rename = "val")]
    value: String,
    #[serde(rename = "unit")]
    unit: String,
    #[serde(rename = "period")]
    period: String,
}

/// Rate-limited BPS API client
pub struct BPSIntegrationService {
    client: Client,
    config: BPSConfig,
    rate_limiter: Arc<RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::QuantaClock>>,
    critical_variables: HashMap<u32, String>,
}

impl BPSIntegrationService {
    /// Create new BPS integration service with rate limiting
    pub fn new(config: BPSConfig) -> Self {
        let quota = Quota::per_second(NonZeroU32::new((config.rate_limit * 100.0) as u32).unwrap())
            .allow_burst(NonZeroU32::new(1).unwrap());
        let rate_limiter = Arc::new(RateLimiter::direct(quota));
        
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .user_agent("Hermes-Intelligence-Dashboard/1.0")
            .build()
            .expect("Failed to create HTTP client");

        // Critical Indonesian economic variables
        let mut critical_variables = HashMap::new();
        critical_variables.insert(1, "Inflation Rate (General)".to_string());
        critical_variables.insert(2, "Inflation Rate (Food)".to_string());
        critical_variables.insert(1709, "Inflation Rate (Core)".to_string());

        Self {
            client,
            config,
            rate_limiter,
            critical_variables,
        }
    }

    /// Collect inflation data with rate limiting compliance
    pub async fn collect_inflation_data(&self) -> Result<Vec<BPSDataPoint>> {
        let mut data_points = Vec::new();
        
        info!("🔄 Starting BPS inflation data collection (rate limit: {} req/s)", self.config.rate_limit);
        
        for (&var_id, var_name) in &self.critical_variables {
            match self.fetch_variable_data(var_id).await {
                Ok(mut points) => {
                    info!("✅ Collected {} data points for {}", points.len(), var_name);
                    data_points.append(&mut points);
                }
                Err(e) => {
                    error!("❌ Failed to collect data for variable {}: {}", var_id, e);
                    // Continue with other variables rather than failing completely
                }
            }
        }
        
        info!("📊 Total BPS data points collected: {}", data_points.len());
        Ok(data_points)
    }

    /// Fetch data for a specific variable with rate limiting
    async fn fetch_variable_data(&self, var_id: u32) -> Result<Vec<BPSDataPoint>> {
        // Enforce rate limiting
        self.rate_limiter.until_ready().await;
        
        let url = format!(
            "{}/api/list/model/data/lang/ind/domain/0000/var/{}?key={}",
            self.config.base_url, var_id, self.config.app_id
        );
        
        debug!("🌐 Fetching BPS data: {}", url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("HTTP request failed: {}", e))?;
            
        if !response.status().is_success() {
            return Err(anyhow!("BPS API error: {}", response.status()));
        }
        
        let api_response: BPSApiResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse BPS response: {}", e))?;
            
        let mut data_points = Vec::new();
        let var_name = self.critical_variables.get(&var_id)
            .unwrap_or(&format!("Variable {}", var_id));
            
        for raw_data in api_response.data {
            if let Ok(value) = raw_data.value.parse::<f64>() {
                let quality_score = self.calculate_data_quality(&raw_data);
                
                let data_point = BPSDataPoint {
                    id: Uuid::new_v4(),
                    variable_id: var_id,
                    variable_name: var_name.clone(),
                    value,
                    unit: raw_data.unit,
                    period: raw_data.period,
                    region: "Indonesia".to_string(),
                    timestamp: Utc::now(),
                    quality_score,
                };
                
                data_points.push(data_point);
            } else {
                warn!("⚠️  Invalid numeric value for variable {}: {}", var_id, raw_data.value);
            }
        }
        
        Ok(data_points)
    }

    /// Calculate data quality score based on completeness and recency
    fn calculate_data_quality(&self, raw_data: &BPSRawData) -> f32 {
        let mut score: f32 = 1.0;
        
        // Check for missing or invalid data
        if raw_data.value.is_empty() || raw_data.value == "null" {
            score -= 0.5;
        }
        
        if raw_data.unit.is_empty() {
            score -= 0.2;
        }
        
        if raw_data.period.is_empty() {
            score -= 0.2;
        }
        
        // Ensure score is between 0.0 and 1.0
        score.max(0.0).min(1.0)
    }

    /// Analyze portfolio impact based on economic indicators
    pub async fn analyze_portfolio_impact(&self, data_points: &[BPSDataPoint]) -> Result<PortfolioImpactAnalysis> {
        info!("🔬 Analyzing portfolio impact for {} data points", data_points.len());
        
        let mut correlation_scores = HashMap::new();
        
        // Indonesian stock portfolio correlation analysis
        // INCO (nickel) - sensitive to inflation and export dynamics
        correlation_scores.insert("INCO".to_string(), self.calculate_commodity_correlation(data_points, "nickel"));
        
        // ANTM (coal mining) - correlated with energy inflation
        correlation_scores.insert("ANTM".to_string(), self.calculate_commodity_correlation(data_points, "coal"));
        
        // PTBA (coal mining) - similar to ANTM
        correlation_scores.insert("PTBA".to_string(), self.calculate_commodity_correlation(data_points, "coal"));
        
        // TAPG (palm oil) - food inflation sensitive
        correlation_scores.insert("TAPG".to_string(), self.calculate_food_inflation_correlation(data_points));
        
        // BMRI & BBRI (banks) - interest rate and general inflation sensitive
        let banking_correlation = self.calculate_banking_correlation(data_points);
        correlation_scores.insert("BMRI".to_string(), banking_correlation);
        correlation_scores.insert("BBRI".to_string(), banking_correlation);
        
        let risk_level = self.assess_overall_risk(&correlation_scores);
        let confidence = self.calculate_analysis_confidence(data_points);
        
        Ok(PortfolioImpactAnalysis {
            correlation_scores,
            risk_level,
            confidence,
            analysis_timestamp: Utc::now(),
        })
    }

    fn calculate_commodity_correlation(&self, data_points: &[BPSDataPoint], commodity_type: &str) -> f64 {
        // Simplified correlation calculation based on general inflation
        let inflation_avg = data_points.iter()
            .filter(|dp| dp.variable_id == 1) // General inflation
            .map(|dp| dp.value)
            .fold(0.0, |acc, val| acc + val) / data_points.len() as f64;
        
        match commodity_type {
            "nickel" => (inflation_avg * 0.7).min(1.0), // High correlation with inflation
            "coal" => (inflation_avg * 0.6).min(1.0),   // Moderate-high correlation
            _ => (inflation_avg * 0.5).min(1.0),
        }
    }

    fn calculate_food_inflation_correlation(&self, data_points: &[BPSDataPoint]) -> f64 {
        // Palm oil is directly correlated with food inflation
        let food_inflation_avg = data_points.iter()
            .filter(|dp| dp.variable_id == 2) // Food inflation
            .map(|dp| dp.value)
            .fold(0.0, |acc, val| acc + val) / data_points.len() as f64;
            
        (food_inflation_avg * 0.8).min(1.0) // Very high correlation with food inflation
    }

    fn calculate_banking_correlation(&self, data_points: &[BPSDataPoint]) -> f64 {
        // Banks are sensitive to general inflation (affects BI rate decisions)
        let general_inflation = data_points.iter()
            .filter(|dp| dp.variable_id == 1)
            .map(|dp| dp.value)
            .fold(0.0, |acc, val| acc + val) / data_points.len() as f64;
            
        // Inverse correlation - higher inflation often leads to rate hikes, affecting banks
        (1.0 - (general_inflation * 0.4)).max(0.0)
    }

    fn assess_overall_risk(&self, correlations: &HashMap<String, f64>) -> RiskLevel {
        let avg_correlation = correlations.values().sum::<f64>() / correlations.len() as f64;
        
        if avg_correlation > 0.8 {
            RiskLevel::Critical
        } else if avg_correlation > 0.6 {
            RiskLevel::High
        } else if avg_correlation > 0.4 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    fn calculate_analysis_confidence(&self, data_points: &[BPSDataPoint]) -> f32 {
        if data_points.is_empty() {
            return 0.0;
        }
        
        let avg_quality = data_points.iter()
            .map(|dp| dp.quality_score)
            .sum::<f32>() / data_points.len() as f32;
            
        // Confidence based on data quality and completeness
        let completeness_factor = (data_points.len() as f32 / self.critical_variables.len() as f32).min(1.0);
        
        (avg_quality * completeness_factor * 100.0).min(100.0)
    }

    /// Health check for BPS API connectivity
    pub async fn health_check(&self) -> Result<bool> {
        self.rate_limiter.until_ready().await;
        
        let url = format!("{}/api/list", self.config.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_bps_service_creation() {
        let config = BPSConfig::default();
        let service = BPSIntegrationService::new(config);
        
        assert_eq!(service.critical_variables.len(), 3);
        assert!(service.critical_variables.contains_key(&1));
        assert!(service.critical_variables.contains_key(&2));
        assert!(service.critical_variables.contains_key(&1709));
    }

    #[tokio::test]
    async fn test_rate_limiting_compliance() {
        let config = BPSConfig::default();
        let service = BPSIntegrationService::new(config);
        
        let start = std::time::Instant::now();
        
        // Simulate 3 API calls
        for _ in 0..3 {
            service.rate_limiter.until_ready().await;
        }
        
        let elapsed = start.elapsed();
        
        // Should take at least ~1 second for 3 calls at 1.9 req/s
        assert!(elapsed.as_secs_f64() >= 1.0);
    }

    #[tokio::test]
    async fn test_data_quality_calculation() {
        let config = BPSConfig::default();
        let service = BPSIntegrationService::new(config);
        
        let good_data = BPSRawData {
            var_id: 1,
            var_name: "Test".to_string(),
            value: "3.5".to_string(),
            unit: "percent".to_string(),
            period: "2024".to_string(),
        };
        
        let quality = service.calculate_data_quality(&good_data);
        assert_eq!(quality, 1.0);
        
        let bad_data = BPSRawData {
            var_id: 1,
            var_name: "Test".to_string(),
            value: "".to_string(),
            unit: "".to_string(),
            period: "".to_string(),
        };
        
        let quality = service.calculate_data_quality(&bad_data);
        assert!(quality < 1.0);
    }
}