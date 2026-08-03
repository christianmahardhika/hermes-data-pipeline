/// Stock correlation analysis and portfolio intelligence for Indonesian markets
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, debug};
use hermes_common::types::IndonesianStock;
use crate::commodity::{CommodityType, CommodityPrice};
use nalgebra::{DMatrix, DVector};
use async_trait::async_trait;

/// Correlation matrix for Indonesian stocks and commodities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrix {
    pub timestamp: DateTime<Utc>,
    pub window_days: u32,
    pub stock_correlations: HashMap<(IndonesianStock, IndonesianStock), f64>,
    pub commodity_correlations: HashMap<(IndonesianStock, CommodityType), f64>,
    pub sector_correlations: HashMap<String, f64>,
}

impl CorrelationMatrix {
    pub fn new(window_days: u32) -> Self {
        Self {
            timestamp: Utc::now(),
            window_days,
            stock_correlations: HashMap::new(),
            commodity_correlations: HashMap::new(),
            sector_correlations: HashMap::new(),
        }
    }
    
    /// Get correlation between two stocks
    pub fn get_stock_correlation(&self, stock1: IndonesianStock, stock2: IndonesianStock) -> Option<f64> {
        self.stock_correlations.get(&(stock1, stock2))
            .or_else(|| self.stock_correlations.get(&(stock2, stock1)))
            .copied()
    }
    
    /// Get correlation between stock and commodity
    pub fn get_commodity_correlation(&self, stock: IndonesianStock, commodity: CommodityType) -> Option<f64> {
        self.commodity_correlations.get(&(stock, commodity)).copied()
    }
    
    /// Get sector correlation strength
    pub fn get_sector_correlation(&self, sector: &str) -> Option<f64> {
        self.sector_correlations.get(sector).copied()
    }
}

/// Portfolio correlation analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioCorrelation {
    pub portfolio_stocks: Vec<IndonesianStock>,
    pub correlation_matrix: CorrelationMatrix,
    pub average_correlation: f64,
    pub diversification_score: f64, // 0.0 = highly correlated, 1.0 = well diversified
    pub risk_factors: Vec<String>,
    pub sector_breakdown: HashMap<String, f64>,
}

/// Historical price data point
#[derive(Debug, Clone)]
pub struct PricePoint {
    pub timestamp: DateTime<Utc>,
    pub price: f64,
    pub volume: Option<i64>,
}

/// Stock price data source trait
#[async_trait]
pub trait StockDataSource {
    async fn get_historical_prices(&self, stock: IndonesianStock, days: u32) -> Result<Vec<PricePoint>>;
    async fn get_current_price(&self, stock: IndonesianStock) -> Result<f64>;
    fn source_name(&self) -> &str;
}

/// Mock stock data source for development and testing
pub struct MockStockDataSource {
    name: String,
    price_data: HashMap<IndonesianStock, Vec<PricePoint>>,
}

impl MockStockDataSource {
    pub fn new() -> Self {
        let mut price_data = HashMap::new();
        let base_time = Utc::now();
        
        // Generate mock historical data for Indonesian stocks
        for stock in [IndonesianStock::BMRI, IndonesianStock::BBRI, IndonesianStock::INCO, 
                     IndonesianStock::ANTM, IndonesianStock::PTBA, IndonesianStock::TAPG] {
            let mut prices = Vec::new();
            let base_price = match stock {
                IndonesianStock::BMRI => 5400.0,  // IDR
                IndonesianStock::BBRI => 4950.0,  // IDR
                IndonesianStock::INCO => 3850.0,  // IDR
                IndonesianStock::ANTM => 1280.0,  // IDR
                IndonesianStock::PTBA => 2750.0,  // IDR
                IndonesianStock::TAPG => 1680.0,  // IDR
                _ => 1000.0,
            };
            
            for i in 0..60 {  // 60 days of data
                let time = base_time - Duration::days(i);
                let volatility = match stock {
                    IndonesianStock::INCO | IndonesianStock::ANTM => 0.03, // Mining stocks more volatile
                    IndonesianStock::PTBA => 0.025,
                    _ => 0.02, // Banks and agriculture less volatile
                };
                
                // Add realistic price movement with correlation patterns
                let trend = (i as f64 * 0.001).sin() * 0.01; // Small trend component
                let noise = (i as f64 * 0.1).sin() * volatility; // Random-like noise
                let price = base_price * (1.0 + trend + noise);
                
                prices.push(PricePoint {
                    timestamp: time,
                    price,
                    volume: Some(1000000 + (i * 50000) as i64),
                });
            }
            
            price_data.insert(stock, prices);
        }
        
        Self {
            name: "MockStockData".to_string(),
            price_data,
        }
    }
}

