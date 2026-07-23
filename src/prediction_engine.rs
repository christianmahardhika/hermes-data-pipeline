use crate::models::*;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc, Duration};
use ndarray::{Array1, Array2, ArrayView1, ArrayView2, s};
use ndarray_stats::QuantileExt;
use statrs::statistics::{Statistics, OrderStatistics};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    LSTM,
    RandomForest,
    XGBoost,
    Prophet,
    LinearRegression,
    Ensemble,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeFrame {
    OneDay,
    OneWeek, 
    OneMonth,
    ThreeMonths,
    OneYear,
}

impl TimeFrame {
    pub fn to_days(&self) -> i64 {
        match self {
            TimeFrame::OneDay => 1,
            TimeFrame::OneWeek => 7,
            TimeFrame::OneMonth => 30,
            TimeFrame::ThreeMonths => 90,
            TimeFrame::OneYear => 365,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            TimeFrame::OneDay => "1D".to_string(),
            TimeFrame::OneWeek => "1W".to_string(),
            TimeFrame::OneMonth => "1M".to_string(),
            TimeFrame::ThreeMonths => "3M".to_string(),
            TimeFrame::OneYear => "1Y".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    pub symbol: String,
    pub model_type: ModelType,
    pub timeframe: TimeFrame,
    pub predicted_price: f64,
    pub confidence: f64,
    pub volatility: f64,
    pub var_95: f64, // 95% Value at Risk
    pub cvar_95: f64, // 95% Conditional Value at Risk
    pub probability_up: f64,
    pub probability_down: f64,
    pub support_levels: Vec<f64>,
    pub resistance_levels: Vec<f64>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketRegime {
    pub regime_type: String, // "Bull", "Bear", "Sideways", "Volatile"
    pub confidence: f64,
    pub duration_estimate: i32, // days
    pub characteristics: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityForecast {
    pub symbol: String,
    pub timeframe: TimeFrame,
    pub garch_volatility: f64,
    pub realized_volatility: f64,
    pub implied_volatility: f64,
    pub volatility_trend: String, // "Increasing", "Decreasing", "Stable"
    pub volatility_regime: String, // "Low", "Normal", "High", "Extreme"
}

pub struct AdvancedPredictionEngine {
    historical_data: HashMap<String, Vec<PricePoint>>,
    models: HashMap<String, Box<dyn PredictionModel>>,
    volatility_models: HashMap<String, VolatilityModel>,
}

#[derive(Debug, Clone)]
pub struct PricePoint {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

pub trait PredictionModel: Send + Sync {
    fn train(&mut self, data: &[PricePoint]) -> Result<()>;
    fn predict(&self, timeframe: &TimeFrame) -> Result<PredictionResult>;
    fn get_model_type(&self) -> ModelType;
    fn get_feature_importance(&self) -> HashMap<String, f64>;
}

// LSTM Neural Network Model
pub struct LSTMModel {
    symbol: String,
    weights: Option<Array2<f64>>,
    sequence_length: usize,
    features: Vec<String>,
}

impl LSTMModel {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            weights: None,
            sequence_length: 60, // 60 day lookback
            features: vec![
                "price".to_string(),
                "volume".to_string(),
                "volatility".to_string(),
                "rsi".to_string(),
                "macd".to_string(),
            ],
        }
    }

    fn extract_features(&self, data: &[PricePoint]) -> Array2<f64> {
        let n = data.len();
        let mut features = Array2::zeros((n, 5));
        
        for (i, point) in data.iter().enumerate() {
            features[[i, 0]] = point.close;
            features[[i, 1]] = point.volume.ln(); // Log volume
            
            // Calculate volatility (rolling 20-day)
            if i >= 20 {
                let prices: Vec<f64> = data[i-19..=i].iter().map(|p| p.close).collect();
                let returns: Vec<f64> = prices.windows(2)
                    .map(|w| (w[1] / w[0] - 1.0))
                    .collect();
                features[[i, 2]] = returns.iter().map(|&x| x * x).sum::<f64>() / returns.len() as f64;
            }
            
            // RSI calculation
            features[[i, 3]] = self.calculate_rsi(data, i, 14);
            
            // MACD
            features[[i, 4]] = self.calculate_macd(data, i);
        }
        
        features
    }

    fn calculate_rsi(&self, data: &[PricePoint], index: usize, period: usize) -> f64 {
        if index < period {
            return 50.0; // Neutral RSI
        }
        
        let mut gains = 0.0;
        let mut losses = 0.0;
        
        for i in (index - period + 1)..=index {
            let change = data[i].close - data[i-1].close;
            if change > 0.0 {
                gains += change;
            } else {
                losses += change.abs();
            }
        }
        
        if losses == 0.0 {
            return 100.0;
        }
        
        let rs = gains / losses;
        100.0 - (100.0 / (1.0 + rs))
    }

    fn calculate_macd(&self, data: &[PricePoint], index: usize) -> f64 {
        if index < 26 {
            return 0.0;
        }
        
        let ema_12 = self.calculate_ema(data, index, 12);
        let ema_26 = self.calculate_ema(data, index, 26);
        ema_12 - ema_26
    }

    fn calculate_ema(&self, data: &[PricePoint], index: usize, period: usize) -> f64 {
        if index < period {
            return data[index].close;
        }
        
        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema = data[index - period + 1].close;
        
        for i in (index - period + 2)..=index {
            ema = alpha * data[i].close + (1.0 - alpha) * ema;
        }
        
        ema
    }
}

impl PredictionModel for LSTMModel {
    fn train(&mut self, data: &[PricePoint]) -> Result<()> {
        if data.len() < self.sequence_length {
            return Err(anyhow!("Insufficient data for training"));
        }

        let features = self.extract_features(data);
        
        // Simplified LSTM implementation - in production you'd use candle-nn or tch
        let input_size = features.ncols();
        let hidden_size = 50;
        
        // Initialize weights randomly
        let mut rng = rand::thread_rng();
        use rand::Rng;
        
        let weights = Array2::from_shape_fn((hidden_size, input_size), |_| {
            rng.gen_range(-0.1..0.1)
        });
        
        self.weights = Some(weights);
        
        info!("LSTM model trained for {} with {} data points", self.symbol, data.len());
        Ok(())
    }

    fn predict(&self, timeframe: &TimeFrame) -> Result<PredictionResult> {
        let days_ahead = timeframe.to_days() as f64;
        
        // Simplified prediction logic
        let base_prediction = match &self.weights {
            Some(_weights) => {
                // In a real implementation, you'd run the LSTM forward pass
                // For now, using a trend-based prediction
                let trend_factor = 1.0 + (days_ahead * 0.001); // Small positive trend
                let volatility = 0.02 + (days_ahead * 0.0005); // Increasing volatility with time
                (trend_factor, volatility)
            }
            None => return Err(anyhow!("Model not trained")),
        };

        let current_price = 4500.0; // You'd get this from latest data
        let predicted_price = current_price * base_prediction.0;
        let volatility = base_prediction.1;
        
        // Calculate VaR and CVaR
        let var_95 = predicted_price * volatility * 1.645; // 95% VaR
        let cvar_95 = predicted_price * volatility * 2.063; // 95% CVaR
        
        Ok(PredictionResult {
            symbol: self.symbol.clone(),
            model_type: ModelType::LSTM,
            timeframe: timeframe.clone(),
            predicted_price,
            confidence: 0.75 - (days_ahead * 0.01), // Confidence decreases with time
            volatility,
            var_95,
            cvar_95,
            probability_up: 0.55,
            probability_down: 0.45,
            support_levels: vec![predicted_price * 0.95, predicted_price * 0.90],
            resistance_levels: vec![predicted_price * 1.05, predicted_price * 1.10],
            timestamp: Utc::now(),
        })
    }

    fn get_model_type(&self) -> ModelType {
        ModelType::LSTM
    }

    fn get_feature_importance(&self) -> HashMap<String, f64> {
        let mut importance = HashMap::new();
        importance.insert("price".to_string(), 0.35);
        importance.insert("volume".to_string(), 0.20);
        importance.insert("volatility".to_string(), 0.25);
        importance.insert("rsi".to_string(), 0.10);
        importance.insert("macd".to_string(), 0.10);
        importance
    }
}

// Random Forest Model
pub struct RandomForestModel {
    symbol: String,
    trees: Vec<DecisionTree>,
    n_trees: usize,
}

impl RandomForestModel {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            trees: Vec::new(),
            n_trees: 100,
        }
    }
}

impl PredictionModel for RandomForestModel {
    fn train(&mut self, data: &[PricePoint]) -> Result<()> {
        info!("Training Random Forest for {} with {} trees", self.symbol, self.n_trees);
        
        // Simplified Random Forest - in production use linfa-trees
        self.trees.clear();
        for _ in 0..self.n_trees {
            let mut tree = DecisionTree::new();
            tree.train(data)?;
            self.trees.push(tree);
        }
        
        Ok(())
    }

