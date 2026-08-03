/// Bank Indonesia (BI) Rate monitoring and monetary policy analysis
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, debug};
use rust_decimal::Decimal;
use async_trait::async_trait;

/// BI Rate and monetary policy data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestRateData {
    pub rate_type: RateType,
    pub current_rate: Decimal,
    pub previous_rate: Option<Decimal>,
    pub change_bps: Option<i32>, // Basis points change
    pub effective_date: NaiveDate,
    pub announcement_date: NaiveDate,
    pub next_meeting_date: Option<NaiveDate>,
    pub policy_stance: PolicyStance,
}

/// Types of Bank Indonesia interest rates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RateType {
    /// BI 7-Day Reverse Repo Rate (primary policy rate)
    BI7DRR,
    /// Deposit Facility Rate
    DFR,
    /// Lending Facility Rate  
    LFR,
}

impl std::fmt::Display for RateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateType::BI7DRR => write!(f, "BI 7-Day Reverse Repo Rate"),
            RateType::DFR => write!(f, "Deposit Facility Rate"),
            RateType::LFR => write!(f, "Lending Facility Rate"),
        }
    }
}

/// Monetary policy stance
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PolicyStance {
    Accommodative,
    Neutral,
    Restrictive,
    Unknown,
}

/// Monetary policy meeting decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonetaryPolicy {
    pub meeting_date: NaiveDate,
    pub decision: PolicyDecision,
    pub rationale: String,
    pub forward_guidance: Option<String>,
    pub inflation_target: Option<Decimal>,
    pub growth_outlook: Option<String>,
    pub key_risks: Vec<String>,
}

/// Policy decision types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PolicyDecision {
    Raise(i32), // Basis points increase
    Cut(i32),   // Basis points decrease
    Hold,       // No change
}

impl std::fmt::Display for PolicyDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyDecision::Raise(bps) => write!(f, "Rate hike: +{} bps", bps),
            PolicyDecision::Cut(bps) => write!(f, "Rate cut: -{} bps", bps),
            PolicyDecision::Hold => write!(f, "Hold rates unchanged"),
        }
    }
}

/// BI Rate data source trait
#[async_trait]
pub trait BIRateDataSource {
    async fn get_current_rates(&self) -> Result<HashMap<RateType, InterestRateData>>;
    async fn get_historical_rates(&self, rate_type: RateType, months: u32) -> Result<Vec<InterestRateData>>;
    async fn get_latest_policy_decision(&self) -> Result<MonetaryPolicy>;
    async fn get_next_meeting_date(&self) -> Result<Option<NaiveDate>>;
    fn source_name(&self) -> &str;
}

/// Mock BI Rate data source for development
pub struct MockBIRateSource {
    name: String,
    current_rates: HashMap<RateType, InterestRateData>,
    policy_history: Vec<MonetaryPolicy>,
}

