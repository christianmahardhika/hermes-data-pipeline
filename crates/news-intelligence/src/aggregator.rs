use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Timeframe {
    Week,      // 7 days
    Month,     // 30 days  
    Quarter,   // 3 months
    HalfYear,  // 6 months
}

impl Timeframe {
    pub fn to_days(&self) -> i64 {
        match self {
            Timeframe::Week => 7,
            Timeframe::Month => 30,
            Timeframe::Quarter => 90,
            Timeframe::HalfYear => 180,
        }
    }
    
    pub fn to_duration(&self) -> Duration {
        Duration::days(self.to_days())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub id: String,
    pub title: String,
    pub content: String,
    pub source: String,
    pub published_at: DateTime<Utc>,
    pub language: Option<String>,
    pub url: String,
    pub sentiment_score: Option<f64>,
    pub sector_tags: Vec<String>,
    pub portfolio_impact: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeframeAnalysis {
    pub timeframe: Timeframe,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_articles: usize,
    pub sentiment_summary: SentimentSummary,
    pub top_themes: Vec<String>,
    pub portfolio_correlation: HashMap<String, f64>, // Stock symbol -> correlation score
    pub professor_jiang_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentSummary {
    pub overall_sentiment: f64, // -1.0 to 1.0
    pub positive_count: usize,
    pub negative_count: usize,
    pub neutral_count: usize,
    pub indonesian_specific: f64,
    pub english_specific: f64,
}

pub struct NewsAggregator {
    pub timeframe_engine: TimeframeEngine,
    pub articles_cache: RwLock<HashMap<String, NewsArticle>>,
}

impl NewsAggregator {
    pub fn new() -> Self {
        Self {
            timeframe_engine: TimeframeEngine::new(),
            articles_cache: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn aggregate_timeframe(&self, timeframe: Timeframe) -> Result<TimeframeAnalysis> {
        let start_time = std::time::Instant::now();
        info!("Starting {:?} timeframe aggregation", timeframe);
        
        let end_date = Utc::now();
        let start_date = end_date - timeframe.to_duration();
        
        let articles = self.collect_articles_in_period(start_date, end_date).await?;
        info!("Collected {} articles for {:?} period", articles.len(), timeframe);
        
        let sentiment_summary = self.calculate_sentiment_summary(&articles).await?;
        let top_themes = self.extract_top_themes(&articles).await?;
        let portfolio_correlation = self.calculate_portfolio_correlations(&articles).await?;
        let professor_jiang_patterns = self.find_jiang_patterns(&articles).await?;
        
        let analysis = TimeframeAnalysis {
            timeframe: timeframe.clone(),
            period_start: start_date,
            period_end: end_date,
            total_articles: articles.len(),
            sentiment_summary,
            top_themes,
            portfolio_correlation,
            professor_jiang_patterns,
        };
        
        let duration = start_time.elapsed();
        info!("Completed {:?} aggregation in {:?}", timeframe, duration);
        
        // Performance requirement: <5s for 7-day, <30s for 6-month
        match timeframe {
            Timeframe::Week if duration.as_secs() > 5 => {
                warn!("7-day aggregation took {}s (target: <5s)", duration.as_secs());
            }
            Timeframe::HalfYear if duration.as_secs() > 30 => {
                warn!("6-month aggregation took {}s (target: <30s)", duration.as_secs());
            }
            _ => {}
        }
        
        Ok(analysis)
    }
    
    async fn collect_articles_in_period(
        &self, 
        start: DateTime<Utc>, 
        end: DateTime<Utc>
    ) -> Result<Vec<NewsArticle>> {
        // Query ArangoDB for articles in time period
        // Implementation depends on existing database schema
        let cache = self.articles_cache.read().await;
        let articles: Vec<NewsArticle> = cache
            .values()
            .filter(|article| article.published_at >= start && article.published_at <= end)
            .cloned()
            .collect();
        
        Ok(articles)
    }
    
    async fn calculate_sentiment_summary(&self, articles: &[NewsArticle]) -> Result<SentimentSummary> {
        let mut positive_count = 0;
        let mut negative_count = 0;
        let mut neutral_count = 0;
        let mut total_sentiment = 0.0;
        let mut indonesian_sentiment = 0.0;
        let mut english_sentiment = 0.0;
        let mut indonesian_count = 0;
        let mut english_count = 0;
        
        for article in articles {
            if let Some(sentiment) = article.sentiment_score {
                total_sentiment += sentiment;
                
                if sentiment > 0.1 {
                    positive_count += 1;
                } else if sentiment < -0.1 {
                    negative_count += 1;
                } else {
                    neutral_count += 1;
                }
                
                // Language-specific sentiment tracking
                match article.language.as_deref() {
                    Some("id") => {
                        indonesian_sentiment += sentiment;
                        indonesian_count += 1;
                    }
                    Some("en") => {
                        english_sentiment += sentiment;
                        english_count += 1;
                    }
                    _ => {}
                }
            }
        }
        
        let overall_sentiment = if articles.is_empty() {
            0.0
        } else {
            total_sentiment / articles.len() as f64
        };
        
        Ok(SentimentSummary {
            overall_sentiment,
            positive_count,
            negative_count,
            neutral_count,
            indonesian_specific: if indonesian_count > 0 {
                indonesian_sentiment / indonesian_count as f64
            } else {
                0.0
            },
            english_specific: if english_count > 0 {
                english_sentiment / english_count as f64
            } else {
                0.0
            },
        })
    }
    
    async fn extract_top_themes(&self, articles: &[NewsArticle]) -> Result<Vec<String>> {
        // Theme extraction implementation
        // For now, aggregate sector tags
        let mut theme_counts = HashMap::new();
        
        for article in articles {
            for tag in &article.sector_tags {
                *theme_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        
        let mut themes: Vec<(String, usize)> = theme_counts.into_iter().collect();
        themes.sort_by(|a, b| b.1.cmp(&a.1));
        
        Ok(themes.into_iter().take(10).map(|(theme, _)| theme).collect())
    }
    
    async fn calculate_portfolio_correlations(
        &self, 
        articles: &[NewsArticle]
    ) -> Result<HashMap<String, f64>> {
        // Portfolio correlation calculation for BMRI, BBRI, INCO, ANTM, PTBA, TAPG
        let portfolio_stocks = ["BMRI", "BBRI", "INCO", "ANTM", "PTBA", "TAPG"];
        let mut correlations = HashMap::new();
        
        for stock in &portfolio_stocks {
            let correlation = self.calculate_stock_correlation(articles, stock).await?;
            correlations.insert(stock.to_string(), correlation);
        }
        
        Ok(correlations)
    }
    
    async fn calculate_stock_correlation(
        &self, 
        articles: &[NewsArticle], 
        stock_symbol: &str
    ) -> Result<f64> {
        // Calculate correlation between news sentiment and stock impact
        let relevant_articles: Vec<&NewsArticle> = articles
            .iter()
            .filter(|article| {
                article.sector_tags.contains(&stock_symbol.to_string()) ||
                article.content.contains(stock_symbol) ||
                article.portfolio_impact.is_some()
            })
            .collect();
        
        if relevant_articles.is_empty() {
            return Ok(0.0);
        }
        
        let avg_sentiment: f64 = relevant_articles
            .iter()
            .filter_map(|article| article.sentiment_score)
            .sum::<f64>() / relevant_articles.len() as f64;
            
        let avg_impact: f64 = relevant_articles
            .iter()
            .filter_map(|article| article.portfolio_impact)
            .sum::<f64>() / relevant_articles.len() as f64;
        
        // Simple correlation approximation
        Ok(avg_sentiment * avg_impact)
    }
    
    async fn find_jiang_patterns(&self, articles: &[NewsArticle]) -> Result<Vec<String>> {
        // Prof Jiang pattern matching implementation
        // This will be enhanced in Phase 2 with actual knowledge base integration
        let mut patterns = Vec::new();
        
        for article in articles {
            if self.matches_geostrategy_pattern(&article.content).await {
                patterns.push("Geostrategy Pattern Detected".to_string());
            }
            if self.matches_game_theory_pattern(&article.content).await {
                patterns.push("Game Theory Pattern Detected".to_string());
            }
            if self.matches_secret_history_pattern(&article.content).await {
                patterns.push("Secret History Pattern Detected".to_string());
            }
        }
        
        Ok(patterns)
    }
    
    async fn matches_geostrategy_pattern(&self, content: &str) -> bool {
        // Basic pattern matching - will be enhanced with actual KB integration
        let geostrategy_keywords = [
            "geopolitik", "strategi", "diplomasi", "kekuatan", "hegemoni",
            "geopolitics", "strategy", "diplomacy", "power", "hegemony"
        ];
        
        geostrategy_keywords.iter().any(|keyword| content.to_lowercase().contains(keyword))
    }
    
    async fn matches_game_theory_pattern(&self, content: &str) -> bool {
        let game_theory_keywords = [
            "nash", "equilibrium", "strategi", "prisoner", "zero-sum",
            "strategy", "game", "payoff", "cooperation", "competition"
        ];
        
        game_theory_keywords.iter().any(|keyword| content.to_lowercase().contains(keyword))
    }
    
    async fn matches_secret_history_pattern(&self, content: &str) -> bool {
        let secret_history_keywords = [
            "rahasia", "klasifikasi", "intelijen", "operasi", "konspirasi",
            "secret", "classified", "intelligence", "operation", "conspiracy"
        ];
        
        secret_history_keywords.iter().any(|keyword| content.to_lowercase().contains(keyword))
    }
}

pub struct TimeframeEngine {
    // Performance optimization for timeframe processing
}

impl TimeframeEngine {
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn process_concurrent_timeframes(
        &self,
        aggregator: &NewsAggregator,
        timeframes: Vec<Timeframe>
    ) -> Result<HashMap<Timeframe, TimeframeAnalysis>> {
        let mut results = HashMap::new();
        
        // Process timeframes concurrently for performance
        let futures: Vec<_> = timeframes
            .iter()
            .map(|tf| aggregator.aggregate_timeframe(tf.clone()))
            .collect();
        
        let analyses = futures::future::join_all(futures).await;
        
        for (timeframe, analysis_result) in timeframes.into_iter().zip(analyses) {
            match analysis_result {
                Ok(analysis) => {
                    results.insert(timeframe, analysis);
                }
                Err(e) => {
                    error!("Failed to process {:?} timeframe: {}", timeframe, e);
                    return Err(e);
                }
            }
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[tokio::test]
    async fn test_news_aggregator_creation() {
        let aggregator = NewsAggregator::new();
        assert!(aggregator.articles_cache.read().await.is_empty());
    }
    
    #[tokio::test]
    async fn test_timeframe_duration() {
        assert_eq!(Timeframe::Week.to_days(), 7);
        assert_eq!(Timeframe::Month.to_days(), 30);
        assert_eq!(Timeframe::Quarter.to_days(), 90);
        assert_eq!(Timeframe::HalfYear.to_days(), 180);
    }
    
    #[tokio::test]
    async fn test_empty_sentiment_calculation() {
        let aggregator = NewsAggregator::new();
        let articles = vec![];
        
        let sentiment = aggregator.calculate_sentiment_summary(&articles).await.unwrap();
        assert_eq!(sentiment.overall_sentiment, 0.0);
        assert_eq!(sentiment.positive_count, 0);
        assert_eq!(sentiment.negative_count, 0);
        assert_eq!(sentiment.neutral_count, 0);
    }
}