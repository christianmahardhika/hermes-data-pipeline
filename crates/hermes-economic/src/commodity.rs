/// Commodity price tracking and analysis for Indonesian market intelligence
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, debug};
use rust_decimal::Decimal;
use async_trait::async_trait;

/// Supported commodity types relevant to Indonesian economy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommodityType {
    /// LME Nickel (correlates with INCO, ANTM)
    Nickel,
    /// Thermal Coal (correlates with PTBA)
    Coal,
    /// Crude Palm Oil (correlates with TAPG agriculture)
    CPO,
    /// Gold (safe haven asset)
    Gold,
    /// Crude Oil (energy sector indicator)
    Oil,
    /// Copper (industrial metals)
    Copper,
    /// Tin (Indonesian export commodity)
    Tin,
}

impl std::fmt::Display for CommodityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommodityType::Nickel => write!(f, "LME Nickel"),
            CommodityType::Coal => write!(f, "Thermal Coal"),
            CommodityType::CPO => write!(f, "Crude Palm Oil"),
            CommodityType::Gold => write!(f, "Gold"),
            CommodityType::Oil => write!(f, "Crude Oil"),
            CommodityType::Copper => write!(f, "Copper"),
            CommodityType::Tin => write!(f, "Tin"),
        }
    }
}

/// Real-time commodity price data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommodityPrice {
    pub commodity: CommodityType,
    pub current_price: Decimal,
    pub currency: String,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub daily_change: Option<Decimal>,
    pub daily_change_percent: Option<Decimal>,
    pub volume: Option<i64>,
    pub market_status: MarketStatus,
    pub source: String,
}

/// Market trading status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MarketStatus {
    Open,
    Closed,
    PreMarket,
    AfterHours,
    Holiday,
    Unknown,
}

/// Commodity data source trait
#[async_trait]
pub trait CommodityDataSource {
    async fn get_price(&self, commodity: CommodityType) -> Result<CommodityPrice>;
    async fn get_historical_prices(&self, commodity: CommodityType, days: u32) -> Result<Vec<CommodityPrice>>;
    fn source_name(&self) -> &str;
    async fn health_check(&self) -> Result<bool>;
}

/// Mock commodity data source for testing and development
pub struct MockCommoditySource {
    name: String,
    prices: HashMap<CommodityType, CommodityPrice>,
}

impl MockCommoditySource {
    pub fn new(name: String) -> Self {
        let mut prices = HashMap::new();
        
        // Initialize with realistic Indonesian market-relevant prices
        prices.insert(CommodityType::Nickel, CommodityPrice {
            commodity: CommodityType::Nickel,
            current_price: Decimal::new(18450, 0), // $18,450/tonne
            currency: "USD".to_string(),
            unit: "per tonne".to_string(),
            timestamp: Utc::now(),
            daily_change: Some(Decimal::new(135, 0)), // +$135
            daily_change_percent: Some(Decimal::new(135, 2)), // +1.35%
            volume: Some(2850000),
            market_status: MarketStatus::Open,
            source: name.clone(),
        });
        
        prices.insert(CommodityType::Coal, CommodityPrice {
            commodity: CommodityType::Coal,
            current_price: Decimal::new(13550, 2), // $135.50/tonne
            currency: "USD".to_string(),
            unit: "per tonne".to_string(),
            timestamp: Utc::now(),
            daily_change: Some(Decimal::new(285, 2)), // +$2.85
            daily_change_percent: Some(Decimal::new(285, 2)), // +2.85%
            volume: Some(1250000),
            market_status: MarketStatus::Open,
            source: name.clone(),
        });
        
        prices.insert(CommodityType::CPO, CommodityPrice {
            commodity: CommodityType::CPO,
            current_price: Decimal::new(965, 0), // $965/tonne
            currency: "USD".to_string(),
            unit: "per tonne".to_string(),
            timestamp: Utc::now(),
            daily_change: Some(Decimal::new(-128, 1)), // -$12.8
            daily_change_percent: Some(Decimal::new(-128, 2)), // -1.28%
            volume: Some(845000),
            market_status: MarketStatus::Open,
            source: name.clone(),
        });
        
        prices.insert(CommodityType::Gold, CommodityPrice {
            commodity: CommodityType::Gold,
            current_price: Decimal::new(201850, 2), // $2,018.50/oz
            currency: "USD".to_string(),
            unit: "per troy ounce".to_string(),
            timestamp: Utc::now(),
            daily_change: Some(Decimal::new(-75, 1)), // -$7.5
            daily_change_percent: Some(Decimal::new(-75, 2)), // -0.75%
            volume: Some(125000),
            market_status: MarketStatus::Open,
            source: name.clone(),
        });
        
        prices.insert(CommodityType::Oil, CommodityPrice {
            commodity: CommodityType::Oil,
            current_price: Decimal::new(7845, 2), // $78.45/barrel
            currency: "USD".to_string(),
            unit: "per barrel".to_string(),
            timestamp: Utc::now(),
            daily_change: Some(Decimal::new(159, 2)), // +$1.59
            daily_change_percent: Some(Decimal::new(159, 2)), // +1.59%
            volume: Some(2150000),
            market_status: MarketStatus::Open,
            source: name.clone(),
        });
        
        Self { name, prices }
    }
}

