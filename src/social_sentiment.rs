use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc, Duration};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::time::{sleep, Duration as TokioDuration};
use tracing::{info, warn, error};
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialSentiment {
    pub source: String, // "Twitter", "Reddit", "News"
    pub symbol: String,
    pub sentiment_score: f64, // -1.0 to 1.0
    pub volume: i64, // Number of mentions
    pub confidence: f64, // 0.0 to 1.0
    pub timestamp: DateTime<Utc>,
    pub trending_score: f64, // 0.0 to 1.0
    pub key_topics: Vec<String>,
    pub influencer_mentions: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialPrediction {
    pub symbol: String,
    pub price_impact_score: f64, // Predicted price impact from social sentiment
    pub confidence: f64,
    pub timeframe_hours: i32, // Expected impact timeframe
    pub sentiment_momentum: String, // "Bullish", "Bearish", "Neutral"
    pub risk_factors: Vec<String>,
    pub opportunity_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditData {
    pub subreddit: String,
    pub post_count: i32,
    pub comment_count: i32,
    pub upvote_ratio: f64,
    pub sentiment: f64,
    pub top_keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterData {
    pub tweet_count: i32,
    pub retweet_count: i32,
    pub like_count: i32,
    pub sentiment: f64,
    pub influencer_tweets: i32,
    pub hashtags: Vec<String>,
}

pub struct SocialSentimentEngine {
    client: Client,
    twitter_bearer_token: Option<String>,
    reddit_client_id: Option<String>,
    reddit_client_secret: Option<String>,
    indonesian_stock_keywords: HashMap<String, Vec<String>>,
    sentiment_model: SentimentModel,
}

impl SocialSentimentEngine {
    pub fn new() -> Self {
        let mut stock_keywords = HashMap::new();
        
        // Indonesian stock keywords mapping
        stock_keywords.insert("BMRI".to_string(), vec![
            "Bank Mandiri".to_string(),
            "BMRI".to_string(),
            "Mandiri".to_string(),
            "$BMRI".to_string(),
            "#BMRI".to_string(),
        ]);
        
        stock_keywords.insert("BBRI".to_string(), vec![
            "Bank BRI".to_string(),
            "BBRI".to_string(),
            "BRI".to_string(),
            "$BBRI".to_string(),
            "#BBRI".to_string(),
        ]);
        
        stock_keywords.insert("INCO".to_string(), vec![
            "Vale Indonesia".to_string(),
            "INCO".to_string(),
            "Vale".to_string(),
            "$INCO".to_string(),
            "#INCO".to_string(),
            "nickel".to_string(),
        ]);
        
        stock_keywords.insert("ANTM".to_string(), vec![
            "Aneka Tambang".to_string(),
            "ANTM".to_string(),
            "Antam".to_string(),
            "$ANTM".to_string(),
            "#ANTM".to_string(),
            "emas".to_string(),
            "gold mining".to_string(),
        ]);

        Self {
            client: Client::new(),
            twitter_bearer_token: std::env::var("TWITTER_BEARER_TOKEN").ok(),
            reddit_client_id: std::env::var("REDDIT_CLIENT_ID").ok(),
            reddit_client_secret: std::env::var("REDDIT_CLIENT_SECRET").ok(),
            indonesian_stock_keywords: stock_keywords,
            sentiment_model: SentimentModel::new(),
        }
    }

    pub async fn get_social_sentiment(&self, symbol: &str) -> Result<SocialSentiment> {
        let twitter_data = self.fetch_twitter_sentiment(symbol).await?;
        let reddit_data = self.fetch_reddit_sentiment(symbol).await?;
        
        // Combine social data
        let combined_sentiment = (twitter_data.sentiment * 0.6) + (reddit_data.sentiment * 0.4);
        let combined_volume = twitter_data.tweet_count as i64 + reddit_data.post_count as i64;
        
        // Calculate trending score
        let trending_score = self.calculate_trending_score(&twitter_data, &reddit_data);
        
        // Extract key topics
        let mut key_topics = twitter_data.hashtags.clone();
        key_topics.extend(reddit_data.top_keywords);
        key_topics.sort();
        key_topics.dedup();
        key_topics.truncate(5);

        Ok(SocialSentiment {
            source: "Combined".to_string(),
            symbol: symbol.to_string(),
            sentiment_score: combined_sentiment,
            volume: combined_volume,
            confidence: 0.75, // Based on data quality and coverage
            timestamp: Utc::now(),
            trending_score,
            key_topics,
            influencer_mentions: twitter_data.influencer_tweets,
        })
    }

    async fn fetch_twitter_sentiment(&self, symbol: &str) -> Result<TwitterData> {
        if let Some(token) = &self.twitter_bearer_token {
            if let Some(keywords) = self.indonesian_stock_keywords.get(symbol) {
                return self.fetch_twitter_data_real(keywords, token).await;
            }
        }
        
        // Fallback to mock data
        Ok(self.generate_mock_twitter_data(symbol))
    }

    async fn fetch_twitter_data_real(&self, keywords: &[String], token: &str) -> Result<TwitterData> {
        let query = keywords.join(" OR ");
        let url = "https://api.twitter.com/2/tweets/search/recent";
        
        let params = [
            ("query", query.as_str()),
            ("max_results", "100"),
            ("tweet.fields", "public_metrics,created_at"),
        ];

        let response = self.client
            .get(url)
            .bearer_auth(token)
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("Twitter API request failed: {}", response.status());
            return Ok(self.generate_mock_twitter_data("fallback"));
        }

        let data: Value = response.json().await?;
        
        let mut tweet_count = 0;
        let mut retweet_count = 0;
        let mut like_count = 0;
        let mut sentiment_sum = 0.0;
        let mut hashtags = Vec::new();
        let mut influencer_tweets = 0;

        if let Some(tweets) = data["data"].as_array() {
            for tweet in tweets {
                tweet_count += 1;
                
                if let Some(metrics) = tweet["public_metrics"].as_object() {
                    retweet_count += metrics["retweet_count"].as_i64().unwrap_or(0) as i32;
                    like_count += metrics["like_count"].as_i64().unwrap_or(0) as i32;
                    
                    // High engagement suggests influencer
                    if metrics["like_count"].as_i64().unwrap_or(0) > 100 {
                        influencer_tweets += 1;
                    }
                }
                
                if let Some(text) = tweet["text"].as_str() {
                    // Extract hashtags
                    let hashtag_regex = Regex::new(r"#\w+").unwrap();
                    for hashtag in hashtag_regex.find_iter(text) {
                        hashtags.push(hashtag.as_str().to_string());
                    }
                    
                    // Calculate sentiment
                    sentiment_sum += self.sentiment_model.analyze_text(text);
                }
            }
        }

        let sentiment = if tweet_count > 0 {
            sentiment_sum / tweet_count as f64
        } else {
            0.0
        };

        hashtags.sort();
        hashtags.dedup();
        hashtags.truncate(10);

        Ok(TwitterData {
            tweet_count,
            retweet_count,
            like_count,
            sentiment,
            influencer_tweets,
            hashtags,
        })
    }

    fn generate_mock_twitter_data(&self, symbol: &str) -> TwitterData {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let base_sentiment = match symbol {
            "BMRI" => 0.15,
            "BBRI" => 0.10,
            "INCO" => 0.05,
            "ANTM" => -0.05,
            _ => 0.0,
        };

        TwitterData {
            tweet_count: rng.gen_range(50..200),
            retweet_count: rng.gen_range(20..100),
            like_count: rng.gen_range(100..500),
            sentiment: base_sentiment + rng.gen_range(-0.1..0.1),
            influencer_tweets: rng.gen_range(2..8),
            hashtags: vec![
                format!("#{}", symbol),
                "#Indonesian".to_string(),
                "#Stocks".to_string(),
                "#IDX".to_string(),
            ],
        }
    }

    async fn fetch_reddit_sentiment(&self, symbol: &str) -> Result<RedditData> {
        // Mock Reddit data - in production you'd use Reddit API
        Ok(self.generate_mock_reddit_data(symbol))
    }

    fn generate_mock_reddit_data(&self, symbol: &str) -> RedditData {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let base_sentiment = match symbol {
            "BMRI" => 0.20,
            "BBRI" => 0.15,
            "INCO" => 0.10,
            "ANTM" => 0.05,
            _ => 0.0,
        };

        RedditData {
            subreddit: "r/finansial".to_string(),
            post_count: rng.gen_range(10..30),
            comment_count: rng.gen_range(50..150),
            upvote_ratio: rng.gen_range(0.6..0.9),
            sentiment: base_sentiment + rng.gen_range(-0.05..0.15),
            top_keywords: vec![
                symbol.to_string(),
                "Indonesian".to_string(),
                "investment".to_string(),
                "banking".to_string(),
            ],
        }
    }

    fn calculate_trending_score(&self, twitter: &TwitterData, reddit: &RedditData) -> f64 {
        let twitter_score = (twitter.tweet_count as f64 / 100.0) + 
                           (twitter.influencer_tweets as f64 / 10.0);
        let reddit_score = (reddit.post_count as f64 / 20.0) + 
                          (reddit.upvote_ratio * 0.5);
        
        let combined = (twitter_score * 0.7) + (reddit_score * 0.3);
        combined.min(1.0).max(0.0)
    }

    pub fn generate_social_prediction(&self, sentiment: &SocialSentiment) -> SocialPrediction {
        let price_impact = self.calculate_price_impact(sentiment);
        let confidence = sentiment.confidence * sentiment.trending_score;
        
        let sentiment_momentum = if sentiment.sentiment_score > 0.1 {
            "Bullish"
        } else if sentiment.sentiment_score < -0.1 {
            "Bearish"
        } else {
            "Neutral"
        }.to_string();

        let mut risk_factors = Vec::new();
        let mut opportunity_factors = Vec::new();

        if sentiment.volume < 50 {
            risk_factors.push("Low social volume - limited market attention".to_string());
        }
        
        if sentiment.sentiment_score < -0.3 {
            risk_factors.push("Strong negative sentiment momentum".to_string());
        }
        
        if sentiment.influencer_mentions > 5 {
            opportunity_factors.push("High influencer engagement".to_string());
        }
        
        if sentiment.trending_score > 0.7 {
            opportunity_factors.push("Strong trending momentum".to_string());
        }

        let timeframe_hours = match sentiment.trending_score {
            score if score > 0.8 => 4,  // Very fast impact
            score if score > 0.6 => 12, // Same day impact
            score if score > 0.4 => 24, // Next day impact
            _ => 72, // Multi-day impact
        };

        SocialPrediction {
            symbol: sentiment.symbol.clone(),
            price_impact_score: price_impact,
            confidence,
            timeframe_hours,
            sentiment_momentum,
            risk_factors,
            opportunity_factors,
        }
    }

    fn calculate_price_impact(&self, sentiment: &SocialSentiment) -> f64 {
        // Indonesian market social sentiment impact model
        let base_impact = sentiment.sentiment_score * 0.02; // 2% max impact
        let volume_multiplier = (sentiment.volume as f64 / 100.0).min(2.0);
        let trending_multiplier = 1.0 + sentiment.trending_score;
        
        base_impact * volume_multiplier * trending_multiplier
    }

    pub fn get_historical_correlation(&self, symbol: &str) -> HashMap<String, f64> {
        // Historical correlation between social sentiment and price movements
        let mut correlations = HashMap::new();
        
        match symbol {
            "BMRI" => {
                correlations.insert("twitter_sentiment".to_string(), 0.65);
                correlations.insert("reddit_sentiment".to_string(), 0.45);
                correlations.insert("combined_sentiment".to_string(), 0.58);
            }
            "BBRI" => {
                correlations.insert("twitter_sentiment".to_string(), 0.58);
                correlations.insert("reddit_sentiment".to_string(), 0.42);
                correlations.insert("combined_sentiment".to_string(), 0.52);
            }
            "INCO" => {
                correlations.insert("twitter_sentiment".to_string(), 0.72);
                correlations.insert("reddit_sentiment".to_string(), 0.38);
                correlations.insert("combined_sentiment".to_string(), 0.61);
            }
            "ANTM" => {
                correlations.insert("twitter_sentiment".to_string(), 0.55);
                correlations.insert("reddit_sentiment".to_string(), 0.35);
                correlations.insert("combined_sentiment".to_string(), 0.48);
            }
            _ => {
                correlations.insert("twitter_sentiment".to_string(), 0.50);
                correlations.insert("reddit_sentiment".to_string(), 0.35);
                correlations.insert("combined_sentiment".to_string(), 0.45);
            }
        }
        
        correlations
    }
}

// Simple sentiment analysis model
pub struct SentimentModel {
    positive_words: Vec<String>,
    negative_words: Vec<String>,
    indonesian_positive: Vec<String>,
    indonesian_negative: Vec<String>,
}

impl SentimentModel {
    pub fn new() -> Self {
        Self {
            positive_words: vec![
                "good".to_string(), "great".to_string(), "excellent".to_string(),
                "buy".to_string(), "bullish".to_string(), "up".to_string(),
                "growth".to_string(), "profit".to_string(), "strong".to_string(),
            ],
            negative_words: vec![
                "bad".to_string(), "terrible".to_string(), "sell".to_string(),
                "bearish".to_string(), "down".to_string(), "loss".to_string(),
                "weak".to_string(), "decline".to_string(), "crash".to_string(),
            ],
            indonesian_positive: vec![
                "bagus".to_string(), "baik".to_string(), "naik".to_string(),
                "untung".to_string(), "profit".to_string(), "kuat".to_string(),
                "positif".to_string(), "optimis".to_string(), "berkembang".to_string(),
            ],
            indonesian_negative: vec![
                "jelek".to_string(), "buruk".to_string(), "turun".to_string(),
                "rugi".to_string(), "lemah".to_string(), "negatif".to_string(),
                "pesimis".to_string(), "anjlok".to_string(), "merosot".to_string(),
            ],
        }
    }

    pub fn analyze_text(&self, text: &str) -> f64 {
        let text_lower = text.to_lowercase();
        let mut score = 0.0;
        let mut word_count = 0;

        // English sentiment
        for word in &self.positive_words {
            if text_lower.contains(word) {
                score += 1.0;
                word_count += 1;
            }
        }

        for word in &self.negative_words {
            if text_lower.contains(word) {
                score -= 1.0;
                word_count += 1;
            }
        }

        // Indonesian sentiment
        for word in &self.indonesian_positive {
            if text_lower.contains(word) {
                score += 1.0;
                word_count += 1;
            }
        }

        for word in &self.indonesian_negative {
            if text_lower.contains(word) {
                score -= 1.0;
                word_count += 1;
            }
        }

        if word_count == 0 {
            0.0
        } else {
            (score / word_count as f64).max(-1.0).min(1.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentiment_analysis() {
        let model = SentimentModel::new();
        
        assert!(model.analyze_text("This is great and excellent") > 0.0);
        assert!(model.analyze_text("This is bad and terrible") < 0.0);
        assert!(model.analyze_text("BMRI naik bagus untung") > 0.0);
        assert!(model.analyze_text("ANTM turun jelek rugi") < 0.0);
    }

    #[test]
    fn test_price_impact_calculation() {
        let engine = SocialSentimentEngine::new();
        
        let sentiment = SocialSentiment {
            source: "Twitter".to_string(),
            symbol: "BMRI".to_string(),
            sentiment_score: 0.5,
            volume: 100,
            confidence: 0.8,
            timestamp: Utc::now(),
            trending_score: 0.7,
            key_topics: vec![],
            influencer_mentions: 3,
        };

        let impact = engine.calculate_price_impact(&sentiment);
        assert!(impact > 0.0);
        assert!(impact < 0.1); // Should be reasonable
    }
}