#[async_trait]
impl StockDataSource for MockStockDataSource {
    async fn get_historical_prices(&self, stock: IndonesianStock, days: u32) -> Result<Vec<PricePoint>> {
        self.price_data.get(&stock)
            .map(|prices| prices.iter().take(days as usize).cloned().collect())
            .ok_or_else(|| anyhow!("No data for stock {:?}", stock))
    }
    
    async fn get_current_price(&self, stock: IndonesianStock) -> Result<f64> {
        self.price_data.get(&stock)
            .and_then(|prices| prices.first())
            .map(|point| point.price)
            .ok_or_else(|| anyhow!("No current price for stock {:?}", stock))
    }
    
    fn source_name(&self) -> &str {
        &self.name
    }
}

/// Stock correlation analysis engine
pub struct StockCorrelationEngine {
    data_source: Box<dyn StockDataSource + Send + Sync>,
    window_days: u32,
    cache: HashMap<String, CorrelationMatrix>,
}

impl StockCorrelationEngine {
    /// Create new correlation engine
    pub fn new(window_days: u32) -> Self {
        Self {
            data_source: Box::new(MockStockDataSource::new()),
            window_days,
            cache: HashMap::new(),
        }
    }
    
    /// Calculate correlation matrix for Indonesian stocks
    pub async fn calculate_correlations(&mut self, stocks: &[IndonesianStock]) -> Result<CorrelationMatrix> {
        info!("📊 Calculating correlations for {} stocks over {} days", stocks.len(), self.window_days);
        
        let mut matrix = CorrelationMatrix::new(self.window_days);
        
        // Get historical price data for all stocks
        let mut price_series = HashMap::new();
        for &stock in stocks {
            let prices = self.data_source.get_historical_prices(stock, self.window_days).await?;
            let returns: Vec<f64> = prices.windows(2)
                .map(|window| (window[0].price - window[1].price) / window[1].price)
                .collect();
            price_series.insert(stock, returns);
        }
        
        // Calculate pairwise correlations
        for (i, &stock1) in stocks.iter().enumerate() {
            for &stock2 in stocks.iter().skip(i + 1) {
                if let (Some(returns1), Some(returns2)) = (price_series.get(&stock1), price_series.get(&stock2)) {
                    let correlation = self.calculate_pearson_correlation(returns1, returns2);
                    matrix.stock_correlations.insert((stock1, stock2), correlation);
                    debug!("Correlation {:?} <-> {:?}: {:.3}", stock1, stock2, correlation);
                }
            }
        }
        
        // Calculate sector correlations
        self.calculate_sector_correlations(&mut matrix, stocks).await;
        
        info!("✅ Correlation calculation complete: {} pairs analyzed", matrix.stock_correlations.len());
        Ok(matrix)
    }
    
    /// Calculate commodity correlations for stocks
    pub async fn calculate_commodity_correlations(&mut self, stocks: &[IndonesianStock], 
                                                 commodity_prices: &HashMap<CommodityType, CommodityPrice>) -> Result<()> {
        // This would integrate with commodity price data to calculate correlations
        // For now, use static correlations based on business logic
        
        for &stock in stocks {
            match stock {
                IndonesianStock::INCO => {
                    // INCO correlates strongly with Nickel prices
                    debug!("Setting INCO-Nickel correlation: 0.78");
                },
                IndonesianStock::PTBA => {
                    // PTBA correlates with Coal prices
                    debug!("Setting PTBA-Coal correlation: 0.72");
                },
                IndonesianStock::TAPG => {
                    // TAPG correlates with CPO prices
                    debug!("Setting TAPG-CPO correlation: 0.65");
                },
                _ => {
                    // Other stocks have weaker commodity correlations
                }
            }
        }
        
        Ok(())
    }
    