    fn predict(&self, timeframe: &TimeFrame) -> Result<PredictionResult> {
        if self.trees.is_empty() {
            return Err(anyhow!("Model not trained"));
        }

        let days_ahead = timeframe.to_days() as f64;
        
        // Aggregate predictions from all trees
        let mut predictions = Vec::new();
        for tree in &self.trees {
            predictions.push(tree.predict(days_ahead));
        }
        
        let predicted_price = predictions.iter().sum::<f64>() / predictions.len() as f64;
        let variance = predictions.iter()
            .map(|&p| (p - predicted_price).powi(2))
            .sum::<f64>() / predictions.len() as f64;
        let volatility = variance.sqrt() / predicted_price;
        
        let var_95 = predicted_price * volatility * 1.645;
        let cvar_95 = predicted_price * volatility * 2.063;
        
        Ok(PredictionResult {
            symbol: self.symbol.clone(),
            model_type: ModelType::RandomForest,
            timeframe: timeframe.clone(),
            predicted_price,
            confidence: 0.80 - (days_ahead * 0.005),
            volatility,
            var_95,
            cvar_95,
            probability_up: if predicted_price > 4500.0 { 0.60 } else { 0.40 },
            probability_down: if predicted_price > 4500.0 { 0.40 } else { 0.60 },
            support_levels: vec![predicted_price * 0.96, predicted_price * 0.92],
            resistance_levels: vec![predicted_price * 1.04, predicted_price * 1.08],
            timestamp: Utc::now(),
        })
    }

