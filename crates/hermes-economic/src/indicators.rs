/// Economic indicators and market sentiment analysis for Indonesian economy
use anyhow::{Result, anyhow};
use rust_decimal::prelude::ToPrimitive;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, debug};
use rust_decimal::Decimal;
use crate::commodity::{CommodityType, CommodityPrice};
use crate::correlation::CorrelationMatrix;

/// Market sentiment analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSentiment {
    pub timestamp: DateTime<Utc>,
    pub overall_sentiment: SentimentScore,
    pub commodity_sentiment: HashMap<CommodityType, SentimentScore>,
    pub sector_sentiment: HashMap<String, SentimentScore>,
    pub confidence_level: f64, // 0.0 to 1.0
    pub key_drivers: Vec<String>,
}

/// Sentiment scoring system
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SentimentScore {
    VeryBearish,  // -2
    Bearish,      // -1  
    Neutral,      //  0
    Bullish,      // +1
    VeryBullish,  // +2
}

impl SentimentScore {
    pub fn to_numeric(&self) -> i8 {
        match self {
            SentimentScore::VeryBearish => -2,
            SentimentScore::Bearish => -1,
            SentimentScore::Neutral => 0,
            SentimentScore::Bullish => 1,
            SentimentScore::VeryBullish => 2,
        }
    }
    
    pub fn from_numeric(value: f64) -> Self {
        match value {
            x if x <= -1.5 => SentimentScore::VeryBearish,
            x if x <= -0.5 => SentimentScore::Bearish,
            x if x <= 0.5 => SentimentScore::Neutral,
            x if x <= 1.5 => SentimentScore::Bullish,
            _ => SentimentScore::VeryBullish,
        }
    }
}

impl std::fmt::Display for SentimentScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SentimentScore::VeryBearish => write!(f, "Very Bearish"),
            SentimentScore::Bearish => write!(f, "Bearish"),
            SentimentScore::Neutral => write!(f, "Neutral"),
            SentimentScore::Bullish => write!(f, "Bullish"),
            SentimentScore::VeryBullish => write!(f, "Very Bullish"),
        }
    }
}

/// Economic trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub timestamp: DateTime<Utc>,
    pub commodity_trends: HashMap<CommodityType, TrendDirection>,
    pub rate_trend: RateTrend,
    pub inflation_outlook: InflationOutlook,
    pub growth_momentum: GrowthMomentum,
    pub risk_assessment: RiskAssessment,
}

/// Trend direction indicators
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    StronglyDown,
    Down,
    Sideways,
    Up,
    StronglyUp,
}

impl std::fmt::Display for TrendDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrendDirection::StronglyDown => write!(f, "📉 Strongly Down"),
            TrendDirection::Down => write!(f, "📉 Down"),
            TrendDirection::Sideways => write!(f, "📊 Sideways"),
            TrendDirection::Up => write!(f, "📈 Up"),
            TrendDirection::StronglyUp => write!(f, "📈 Strongly Up"),
        }
    }
}

/// Interest rate trend
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RateTrend {
    Easing,     // Rates falling
    Stable,     // Rates unchanged
    Tightening, // Rates rising
}

/// Inflation outlook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InflationOutlook {
    pub current_estimate: Option<Decimal>,
    pub target_range: (Decimal, Decimal), // BI target: 3% +/- 1%
    pub trend: TrendDirection,
    pub key_risks: Vec<String>,
}

/// Economic growth momentum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthMomentum {
    pub gdp_growth_estimate: Option<Decimal>,
    pub momentum: TrendDirection,
    pub leading_indicators: Vec<String>,
    pub sector_contributions: HashMap<String, f64>,
}

/// Risk assessment framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk_level: RiskLevel,
    pub domestic_risks: Vec<String>,
    pub external_risks: Vec<String>,
    pub commodity_risks: Vec<String>,
    pub financial_stability_risks: Vec<String>,
}

/// Risk level categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Moderate,
    Elevated,
    High,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "🟢 Low"),
            RiskLevel::Moderate => write!(f, "🟡 Moderate"),
            RiskLevel::Elevated => write!(f, "🟠 Elevated"),
            RiskLevel::High => write!(f, "🔴 High"),
        }
    }
}

/// Economic indicators analysis engine
pub struct EconomicIndicators {
    sentiment_weights: HashMap<String, f64>,
    trend_lookback_days: u32,
}

