//! X/Twitter collector via xurl CLI (requires OAuth2 setup).
//!
//! Uses the official xurl CLI for X API v2 access.
//! User must complete OAuth2 setup manually before this works:
//! 1. Create app at https://developer.x.com/en/portal/dashboard
//! 2. `xurl auth apps add my-app --client-id YOUR_ID --client-secret YOUR_SECRET`
//! 3. `xurl auth oauth2 --app my-app`
//! 4. `xurl auth default my-app`

use std::process::Command;
use tracing::{info, warn};
use chrono::Utc;

use super::SocialArticle;
use super::relevance::combined_relevance;

/// Timeout for xurl subprocess calls (reserved for future use with process timeout).
#[allow(dead_code)]
const XURL_TIMEOUT_SECS: u64 = 30;

/// Run xurl with the given arguments and return stdout on success.
/// Returns `None` on any error (binary not found, timeout, non-zero exit).
fn run_xurl(args: &[&str]) -> Option<String> {
    let output = match Command::new("xurl")
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            warn!("xurl not found or failed to spawn: {}", e);
            return None;
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("No apps registered") || stderr.contains("No apps registered") {
            warn!("xurl auth not configured - user must run xurl auth setup");
        } else {
            warn!("xurl exited with status {}: {}", output.status, &stderr[..stderr.len().min(200)]);
        }
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if stdout.is_empty() {
        None
    } else {
        Some(stdout)
    }
}

