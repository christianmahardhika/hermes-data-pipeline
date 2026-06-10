//! Article Cleaner
//! 
//! Phase 2: Raw XML → Parsed → Cleaned → Deduplicated

use anyhow::Result;
use chrono::{DateTime, Utc};
use feed_rs::parser;
use sha2::{Sha256, Digest};
use tracing::{info, warn};

use crate::storage::{Database, RawFeed, CleanedArticle};

/// Article cleaner - parse, strip HTML, normalize, dedup
pub struct ArticleCleaner;

impl ArticleCleaner {
    pub fn new() -> Self {
        Self
    }

    /// Process pending raw feeds
    pub async fn process_pending(&self, db: &Database, limit: i64) -> Result<CleanStats> {
        let pending = db.get_pending_raw(limit)?;
        let mut stats = CleanStats::default();

        for raw in pending {
            match self.process_raw(&raw, db) {
                Ok(count) => {
                    db.update_raw_status(raw.id.unwrap(), "processed")?;
                    stats.processed += count;
                }
                Err(e) => {
                    db.record_parse_error(
                        raw.id.unwrap(),
                        &raw.feed_name,
                        "parse_error",
                        &e.to_string(),
                    )?;
                    db.update_raw_status(raw.id.unwrap(), "failed")?;
                    stats.errors += 1;
                    warn!("⚠️ Failed to process {}: {}", raw.feed_name, e);
                }
            }
        }

        Ok(stats)
    }

    /// Process single raw feed
    fn process_raw(&self, raw: &RawFeed, db: &Database) -> Result<usize> {
        let feed = parser::parse(&raw.raw_content[..])?;
        let mut count = 0;

        for entry in feed.entries {
            // Extract fields
            let title = entry.title
                .map(|t| t.content)
                .unwrap_or_default();
            
            let content = entry.summary
                .map(|s| s.content)
                .or_else(|| entry.content.and_then(|c| c.body))
                .unwrap_or_default();

            let url = entry.links
                .first()
                .map(|l| l.href.clone())
                .unwrap_or_default();

            let published_at = entry.published
                .or(entry.updated)
                .map(|dt| dt.with_timezone(&Utc));

            // Skip if missing required fields
            if title.is_empty() || url.is_empty() {
                continue;
            }

            // Clean HTML
            let clean_title = self.strip_html(&title);
            let clean_content = self.strip_html(&content);

            // Hash for dedup
            let content_hash = self.compute_hash(&clean_title, &clean_content);

            // Check hash dedup
            if db.hash_exists(&content_hash)? {
                continue;
            }

            // Insert cleaned article
            let cleaned = CleanedArticle {
                id: None,
                raw_id: raw.id.unwrap(),
                title: clean_title,
                content: clean_content,
                published_at,
                source: raw.feed_name.clone(),
                url,
                content_hash,
                cleaned_at: Utc::now(),
            };

            db.insert_cleaned(&cleaned)?;
            count += 1;
        }

        info!("✅ Cleaned {} articles from {}", count, raw.feed_name);
        Ok(count)
    }

    /// Strip HTML tags and clean text
    fn strip_html(&self, html: &str) -> String {
        // Use ammonia to strip all HTML
        let clean = ammonia::clean(html);
        
        // Normalize whitespace
        clean
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    /// Compute SHA256 hash for deduplication
    fn compute_hash(&self, title: &str, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(title.as_bytes());
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// Cleaning statistics
#[derive(Debug, Default)]
pub struct CleanStats {
    pub processed: usize,
    pub errors: usize,
}

impl std::fmt::Display for CleanStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cleaned: {} articles, {} errors", self.processed, self.errors)
    }
}