impl EconomicIndicators {
    /// Create new economic indicators analyzer
    pub fn new() -> Self {
        let mut sentiment_weights = HashMap::new();
        
        // Weight different factors for sentiment analysis
        sentiment_weights.insert("commodity_price_momentum".to_string(), 0.30);
        sentiment_weights.insert("stock_correlation_strength".to_string(), 0.25);
        sentiment_weights.insert("interest_rate_outlook".to_string(), 0.20);
        sentiment_weights.insert("volatility_levels".to_string(), 0.15);
        sentiment_weights.insert("external_factors".to_string(), 0.10);
        
        Self {
            sentiment_weights,
            trend_lookback_days: 30,
        }
    }
    
    /// Analyze market sentiment based on commodity prices and correlations
    pub async fn analyze_sentiment(&self, commodities: &HashMap<CommodityType, CommodityPrice>,
                                 correlations: &CorrelationMatrix) -> Result<MarketSentiment> {
        info!("📊 Analyzing market sentiment across {} commodities", commodities.len());
        
        let timestamp = Utc::now();
        let mut commodity_sentiment = HashMap::new();
        let mut sector_sentiment = HashMap::new();
        let mut key_drivers = Vec::new();
        
        // Analyze commodity sentiment
        for (commodity_type, price) in commodities {
            let sentiment = self.calculate_commodity_sentiment(price);
            commodity_sentiment.insert(*commodity_type, sentiment);
            
            if let Some(change_pct) = price.daily_change_percent {
                if change_pct.abs() > Decimal::new(3, 0) { // 3% threshold
                    key_drivers.push(format!("{} {:+.1}%", commodity_type, change_pct));
                }
            }
        }
        
        // Analyze sector sentiment based on correlations
        for (sector, correlation) in &correlations.sector_correlations {
            let sentiment = if *correlation > 0.7 {
                if self.is_sector_positive(sector, commodities) {
                    SentimentScore::Bullish
                } else {
                    SentimentScore::Bearish
                }
            } else {
                SentimentScore::Neutral
            };
            sector_sentiment.insert(sector.clone(), sentiment);
        }
        
        // Calculate overall sentiment
        let sentiment_values: Vec<i8> = commodity_sentiment.values()
            .map(|s| s.to_numeric())
            .collect();
        
        let avg_sentiment = if sentiment_values.is_empty() {
            0.0
        } else {
            sentiment_values.iter().sum::<i8>() as f64 / sentiment_values.len() as f64
        };
        
        let overall_sentiment = SentimentScore::from_numeric(avg_sentiment);
        
        // Calculate confidence level based on data quality
        let confidence_level = self.calculate_confidence_level(commodities, correlations);
        
        info!("✅ Market sentiment analysis complete: {} (confidence: {:.1}%)", 
              overall_sentiment, confidence_level * 100.0);
        
        Ok(MarketSentiment {
            timestamp,
            overall_sentiment,
            commodity_sentiment,
            sector_sentiment,
            confidence_level,
            key_drivers,
        })
    }
    
    /// Analyze economic trends
    pub async fn analyze_trends(&self, commodities: &HashMap<CommodityType, CommodityPrice>,
                               bi_rate: Option<Decimal>) -> Result<TrendAnalysis> {
        info!("📈 Analyzing economic trends and momentum");
        
        let timestamp = Utc::now();
        let mut commodity_trends = HashMap::new();
        
        // Analyze commodity trends
        for (commodity_type, price) in commodities {
            let trend = self.calculate_price_trend(price);
            commodity_trends.insert(*commodity_type, trend);
        }
        
        // Analyze rate trend
        let rate_trend = self.analyze_rate_trend(bi_rate);
        
        // Inflation outlook
        let inflation_outlook = InflationOutlook {
            current_estimate: Some(Decimal::new(295, 2)), // 2.95%
            target_range: (Decimal::new(2, 0), Decimal::new(4, 0)), // 2-4%
            trend: TrendDirection::Sideways,
            key_risks: vec![
                "Commodity price volatility".to_string(),
                "Global supply chain disruptions".to_string(),
                "Exchange rate fluctuations".to_string(),
            ],
        };
        
        // Growth momentum
        let growth_momentum = GrowthMomentum {
            gdp_growth_estimate: Some(Decimal::new(510, 2)), // 5.10%
            momentum: TrendDirection::Up,
            leading_indicators: vec![
                "Manufacturing PMI expansion".to_string(),
                "Export growth recovery".to_string(),
                "Domestic consumption resilience".to_string(),
            ],
            sector_contributions: [
                ("Manufacturing".to_string(), 0.25),
                ("Services".to_string(), 0.35),
                ("Agriculture".to_string(), 0.15),
                ("Mining".to_string(), 0.12),
                ("Construction".to_string(), 0.13),
            ].iter().cloned().collect(),
        };
        
        // Risk assessment
        let risk_assessment = self.assess_risks(commodities, bi_rate);
        
        info!("✅ Economic trend analysis complete");
        
        Ok(TrendAnalysis {
            timestamp,
            commodity_trends,
            rate_trend,
            inflation_outlook,
            growth_momentum,
            risk_assessment,
        })
    }
    
