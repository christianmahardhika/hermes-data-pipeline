//! Kiromania Health Check
//! 
//! Phase 5: Health check + reauthentication for self-healing

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Duration;
use tracing::{info, warn, error};

/// Kiromania health checker
pub struct KiroHealth {
    client: Client,
    kiro_url: String,
    api_key: String,
    login_script: String,
}

impl KiroHealth {
    /// Create new health checker
    pub fn new(kiro_url: &str, api_key: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            kiro_url: kiro_url.to_string(),
            api_key: api_key.to_string(),
            login_script: std::env::var("KIRO_LOGIN_SCRIPT")
                .unwrap_or_else(|_| "/home/ctianm/.hermes/scripts/kiro_login.py".to_string()),
        }
    }

    /// Check if Kiromania is healthy
    pub async fn check(&self) -> Result<bool> {
        // 1. Check if gateway is responding via /v1/models
        //    This also validates the API key/token
        let models_url = format!("{}/v1/models", self.kiro_url);
        match self.client
            .get(&models_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                info!("✅ Kiro gateway responding, token valid");
                return Ok(true);
            }
            Ok(resp) if resp.status().as_u16() == 401 => {
                warn!("⚠️ Kiro token expired (401 Unauthorized)");
                return Ok(false);
            }
            Ok(resp) => {
                warn!("⚠️ Kiro /v1/models returned: {}", resp.status());
                return Ok(false);
            }
            Err(e) => {
                warn!("⚠️ Kiro gateway unreachable: {}", e);
                return Ok(false);
            }
        }

    }

    /// Reauthenticate Kiromania
    pub async fn reauthenticate(&self) -> Result<()> {
        info!("🔄 Attempting Kiro reauthentication...");

        // Run login script
        let output = Command::new("python3")
            .arg(&self.login_script)
            .output()?;

        if output.status.success() {
            info!("✅ Kiro reauthentication successful");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("❌ Kiro reauthentication failed: {}", stderr);
            Err(anyhow::anyhow!("Reauthentication failed: {}", stderr))
        }
    }

    /// Send alert to Telegram
    pub async fn alert_telegram(&self, message: &str) -> Result<()> {
        // Use hermes CLI to send alert
        let output = Command::new("hermes")
            .args(["send", "-m", message, "-t", "telegram"])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                info!("📢 Telegram alert sent: {}", message);
                Ok(())
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                warn!("⚠️ Telegram alert failed: {}", stderr);
                Ok(()) // Non-fatal
            }
            Err(e) => {
                warn!("⚠️ Telegram alert error: {}", e);
                Ok(()) // Non-fatal
            }
        }
    }
}

/// Source freshness report entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaleFeedReport {
    pub source_name: String,
    pub last_article_at: String,
}

/// Get stale sources from the database and format as a report.
/// Returns sources that have not produced articles within `stale_hours`
/// but are historically active (avg_articles_per_day > 0).
pub fn get_stale_sources_report(db: &crate::storage::Database, stale_hours: i32) -> Result<Vec<StaleFeedReport>> {
    let stale = db.get_stale_sources(stale_hours)?;
    Ok(stale.into_iter().map(|(name, last_at)| StaleFeedReport {
        source_name: name,
        last_article_at: last_at,
    }).collect())
}

/// Self-healing monitor
pub struct SelfHealingMonitor {
    health: KiroHealth,
    error_threshold: i64,
}

impl SelfHealingMonitor {
    pub fn new(kiro_url: &str, api_key: &str, error_threshold: i64) -> Self {
        Self {
            health: KiroHealth::new(kiro_url, api_key),
            error_threshold,
        }
    }

    /// Check for errors and trigger self-healing
    pub async fn check_and_heal(&self, db: &crate::storage::Database) -> Result<()> {
        // Get error counts from last hour
        let error_counts = db.get_error_counts(1)?;

        for (feed_name, count) in error_counts {
            if count >= self.error_threshold {
                info!("🔧 Feed {} has {} errors, triggering self-heal", feed_name, count);

                // Check Kiro health
                if self.health.check().await? {
                    // TODO: Call LLM to generate adapter for this feed
                    info!("📝 Would generate adapter for {}", feed_name);
                } else {
                    // Try to reauthenticate
                    if let Err(e) = self.health.reauthenticate().await {
                        self.health.alert_telegram(&format!(
                            "🚨 Kiro auth failed, self-healing blocked: {}", e
                        )).await?;
                    }
                }
            }
        }

        // Check unhealthy feeds
        let unhealthy = db.get_unhealthy_feeds(10)?;
        if !unhealthy.is_empty() {
            let names: Vec<_> = unhealthy.iter().map(|f| f.feed_name.as_str()).collect();
            self.health.alert_telegram(&format!(
                "⚠️ Unhealthy feeds (10+ consecutive failures): {}", 
                names.join(", ")
            )).await?;
        }

        // Check stale sources (no articles in 48h but historically active)
        let stale = get_stale_sources_report(db, 48)?;
        if !stale.is_empty() {
            let names: Vec<_> = stale.iter().map(|s| s.source_name.as_str()).collect();
            info!("⚠️ Stale sources (48h no articles): {}", names.join(", "));
            self.health.alert_telegram(&format!(
                "⚠️ Stale sources (48h no new articles): {}",
                names.join(", ")
            )).await?;
        }

        Ok(())
    }
}