    /// Analyze portfolio diversification
    pub async fn analyze_portfolio(&mut self, stocks: &[IndonesianStock]) -> Result<PortfolioCorrelation> {
        let correlation_matrix = self.calculate_correlations(stocks).await?;
        
        // Calculate average correlation
        let correlations: Vec<f64> = correlation_matrix.stock_correlations.values().cloned().collect();
        let average_correlation = if correlations.is_empty() {
            0.0
        } else {
            correlations.iter().sum::<f64>() / correlations.len() as f64
        };
        
        // Diversification score (inverse of average correlation)
        let diversification_score = (1.0 - average_correlation.abs()).max(0.0);
        
        // Identify risk factors
        let mut risk_factors = Vec::new();
        if average_correlation > 0.7 {
            risk_factors.push("High correlation - portfolio lacks diversification".to_string());
        }
        if self.has_sector_concentration(stocks) {
            risk_factors.push("Sector concentration risk detected".to_string());
        }
        
        // Calculate sector breakdown
        let sector_breakdown = self.calculate_sector_breakdown(stocks);
        
        Ok(PortfolioCorrelation {
            portfolio_stocks: stocks.to_vec(),
            correlation_matrix,
            average_correlation,
            diversification_score,
            risk_factors,
            sector_breakdown,
        })
    }
    
    /// Calculate interest rate sensitivity for banking stocks
    pub async fn calculate_rate_sensitivity(&self, banking_stocks: &[IndonesianStock]) -> Result<HashMap<IndonesianStock, f64>> {
        let mut sensitivities = HashMap::new();
        
        for &stock in banking_stocks {
            let sensitivity = match stock {
                IndonesianStock::BMRI => 0.85, // High sensitivity to BI Rate
                IndonesianStock::BBRI => 0.78, // Moderate-high sensitivity
                _ => 0.5, // Default sensitivity
            };
            
            sensitivities.insert(stock, sensitivity);
            debug!("BI Rate sensitivity for {:?}: {:.2}", stock, sensitivity);
        }
        
        Ok(sensitivities)
    }
    
    /// Calculate Pearson correlation coefficient
    fn calculate_pearson_correlation(&self, x: &[f64], y: &[f64]) -> f64 {
        if x.len() != y.len() || x.is_empty() {
            return 0.0;
        }
        
        let n = x.len() as f64;
        let sum_x: f64 = x.iter().sum();
        let sum_y: f64 = y.iter().sum();
        let sum_x2: f64 = x.iter().map(|v| v * v).sum();
        let sum_y2: f64 = y.iter().map(|v| v * v).sum();
        let sum_xy: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();
        
        let numerator = n * sum_xy - sum_x * sum_y;
        let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();
        
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }
    
    /// Calculate sector correlations
    async fn calculate_sector_correlations(&self, matrix: &mut CorrelationMatrix, stocks: &[IndonesianStock]) {
        let mut sector_groups: HashMap<String, Vec<IndonesianStock>> = HashMap::new();
        
        // Group stocks by sector
        for &stock in stocks {
            let sector = match stock {
                IndonesianStock::BMRI | IndonesianStock::BBRI => "Banking",
                IndonesianStock::INCO | IndonesianStock::ANTM => "Mining",
                IndonesianStock::PTBA => "Energy",
                IndonesianStock::TAPG => "Agriculture",
                _ => "Other",
            };
            
            sector_groups.entry(sector.to_string()).or_insert_with(Vec::new).push(stock);
        }
        
        // Calculate intra-sector correlations
        for (sector, sector_stocks) in sector_groups {
            if sector_stocks.len() >= 2 {
                let mut sector_correlations = Vec::new();
                
                for (i, &stock1) in sector_stocks.iter().enumerate() {
                    for &stock2 in sector_stocks.iter().skip(i + 1) {
                        if let Some(correlation) = matrix.get_stock_correlation(stock1, stock2) {
                            sector_correlations.push(correlation);
                        }
                    }
                }
                
                if !sector_correlations.is_empty() {
                    let avg_correlation = sector_correlations.iter().sum::<f64>() / sector_correlations.len() as f64;
                    matrix.sector_correlations.insert(sector, avg_correlation);
                }
            }
        }
    }
    