    fn get_model_type(&self) -> ModelType {
        ModelType::RandomForest
    }

    fn get_feature_importance(&self) -> HashMap<String, f64> {
        let mut importance = HashMap::new();
        importance.insert("price".to_string(), 0.30);
        importance.insert("volume".to_string(), 0.25);
        importance.insert("volatility".to_string(), 0.20);
        importance.insert("technical_indicators".to_string(), 0.25);
        importance
    }
}

// Simple Decision Tree for Random Forest
pub struct DecisionTree {
    prediction_base: f64,
}

impl DecisionTree {
    pub fn new() -> Self {
        Self {
            prediction_base: 4500.0, // Base Indonesian stock price
        }
    }

    pub fn train(&mut self, data: &[PricePoint]) -> Result<()> {
        if !data.is_empty() {
            self.prediction_base = data.last().unwrap().close;
        }
        Ok(())
    }

    pub fn predict(&self, days_ahead: f64) -> f64 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Simple prediction with random variation
        let trend = rng.gen_range(-0.02..0.02);
        let time_decay = 1.0 + (trend * days_ahead / 365.0);
        
        self.prediction_base * time_decay
    }
}

// Volatility Model for GARCH-style volatility forecasting
pub struct VolatilityModel {
    symbol: String,
    alpha: f64, // ARCH parameter
    beta: f64,  // GARCH parameter
    omega: f64, // Constant term
}

impl VolatilityModel {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            alpha: 0.1,
            beta: 0.85,
            omega: 0.00001,
        }
    }

    pub fn forecast_volatility(&self, historical_returns: &[f64], days_ahead: i64) -> VolatilityForecast {
        if historical_returns.is_empty() {
            return VolatilityForecast {
                symbol: self.symbol.clone(),
                timeframe: TimeFrame::OneDay,
                garch_volatility: 0.02,
                realized_volatility: 0.02,
                implied_volatility: 0.02,
                volatility_trend: "Stable".to_string(),
                volatility_regime: "Normal".to_string(),
            };
        }

        // GARCH(1,1) volatility forecast
        let latest_return = historical_returns.last().unwrap_or(&0.0);
        let latest_variance = historical_returns.iter()
            .map(|&r| r * r)
            .sum::<f64>() / historical_returns.len() as f64;

        let mut forecasted_variance = self.omega + self.alpha * latest_return.powi(2) + self.beta * latest_variance;
        
        // Multi-step forecast
        for _ in 1..days_ahead {
            forecasted_variance = self.omega + (self.alpha + self.beta) * forecasted_variance;
        }

        let garch_volatility = forecasted_variance.sqrt() * (252.0_f64).sqrt(); // Annualized

        // Realized volatility calculation
        let realized_volatility = latest_variance.sqrt() * (252.0_f64).sqrt();

        // Implied volatility (simplified - would come from options data)
        let implied_volatility = garch_volatility * 1.1; // Typically higher than realized

        let volatility_trend = if garch_volatility > realized_volatility * 1.05 {
            "Increasing"
        } else if garch_volatility < realized_volatility * 0.95 {
            "Decreasing" 
        } else {
            "Stable"
        }.to_string();

        let volatility_regime = match garch_volatility {
            v if v < 0.15 => "Low",
            v if v < 0.25 => "Normal",
            v if v < 0.40 => "High",
            _ => "Extreme",
        }.to_string();

        VolatilityForecast {
            symbol: self.symbol.clone(),
            timeframe: if days_ahead <= 7 { TimeFrame::OneWeek } else if days_ahead <= 30 { TimeFrame::OneMonth } else { TimeFrame::ThreeMonths },
            garch_volatility,
            realized_volatility,
            implied_volatility,
            volatility_trend,
            volatility_regime,
        }
    }
}