    /// Calculate sentiment for individual commodity
    fn calculate_commodity_sentiment(&self, price: &CommodityPrice) -> SentimentScore {
        if let Some(change_pct) = price.daily_change_percent {
            let change_val = change_pct.to_f64().unwrap_or(0.0);
            match change_val {
                x if x > 5.0 => SentimentScore::VeryBullish,
                x if x > 2.0 => SentimentScore::Bullish,
                x if x > -2.0 => SentimentScore::Neutral,
                x if x > -5.0 => SentimentScore::Bearish,
                _ => SentimentScore::VeryBearish,
            }
        } else {
            SentimentScore::Neutral
        }
    }
    
    /// Check if sector is showing positive signals
    fn is_sector_positive(&self, sector: &str, commodities: &HashMap<CommodityType, CommodityPrice>) -> bool {
        let relevant_commodities = match sector {
            "Mining" => vec![CommodityType::Nickel, CommodityType::Copper],
            "Energy" => vec![CommodityType::Coal, CommodityType::Oil],
            "Agriculture" => vec![CommodityType::CPO],
            _ => vec![CommodityType::Gold], // Safe haven indicator
        };
        
        let positive_signals = relevant_commodities.iter()
            .filter_map(|commodity| commodities.get(commodity))
            .filter(|price| {
                price.daily_change_percent.unwrap_or(Decimal::ZERO) > Decimal::ZERO
            })
            .count();
        
        positive_signals > relevant_commodities.len() / 2
    }
    
    /// Calculate price trend for commodity
    fn calculate_price_trend(&self, price: &CommodityPrice) -> TrendDirection {
        if let Some(change_pct) = price.daily_change_percent {
            let change_val = change_pct.to_f64().unwrap_or(0.0);
            match change_val {
                x if x > 3.0 => TrendDirection::StronglyUp,
                x if x > 1.0 => TrendDirection::Up,
                x if x > -1.0 => TrendDirection::Sideways,
                x if x > -3.0 => TrendDirection::Down,
                _ => TrendDirection::StronglyDown,
            }
        } else {
            TrendDirection::Sideways
        }
    }
    
    /// Analyze interest rate trend
    fn analyze_rate_trend(&self, bi_rate: Option<Decimal>) -> RateTrend {
        // In a real implementation, this would analyze historical rate changes
        // For now, assume neutral stance based on current economic conditions
        RateTrend::Stable
    }
    
    /// Assess economic risks
    fn assess_risks(&self, commodities: &HashMap<CommodityType, CommodityPrice>, 
                   bi_rate: Option<Decimal>) -> RiskAssessment {
        
        // Calculate volatility-based risk
        let high_volatility_count = commodities.values()
            .filter(|price| {
                if let Some(change) = price.daily_change_percent {
                    change.abs() > Decimal::new(4, 0) // 4% threshold
                } else {
                    false
                }
            })
            .count();
        
        let overall_risk_level = if high_volatility_count > commodities.len() / 2 {
            RiskLevel::Elevated
        } else {
            RiskLevel::Moderate
        };
        
        RiskAssessment {
            overall_risk_level,
            domestic_risks: vec![
                "Inflation pressures from commodity prices".to_string(),
                "Rupiah volatility impact on imports".to_string(),
                "Domestic demand sustainability".to_string(),
            ],
            external_risks: vec![
                "Global supply chain disruptions".to_string(),
                "Geopolitical tensions affecting trade".to_string(),
                "Major central bank policy divergence".to_string(),
            ],
            commodity_risks: vec![
                "Nickel price volatility (INCO exposure)".to_string(),
                "Coal demand uncertainty (PTBA impact)".to_string(),
                "Palm oil sustainability regulations".to_string(),
            ],
            financial_stability_risks: vec![
                "Banking sector credit growth sustainability".to_string(),
                "Corporate debt servicing capacity".to_string(),
                "External financing conditions".to_string(),
            ],
        }
    }
    
