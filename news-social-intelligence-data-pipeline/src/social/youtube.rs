//! YouTube collector via yt-dlp subprocess (no API key required).

use serde::Deserialize;
use std::process::Command;
use tracing::{info, warn};
use chrono::Utc;

use super::SocialArticle;
use super::relevance::combined_relevance;

/// Timeout for yt-dlp subprocess calls (reserved for future use with process timeout).
#[allow(dead_code)]
const YTDLP_TIMEOUT_SECS: u64 = 30;

/// Parsed yt-dlp JSON output for a single video.
#[derive(Debug, Deserialize)]
struct YtDlpOutput {
    id: Option<String>,
    title: Option<String>,
    description: Option<String>,
    webpage_url: Option<String>,
    uploader: Option<String>,
    channel: Option<String>,
    channel_id: Option<String>,
    upload_date: Option<String>, // "YYYYMMDD"
    timestamp: Option<i64>,
    view_count: Option<i64>,
    like_count: Option<i64>,
    comment_count: Option<i64>,
    duration: Option<i64>,
    categories: Option<Vec<String>>,
    tags: Option<Vec<String>>,
}

/// Run yt-dlp with the given arguments and return stdout on success.
/// Returns `None` on any error (binary not found, timeout, non-zero exit).
fn run_ytdlp(args: &[&str]) -> Option<String> {
    let mut cmd_args = vec!["--no-warnings", "-q"];
    cmd_args.extend_from_slice(args);

    let output = match Command::new("yt-dlp")
        .args(&cmd_args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            warn!("yt-dlp not found or failed to spawn: {}", e);
            return None;
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("yt-dlp exited with status {}: {}", output.status, &stderr[..stderr.len().min(200)]);
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if stdout.is_empty() {
        None
    } else {
        Some(stdout)
    }
}

/// Convert a YtDlpOutput into a normalized SocialArticle.
/// Returns `None` if required fields (id, title) are missing.
fn parse_video(data: &YtDlpOutput, query: &str) -> Option<SocialArticle> {
    let video_id = data.id.as_deref()?;
    let title = data.title.as_deref()?;

    if video_id.is_empty() || title.is_empty() {
        return None;
    }

    let description = data
        .description
        .as_deref()
        .unwrap_or("")
        .chars()
        .take(500)
        .collect::<String>();

    let relevance = if query.is_empty() {
        0.0
    } else {
        combined_relevance(query, title, &description, 0.7)
    };

    // Parse upload_date "YYYYMMDD" → "YYYY-MM-DD"
    let date = data.upload_date.as_deref().and_then(|d| {
        if d.len() == 8 {
            Some(format!("{}-{}-{}", &d[..4], &d[4..6], &d[6..8]))
        } else {
            None
        }
    });

    let url = data
        .webpage_url
        .clone()
        .unwrap_or_else(|| format!("https://www.youtube.com/watch?v={}", video_id));

    let author = data
        .uploader
        .as_deref()
        .or(data.channel.as_deref())
        .unwrap_or("")
        .to_string();

    let view_count = data.view_count.unwrap_or(0);
    let comment_count = data.comment_count.unwrap_or(0);

    let tags: Vec<String> = data
        .tags
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .take(10)
        .cloned()
        .collect();

    let metadata = serde_json::json!({
        "channel_id": data.channel_id,
        "channel": data.channel,
        "duration": data.duration,
        "view_count": data.view_count,
        "like_count": data.like_count,
        "categories": data.categories.as_deref().unwrap_or(&[]),
        "tags": tags,
    });

    Some(SocialArticle {
        id: video_id.to_string(),
        title: title.to_string(),
        url,
        description,
        source: "YouTube".to_string(),
        author,
        score: view_count,
        num_comments: comment_count,
        created_utc: data.timestamp.map(|t| t as f64),
        date,
        relevance,
        collected_at: Utc::now().to_rfc3339(),
        content_type: "youtube".to_string(),
        metadata,
    })
}

/// Search YouTube for videos matching the query.
/// Uses `yt-dlp --flat-playlist` to get metadata without downloading.
/// Returns an empty Vec on any error.
pub fn search_youtube(query: &str, limit: usize) -> Vec<SocialArticle> {
    let search_url = format!("ytsearch{}:{}", limit, query);
    info!("🎬 YouTube search: {} (limit={})", query, limit);

    let output = match run_ytdlp(&["--dump-json", "--flat-playlist", &search_url]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    let mut results = Vec::new();
    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<YtDlpOutput>(line) {
            Ok(data) => {
                if let Some(article) = parse_video(&data, query) {
                    results.push(article);
                }
            }
            Err(e) => {
                warn!("Failed to parse yt-dlp JSON line: {}", e);
            }
        }
    }

    info!("🎬 YouTube found {} results", results.len());
    results
}

/// Get metadata for a specific YouTube video URL.
/// Returns `None` on any error.
pub fn get_video_info(url: &str) -> Option<SocialArticle> {
    info!("🎬 YouTube get info: {}", url);

    let output = run_ytdlp(&["--dump-json", "--no-playlist", url])?;

    let data: YtDlpOutput = serde_json::from_str(&output).ok()?;
    parse_video(&data, "")
}

/// Get recent videos from a YouTube channel.
/// Returns an empty Vec on any error.
pub fn get_channel_videos(channel_url: &str, limit: usize) -> Vec<SocialArticle> {
    info!("🎬 YouTube channel videos: {} (limit={})", channel_url, limit);

    let url = if channel_url.ends_with("/videos") {
        channel_url.to_string()
    } else {
        format!("{}/videos", channel_url.trim_end_matches('/'))
    };

    let limit_str = limit.to_string();
    let output = match run_ytdlp(&[
        "--dump-json",
        "--flat-playlist",
        "--playlist-end",
        &limit_str,
        &url,
    ]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    let mut results = Vec::new();
    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<YtDlpOutput>(line) {
            Ok(data) => {
                if let Some(article) = parse_video(&data, "") {
                    results.push(article);
                }
            }
            Err(e) => {
                warn!("Failed to parse yt-dlp JSON line: {}", e);
            }
        }
    }

    info!("🎬 YouTube channel found {} videos", results.len());
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ytdlp_output() -> YtDlpOutput {
        YtDlpOutput {
            id: Some("dQw4w9WgXcQ".to_string()),
            title: Some("Rick Astley - Never Gonna Give You Up".to_string()),
            description: Some("The official video for Rick Astley".to_string()),
            webpage_url: Some("https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string()),
            uploader: Some("Rick Astley".to_string()),
            channel: Some("Rick Astley".to_string()),
            channel_id: Some("UCuAXFkgsw1L7xaCfnd5JJOw".to_string()),
            upload_date: Some("20091025".to_string()),
            timestamp: Some(1256428800),
            view_count: Some(1_500_000_000),
            like_count: Some(15_000_000),
            comment_count: Some(3_000_000),
            duration: Some(212),
            categories: Some(vec!["Music".to_string()]),
            tags: Some(vec![
                "rick astley".to_string(),
                "never gonna give you up".to_string(),
                "music".to_string(),
            ]),
        }
    }

    #[test]
    fn test_parse_video_valid() {
        let data = sample_ytdlp_output();
        let article = parse_video(&data, "rick astley").unwrap();

        assert_eq!(article.id, "dQw4w9WgXcQ");
        assert_eq!(article.title, "Rick Astley - Never Gonna Give You Up");
        assert_eq!(article.url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        assert_eq!(article.source, "YouTube");
        assert_eq!(article.content_type, "youtube");
        assert_eq!(article.author, "Rick Astley");
        assert_eq!(article.score, 1_500_000_000);
        assert_eq!(article.num_comments, 3_000_000);
        assert_eq!(article.date, Some("2009-10-25".to_string()));
        assert_eq!(article.created_utc, Some(1256428800.0));
        assert!(article.relevance > 0.0);

        // Check metadata
        let meta = &article.metadata;
        assert_eq!(meta["channel_id"], "UCuAXFkgsw1L7xaCfnd5JJOw");
        assert_eq!(meta["channel"], "Rick Astley");
        assert_eq!(meta["duration"], 212);
        assert_eq!(meta["view_count"], 1_500_000_000i64);
        assert_eq!(meta["like_count"], 15_000_000);
    }

    #[test]
    fn test_parse_video_missing_title() {
        let data = YtDlpOutput {
            id: Some("abc123".to_string()),
            title: None,
            description: None,
            webpage_url: None,
            uploader: None,
            channel: None,
            channel_id: None,
            upload_date: None,
            timestamp: None,
            view_count: None,
            like_count: None,
            comment_count: None,
            duration: None,
            categories: None,
            tags: None,
        };

        let result = parse_video(&data, "test");
        assert!(result.is_none());
    }

    #[test]
    fn test_upload_date_conversion() {
        let data = YtDlpOutput {
            id: Some("test123".to_string()),
            title: Some("Test Video".to_string()),
            description: None,
            webpage_url: None,
            uploader: None,
            channel: None,
            channel_id: None,
            upload_date: Some("20260610".to_string()),
            timestamp: None,
            view_count: None,
            like_count: None,
            comment_count: None,
            duration: None,
            categories: None,
            tags: None,
        };

        let article = parse_video(&data, "").unwrap();
        assert_eq!(article.date, Some("2026-06-10".to_string()));
    }

    #[test]
    fn test_run_ytdlp_not_found() {
        // run_ytdlp uses "yt-dlp" binary. If not installed, returns None.
        // On machines with yt-dlp, this will succeed but we just verify no panic.
        let result = run_ytdlp(&["--version"]);
        // Either Some (if installed) or None (if not) — both valid.
        let _ = result;
    }
}
