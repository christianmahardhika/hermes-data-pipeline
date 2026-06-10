//! Reddit collector via RSS/Atom feeds.
//!
//! Reddit's .json endpoints return 403 (anti-bot), but RSS/Atom feeds still work.
//! This module fetches Atom feeds from Reddit search and subreddit listings,
//! parses entries, and returns normalized `SocialArticle` items.

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use quick_xml::Reader;
use quick_xml::events::Event;
use regex::Regex;
use reqwest::Client;
use serde_json::json;
use tracing::{info, warn};

use super::SocialArticle;
use super::relevance::combined_relevance;

const USER_AGENT: &str = "Mozilla/5.0 (compatible; SocialIntel/1.0)";
const MAX_CONCURRENT: usize = 4;

/// State machine for Atom XML parsing.
enum ParseState {
    Outside,
    InEntry,
    InTitle,
    InAuthorName,
    InContent,
    InUpdated,
}

/// Intermediate representation of a parsed Atom entry.
struct AtomEntry {
    title: String,
    url: String,
    author: String,
    content: String,
    updated: String,
    category: String,
}

/// Strip HTML tags from text.
fn strip_html(text: &str) -> String {
    let re = Regex::new(r"<[^>]+>").unwrap();
    let stripped = re.replace_all(text, " ");
    let ws_re = Regex::new(r"\s+").unwrap();
    ws_re.replace_all(stripped.trim(), " ").to_string()
}

/// Extract subreddit name from category term or URL path.
fn extract_subreddit(category: &str, url: &str) -> String {
    if !category.is_empty() {
        return category.to_string();
    }
    if let Some(after) = url.split("/r/").nth(1) {
        if let Some(sub) = after.split('/').next() {
            if !sub.is_empty() {
                return sub.to_string();
            }
        }
    }
    String::new()
}

/// Extract post ID from a Reddit comments URL (`/comments/{id}/`).
fn extract_post_id(url: &str) -> String {
    if let Some(after) = url.split("/comments/").nth(1) {
        if let Some(id) = after.split('/').next() {
            if !id.is_empty() {
                return id.to_string();
            }
        }
    }
    String::new()
}

/// Parse ISO-8601 date string to epoch seconds.
fn iso_to_epoch(s: &str) -> Option<f64> {
    s.parse::<DateTime<Utc>>()
        .ok()
        .map(|dt| dt.timestamp() as f64)
}

/// Parse ISO-8601 date string to "YYYY-MM-DD".
fn iso_to_date(s: &str) -> Option<String> {
    s.parse::<DateTime<Utc>>()
        .ok()
        .map(|dt| dt.format("%Y-%m-%d").to_string())
}