impl MockBIRateSource {
    pub fn new() -> Self {
        let mut current_rates = HashMap::new();
        let today = Utc::now().date_naive();
        
        // Current BI Rate structure (as of 2026)
        current_rates.insert(RateType::BI7DRR, InterestRateData {
            rate_type: RateType::BI7DRR,
            current_rate: Decimal::new(575, 2), // 5.75%
            previous_rate: Some(Decimal::new(600, 2)), // Previous: 6.00%
            change_bps: Some(-25), // Last cut was 25bps
            effective_date: today,
            announcement_date: today - chrono::Duration::days(2),
            next_meeting_date: Some(today + chrono::Duration::days(45)),
            policy_stance: PolicyStance::Neutral,
        });
        
        current_rates.insert(RateType::DFR, InterestRateData {
            rate_type: RateType::DFR,
            current_rate: Decimal::new(400, 2), // 4.00%
            previous_rate: Some(Decimal::new(425, 2)),
            change_bps: Some(-25),
            effective_date: today,
            announcement_date: today - chrono::Duration::days(2),
            next_meeting_date: Some(today + chrono::Duration::days(45)),
            policy_stance: PolicyStance::Neutral,
        });
        
        current_rates.insert(RateType::LFR, InterestRateData {
            rate_type: RateType::LFR,
            current_rate: Decimal::new(750, 2), // 7.50%
            previous_rate: Some(Decimal::new(775, 2)),
            change_bps: Some(-25),
            effective_date: today,
            announcement_date: today - chrono::Duration::days(2),
            next_meeting_date: Some(today + chrono::Duration::days(45)),
            policy_stance: PolicyStance::Neutral,
        });
        
        // Mock policy history
        let policy_history = vec![
            MonetaryPolicy {
                meeting_date: today - chrono::Duration::days(2),
                decision: PolicyDecision::Cut(25),
                rationale: "Supporting economic recovery while maintaining price stability".to_string(),
                forward_guidance: Some("BI will continue to monitor global and domestic economic developments".to_string()),
                inflation_target: Some(Decimal::new(300, 2)), // 3.0% +/- 1%
                growth_outlook: Some("GDP growth expected 4.7-5.5% in 2026".to_string()),
                key_risks: vec![
                    "Global financial market volatility".to_string(),
                    "Commodity price fluctuations".to_string(),
                    "Geopolitical tensions".to_string(),
                ],
            },
        ];
        
        Self {
            name: "Bank Indonesia API".to_string(),
            current_rates,
            policy_history,
        }
    }
    
    /// Generate historical rate data
    fn generate_historical_rates(&self, rate_type: RateType, months: u32) -> Vec<InterestRateData> {
        let mut rates = Vec::new();
        let base_rate = self.current_rates.get(&rate_type).unwrap();
        
        for i in 0..months {
            let mut rate = base_rate.clone();
            
            // Add realistic rate evolution over time
            let months_ago = i as i64;
            rate.effective_date = Utc::now().date_naive() - chrono::Duration::days(months_ago * 30);
            rate.announcement_date = rate.effective_date - chrono::Duration::days(2);
            
            // Simulate rate cycle (rates were higher in the past, gradually cut)
            let rate_adjustment = (i as i64 * 25) / 100; // 25bps higher per month back
            rate.current_rate = base_rate.current_rate + Decimal::new(rate_adjustment, 2);
            
            rates.push(rate);
        }
        
        rates.reverse(); // Oldest first
        rates
    }
}

#[async_trait]
impl BIRateDataSource for MockBIRateSource {
    async fn get_current_rates(&self) -> Result<HashMap<RateType, InterestRateData>> {
        Ok(self.current_rates.clone())
    }
    
    async fn get_historical_rates(&self, rate_type: RateType, months: u32) -> Result<Vec<InterestRateData>> {
        Ok(self.generate_historical_rates(rate_type, months))
    }
    
    async fn get_latest_policy_decision(&self) -> Result<MonetaryPolicy> {
        self.policy_history.first()
            .cloned()
            .ok_or_else(|| anyhow!("No policy decisions available"))
    }
    
    async fn get_next_meeting_date(&self) -> Result<Option<NaiveDate>> {
        Ok(self.current_rates.get(&RateType::BI7DRR)
           .and_then(|rate| rate.next_meeting_date))
    }
    
    fn source_name(&self) -> &str {
        &self.name
    }
}

/// BI Rate monitoring system
pub struct BIRateMonitor {
    data_source: Box<dyn BIRateDataSource + Send + Sync>,
    endpoint_url: String,
    cache: HashMap<RateType, InterestRateData>,
}

impl BIRateMonitor {
    /// Create new BI Rate monitor
    pub fn new(endpoint_url: &str) -> Self {
        Self {
            data_source: Box::new(MockBIRateSource::new()),
            endpoint_url: endpoint_url.to_string(),
            cache: HashMap::new(),
        }
    }
    
    /// Get current BI 7-Day Reverse Repo Rate
    pub async fn get_current_rate(&mut self) -> Result<Option<Decimal>> {
        let rates = self.data_source.get_current_rates().await?;
        
        Ok(rates.get(&RateType::BI7DRR).map(|rate| rate.current_rate))
    }
    