    /// Calculate confidence level for analysis
    fn calculate_confidence_level(&self, commodities: &HashMap<CommodityType, CommodityPrice>,
                                 correlations: &CorrelationMatrix) -> f64 {
        let mut factors = Vec::new();
        
        // Data freshness factor
        let data_freshness = commodities.values()
            .map(|price| {
                let hours_old = (Utc::now() - price.timestamp).num_hours();
                (24.0 - hours_old.min(24) as f64) / 24.0
            })
            .collect::<Vec<f64>>();
        
        if !data_freshness.is_empty() {
            factors.push(data_freshness.iter().sum::<f64>() / data_freshness.len() as f64);
        }
        
        // Market status factor
        let market_open_count = commodities.values()
            .filter(|price| price.market_status == crate::commodity::MarketStatus::Open)
            .count();
        
        let market_factor = market_open_count as f64 / commodities.len() as f64;
        factors.push(market_factor);
        
        // Correlation data quality
        let correlation_count = correlations.stock_correlations.len();
        let correlation_factor = if correlation_count > 0 { 0.9 } else { 0.5 };
        factors.push(correlation_factor);
        
        // Average all factors
        if factors.is_empty() {
            0.5
        } else {
            factors.iter().sum::<f64>() / factors.len() as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commodity::MarketStatus;
    
    #[test]
    fn test_sentiment_score_conversion() {
        assert_eq!(SentimentScore::VeryBullish.to_numeric(), 2);
        assert_eq!(SentimentScore::Neutral.to_numeric(), 0);
        assert_eq!(SentimentScore::VeryBearish.to_numeric(), -2);
        
        assert_eq!(SentimentScore::from_numeric(1.8), SentimentScore::VeryBullish);
        assert_eq!(SentimentScore::from_numeric(0.0), SentimentScore::Neutral);
        assert_eq!(SentimentScore::from_numeric(-1.8), SentimentScore::VeryBearish);
    }
    
    #[test]
    fn test_trend_direction_display() {
        assert!(TrendDirection::StronglyUp.to_string().contains("Strongly Up"));
        assert!(TrendDirection::Down.to_string().contains("Down"));
        assert!(TrendDirection::Sideways.to_string().contains("Sideways"));
    }
    
    #[test]
    fn test_risk_level_display() {
        assert!(RiskLevel::High.to_string().contains("High"));
        assert!(RiskLevel::Low.to_string().contains("Low"));
    }
    
    #[tokio::test]
    async fn test_economic_indicators_creation() {
        let indicators = EconomicIndicators::new();
        assert_eq!(indicators.trend_lookback_days, 30);
        assert!(!indicators.sentiment_weights.is_empty());
    }
    
    #[test]
    fn test_calculate_commodity_sentiment() {
        let indicators = EconomicIndicators::new();
        
        // Test bullish sentiment
        let bullish_price = CommodityPrice {
            commodity: CommodityType::Nickel,
            current_price: Decimal::new(18450, 0),
            currency: "USD".to_string(),
            unit: "per tonne".to_string(),
            timestamp: Utc::now(),
            daily_change: Some(Decimal::new(500, 0)),
            daily_change_percent: Some(Decimal::new(3, 0)), // +3%
            volume: Some(1000000),
            market_status: MarketStatus::Open,
            source: "test".to_string(),
        };
        
        let sentiment = indicators.calculate_commodity_sentiment(&bullish_price);
        assert_eq!(sentiment, SentimentScore::Bullish);
        
        // Test bearish sentiment
        let mut bearish_price = bullish_price.clone();
        bearish_price.daily_change_percent = Some(Decimal::new(-4, 0)); // -4%
        
        let sentiment = indicators.calculate_commodity_sentiment(&bearish_price);
        assert_eq!(sentiment, SentimentScore::Bearish);
    }
    
    #[test]
    fn test_calculate_price_trend() {
        let indicators = EconomicIndicators::new();
        
        let price = CommodityPrice {
            commodity: CommodityType::Gold,
            current_price: Decimal::new(2000, 0),
            currency: "USD".to_string(),
            unit: "per ounce".to_string(),
            timestamp: Utc::now(),
            daily_change: None,
            daily_change_percent: Some(Decimal::new(25, 1)), // +2.5%
            volume: Some(100000),
            market_status: MarketStatus::Open,
            source: "test".to_string(),
        };
        
        let trend = indicators.calculate_price_trend(&price);
        assert_eq!(trend, TrendDirection::Up);
    }
}