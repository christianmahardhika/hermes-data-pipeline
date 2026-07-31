//! Actor domain model for Indonesian intelligence analysis

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Actor types in Indonesian political and economic landscape
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    Individual,
    Organization,
    Government,
    Corporation,
    PoliticalParty,
    Military,
    InternationalActor,
}

/// Actor model for tracking key players in Indonesian intelligence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub id: Uuid,
    pub name: String,
    pub actor_type: ActorType,
    pub description: Option<String>,
    pub sector: Option<String>, // e.g., "Banking", "Mining", "Politics"
    pub influence_score: Option<f64>, // 0.0 to 1.0
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Actor {
    /// Create a new actor
    pub fn new(name: String, actor_type: ActorType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            actor_type,
            description: None,
            sector: None,
            influence_score: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set actor description and sector
    pub fn with_details(mut self, description: Option<String>, sector: Option<String>) -> Self {
        self.description = description;
        self.sector = sector;
        self.updated_at = Utc::now();
        self
    }

    /// Set influence score (0.0 to 1.0)
    pub fn with_influence_score(mut self, score: f64) -> Self {
        self.influence_score = Some(score.max(0.0).min(1.0));
        self.updated_at = Utc::now();
        self
    }

    /// Check if actor is Indonesian government related
    pub fn is_government_related(&self) -> bool {
        matches!(
            self.actor_type,
            ActorType::Government | ActorType::PoliticalParty | ActorType::Military
        )
    }

    /// Check if actor is in financial sector (BMRI, BBRI focus)
    pub fn is_financial_sector(&self) -> bool {
        self.sector
            .as_ref()
            .map(|s| s.to_lowercase().contains("bank") || s.to_lowercase().contains("financial"))
            .unwrap_or(false)
    }

    /// Check if actor is in mining sector (INCO, ANTM focus)
    pub fn is_mining_sector(&self) -> bool {
        self.sector
            .as_ref()
            .map(|s| s.to_lowercase().contains("mining") || s.to_lowercase().contains("mineral"))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_creation() {
        let actor = Actor::new(
            "Bank Mandiri".to_string(),
            ActorType::Corporation,
        );

        assert_eq!(actor.name, "Bank Mandiri");
        assert_eq!(actor.actor_type, ActorType::Corporation);
        assert!(actor.id.to_string().len() > 0);
    }

    #[test]
    fn test_actor_with_details() {
        let actor = Actor::new(
            "BMRI".to_string(),
            ActorType::Corporation,
        )
        .with_details(
            Some("Bank Mandiri (Persero) Tbk".to_string()),
            Some("Banking".to_string()),
        );

        assert_eq!(actor.description, Some("Bank Mandiri (Persero) Tbk".to_string()));
        assert_eq!(actor.sector, Some("Banking".to_string()));
    }

    #[test]
    fn test_actor_influence_score() {
        let actor = Actor::new(
            "Indonesia Central Bank".to_string(),
            ActorType::Government,
        )
        .with_influence_score(0.95);

        assert_eq!(actor.influence_score, Some(0.95));

        // Test bounds checking
        let actor2 = Actor::new(
            "Test".to_string(),
            ActorType::Individual,
        )
        .with_influence_score(1.5); // Should be clamped to 1.0

        assert_eq!(actor2.influence_score, Some(1.0));
    }

    #[test]
    fn test_government_related_check() {
        let gov_actor = Actor::new(
            "Indonesian Parliament".to_string(),
            ActorType::Government,
        );
        assert!(gov_actor.is_government_related());

        let corp_actor = Actor::new(
            "BMRI".to_string(),
            ActorType::Corporation,
        );
        assert!(!corp_actor.is_government_related());
    }

    #[test]
    fn test_sector_checks() {
        let financial_actor = Actor::new(
            "BMRI".to_string(),
            ActorType::Corporation,
        )
        .with_details(None, Some("Banking".to_string()));

        assert!(financial_actor.is_financial_sector());
        assert!(!financial_actor.is_mining_sector());

        let mining_actor = Actor::new(
            "INCO".to_string(),
            ActorType::Corporation,
        )
        .with_details(None, Some("Mining".to_string()));

        assert!(mining_actor.is_mining_sector());
        assert!(!mining_actor.is_financial_sector());
    }
}