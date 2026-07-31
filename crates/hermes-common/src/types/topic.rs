//! Topic domain model for content categorization and analysis

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Topic categories for Indonesian intelligence analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopicCategory {
    Politics,
    Economics,
    Banking,
    Mining,
    Energy,
    Technology,
    Social,
    Geopolitics,
    Military,
    Trade,
    Regulation,
    Markets,
}

/// Topic sentiment classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopicSentiment {
    Positive,
    Neutral,
    Negative,
    Mixed,
}

/// Topic model for content categorization and trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub id: Uuid,
    pub name: String,
    pub category: TopicCategory,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub sentiment: Option<TopicSentiment>,
    pub relevance_score: Option<f64>, // 0.0 to 1.0
    pub article_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Topic {
    /// Create a new topic
    pub fn new(name: String, category: TopicCategory) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            category,
            description: None,
            keywords: Vec::new(),
            sentiment: None,
            relevance_score: None,
            article_count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set topic description and keywords
    pub fn with_details(mut self, description: Option<String>, keywords: Vec<String>) -> Self {
        self.description = description;
        self.keywords = keywords;
        self.updated_at = Utc::now();
        self
    }

    /// Update sentiment and relevance score
    pub fn with_analysis(mut self, sentiment: TopicSentiment, relevance_score: f64) -> Self {
        self.sentiment = Some(sentiment);
        self.relevance_score = Some(relevance_score.max(0.0).min(1.0));
        self.updated_at = Utc::now();
        self
    }

    /// Increment article count
    pub fn increment_article_count(mut self) -> Self {
        self.article_count += 1;
        self.updated_at = Utc::now();
        self
    }

    /// Check if topic is Indonesian market related
    pub fn is_indonesian_market_related(&self) -> bool {
        matches!(
            self.category,
            TopicCategory::Banking | TopicCategory::Mining | TopicCategory::Economics | TopicCategory::Markets
        ) || self.keywords.iter().any(|k| {
            let k_lower = k.to_lowercase();
            k_lower.contains("indonesia") ||
            k_lower.contains("bmri") ||
            k_lower.contains("bbri") ||
            k_lower.contains("inco") ||
            k_lower.contains("antm") ||
            k_lower.contains("idx") ||
            k_lower.contains("rupiah")
        })
    }

    /// Check if topic is geopolitically sensitive
    pub fn is_geopolitically_sensitive(&self) -> bool {
        matches!(
            self.category,
            TopicCategory::Geopolitics | TopicCategory::Military | TopicCategory::Politics
        ) || self.keywords.iter().any(|k| {
            let k_lower = k.to_lowercase();
            k_lower.contains("china") ||
            k_lower.contains("usa") ||
            k_lower.contains("trade war") ||
            k_lower.contains("sanction") ||
            k_lower.contains("conflict") ||
            k_lower.contains("election")
        })
    }

    /// Get Prof Jiang framework relevance (elite overproduction theory)
    pub fn prof_jiang_relevance(&self) -> f64 {
        let mut score: f64 = 0.0;

        // Base category relevance
        match self.category {
            TopicCategory::Politics => score += 0.8,
            TopicCategory::Geopolitics => score += 0.9,
            TopicCategory::Economics => score += 0.7,
            TopicCategory::Banking => score += 0.6,
            TopicCategory::Regulation => score += 0.8,
            _ => score += 0.3,
        }

        // Keyword-based adjustments
        for keyword in &self.keywords {
            let k_lower = keyword.to_lowercase();
            if k_lower.contains("elite") || k_lower.contains("corruption") || k_lower.contains("oligarch") {
                score += 0.3;
            }
            if k_lower.contains("overproduction") || k_lower.contains("competition") {
                score += 0.4;
            }
            if k_lower.contains("institution") || k_lower.contains("capture") {
                score += 0.2;
            }
        }

        score.min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_creation() {
        let topic = Topic::new(
            "Indonesian Banking Sector".to_string(),
            TopicCategory::Banking,
        );

        assert_eq!(topic.name, "Indonesian Banking Sector");
        assert_eq!(topic.category, TopicCategory::Banking);
        assert_eq!(topic.article_count, 0);
        assert!(topic.id.to_string().len() > 0);
    }

    #[test]
    fn test_topic_with_details() {
        let topic = Topic::new(
            "Mining Regulations".to_string(),
            TopicCategory::Mining,
        )
        .with_details(
            Some("Indonesian mining policy changes".to_string()),
            vec!["mining".to_string(), "regulation".to_string(), "indonesia".to_string()],
        );

        assert_eq!(topic.description, Some("Indonesian mining policy changes".to_string()));
        assert_eq!(topic.keywords.len(), 3);
        assert!(topic.keywords.contains(&"mining".to_string()));
    }

    #[test]
    fn test_topic_with_analysis() {
        let topic = Topic::new(
            "Market Volatility".to_string(),
            TopicCategory::Markets,
        )
        .with_analysis(TopicSentiment::Negative, 0.85);

        assert_eq!(topic.sentiment, Some(TopicSentiment::Negative));
        assert_eq!(topic.relevance_score, Some(0.85));

        // Test bounds checking
        let topic2 = Topic::new(
            "Test".to_string(),
            TopicCategory::Economics,
        )
        .with_analysis(TopicSentiment::Positive, 1.5); // Should be clamped to 1.0

        assert_eq!(topic2.relevance_score, Some(1.0));
    }

    #[test]
    fn test_article_count_increment() {
        let topic = Topic::new(
            "Test Topic".to_string(),
            TopicCategory::Politics,
        )
        .increment_article_count()
        .increment_article_count();

        assert_eq!(topic.article_count, 2);
    }

    #[test]
    fn test_indonesian_market_related() {
        let banking_topic = Topic::new(
            "Banking Analysis".to_string(),
            TopicCategory::Banking,
        );
        assert!(banking_topic.is_indonesian_market_related());

        let bmri_topic = Topic::new(
            "General News".to_string(),
            TopicCategory::Social,
        )
        .with_details(None, vec!["BMRI".to_string(), "earnings".to_string()]);
        assert!(bmri_topic.is_indonesian_market_related());

        let unrelated_topic = Topic::new(
            "Global Tech".to_string(),
            TopicCategory::Technology,
        );
        assert!(!unrelated_topic.is_indonesian_market_related());
    }

    #[test]
    fn test_geopolitically_sensitive() {
        let geopolitics_topic = Topic::new(
            "US-China Relations".to_string(),
            TopicCategory::Geopolitics,
        );
        assert!(geopolitics_topic.is_geopolitically_sensitive());

        let trade_war_topic = Topic::new(
            "Economic News".to_string(),
            TopicCategory::Economics,
        )
        .with_details(None, vec!["trade war".to_string(), "tariffs".to_string()]);
        assert!(trade_war_topic.is_geopolitically_sensitive());
    }

    #[test]
    fn test_prof_jiang_relevance() {
        let political_topic = Topic::new(
            "Elite Competition".to_string(),
            TopicCategory::Politics,
        )
        .with_details(None, vec!["elite".to_string(), "competition".to_string()]);

        let relevance = political_topic.prof_jiang_relevance();
        assert!(relevance > 0.8); // Should be high due to category + keywords

        let tech_topic = Topic::new(
            "Software Development".to_string(),
            TopicCategory::Technology,
        );

        let tech_relevance = tech_topic.prof_jiang_relevance();
        assert!(tech_relevance < 0.5); // Should be lower for non-political topics
    }
}