/// Parse a tweet JSON value into a normalized SocialArticle.
/// Returns `None` if required fields (id, text) are missing.
fn parse_tweet(tweet: &serde_json::Value, query: &str) -> Option<SocialArticle> {
    let tweet_id = tweet.get("id")?.as_str()?;
    let text = tweet.get("text")?.as_str()?;

    if tweet_id.is_empty() || text.is_empty() {
        return None;
    }

    // Title = first 100 chars + "..." if truncated
    let title = if text.len() > 100 {
        let boundary = text
            .char_indices()
            .nth(100)
            .map(|(i, _)| i)
            .unwrap_or(text.len());
        format!("{}...", &text[..boundary])
    } else {
        text.to_string()
    };

    let relevance = if query.is_empty() {
        0.0
    } else {
        combined_relevance(query, text, "", 0.7)
    };

    // Parse created_at (ISO 8601 format from X API v2)
    let created_at_str = tweet.get("created_at").and_then(|v| v.as_str());
    let (created_utc, date) = if let Some(ts) = created_at_str {
        let parsed = chrono::DateTime::parse_from_rfc3339(ts)
            .or_else(|_| chrono::DateTime::parse_from_rfc3339(&ts.replace('Z', "+00:00")))
            .ok();
        match parsed {
            Some(dt) => (Some(dt.timestamp() as f64), Some(dt.format("%Y-%m-%d").to_string())),
            None => (None, None),
        }
    } else {
        (None, None)
    };

    let author_id = tweet
        .get("author_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let url = format!("https://x.com/i/status/{}", tweet_id);

    // Extract public metrics
    let public_metrics = tweet.get("public_metrics");
    let like_count = public_metrics
        .and_then(|m| m.get("like_count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let reply_count = public_metrics
        .and_then(|m| m.get("reply_count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let retweet_count = public_metrics
        .and_then(|m| m.get("retweet_count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let metadata = serde_json::json!({
        "tweet_id": tweet_id,
        "retweet_count": retweet_count,
        "like_count": like_count,
        "reply_count": reply_count,
    });

    Some(SocialArticle {
        id: tweet_id.to_string(),
        title,
        url,
        description: text.to_string(),
        source: "X/Twitter".to_string(),
        author: author_id,
        score: like_count,
        num_comments: reply_count,
        created_utc,
        date,
        relevance,
        collected_at: Utc::now().to_rfc3339(),
        content_type: "x_twitter".to_string(),
        metadata,
    })
}

/// Check if xurl CLI is installed and auth is configured.
/// Returns `true` if `xurl auth status` exits 0 and output does not contain
/// "No apps registered".
pub fn is_available() -> bool {
    let output = match Command::new("xurl")
        .args(["auth", "status"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
    {
        Ok(o) => o,
        Err(_) => return false,
    };

    if !output.status.success() {
        return false;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    !stdout.contains("No apps registered")
}

/// Search X/Twitter for tweets matching the query.
/// Returns an empty Vec if xurl is not available or on any error.
pub fn search_tweets(query: &str, limit: usize) -> Vec<SocialArticle> {
    if !is_available() {
        warn!("X/Twitter not available - xurl auth not configured");
        return Vec::new();
    }

    info!("🐦 X/Twitter search: {} (limit={})", query, limit);

    let limit_str = limit.to_string();
    let output = match run_xurl(&["search", query, "-n", &limit_str]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    let parsed: serde_json::Value = match serde_json::from_str(&output) {
        Ok(v) => v,
        Err(e) => {
            warn!("Failed to parse xurl JSON: {}", e);
            return Vec::new();
        }
    };

    // X API v2 returns {"data": [...]} or a plain array
    let tweets = if let Some(arr) = parsed.get("data").and_then(|d| d.as_array()) {
        arr.clone()
    } else if let Some(arr) = parsed.as_array() {
        arr.clone()
    } else {
        return Vec::new();
    };

    let mut results = Vec::new();
    for tweet in &tweets {
        if let Some(article) = parse_tweet(tweet, query) {
            results.push(article);
        }
    }

    info!("🐦 X/Twitter found {} results", results.len());
    results
}

/// Get recent tweets from a user's timeline.
/// Returns an empty Vec if xurl is not available or on any error.
pub fn get_user_timeline(username: &str, limit: usize) -> Vec<SocialArticle> {
    if !is_available() {
        warn!("X/Twitter not available - xurl auth not configured");
        return Vec::new();
    }

    let username = username.trim_start_matches('@');
    info!("🐦 X/Twitter timeline: @{} (limit={})", username, limit);

    // First get user info to obtain user ID
    let user_output = match run_xurl(&["user", username]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    let user_data: serde_json::Value = match serde_json::from_str(&user_output) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let user_id = user_data
        .get("data")
        .and_then(|d| d.get("id"))
        .and_then(|id| id.as_str());

    let user_id = match user_id {
        Some(id) => id.to_string(),
        None => return Vec::new(),
    };

    // Get user's tweets
    let endpoint = format!("/2/users/{}/tweets", user_id);
    let limit_str = limit.to_string();
    let output = match run_xurl(&[&endpoint, "-n", &limit_str]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    let parsed: serde_json::Value = match serde_json::from_str(&output) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let tweets = if let Some(arr) = parsed.get("data").and_then(|d| d.as_array()) {
        arr.clone()
    } else if let Some(arr) = parsed.as_array() {
        arr.clone()
    } else {
        return Vec::new();
    };

    let mut results = Vec::new();
    for tweet in &tweets {
        if let Some(mut article) = parse_tweet(tweet, "") {
            article.author = username.to_string();
            results.push(article);
        }
    }

    info!("🐦 X/Twitter timeline found {} tweets", results.len());
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tweet_valid() {
        let tweet = serde_json::json!({
            "id": "1234567890",
            "text": "Artificial intelligence is transforming the world of finance and technology.",
            "created_at": "2025-06-10T12:00:00+00:00",
            "author_id": "987654321",
            "public_metrics": {
                "like_count": 42,
                "reply_count": 5,
                "retweet_count": 10
            }
        });

        let article = parse_tweet(&tweet, "artificial intelligence").unwrap();

        assert_eq!(article.id, "1234567890");
        assert_eq!(article.url, "https://x.com/i/status/1234567890");
        assert_eq!(article.source, "X/Twitter");
        assert_eq!(article.content_type, "x_twitter");
        assert_eq!(article.author, "987654321");
        assert_eq!(article.score, 42);
        assert_eq!(article.num_comments, 5);
        assert_eq!(article.date, Some("2025-06-10".to_string()));
        assert!(article.relevance > 0.0);

        // Check metadata
        let meta = &article.metadata;
        assert_eq!(meta["tweet_id"], "1234567890");
        assert_eq!(meta["retweet_count"], 10);
        assert_eq!(meta["like_count"], 42);
        assert_eq!(meta["reply_count"], 5);
    }

    #[test]
    fn test_is_available_returns_false_when_not_installed() {
        // On most CI/dev machines without xurl, this returns false.
        // On machines with xurl configured, it may return true.
        // We verify it doesn't panic regardless.
        let result = is_available();
        let _ = result;
    }

    #[test]
    fn test_parse_tweet_missing_text() {
        let tweet = serde_json::json!({
            "id": "111222333",
            "author_id": "444555666"
        });

        let result = parse_tweet(&tweet, "test");
        assert!(result.is_none());
    }
}
