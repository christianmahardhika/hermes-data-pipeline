//! HackerNews collector via Algolia API.
//!
//! Provides search and front-page retrieval from HackerNews using the free
//! Algolia search API (no auth required).
//! API docs: <https://hn.algolia.com/api>

use chrono::{DateTime, Utc};
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use tracing::{info, warn};

use super::relevance::combined_relevance;
use super::SocialArticle;

const ALGOLIA_BASE: &str = "https://hn.algolia.com/api/v1";

// ---------------------------------------------------------------------------
// Algolia response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct AlgoliaResponse {
    hits: Vec<AlgoliaHit>,
}

#[derive(Debug, Deserialize)]
struct AlgoliaHit {
    #[serde(rename = "objectID")]
    object_id: String,
    title: Option<String>,
    story_title: Option<String>,
    url: Option<String>,
    story_text: Option<String>,
    comment_text: Option<String>,
    author: Option<String>,
    points: Option<i64>,
    num_comments: Option<i64>,
    created_at_i: Option<i64>,
    #[serde(rename = "_tags", default)]
    tags: Vec<String>,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Strip HTML tags from text and collapse whitespace.
fn strip_html(text: &str) -> String {
    let re = Regex::new(r"<[^>]+>").unwrap();
    let stripped = re.replace_all(text, " ");
    // Collapse multiple whitespace into single space
    let ws = Regex::new(r"\s+").unwrap();
    ws.replace_all(&stripped, " ").trim().to_string()
}

/// Truncate a string to at most `max_chars` characters (on a char boundary).
fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        s.chars().take(max_chars).collect()
    }
}

/// Parse a single Algolia hit into a SocialArticle.
/// Returns `None` if both `title` and `story_title` are missing.
fn parse_hit(hit: &AlgoliaHit, query: &str) -> Option<SocialArticle> {
    let title = hit.title.clone().or_else(|| hit.story_title.clone())?;

    let url = hit
        .url
        .clone()
        .unwrap_or_else(|| format!("https://news.ycombinator.com/item?id={}", hit.object_id));

    // Build description from story_text or comment_text, strip HTML, truncate
    let raw_desc = hit
        .story_text
        .clone()
        .or_else(|| hit.comment_text.clone())
        .unwrap_or_default();
    let description = truncate(&strip_html(&raw_desc), 500);

    // Relevance scoring
    let relevance = if query.is_empty() {
        0.0
    } else {
        combined_relevance(query, &title, &description, 0.7)
    };

    // Convert unix timestamp to date string
    let date = hit.created_at_i.and_then(|ts| {
        DateTime::from_timestamp(ts, 0).map(|dt| dt.format("%Y-%m-%d").to_string())
    });

    let created_utc = hit.created_at_i.map(|ts| ts as f64);

    Some(SocialArticle {
        id: hit.object_id.clone(),
        title,
        url,
        description,
        source: "HackerNews".to_string(),
        author: hit.author.clone().unwrap_or_default(),
        score: hit.points.unwrap_or(0),
        num_comments: hit.num_comments.unwrap_or(0),
        created_utc,
        date,
        relevance,
        collected_at: Utc::now().to_rfc3339(),
        content_type: "hackernews".to_string(),
        metadata: serde_json::json!({
            "hn_id": hit.object_id,
            "tags": hit.tags,
        }),
    })
}

