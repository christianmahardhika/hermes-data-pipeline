//! Kiromania Labeler
//! 
//! Phase 3: Prof Jiang Game Theory extraction via Kiro Gateway

use anyhow::Result;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tracing::{info, warn, error};

use crate::storage::{Database, CleanedArticle, LabeledArticle};
use crate::health::KiroHealth;

/// Prof Jiang BATCH extraction prompt (20 articles per request)
const PROF_JIANG_BATCH_PROMPT: &str = r#"You are a JSON-only API. Return ONLY valid JSON array with no markdown, no code blocks, no explanation.

Extract events using Prof Jiang Game Theory framework for ALL articles below.
Analyze each as chess moves on a geopolitical/economic board.

ARTICLES:
{articles}

Return a JSON ARRAY where each element has "index" matching the article index above:
[
  {
    "index": 0,
    "sentiment": "positive" | "negative" | "neutral",
    "sentiment_score": -1.0 to 1.0,
    "news_type": "politik" | "ekonomi" | "kriminal" | "tech" | "olahraga" | "hiburan" | "kesehatan" | "internasional",
    "news_subtype": "optional specific category",
    "events": [{"id": "e1", "action": "...", "trigger": "...", "tense": "past|present|future"}],
    "actors": [{"id": "a1", "name": "...", "type": "government|company|person|military|ngo", "role": "authority|perpetrator|victim|ally|opponent|neutral", "incentives": [], "constraints": []}],
    "relations": [{"event": "e1", "actor": "a1", "target": "a2", "relation": "..."}],
    "context": {"geopolitical": "...", "economic": "...", "social": "..."},
    "pattern_match": {"template": "trade_war|currency_crisis|regional_conflict|political_transition|infrastructure_crisis|corporate_scandal|null", "historical_parallel": [], "current_phase": "...", "next_likely": "...", "confidence": 0.0-1.0},
    "investment_signal": {"signal": "...", "action": "buy|sell|hold|defensive|avoid|null", "sectors": [], "confidence": 0.0-1.0}
  }
]
"#;

/// Kiromania response
#[derive(Debug, Deserialize)]
struct KiroResponse {
    choices: Vec<KiroChoice>,
}

#[derive(Debug, Deserialize)]
struct KiroChoice {
    message: KiroMessage,
}

#[derive(Debug, Deserialize)]
struct KiroMessage {
    content: String,
}

/// Prof Jiang extraction output (with index for batch matching)
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfJiangOutput {
    pub index: usize,
    pub sentiment: String,
    pub sentiment_score: f32,
    pub news_type: String,
    pub news_subtype: Option<String>,
    pub events: serde_json::Value,
    pub actors: serde_json::Value,
    pub relations: serde_json::Value,
    pub context: serde_json::Value,
    pub pattern_match: serde_json::Value,
    pub investment_signal: serde_json::Value,
}

/// Kiromania labeler
pub struct KiroLabeler {
    client: Client,
    kiro_url: String,
    api_key: String,
    model: String,
    health: KiroHealth,
    batch_size: usize,
}

