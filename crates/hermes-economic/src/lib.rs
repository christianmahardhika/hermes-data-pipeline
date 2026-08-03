/// hermes-economic: Economic Intelligence Service
/// 
/// Comprehensive economic data collection and analysis for Indonesian markets.
/// Tracks commodities (Nickel, Coal, CPO), BI Rate, and stock correlations.

pub mod commodity;
pub mod correlation;
pub mod indicators;
pub mod bi_rate;
pub mod bps_integration;

pub use commodity::{CommodityTracker, CommodityPrice, CommodityType};
pub use correlation::{StockCorrelationEngine, CorrelationMatrix, PortfolioCorrelation};
pub use indicators::{EconomicIndicators, MarketSentiment, TrendAnalysis};
pub use bi_rate::{BIRateMonitor, InterestRateData, MonetaryPolicy};
pub use bps_integration::{BPSIntegrationService, BPSDataPoint, BPSConfig, PortfolioImpactAnalysis, RiskLevel};

use anyhow::Result;
use hermes_common::types::IndonesianStock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// Economic intelligence system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicConfig {
    pub commodity_sources: Vec<String>,
    pub bi_rate_endpoint: String,
    pub correlation_window_days: u32,
    pub update_frequency_minutes: u32,
    pub indonesian_stocks: Vec<IndonesianStock>,
}

impl Default for EconomicConfig {
    fn default() -> Self {
        Self {
            commodity_sources: vec![
                "https://api.metals-api.com".to_string(),
                "https://api.marketdata.app".to_string(),
                "https://api.alphavantage.co".to_string(),
            ],
            bi_rate_endpoint: "https://api.bi.go.id/v2".to_string(),
            correlation_window_days: 30,
            update_frequency_minutes: 30,
            indonesian_stocks: vec![
                IndonesianStock::INCO,  // Nickel correlation
                IndonesianStock::PTBA,  // Coal correlation
                IndonesianStock::TAPG,  // Agriculture/CPO correlation
                IndonesianStock::BMRI,  // Banking/BI Rate correlation
                IndonesianStock::BBRI,  // Banking/BI Rate correlation
                IndonesianStock::ANTM,  // Mining correlation
            ],
        }
    }
}

/// Comprehensive economic data snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicSnapshot {
    pub timestamp: DateTime<Utc>,
    pub commodities: HashMap<CommodityType, CommodityPrice>,
    pub bi_rate: Option<Decimal>,
    pub stock_correlations: CorrelationMatrix,
    pub market_sentiment: MarketSentiment,
    pub trend_analysis: TrendAnalysis,
}

/// Main economic intelligence orchestrator
pub struct EconomicIntelligence {
    config: EconomicConfig,
    commodity_tracker: CommodityTracker,
    correlation_engine: StockCorrelationEngine,
    bi_rate_monitor: BIRateMonitor,
    indicators: EconomicIndicators,
}

impl EconomicIntelligence {
    /// Create new economic intelligence system
    pub fn new(config: EconomicConfig) -> Self {
        Self {
            commodity_tracker: CommodityTracker::new(&config.commodity_sources),
            correlation_engine: StockCorrelationEngine::new(config.correlation_window_days),
            bi_rate_monitor: BIRateMonitor::new(&config.bi_rate_endpoint),
            indicators: EconomicIndicators::new(),
            config,
        }
    }
    
    /// Create with default Indonesian market configuration
    pub fn new_indonesian_markets() -> Self {
        Self::new(EconomicConfig::default())
    }
    
    /// Collect comprehensive economic snapshot
    pub async fn collect_snapshot(&mut self) -> Result<EconomicSnapshot> {
        info!("📊 Collecting comprehensive economic intelligence snapshot");
        
        // Parallel data collection
        let (commodities, bi_rate, correlations) = tokio::try_join!(
            self.commodity_tracker.get_all_prices(),
            self.bi_rate_monitor.get_current_rate(),
            self.correlation_engine.calculate_correlations(&self.config.indonesian_stocks)
        )?;
        
        // Analyze market sentiment and trends
        let market_sentiment = self.indicators.analyze_sentiment(&commodities, &correlations).await?;
        let trend_analysis = self.indicators.analyze_trends(&commodities, bi_rate).await?;
        
        let snapshot = EconomicSnapshot {
            timestamp: Utc::now(),
            commodities,
            bi_rate,
            stock_correlations: correlations,
            market_sentiment,
            trend_analysis,
        };
        
        info!("✅ Economic snapshot complete: {} commodities, BI Rate: {:?}", 
              snapshot.commodities.len(), snapshot.bi_rate);
        
        Ok(snapshot)
    }
    
    /// Get Indonesian mining sector correlation analysis
    pub async fn get_mining_correlations(&mut self) -> Result<PortfolioCorrelation> {
        let mining_stocks = vec![
            IndonesianStock::INCO,
            IndonesianStock::ANTM,
            IndonesianStock::PTBA,
        ];
        
        self.correlation_engine.analyze_portfolio(&mining_stocks).await
    }
    
    /// Get banking sector BI Rate sensitivity
    pub async fn get_banking_rate_sensitivity(&mut self) -> Result<HashMap<IndonesianStock, f64>> {
        let banking_stocks = vec![
            IndonesianStock::BMRI,
            IndonesianStock::BBRI,
        ];
        
        self.correlation_engine.calculate_rate_sensitivity(&banking_stocks).await
    }
    
    /// Monitor commodity price alerts
    pub async fn check_commodity_alerts(&self) -> Result<Vec<String>> {
        let mut alerts = Vec::new();
        
        let prices = self.commodity_tracker.get_all_prices().await?;
        
        // Check for significant price movements
        for (commodity, price) in prices {
            if let Some(change_pct) = price.daily_change_percent {
                if change_pct.abs() > rust_decimal::Decimal::new(5, 0) { // 5% threshold
                    alerts.push(format!(
                        "🚨 {} price alert: ${} ({:+.2}%)", 
                        commodity, price.current_price, change_pct
                    ));
                }
            }
        }
        
        Ok(alerts)
    }
}

/// Initialize economic intelligence system
pub async fn init_economic_system() -> Result<EconomicIntelligence> {
    info!("🏭 Initializing Hermes Economic Intelligence System");
    info!("📈 Tracking: LME Nickel, Coal, CPO, Gold, Oil");
    info!("🏦 Monitoring: BI Rate, Indonesian stock correlations");
    info!("📊 Supporting: INCO, PTBA, TAPG, BMRI, BBRI, ANTM");
    
    Ok(EconomicIntelligence::new_indonesian_markets())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_economic_intelligence_creation() {
        let economic = EconomicIntelligence::new_indonesian_markets();
        assert_eq!(economic.config.indonesian_stocks.len(), 6);
        assert_eq!(economic.config.correlation_window_days, 30);
    }
    
    #[test]
    fn test_economic_config_default() {
        let config = EconomicConfig::default();
        assert!(!config.commodity_sources.is_empty());
        assert!(config.bi_rate_endpoint.contains("bi.go.id"));
        assert!(config.indonesian_stocks.contains(&IndonesianStock::INCO));
        assert!(config.indonesian_stocks.contains(&IndonesianStock::BMRI));
    }
    
    #[tokio::test]
    async fn test_init_economic_system() {
        let result = init_economic_system().await;
        assert!(result.is_ok());
    }
}