/// Fetch and parse an Algolia URL, returning parsed articles.
/// On any HTTP or parse error, logs a warning and returns an empty Vec.
async fn fetch_algolia(client: &Client, url: &str, query: &str) -> Vec<SocialArticle> {
    let response = match client.get(url).send().await {
        Ok(r) => r,
        Err(e) => {
            warn!("⚠️ HackerNews HTTP error: {}", e);
            return Vec::new();
        }
    };

    if !response.status().is_success() {
        warn!(
            "⚠️ HackerNews API returned status {}",
            response.status()
        );
        return Vec::new();
    }

    let body = match response.json::<AlgoliaResponse>().await {
        Ok(b) => b,
        Err(e) => {
            warn!("⚠️ HackerNews JSON parse error: {}", e);
            return Vec::new();
        }
    };

    body.hits
        .iter()
        .filter_map(|hit| parse_hit(hit, query))
        .collect()
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Search HackerNews stories via Algolia full-text search.
///
/// Returns up to `limit` results matching `query`. Never panics — returns
/// empty Vec on error.
pub async fn search_stories(client: &Client, query: &str, limit: usize) -> Vec<SocialArticle> {
    let encoded_query = urlencoding::encode(query);
    let url = format!(
        "{}/search?query={}&hitsPerPage={}",
        ALGOLIA_BASE, encoded_query, limit
    );

    info!("📥 HackerNews search: '{}' (limit={})", query, limit);
    let results = fetch_algolia(client, &url, query).await;
    info!("📥 HackerNews search found {} results", results.len());
    results
}

/// Get current HackerNews front page stories.
///
/// Returns up to `limit` stories. Never panics — returns empty Vec on error.
pub async fn get_front_page(client: &Client, limit: usize) -> Vec<SocialArticle> {
    let url = format!(
        "{}/search?tags=front_page&hitsPerPage={}",
        ALGOLIA_BASE, limit
    );

    info!("📥 HackerNews front page (limit={})", limit);
    let results = fetch_algolia(client, &url, "").await;
    info!("📥 HackerNews front page got {} stories", results.len());
    results
}

/// Get top stories for a given time period.
///
/// `period` can be `"day"` (24h), `"week"` (7d), or `"month"` (30d).
/// Returns up to `limit` stories sorted by recency from Algolia.
/// Never panics — returns empty Vec on error.
pub async fn get_top_stories(client: &Client, period: &str, limit: usize) -> Vec<SocialArticle> {
    let seconds_ago: i64 = match period {
        "day" => 86400,
        "week" => 604800,
        "month" => 2592000,
        _ => 604800, // default to week
    };

    let cutoff = Utc::now().timestamp() - seconds_ago;
    let url = format!(
        "{}/search?tags=story&numericFilters=created_at_i>{}&hitsPerPage={}",
        ALGOLIA_BASE, cutoff, limit
    );

    info!(
        "📥 HackerNews top stories (period={}, limit={})",
        period, limit
    );
    let results = fetch_algolia(client, &url, "").await;
    info!("📥 HackerNews top stories got {} results", results.len());
    results
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_valid_hit() -> AlgoliaHit {
        AlgoliaHit {
            object_id: "12345".to_string(),
            title: Some("Rust is awesome".to_string()),
            story_title: None,
            url: Some("https://example.com/rust".to_string()),
            story_text: Some("<p>Rust offers memory safety</p>".to_string()),
            comment_text: None,
            author: Some("rustacean".to_string()),
            points: Some(150),
            num_comments: Some(42),
            created_at_i: Some(1700000000),
            tags: vec!["story".to_string(), "front_page".to_string()],
        }
    }

    #[test]
    fn test_parse_hit_valid() {
        let hit = make_valid_hit();
        let article = parse_hit(&hit, "rust").unwrap();

        assert_eq!(article.id, "12345");
        assert_eq!(article.title, "Rust is awesome");
        assert_eq!(article.url, "https://example.com/rust");
        assert_eq!(article.source, "HackerNews");
        assert_eq!(article.author, "rustacean");
        assert_eq!(article.score, 150);
        assert_eq!(article.num_comments, 42);
        assert_eq!(article.content_type, "hackernews");
        assert!(article.relevance > 0.0);
        // Description should have HTML stripped
        assert!(!article.description.contains("<p>"));
        assert!(article.description.contains("Rust offers memory safety"));
        // Date should be set
        assert!(article.date.is_some());
        // Metadata should contain hn_id
        assert_eq!(article.metadata["hn_id"], "12345");
        assert!(article.metadata["tags"].is_array());
    }

    #[test]
    fn test_parse_hit_missing_title() {
        let hit = AlgoliaHit {
            object_id: "99999".to_string(),
            title: None,
            story_title: None,
            url: None,
            story_text: None,
            comment_text: None,
            author: None,
            points: None,
            num_comments: None,
            created_at_i: None,
            tags: Vec::new(),
        };

        let result = parse_hit(&hit, "anything");
        assert!(result.is_none());
    }

    #[test]
    fn test_url_fallback() {
        let hit = AlgoliaHit {
            object_id: "67890".to_string(),
            title: Some("Ask HN: Something".to_string()),
            story_title: None,
            url: None, // No external URL
            story_text: None,
            comment_text: None,
            author: Some("someone".to_string()),
            points: Some(10),
            num_comments: Some(5),
            created_at_i: Some(1700000000),
            tags: vec!["ask_hn".to_string()],
        };

        let article = parse_hit(&hit, "").unwrap();
        assert_eq!(
            article.url,
            "https://news.ycombinator.com/item?id=67890"
        );
    }
}