impl AdvancedPredictionEngine {
    pub fn new() -> Self {
        Self {
            historical_data: HashMap::new(),
            models: HashMap::new(),
            volatility_models: HashMap::new(),
        }
    }

    pub fn initialize_models(&mut self, symbols: &[String]) {
        for symbol in symbols {
            // Initialize multiple models for each symbol
            let lstm_model = Box::new(LSTMModel::new(symbol.clone()));
            let rf_model = Box::new(RandomForestModel::new(symbol.clone()));
            
            self.models.insert(format!("{}_LSTM", symbol), lstm_model);
            self.models.insert(format!("{}_RF", symbol), rf_model);
            
            // Initialize volatility model
            self.volatility_models.insert(symbol.clone(), VolatilityModel::new(symbol.clone()));
        }
        
        info!("Initialized {} models for {} symbols", self.models.len(), symbols.len());
    }

    pub fn train_all_models(&mut self, stock_data: &HashMap<String, Vec<PricePoint>>) -> Result<()> {
        for (symbol, data) in stock_data {
            if data.len() < 100 {
                warn!("Insufficient data for {}: {} points", symbol, data.len());
                continue;
            }

            // Train LSTM
            if let Some(model) = self.models.get_mut(&format!("{}_LSTM", symbol)) {
                if let Err(e) = model.train(data) {
                    error!("Failed to train LSTM for {}: {}", symbol, e);
                }
            }

            // Train Random Forest
            if let Some(model) = self.models.get_mut(&format!("{}_RF", symbol)) {
                if let Err(e) = model.train(data) {
                    error!("Failed to train Random Forest for {}: {}", symbol, e);
                }
            }
        }

        info!("Training completed for all models");
        Ok(())
    }

    pub fn get_multi_model_prediction(&self, symbol: &str, timeframe: &TimeFrame) -> Result<Vec<PredictionResult>> {
        let mut predictions = Vec::new();

        // Get predictions from all models
        if let Some(lstm_model) = self.models.get(&format!("{}_LSTM", symbol)) {
            match lstm_model.predict(timeframe) {
                Ok(prediction) => predictions.push(prediction),
                Err(e) => warn!("LSTM prediction failed for {}: {}", symbol, e),
            }
        }

        if let Some(rf_model) = self.models.get(&format!("{}_RF", symbol)) {
            match rf_model.predict(timeframe) {
                Ok(prediction) => predictions.push(prediction),
                Err(e) => warn!("Random Forest prediction failed for {}: {}", symbol, e),
            }
        }

        if predictions.is_empty() {
            return Err(anyhow!("No models available for prediction"));
        }

        Ok(predictions)
    }

    pub fn detect_market_regime(&self, symbol: &str, data: &[PricePoint]) -> MarketRegime {
        if data.len() < 30 {
            return MarketRegime {
                regime_type: "Unknown".to_string(),
                confidence: 0.0,
                duration_estimate: 0,
                characteristics: HashMap::new(),
            };
        }

        // Calculate market characteristics
        let returns: Vec<f64> = data.windows(2)
            .map(|w| (w[1].close / w[0].close - 1.0))
            .collect();

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let volatility = returns.iter()
            .map(|&r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let volatility = volatility.sqrt();

        // Trend analysis
        let price_start = data[0].close;
        let price_end = data.last().unwrap().close;
        let total_return = price_end / price_start - 1.0;

        // Regime detection logic
        let (regime_type, confidence) = if total_return > 0.15 && volatility < 0.25 {
            ("Bull".to_string(), 0.85)
        } else if total_return < -0.15 && volatility > 0.30 {
            ("Bear".to_string(), 0.80)
        } else if volatility > 0.40 {
            ("Volatile".to_string(), 0.75)
        } else {
            ("Sideways".to_string(), 0.70)
        };

        let mut characteristics = HashMap::new();
        characteristics.insert("mean_return".to_string(), mean_return);
        characteristics.insert("volatility".to_string(), volatility);
        characteristics.insert("total_return".to_string(), total_return);
        characteristics.insert("sharpe_ratio".to_string(), mean_return / volatility);

        MarketRegime {
            regime_type,
            confidence,
            duration_estimate: (30.0 * (1.0 + volatility)) as i32,
            characteristics,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeframe_conversion() {
        assert_eq!(TimeFrame::OneDay.to_days(), 1);
        assert_eq!(TimeFrame::OneMonth.to_days(), 30);
        assert_eq!(TimeFrame::OneYear.to_days(), 365);
    }

    #[test]
    fn test_lstm_model_creation() {
        let model = LSTMModel::new("BMRI".to_string());
        assert_eq!(model.symbol, "BMRI");
        assert_eq!(model.sequence_length, 60);
    }
}