    /// Get all current BI rates
    pub async fn get_all_rates(&mut self) -> Result<HashMap<RateType, InterestRateData>> {
        info!("📊 Fetching current BI rates from Bank Indonesia");
        
        let rates = self.data_source.get_current_rates().await?;
        
        for (rate_type, rate_data) in &rates {
            debug!("Current {}: {:.2}%", rate_type, rate_data.current_rate);
            if let Some(change_bps) = rate_data.change_bps {
                debug!("  Change: {:+} bps from previous meeting", change_bps);
            }
        }
        
        // Update cache
        self.cache = rates.clone();
        
        info!("✅ Retrieved {} BI rates", rates.len());
        Ok(rates)
    }
    
    /// Get rate change impact analysis
    pub async fn analyze_rate_impact(&mut self) -> Result<RateImpactAnalysis> {
        let rates = self.get_all_rates().await?;
        let policy = self.data_source.get_latest_policy_decision().await?;
        
        let bi_rate = rates.get(&RateType::BI7DRR)
            .ok_or_else(|| anyhow!("BI 7DRR not found"))?;
        
        // Analyze impact on different sectors
        let banking_impact = self.calculate_banking_impact(bi_rate);
        let credit_impact = self.calculate_credit_impact(bi_rate);
        let currency_impact = self.calculate_currency_impact(bi_rate);
        
        Ok(RateImpactAnalysis {
            current_rate: bi_rate.current_rate,
            policy_decision: policy.decision,
            banking_sector_impact: banking_impact,
            credit_market_impact: credit_impact,
            currency_impact,
            next_meeting: bi_rate.next_meeting_date,
        })
    }
    
    /// Calculate banking sector impact
    fn calculate_banking_impact(&self, rate_data: &InterestRateData) -> SectorImpact {
        let change_bps = rate_data.change_bps.unwrap_or(0);
        
        let impact_score = match change_bps {
            bps if bps > 50 => 0.8,   // Major positive impact (rate hike)
            bps if bps > 0 => 0.6,    // Moderate positive impact
            0 => 0.0,                 // Neutral
            bps if bps > -50 => -0.4, // Moderate negative impact (rate cut)
            _ => -0.6,                // Significant negative impact
        };
        
        SectorImpact {
            sector: "Banking".to_string(),
            impact_score,
            reasoning: format!("Net Interest Margin impact from {}bps rate change", change_bps),
            affected_stocks: vec!["BMRI".to_string(), "BBRI".to_string()],
        }
    }
    
    /// Calculate credit market impact
    fn calculate_credit_impact(&self, rate_data: &InterestRateData) -> SectorImpact {
        let change_bps = rate_data.change_bps.unwrap_or(0);
        
        let impact_score = match change_bps {
            bps if bps > 0 => -0.5,   // Rate hikes reduce credit demand
            0 => 0.0,
            _ => 0.4,                 // Rate cuts stimulate credit growth
        };
        
        SectorImpact {
            sector: "Credit Market".to_string(),
            impact_score,
            reasoning: "Rate changes affect borrowing costs and credit demand".to_string(),
            affected_stocks: vec!["BMRI".to_string(), "BBRI".to_string()],
        }
    }
    
    /// Calculate currency impact
    fn calculate_currency_impact(&self, rate_data: &InterestRateData) -> SectorImpact {
        let change_bps = rate_data.change_bps.unwrap_or(0);
        
        let impact_score = match change_bps {
            bps if bps > 25 => 0.6,   // Rate hikes strengthen IDR
            bps if bps > 0 => 0.3,
            0 => 0.0,
            bps if bps > -25 => -0.3, // Rate cuts weaken IDR
            _ => -0.5,
        };
        
        SectorImpact {
            sector: "Currency (IDR)".to_string(),
            impact_score,
            reasoning: "Interest rate differentials affect capital flows".to_string(),
            affected_stocks: vec!["All export-oriented companies".to_string()],
        }
    }
    