#[async_trait]
impl CommodityDataSource for MockCommoditySource {
    async fn get_price(&self, commodity: CommodityType) -> Result<CommodityPrice> {
        self.prices.get(&commodity)
            .cloned()
            .ok_or_else(|| anyhow!("Commodity {} not found in mock data", commodity))
    }
    
    async fn get_historical_prices(&self, commodity: CommodityType, days: u32) -> Result<Vec<CommodityPrice>> {
        let base_price = self.get_price(commodity).await?;
        let mut prices = Vec::new();
        
        // Generate mock historical data
        for i in 0..days {
            let mut price = base_price.clone();
            // Add some realistic price variation
            let variation = (i as f64 * 0.01 - days as f64 * 0.005) / 100.0;
            price.current_price = base_price.current_price * Decimal::new((1.0 + variation * 1000.0) as i64, 3);
            price.timestamp = Utc::now() - chrono::Duration::days(i as i64);
            prices.push(price);
        }
        
        Ok(prices)
    }
    
    fn source_name(&self) -> &str {
        &self.name
    }
    
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

/// Main commodity tracking orchestrator
pub struct CommodityTracker {
    sources: Vec<Box<dyn CommodityDataSource + Send + Sync>>,
    supported_commodities: Vec<CommodityType>,
}

impl CommodityTracker {
    /// Create new commodity tracker with data sources
    pub fn new(source_urls: &[String]) -> Self {
        let mut sources: Vec<Box<dyn CommodityDataSource + Send + Sync>> = Vec::new();
        
        // For now, use mock sources (in production, these would be real API clients)
        for url in source_urls {
            let source_name = url.split('/').last().unwrap_or("unknown").to_string();
            sources.push(Box::new(MockCommoditySource::new(source_name)));
        }
        
        Self {
            sources,
            supported_commodities: vec![
                CommodityType::Nickel,
                CommodityType::Coal,
                CommodityType::CPO,
                CommodityType::Gold,
                CommodityType::Oil,
            ],
        }
    }
    
    /// Get current prices for all supported commodities
    pub async fn get_all_prices(&self) -> Result<HashMap<CommodityType, CommodityPrice>> {
        info!("📊 Fetching current prices for {} commodities", self.supported_commodities.len());
        
        let mut prices = HashMap::new();
        let source = &self.sources[0]; // Use primary source for now
        
        for &commodity in &self.supported_commodities {
            match source.get_price(commodity).await {
                Ok(price) => {
                    debug!("✅ Got price for {}: ${}", commodity, price.current_price);
                    prices.insert(commodity, price);
                },
                Err(e) => {
                    warn!("❌ Failed to get price for {}: {}", commodity, e);
                }
            }
        }
        
        info!("✅ Retrieved {} commodity prices", prices.len());
        Ok(prices)
    }
    
    /// Get price for specific commodity
    pub async fn get_price(&self, commodity: CommodityType) -> Result<CommodityPrice> {
        let source = &self.sources[0];
        source.get_price(commodity).await
    }
    
    /// Get historical price data for analysis
    pub async fn get_historical(&self, commodity: CommodityType, days: u32) -> Result<Vec<CommodityPrice>> {
        let source = &self.sources[0];
        source.get_historical_prices(commodity, days).await
    }
    