impl KiroLabeler {
    /// Create new labeler
    pub fn new(kiro_url: &str, api_key: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(180)) // longer timeout for batch
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client: client.clone(),
            kiro_url: kiro_url.to_string(),
            api_key: api_key.to_string(),
            model: "claude-sonnet-4".to_string(),
            health: KiroHealth::new(kiro_url, api_key),
            batch_size: 20,
        }
    }

    /// Process pending cleaned articles in batches
    pub async fn process_pending(&self, db: &Database, limit: i64) -> Result<LabelStats> {
        // Health check first
        if !self.health.check().await? {
            warn!("⚠️ Kiro unhealthy, attempting reauth...");
            
            if let Err(e) = self.health.reauthenticate().await {
                error!("❌ Reauth failed: {}", e);
                return Err(anyhow::anyhow!("Kiro unavailable"));
            }
            
            // Retry health check
            if !self.health.check().await? {
                error!("❌ Kiro still unhealthy after reauth");
                return Err(anyhow::anyhow!("Kiro unavailable after reauth"));
            }
        }

        let pending = db.get_pending_cleaned(limit)?;
        let mut stats = LabelStats::default();

        // Process in batches
        for batch in pending.chunks(self.batch_size) {
            info!("🏷️ Processing batch of {} articles...", batch.len());
            
            match self.label_batch(batch).await {
                Ok(outputs) => {
                    // Map outputs by index
                    for output in outputs {
                        if output.index >= batch.len() {
                            warn!("⚠️ Invalid index {} in batch response", output.index);
                            continue;
                        }
                        
                        let article = &batch[output.index];
                        let labeled = LabeledArticle {
                            id: None,
                            cleaned_id: article.id.unwrap(),
                            sentiment: output.sentiment,
                            sentiment_score: output.sentiment_score,
                            news_type: output.news_type,
                            news_subtype: output.news_subtype,
                            events: output.events,
                            actors: output.actors,
                            relations: output.relations,
                            context: output.context,
                            pattern_match: output.pattern_match,
                            investment_signal: output.investment_signal,
                            labeled_at: Utc::now(),
                            labeled_by: "kiromania".to_string(),
                        };

                        db.insert_labeled(&labeled)?;
                        stats.success += 1;
                        info!("✅ Labeled: {}", article.title);
                    }
                }
                Err(e) => {
                    // Record error for all articles in failed batch
                    for article in batch {
                        db.record_parse_error(
                            article.raw_id,
                            &article.source,
                            "batch_label_error",
                            &e.to_string(),
                        )?;
                        stats.errors += 1;
                    }
                    warn!("⚠️ Batch label failed: {}", e);
                }
            }
        }

        Ok(stats)
    }

    /// Label batch of articles in single API call
    async fn label_batch(&self, articles: &[CleanedArticle]) -> Result<Vec<ProfJiangOutput>> {
        // Build articles list for prompt
        let articles_text: String = articles
            .iter()
            .enumerate()
            .map(|(i, a)| format!("[{}] Title: {}\nContent: {}\n", i, a.title, &a.content[..a.content.len().min(500)]))
            .collect::<Vec<_>>()
            .join("\n---\n");

        let prompt = PROF_JIANG_BATCH_PROMPT.replace("{articles}", &articles_text);

        let request_body = json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "response_format": {"type": "json_object"},
            "max_tokens": 16000  // larger for batch
        });

        let response = self.client
            .post(format!("{}/v1/chat/completions", self.kiro_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Kiro API error {}: {}", status, text));
        }

        let kiro_response: KiroResponse = response.json().await?;
        let content = &kiro_response.choices
            .first()
            .ok_or_else(|| anyhow::anyhow!("No response from Kiro"))?
            .message
            .content;

        // Clean up JSON - strip markdown code blocks if present
        let cleaned = content.trim();
        let cleaned = if cleaned.starts_with("```json") {
            cleaned.strip_prefix("```json").unwrap_or(cleaned)
        } else if cleaned.starts_with("```") {
            cleaned.strip_prefix("```").unwrap_or(cleaned)
        } else {
            cleaned
        };
        let cleaned = cleaned.strip_suffix("```").unwrap_or(cleaned).trim();

        // Parse JSON array output
        let outputs: Vec<ProfJiangOutput> = serde_json::from_str(cleaned)
            .map_err(|e| anyhow::anyhow!("Failed to parse Kiro batch output: {} | Raw: {}", e, &cleaned[..cleaned.len().min(500)]))?;

        Ok(outputs)
    }
}

/// Labeling statistics
#[derive(Debug, Default)]
pub struct LabelStats {
    pub success: usize,
    pub errors: usize,
}

impl std::fmt::Display for LabelStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Labeled: {} success, {} errors", self.success, self.errors)
    }
}
