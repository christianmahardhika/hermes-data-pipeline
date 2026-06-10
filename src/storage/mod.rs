//! SQLite storage for staging data
//! 
//! Tables: raw_feeds, cleaned, labeled, rejected, adapters, feed_health

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

/// Raw feed entry from RSS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFeed {
    pub id: Option<i64>,
    pub feed_name: String,
    pub raw_content: Vec<u8>,
    pub content_type: String,
    pub fetched_at: DateTime<Utc>,
    pub status: String,
    pub retry_count: i32,
}

/// Cleaned article ready for labeling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanedArticle {
    pub id: Option<i64>,
    pub raw_id: i64,
    pub title: String,
    pub content: String,
    pub published_at: Option<DateTime<Utc>>,
    pub source: String,
    pub url: String,
    pub content_hash: String,
    pub cleaned_at: DateTime<Utc>,
}

/// Prof Jiang labeled article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledArticle {
    pub id: Option<i64>,
    pub cleaned_id: i64,
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
    pub labeled_at: DateTime<Utc>,
    pub labeled_by: String,
}

/// Feed health tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedHealth {
    pub feed_name: String,
    pub last_success: Option<DateTime<Utc>>,
    pub consecutive_failures: i32,
    pub last_error: Option<String>,
}

/// Database connection and operations
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open database and initialize schema
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Initialize all tables
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(r#"
            -- Raw feeds (staging)
            CREATE TABLE IF NOT EXISTS raw_feeds (
                id INTEGER PRIMARY KEY,
                feed_name TEXT NOT NULL,
                raw_content BLOB,
                content_type TEXT,
                fetched_at TEXT DEFAULT CURRENT_TIMESTAMP,
                status TEXT DEFAULT 'pending',
                retry_count INTEGER DEFAULT 0
            );

            -- Cleaned articles
            CREATE TABLE IF NOT EXISTS cleaned (
                id INTEGER PRIMARY KEY,
                raw_id INTEGER REFERENCES raw_feeds(id),
                title TEXT,
                content TEXT,
                published_at TEXT,
                source TEXT,
                url TEXT UNIQUE,
                content_hash TEXT,
                cleaned_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Labeled articles (Prof Jiang model)
            CREATE TABLE IF NOT EXISTS labeled (
                id INTEGER PRIMARY KEY,
                cleaned_id INTEGER REFERENCES cleaned(id),
                sentiment TEXT,
                sentiment_score REAL,
                news_type TEXT,
                news_subtype TEXT,
                events JSON,
                actors JSON,
                relations JSON,
                context JSON,
                pattern_match JSON,
                investment_signal JSON,
                labeled_at TEXT DEFAULT CURRENT_TIMESTAMP,
                labeled_by TEXT
            );

            -- Rejected articles
            CREATE TABLE IF NOT EXISTS rejected (
                id INTEGER PRIMARY KEY,
                raw_id INTEGER,
                reason TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Parse errors for self-healing
            CREATE TABLE IF NOT EXISTS parse_errors (
                id INTEGER PRIMARY KEY,
                raw_id INTEGER,
                feed_name TEXT,
                error_type TEXT,
                error_msg TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Ingested tracking (for prune after Qdrant insert)
            CREATE TABLE IF NOT EXISTS ingested (
                id INTEGER PRIMARY KEY,
                labeled_id INTEGER UNIQUE REFERENCES labeled(id),
                qdrant_id TEXT,
                ingested_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Feed health tracking
            CREATE TABLE IF NOT EXISTS feed_health (
                feed_name TEXT PRIMARY KEY,
                last_success TEXT,
                consecutive_failures INTEGER DEFAULT 0,
                last_error TEXT
            );

            -- LLM-generated adapters for self-healing
            CREATE TABLE IF NOT EXISTS adapters (
                feed_name TEXT PRIMARY KEY,
                transform_rules JSON,
                updated_at TEXT,
                updated_by TEXT
            );

            -- Indexes
            CREATE INDEX IF NOT EXISTS idx_raw_status ON raw_feeds(status, fetched_at);
            CREATE INDEX IF NOT EXISTS idx_cleaned_hash ON cleaned(content_hash);
            CREATE INDEX IF NOT EXISTS idx_labeled_sentiment ON labeled(sentiment);
            CREATE INDEX IF NOT EXISTS idx_labeled_type ON labeled(news_type);
        "#)?;
        Ok(())
    }

    // ========== Raw Feeds ==========

    /// Insert raw feed
    pub fn insert_raw(&self, feed: &RawFeed) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO raw_feeds (feed_name, raw_content, content_type, fetched_at, status, retry_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                feed.feed_name,
                feed.raw_content,
                feed.content_type,
                feed.fetched_at.to_rfc3339(),
                feed.status,
                feed.retry_count,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get pending raw feeds
    pub fn get_pending_raw(&self, limit: i64) -> Result<Vec<RawFeed>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, feed_name, raw_content, content_type, fetched_at, status, retry_count
             FROM raw_feeds WHERE status = 'pending' ORDER BY fetched_at LIMIT ?1"
        )?;
        
        let rows = stmt.query_map([limit], |row| {
            Ok(RawFeed {
                id: Some(row.get(0)?),
                feed_name: row.get(1)?,
                raw_content: row.get(2)?,
                content_type: row.get(3)?,
                fetched_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                status: row.get(5)?,
                retry_count: row.get(6)?,
            })
        })?;
        
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Update raw feed status
    pub fn update_raw_status(&self, id: i64, status: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE raw_feeds SET status = ?1 WHERE id = ?2",
            params![status, id],
        )?;
        Ok(())
    }

    // ========== Cleaned Articles ==========

    /// Insert cleaned article
    pub fn insert_cleaned(&self, article: &CleanedArticle) -> Result<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO cleaned 
             (raw_id, title, content, published_at, source, url, content_hash, cleaned_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                article.raw_id,
                article.title,
                article.content,
                article.published_at.map(|d| d.to_rfc3339()),
                article.source,
                article.url,
                article.content_hash,
                article.cleaned_at.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Check if hash exists (dedup)
    pub fn hash_exists(&self, hash: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM cleaned WHERE content_hash = ?1",
            [hash],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Get pending cleaned articles for labeling
    pub fn get_pending_cleaned(&self, limit: i64) -> Result<Vec<CleanedArticle>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.raw_id, c.title, c.content, c.published_at, c.source, c.url, c.content_hash, c.cleaned_at
             FROM cleaned c
             LEFT JOIN labeled l ON c.id = l.cleaned_id
             WHERE l.id IS NULL
             ORDER BY c.cleaned_at
             LIMIT ?1"
        )?;
        
        let rows = stmt.query_map([limit], |row| {
            Ok(CleanedArticle {
                id: Some(row.get(0)?),
                raw_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                published_at: row.get::<_, Option<String>>(4)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                source: row.get(5)?,
                url: row.get(6)?,
                content_hash: row.get(7)?,
                cleaned_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;
        
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    // ========== Labeled Articles ==========

    /// Insert labeled article
    pub fn insert_labeled(&self, article: &LabeledArticle) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO labeled 
             (cleaned_id, sentiment, sentiment_score, news_type, news_subtype,
              events, actors, relations, context, pattern_match, investment_signal,
              labeled_at, labeled_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                article.cleaned_id,
                article.sentiment,
                article.sentiment_score,
                article.news_type,
                article.news_subtype,
                article.events.to_string(),
                article.actors.to_string(),
                article.relations.to_string(),
                article.context.to_string(),
                article.pattern_match.to_string(),
                article.investment_signal.to_string(),
                article.labeled_at.to_rfc3339(),
                article.labeled_by,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get labeled articles ready for embedding
    pub fn get_pending_embed(&self, limit: i64) -> Result<Vec<(CleanedArticle, LabeledArticle)>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.raw_id, c.title, c.content, c.published_at, c.source, c.url, c.content_hash, c.cleaned_at,
                    l.id, l.cleaned_id, l.sentiment, l.sentiment_score, l.news_type, l.news_subtype,
                    l.events, l.actors, l.relations, l.context, l.pattern_match, l.investment_signal,
                    l.labeled_at, l.labeled_by
             FROM labeled l
             JOIN cleaned c ON l.cleaned_id = c.id
             WHERE l.id NOT IN (SELECT labeled_id FROM ingested WHERE labeled_id IS NOT NULL)
             ORDER BY l.labeled_at
             LIMIT ?1"
        )?;

        let rows = stmt.query_map([limit], |row| {
            let cleaned = CleanedArticle {
                id: Some(row.get(0)?),
                raw_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                published_at: row.get::<_, Option<String>>(4)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                source: row.get(5)?,
                url: row.get(6)?,
                content_hash: row.get(7)?,
                cleaned_at: row.get::<_, String>(8)?
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .unwrap_or_else(|_| chrono::Utc::now()),
            };

            let labeled = LabeledArticle {
                id: Some(row.get(9)?),
                cleaned_id: row.get(10)?,
                sentiment: row.get(11)?,
                sentiment_score: row.get(12)?,
                news_type: row.get(13)?,
                news_subtype: row.get(14)?,
                events: serde_json::from_str(&row.get::<_, String>(15)?).unwrap_or_default(),
                actors: serde_json::from_str(&row.get::<_, String>(16)?).unwrap_or_default(),
                relations: serde_json::from_str(&row.get::<_, String>(17)?).unwrap_or_default(),
                context: serde_json::from_str(&row.get::<_, String>(18)?).unwrap_or_default(),
                pattern_match: serde_json::from_str(&row.get::<_, String>(19)?).unwrap_or_default(),
                investment_signal: serde_json::from_str(&row.get::<_, String>(20)?).unwrap_or_default(),
                labeled_at: row.get::<_, String>(21)?
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .unwrap_or_else(|_| chrono::Utc::now()),
                labeled_by: row.get(22)?,
            };

            Ok((cleaned, labeled))
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    // ========== Ingested Tracking ==========

    /// Mark labeled articles as ingested into Qdrant
    pub fn mark_ingested(&self, labeled_ids: &[i64], qdrant_id: Option<&str>) -> Result<usize> {
        let mut count = 0;
        for id in labeled_ids {
            self.conn.execute(
                "INSERT OR IGNORE INTO ingested (labeled_id, qdrant_id) VALUES (?1, ?2)",
                params![id, qdrant_id],
            )?;
            count += 1;
        }
        Ok(count)
    }

    // ========== Feed Health ==========

    /// Update feed health on success
    pub fn record_feed_success(&self, feed_name: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO feed_health (feed_name, last_success, consecutive_failures, last_error)
             VALUES (?1, CURRENT_TIMESTAMP, 0, NULL)
             ON CONFLICT(feed_name) DO UPDATE SET
                last_success = CURRENT_TIMESTAMP,
                consecutive_failures = 0,
                last_error = NULL",
            [feed_name],
        )?;
        Ok(())
    }

    /// Update feed health on failure
    pub fn record_feed_failure(&self, feed_name: &str, error: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO feed_health (feed_name, consecutive_failures, last_error)
             VALUES (?1, 1, ?2)
             ON CONFLICT(feed_name) DO UPDATE SET
                consecutive_failures = consecutive_failures + 1,
                last_error = ?2",
            params![feed_name, error],
        )?;
        Ok(())
    }

    /// Get unhealthy feeds (for alerting)
    pub fn get_unhealthy_feeds(&self, threshold: i32) -> Result<Vec<FeedHealth>> {
        let mut stmt = self.conn.prepare(
            "SELECT feed_name, last_success, consecutive_failures, last_error
             FROM feed_health WHERE consecutive_failures >= ?1"
        )?;
        
        let rows = stmt.query_map([threshold], |row| {
            Ok(FeedHealth {
                feed_name: row.get(0)?,
                last_success: row.get::<_, Option<String>>(1)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                consecutive_failures: row.get(2)?,
                last_error: row.get(3)?,
            })
        })?;
        
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    // ========== Cleanup ==========

    /// Prune ingested records
    pub fn prune_ingested(&self) -> Result<usize> {
        // Delete labeled → cleaned → raw in order
        let labeled = self.conn.execute(
            "DELETE FROM labeled WHERE cleaned_id IN (
                SELECT cleaned_id FROM labeled WHERE id IN (
                    SELECT labeled_id FROM ingested
                )
            )",
            [],
        )?;
        
        let cleaned = self.conn.execute(
            "DELETE FROM cleaned WHERE id NOT IN (SELECT cleaned_id FROM labeled)",
            [],
        )?;
        
        let raw = self.conn.execute(
            "DELETE FROM raw_feeds WHERE id NOT IN (SELECT raw_id FROM cleaned)
             AND status = 'processed'",
            [],
        )?;
        
        Ok(labeled + cleaned + raw)
    }

    // ========== Parse Errors (for self-healing) ==========

    /// Record parse error
    pub fn record_parse_error(&self, raw_id: i64, feed_name: &str, error_type: &str, error_msg: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO parse_errors (raw_id, feed_name, error_type, error_msg)
             VALUES (?1, ?2, ?3, ?4)",
            params![raw_id, feed_name, error_type, error_msg],
        )?;
        Ok(())
    }

    /// Get error counts by feed (for self-healing trigger)
    pub fn get_error_counts(&self, hours: i32) -> Result<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT feed_name, COUNT(*) as cnt
             FROM parse_errors
             WHERE created_at > datetime('now', ?1)
             GROUP BY feed_name
             HAVING cnt >= 5"
        )?;
        
        let modifier = format!("-{} hours", hours);
        let rows = stmt.query_map([modifier], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
        
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
}