    /// Calculate price volatility over period
    pub async fn calculate_volatility(&self, commodity: CommodityType, days: u32) -> Result<f64> {
        let prices = self.get_historical(commodity, days).await?;
        
        if prices.len() < 2 {
            return Ok(0.0);
        }
        
        let price_changes: Vec<f64> = prices.windows(2)
            .map(|window| {
                let change = (window[0].current_price - window[1].current_price).to_f64().unwrap_or(0.0);
                let base = window[1].current_price.to_f64().unwrap_or(1.0);
                change / base
            })
            .collect();
        
        let mean = price_changes.iter().sum::<f64>() / price_changes.len() as f64;
        let variance = price_changes.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / price_changes.len() as f64;
        
        Ok(variance.sqrt())
    }
    
    /// Get commodities relevant to Indonesian stock
    pub fn get_relevant_commodities(&self, stock: hermes_common::types::IndonesianStock) -> Vec<CommodityType> {
        use hermes_common::types::IndonesianStock;
        
        match stock {
            IndonesianStock::INCO | IndonesianStock::ANTM => vec![CommodityType::Nickel, CommodityType::Copper],
            IndonesianStock::PTBA => vec![CommodityType::Coal],
            IndonesianStock::TAPG => vec![CommodityType::CPO],
            IndonesianStock::BMRI | IndonesianStock::BBRI => vec![CommodityType::Gold, CommodityType::Oil], // Safe haven/inflation hedge
            _ => vec![CommodityType::Gold, CommodityType::Oil], // General economic indicators
        }
    }
    
    /// Health check all data sources
    pub async fn health_check_all(&self) -> Result<Vec<String>> {
        let mut results = Vec::new();
        
        for source in &self.sources {
            match source.health_check().await {
                Ok(true) => results.push(format!("✅ {} - Healthy", source.source_name())),
                Ok(false) => results.push(format!("⚠️  {} - Degraded", source.source_name())),
                Err(e) => results.push(format!("❌ {} - Error: {}", source.source_name(), e)),
            }
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_commodity_tracker_creation() {
        let sources = vec!["https://api.example.com".to_string()];
        let tracker = CommodityTracker::new(&sources);
        assert_eq!(tracker.supported_commodities.len(), 5);
        assert_eq!(tracker.sources.len(), 1);
    }
    
    #[tokio::test]
    async fn test_mock_commodity_source() {
        let source = MockCommoditySource::new("test".to_string());
        
        let nickel_price = source.get_price(CommodityType::Nickel).await;
        assert!(nickel_price.is_ok());
        
        let price = nickel_price.unwrap();
        assert_eq!(price.commodity, CommodityType::Nickel);
        assert_eq!(price.currency, "USD");
        assert!(price.current_price > Decimal::ZERO);
    }
    
    #[tokio::test]
    async fn test_get_all_prices() {
        let sources = vec!["https://api.test.com".to_string()];
        let tracker = CommodityTracker::new(&sources);
        
        let prices = tracker.get_all_prices().await;
        assert!(prices.is_ok());
        
        let price_map = prices.unwrap();
        assert!(price_map.contains_key(&CommodityType::Nickel));
        assert!(price_map.contains_key(&CommodityType::Coal));
    }
    
    #[tokio::test]
    async fn test_relevant_commodities_mapping() {
        let sources = vec!["https://api.test.com".to_string()];
        let tracker = CommodityTracker::new(&sources);
        
        let inco_commodities = tracker.get_relevant_commodities(hermes_common::types::IndonesianStock::INCO);
        assert!(inco_commodities.contains(&CommodityType::Nickel));
        
        let ptba_commodities = tracker.get_relevant_commodities(hermes_common::types::IndonesianStock::PTBA);
        assert!(ptba_commodities.contains(&CommodityType::Coal));
    }
    
    #[tokio::test]
    async fn test_volatility_calculation() {
        let sources = vec!["https://api.test.com".to_string()];
        let tracker = CommodityTracker::new(&sources);
        
        let volatility = tracker.calculate_volatility(CommodityType::Nickel, 10).await;
        assert!(volatility.is_ok());
        assert!(volatility.unwrap() >= 0.0);
    }
}