/// Parse Atom XML feed text into a list of `AtomEntry`.
fn parse_atom_feed(xml_text: &str) -> Vec<AtomEntry> {
    let mut reader = Reader::from_str(xml_text);
    reader.config_mut().trim_text(true);

    let mut entries: Vec<AtomEntry> = Vec::new();
    let mut state = ParseState::Outside;
    let mut buf = Vec::new();

    // Current entry fields
    let mut title = String::new();
    let mut url = String::new();
    let mut author = String::new();
    let mut content = String::new();
    let mut updated = String::new();
    let mut category = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"entry" => {
                        state = ParseState::InEntry;
                        title.clear();
                        url.clear();
                        author.clear();
                        content.clear();
                        updated.clear();
                        category.clear();
                    }
                    b"title" => {
                        if matches!(state, ParseState::InEntry) {
                            state = ParseState::InTitle;
                        }
                    }
                    b"name" => {
                        if matches!(state, ParseState::InEntry) {
                            state = ParseState::InAuthorName;
                        }
                    }
                    b"content" => {
                        if matches!(state, ParseState::InEntry) {
                            state = ParseState::InContent;
                        }
                    }
                    b"updated" => {
                        if matches!(state, ParseState::InEntry) {
                            state = ParseState::InUpdated;
                        }
                    }
                    b"link" => {
                        if matches!(state, ParseState::InEntry) {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"href" {
                                    if let Ok(val) = attr.unescape_value() {
                                        url = val.to_string();
                                    }
                                }
                            }
                        }
                    }
                    b"category" => {
                        if matches!(state, ParseState::InEntry) {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"term" {
                                    if let Ok(val) = attr.unescape_value() {
                                        category = val.to_string();
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"link" => {
                        if matches!(state, ParseState::InEntry) {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"href" {
                                    if let Ok(val) = attr.unescape_value() {
                                        url = val.to_string();
                                    }
                                }
                            }
                        }
                    }
                    b"category" => {
                        if matches!(state, ParseState::InEntry) {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"term" {
                                    if let Ok(val) = attr.unescape_value() {
                                        category = val.to_string();
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if let Ok(text) = e.unescape() {
                    match state {
                        ParseState::InTitle => {
                            title.push_str(&text);
                        }
                        ParseState::InAuthorName => {
                            author.push_str(&text);
                        }
                        ParseState::InContent => {
                            content.push_str(&text);
                        }
                        ParseState::InUpdated => {
                            updated.push_str(&text);
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"entry" => {
                        if !title.is_empty() && !url.is_empty() {
                            entries.push(AtomEntry {
                                title: title.clone(),
                                url: url.clone(),
                                author: author.clone(),
                                content: content.clone(),
                                updated: updated.clone(),
                                category: category.clone(),
                            });
                        }
                        state = ParseState::Outside;
                    }
                    b"title" => {
                        if matches!(state, ParseState::InTitle) {
                            state = ParseState::InEntry;
                        }
                    }
                    b"name" => {
                        if matches!(state, ParseState::InAuthorName) {
                            state = ParseState::InEntry;
                        }
                    }
                    b"content" => {
                        if matches!(state, ParseState::InContent) {
                            state = ParseState::InEntry;
                        }
                    }
                    b"updated" => {
                        if matches!(state, ParseState::InUpdated) {
                            state = ParseState::InEntry;
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    entries
}

/// Convert parsed `AtomEntry` items into `SocialArticle` items.
fn entries_to_articles(entries: Vec<AtomEntry>, query: &str) -> Vec<SocialArticle> {
    entries
        .into_iter()
        .filter_map(|entry| {
            if !entry.url.contains("/comments/") {
                return None;
            }

            let subreddit = extract_subreddit(&entry.category, &entry.url);
            let post_id = extract_post_id(&entry.url);

            let author = entry
                .author
                .trim()
                .trim_start_matches("/u/")
                .trim_start_matches("u/")
                .to_string();

            let author = if author.is_empty() || author == "[deleted]" || author == "[removed]" {
                "[deleted]".to_string()
            } else {
                author
            };

            let description = {
                let cleaned = strip_html(&entry.content);
                if cleaned.len() > 500 {
                    cleaned[..500].to_string()
                } else {
                    cleaned
                }
            };

            let relevance = if query.is_empty() {
                0.0
            } else {
                combined_relevance(query, &entry.title, &description, 0.7)
            };

            let source = if subreddit.is_empty() {
                "Reddit".to_string()
            } else {
                format!("Reddit/r/{}", subreddit)
            };

            Some(SocialArticle {
                id: post_id,
                title: entry.title,
                url: entry.url,
                description,
                source,
                author,
                score: 0,
                num_comments: 0,
                created_utc: iso_to_epoch(&entry.updated),
                date: iso_to_date(&entry.updated),
                relevance,
                collected_at: Utc::now().to_rfc3339(),
                content_type: "reddit".to_string(),
                metadata: json!({ "subreddit": subreddit }),
            })
        })
        .collect()
}

/// Build RSS feed URLs for a search query, optionally scoped to subreddits.
fn build_search_urls(query: &str, subreddits: Option<&[&str]>) -> Vec<String> {
    let encoded = urlencoding::encode(query);
    let mut urls = vec![format!(
        "https://www.reddit.com/search.rss?q={}&sort=relevance&t=month",
        encoded
    )];

    if let Some(subs) = subreddits {
        for raw_sub in subs {
            let sub = raw_sub.trim_start_matches("r/").trim();
            if sub.is_empty() {
                continue;
            }
            urls.push(format!(
                "https://www.reddit.com/r/{}/search.rss?q={}&restrict_sr=on&sort=relevance&t=month",
                sub, encoded
            ));
            urls.push(format!(
                "https://www.reddit.com/r/{}/hot.rss?t=month",
                sub
            ));
        }
    }

    urls
}

/// Fetch a single RSS feed URL and return parsed articles.
async fn fetch_feed(client: &Client, url: &str, query: &str) -> Vec<SocialArticle> {
    let response = match client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/atom+xml, text/xml, */*")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            warn!("⚠️ Reddit feed fetch failed for {}: {}", url, e);
            return Vec::new();
        }
    };

    if !response.status().is_success() {
        warn!(
            "⚠️ Reddit feed returned {} for {}",
            response.status(),
            url
        );
        return Vec::new();
    }

    let text = match response.text().await {
        Ok(t) => t,
        Err(e) => {
            warn!("⚠️ Reddit feed body read failed for {}: {}", url, e);
            return Vec::new();
        }
    };

    let entries = parse_atom_feed(&text);
    entries_to_articles(entries, query)
}

/// Search Reddit via RSS feeds. Never panics; returns empty Vec on any error.
///
/// Fetches global search and optional subreddit-scoped searches concurrently,
/// deduplicates by URL, sorts by relevance, and truncates to `limit`.
pub async fn search_reddit(
    client: &Client,
    query: &str,
    limit: usize,
    subreddits: Option<&[&str]>,
) -> Vec<SocialArticle> {
    let urls = build_search_urls(query, subreddits);

    info!(
        "🔍 Reddit: searching '{}' across {} feeds",
        query,
        urls.len()
    );

    let mut all_articles: Vec<SocialArticle> = Vec::new();
    let mut seen_urls: HashSet<String> = HashSet::new();

    // Fetch feeds concurrently using JoinSet (up to MAX_CONCURRENT per batch)
    for chunk in urls.chunks(MAX_CONCURRENT) {
        let mut join_set = tokio::task::JoinSet::new();

        for url in chunk {
            let client = client.clone();
            let url = url.clone();
            let query = query.to_string();
            join_set.spawn(async move { fetch_feed(&client, &url, &query).await });
        }

        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(articles) => {
                    for article in articles {
                        if seen_urls.insert(article.url.clone()) {
                            all_articles.push(article);
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️ Reddit feed task panicked: {}", e);
                }
            }
        }
    }

    // Sort by relevance descending
    all_articles.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    all_articles.truncate(limit);

    info!("📋 Reddit: found {} unique results", all_articles.len());
    all_articles
}

/// Get hot posts from a specific subreddit. Never panics; returns empty Vec on any error.
pub async fn get_subreddit_hot(
    client: &Client,
    subreddit: &str,
    limit: usize,
) -> Vec<SocialArticle> {
    let sub = subreddit.trim_start_matches("r/").trim();
    if sub.is_empty() {
        warn!("⚠️ Reddit: empty subreddit name");
        return Vec::new();
    }

    let url = format!("https://www.reddit.com/r/{}/hot.rss?t=month", sub);

    info!("🔥 Reddit: fetching r/{} hot posts", sub);

    let mut articles = fetch_feed(client, &url, "").await;
    articles.truncate(limit);
    articles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html() {
        let input = "<p>Hello <strong>world</strong></p>";
        let output = strip_html(input);
        assert_eq!(output, "Hello world");

        let input2 = "No tags here";
        assert_eq!(strip_html(input2), "No tags here");

        let input3 = "<a href=\"http://example.com\">link</a> and <br/>text";
        let output3 = strip_html(input3);
        assert_eq!(output3, "link and text");
    }

    #[test]
    fn test_parse_feed_valid_atom() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Reddit search results</title>
  <entry>
    <title>Rust is amazing for data pipelines</title>
    <link href="https://www.reddit.com/r/rust/comments/abc123/rust_is_amazing/"/>
    <author><name>/u/rustacean</name></author>
    <updated>2024-01-15T10:30:00+00:00</updated>
    <content type="html">&lt;p&gt;Great for async and performance&lt;/p&gt;</content>
    <category term="rust"/>
  </entry>
  <entry>
    <title>Python vs Rust performance</title>
    <link href="https://www.reddit.com/r/programming/comments/def456/python_vs_rust/"/>
    <author><name>/u/coder42</name></author>
    <updated>2024-01-14T08:00:00+00:00</updated>
    <content type="html">&lt;p&gt;Benchmark comparison&lt;/p&gt;</content>
    <category term="programming"/>
  </entry>
</feed>"#;

        let entries = parse_atom_feed(xml);
        assert_eq!(entries.len(), 2);

        assert_eq!(entries[0].title, "Rust is amazing for data pipelines");
        assert_eq!(
            entries[0].url,
            "https://www.reddit.com/r/rust/comments/abc123/rust_is_amazing/"
        );
        assert_eq!(entries[0].author, "/u/rustacean");
        assert_eq!(entries[0].category, "rust");
        assert!(entries[0].updated.contains("2024-01-15"));

        // Test conversion to articles
        let articles = entries_to_articles(entries, "rust performance");
        assert_eq!(articles.len(), 2);
        assert_eq!(articles[0].source, "Reddit/r/rust");
        assert_eq!(articles[0].id, "abc123");
        assert_eq!(articles[0].author, "rustacean");
        assert_eq!(articles[0].content_type, "reddit");
        assert!(articles[0].relevance > 0.0);
        assert!(articles[0].date.is_some());
        assert_eq!(articles[0].score, 0);
        assert_eq!(articles[0].num_comments, 0);

        assert_eq!(articles[1].source, "Reddit/r/programming");
        assert_eq!(articles[1].id, "def456");
    }

    #[test]
    fn test_build_urls_global() {
        let urls = build_search_urls("artificial intelligence", None);
        assert_eq!(urls.len(), 1);
        assert!(urls[0].contains("search.rss"));
        assert!(urls[0].contains("artificial"));
        assert!(urls[0].contains("intelligence"));
        assert!(urls[0].contains("sort=relevance"));
        assert!(urls[0].contains("t=month"));
    }

    #[test]
    fn test_build_urls_with_subreddits() {
        let subs: Vec<&str> = vec!["MachineLearning", "rust"];
        let urls = build_search_urls("AI", Some(&subs));
        // 1 global + 2 subreddit search + 2 subreddit hot = 5
        assert_eq!(urls.len(), 5);

        // Global search
        assert!(urls[0].contains("/search.rss?q=AI"));

        // Subreddit search
        assert!(urls[1].contains("/r/MachineLearning/search.rss"));
        assert!(urls[1].contains("restrict_sr=on"));

        // Subreddit hot
        assert!(urls[2].contains("/r/MachineLearning/hot.rss"));

        // Second subreddit
        assert!(urls[3].contains("/r/rust/search.rss"));
        assert!(urls[4].contains("/r/rust/hot.rss"));
    }

    #[test]
    fn test_extract_subreddit() {
        assert_eq!(
            extract_subreddit("rust", "https://www.reddit.com/r/rust/comments/abc/title/"),
            "rust"
        );
        assert_eq!(
            extract_subreddit("", "https://www.reddit.com/r/programming/comments/x/y/"),
            "programming"
        );
        assert_eq!(extract_subreddit("", "https://www.reddit.com/search"), "");
    }

    #[test]
    fn test_extract_post_id() {
        assert_eq!(
            extract_post_id("https://www.reddit.com/r/rust/comments/abc123/some_title/"),
            "abc123"
        );
        assert_eq!(
            extract_post_id("https://www.reddit.com/r/news/comments/xyz789/post/"),
            "xyz789"
        );
        assert_eq!(extract_post_id("https://www.reddit.com/r/rust/"), "");
    }

    #[test]
    fn test_iso_to_date() {
        assert_eq!(
            iso_to_date("2024-01-15T10:30:00+00:00"),
            Some("2024-01-15".to_string())
        );
        assert_eq!(iso_to_date("invalid"), None);
    }

    #[test]
    fn test_iso_to_epoch() {
        let epoch = iso_to_epoch("2024-01-15T10:30:00+00:00");
        assert!(epoch.is_some());
        assert!(epoch.unwrap() > 1700000000.0);
        assert_eq!(iso_to_epoch("not a date"), None);
    }

    #[test]
    fn test_parse_feed_empty_xml() {
        let entries = parse_atom_feed("");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_feed_malformed_xml() {
        let entries = parse_atom_feed("<feed><entry><title>No closing tags");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_dedup_by_url() {
        let mut seen: HashSet<String> = HashSet::new();
        let url = "https://www.reddit.com/r/rust/comments/abc/title/".to_string();
        assert!(seen.insert(url.clone()));
        assert!(!seen.insert(url));
    }
}