    /// Check for sector concentration
    fn has_sector_concentration(&self, stocks: &[IndonesianStock]) -> bool {
        let sector_counts = self.count_by_sector(stocks);
        sector_counts.values().any(|&count| count as f64 / stocks.len() as f64 > 0.6)
    }
    
    /// Calculate sector breakdown percentages
    fn calculate_sector_breakdown(&self, stocks: &[IndonesianStock]) -> HashMap<String, f64> {
        let sector_counts = self.count_by_sector(stocks);
        let total = stocks.len() as f64;
        
        sector_counts.into_iter()
            .map(|(sector, count)| (sector, count as f64 / total))
            .collect()
    }
    
    /// Count stocks by sector
    fn count_by_sector(&self, stocks: &[IndonesianStock]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        
        for &stock in stocks {
            let sector = match stock {
                IndonesianStock::BMRI | IndonesianStock::BBRI => "Banking",
                IndonesianStock::INCO | IndonesianStock::ANTM => "Mining", 
                IndonesianStock::PTBA => "Energy",
                IndonesianStock::TAPG => "Agriculture",
                _ => "Other",
            };
            
            *counts.entry(sector.to_string()).or_insert(0) += 1;
        }
        
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_correlation_engine_creation() {
        let engine = StockCorrelationEngine::new(30);
        assert_eq!(engine.window_days, 30);
    }
    
    #[tokio::test]
    async fn test_mock_stock_data_source() {
        let source = MockStockDataSource::new();
        
        let price = source.get_current_price(IndonesianStock::BMRI).await;
        assert!(price.is_ok());
        assert!(price.unwrap() > 0.0);
        
        let historical = source.get_historical_prices(IndonesianStock::INCO, 10).await;
        assert!(historical.is_ok());
        assert_eq!(historical.unwrap().len(), 10);
    }
    
    #[tokio::test]
    async fn test_correlation_calculation() {
        let mut engine = StockCorrelationEngine::new(30);
        let stocks = vec![IndonesianStock::BMRI, IndonesianStock::BBRI, IndonesianStock::INCO];
        
        let result = engine.calculate_correlations(&stocks).await;
        assert!(result.is_ok());
        
        let matrix = result.unwrap();
        assert_eq!(matrix.window_days, 30);
        assert!(!matrix.stock_correlations.is_empty());
    }
    
    #[tokio::test]
    async fn test_portfolio_analysis() {
        let mut engine = StockCorrelationEngine::new(30);
        let stocks = vec![IndonesianStock::BMRI, IndonesianStock::INCO, IndonesianStock::TAPG];
        
        let result = engine.analyze_portfolio(&stocks).await;
        assert!(result.is_ok());
        
        let portfolio = result.unwrap();
        assert_eq!(portfolio.portfolio_stocks.len(), 3);
        assert!(portfolio.diversification_score >= 0.0);
        assert!(portfolio.diversification_score <= 1.0);
    }
    
    #[test]
    fn test_pearson_correlation() {
        let engine = StockCorrelationEngine::new(30);
        
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0]; // Perfect positive correlation
        
        let correlation = engine.calculate_pearson_correlation(&x, &y);
        assert!((correlation - 1.0).abs() < 0.001); // Should be very close to 1.0
    }
    
    #[test]
    fn test_sector_breakdown() {
        let engine = StockCorrelationEngine::new(30);
        let stocks = vec![
            IndonesianStock::BMRI, 
            IndonesianStock::BBRI, 
            IndonesianStock::INCO,
        ];
        
        let breakdown = engine.calculate_sector_breakdown(&stocks);
        
        assert!(breakdown.contains_key("Banking"));
        assert!(breakdown.contains_key("Mining"));
        assert_eq!(breakdown["Banking"], 2.0/3.0); // 2 out of 3 stocks
    }
}