    /// Get rate trend over period
    pub async fn get_rate_trend(&self, months: u32) -> Result<RateTrend> {
        let historical = self.data_source.get_historical_rates(RateType::BI7DRR, months).await?;
        
        if historical.len() < 2 {
            return Ok(RateTrend::Stable);
        }
        
        let first_rate = historical.first().unwrap().current_rate;
        let last_rate = historical.last().unwrap().current_rate;
        let total_change = last_rate - first_rate;
        
        let trend = if total_change > Decimal::new(50, 2) {
            RateTrend::Rising
        } else if total_change < Decimal::new(-50, 2) {
            RateTrend::Falling
        } else {
            RateTrend::Stable
        };
        
        Ok(trend)
    }
    
    /// Check if rate meeting is upcoming
    pub async fn is_meeting_upcoming(&self, days_ahead: u32) -> Result<bool> {
        let next_meeting = self.data_source.get_next_meeting_date().await?;
        
        if let Some(meeting_date) = next_meeting {
            let days_until = (meeting_date - Utc::now().date_naive()).num_days();
            Ok(days_until >= 0 && days_until <= days_ahead as i64)
        } else {
            Ok(false)
        }
    }
}

/// Rate impact analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateImpactAnalysis {
    pub current_rate: Decimal,
    pub policy_decision: PolicyDecision,
    pub banking_sector_impact: SectorImpact,
    pub credit_market_impact: SectorImpact,
    pub currency_impact: SectorImpact,
    pub next_meeting: Option<NaiveDate>,
}

/// Sector-specific rate impact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorImpact {
    pub sector: String,
    pub impact_score: f64, // -1.0 (very negative) to +1.0 (very positive)
    pub reasoning: String,
    pub affected_stocks: Vec<String>,
}

/// Rate trend direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RateTrend {
    Rising,
    Falling,
    Stable,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_bi_rate_monitor_creation() {
        let monitor = BIRateMonitor::new("https://api.bi.go.id/v2");
        assert!(monitor.endpoint_url.contains("bi.go.id"));
    }
    
    #[tokio::test]
    async fn test_mock_bi_rate_source() {
        let source = MockBIRateSource::new();
        
        let rates = source.get_current_rates().await;
        assert!(rates.is_ok());
        
        let rate_map = rates.unwrap();
        assert!(rate_map.contains_key(&RateType::BI7DRR));
        assert!(rate_map.contains_key(&RateType::DFR));
        assert!(rate_map.contains_key(&RateType::LFR));
    }
    
    #[tokio::test]
    async fn test_get_current_rate() {
        let mut monitor = BIRateMonitor::new("https://api.bi.go.id/v2");
        
        let rate = monitor.get_current_rate().await;
        assert!(rate.is_ok());
        assert!(rate.unwrap().is_some());
    }
    
    #[tokio::test]
    async fn test_rate_impact_analysis() {
        let mut monitor = BIRateMonitor::new("https://api.bi.go.id/v2");
        
        let analysis = monitor.analyze_rate_impact().await;
        assert!(analysis.is_ok());
        
        let impact = analysis.unwrap();
        assert!(impact.banking_sector_impact.impact_score >= -1.0);
        assert!(impact.banking_sector_impact.impact_score <= 1.0);
    }
    
    #[tokio::test]
    async fn test_meeting_upcoming() {
        let monitor = BIRateMonitor::new("https://api.bi.go.id/v2");
        
        let upcoming = monitor.is_meeting_upcoming(60).await; // Check next 60 days
        assert!(upcoming.is_ok());
    }
    
    #[test]
    fn test_policy_decision_display() {
        assert_eq!(PolicyDecision::Raise(25).to_string(), "Rate hike: +25 bps");
        assert_eq!(PolicyDecision::Cut(50).to_string(), "Rate cut: -50 bps");
        assert_eq!(PolicyDecision::Hold.to_string(), "Hold rates unchanged");
    }
    
    #[test]
    fn test_rate_type_display() {
        assert_eq!(RateType::BI7DRR.to_string(), "BI 7-Day Reverse Repo Rate");
        assert_eq!(RateType::DFR.to_string(), "Deposit Facility Rate");
        assert_eq!(RateType::LFR.to_string(), "Lending Facility Rate");
    }
}