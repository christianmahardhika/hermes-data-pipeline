//! Unlimited News Collector (Rust rewrite)
//! 
//! Replaces Python unlimited_indonesian_collector.py
//! Uses TEI HTTP endpoint (768-dim) instead of local SentenceTransformer (384-dim)
//! Memory: ~20MB vs Python's ~500MB

use anyhow::Result;
use chrono::{DateTime, Utc};
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, VectorParamsBuilder,
    PointStruct, SearchPointsBuilder, UpsertPointsBuilder,
};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Sha256, Digest};
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};

/// RSS feed source
struct RssFeed {
    name: &'static str,
    url: &'static str,
    category: FeedCategory,
}

#[derive(Clone, Copy)]
enum FeedCategory {
    Indonesian,
    International,
}

/// Indonesian RSS feeds
const INDONESIAN_FEEDS: &[RssFeed] = &[
    RssFeed { name: "Tempo", url: "https://rss.tempo.co/", category: FeedCategory::Indonesian },
    RssFeed { name: "CNN Indonesia", url: "https://www.cnnindonesia.com/rss", category: FeedCategory::Indonesian },
    RssFeed { name: "Antara News", url: "https://www.antaranews.com/rss/terkini.xml", category: FeedCategory::Indonesian },
    RssFeed { name: "Republika", url: "https://www.republika.co.id/rss", category: FeedCategory::Indonesian },
    RssFeed { name: "Merdeka", url: "https://www.merdeka.com/feed/", category: FeedCategory::Indonesian },
    RssFeed { name: "Tribunnews", url: "https://www.tribunnews.com/rss", category: FeedCategory::Indonesian },
    RssFeed { name: "Jpnn", url: "https://www.jpnn.com/rss/news", category: FeedCategory::Indonesian },
];

/// International RSS feeds
const INTERNATIONAL_FEEDS: &[RssFeed] = &[
    RssFeed { name: "BBC Business", url: "http://feeds.bbci.co.uk/news/business/rss.xml", category: FeedCategory::International },
    RssFeed { name: "BBC World", url: "http://feeds.bbci.co.uk/news/world/rss.xml", category: FeedCategory::International },
    RssFeed { name: "Reuters", url: "https://feeds.reuters.com/reuters/businessNews", category: FeedCategory::International },
    RssFeed { name: "Google News Business", url: "https://news.google.com/rss/topics/CAAqJggKIiBDQkFTRWdvSUwyMHZNRGx6TVdZU0FtVnVHZ0pWVXlnQVAB", category: FeedCategory::International },
];

/// TEI embedding response
#[derive(Debug, Deserialize)]
struct TeiResponse(Vec<Vec<f32>>);

/// Collected article
#[derive(Debug, Clone)]
pub struct CollectedArticle {
    pub title: String,
    pub description: String,
    pub url: String,
    pub source: String,
    pub published_date: Option<String>,
    pub collected_at: DateTime<Utc>,
    pub url_hash: String,
    pub category: String,
}

/// Unlimited collector statistics
#[derive(Debug, Default)]
pub struct UnlimitedStats {
    pub indonesian_total: usize,
    pub indonesian_stored: usize,
    pub international_total: usize,
    pub international_stored: usize,
    pub duplicates_skipped: usize,
    pub near_duplicates: usize,
    pub errors: usize,
}

/// Unlimited News Collector
pub struct UnlimitedCollector {
    http_client: Client,
    qdrant_client: Qdrant,
    tei_url: String,
    indonesian_collection: String,
    international_collection: String,
    similarity_threshold: f32,
    processed_hashes: HashSet<String>,
    stats: UnlimitedStats,
    start_time: DateTime<Utc>,
}

impl UnlimitedCollector {
    /// Create new unlimited collector
    pub async fn new(
        tei_url: &str,
        qdrant_url: &str,
        similarity_threshold: f32,
    ) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; HermesCollector/2.0)")
            .build()?;

        let qdrant_client = Qdrant::from_url(qdrant_url).build()?;

        let mut collector = Self {
            http_client,
            qdrant_client,
            tei_url: tei_url.to_string(),
            indonesian_collection: "indonesian_news_768".to_string(),
            international_collection: "international_news_768".to_string(),
            similarity_threshold,
            processed_hashes: HashSet::new(),
            stats: UnlimitedStats::default(),
            start_time: Utc::now(),
        };

        // Ensure collections exist with 768-dim
        collector.ensure_collections().await?;

        Ok(collector)
    }

    /// Ensure Qdrant collections exist with 768-dim vectors
    async fn ensure_collections(&self) -> Result<()> {
        for collection in [&self.indonesian_collection, &self.international_collection] {
            let exists = self.qdrant_client
                .collection_exists(collection)
                .await?;

            if !exists {
                info!("📦 Creating collection: {} (768-dim)", collection);
                
                self.qdrant_client
                    .create_collection(
                        CreateCollectionBuilder::new(collection)
                            .vectors_config(VectorParamsBuilder::new(768, Distance::Cosine))
                    )
                    .await?;
                
                info!("✅ Collection created: {}", collection);
            } else {
                info!("✅ Collection exists: {}", collection);
            }
        }

        Ok(())
    }

    /// Generate embedding via TEI HTTP endpoint
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let response = self.http_client
            .post(format!("{}/embed", self.tei_url))
            .json(&serde_json::json!({
                "inputs": [text]
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("TEI error: {}", response.status()));
        }

        let tei_response: TeiResponse = response.json().await?;
        tei_response.0.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("Empty TEI response"))
    }

    /// Check for near-duplicates using vector similarity
    async fn is_near_duplicate(&self, embedding: &[f32], collection: &str) -> Result<bool> {
        let results = self.qdrant_client
            .search_points(
                SearchPointsBuilder::new(collection, embedding.to_vec(), 1)
                    .score_threshold(self.similarity_threshold)
            )
            .await?;

        Ok(!results.result.is_empty())
    }

    /// Fetch RSS feed
    async fn fetch_feed(&self, feed: &RssFeed) -> Vec<CollectedArticle> {
        let mut articles = Vec::new();

        match self.http_client.get(feed.url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    warn!("⚠️ {}: HTTP {}", feed.name, response.status());
                    return articles;
                }

                match response.text().await {
                    Ok(body) => {
                        // Parse RSS/Atom feed
                        if let Ok(channel) = rss::Channel::read_from(body.as_bytes()) {
                            for item in channel.items() {
                                let title = item.title().unwrap_or("").to_string();
                                let link = item.link().unwrap_or("").to_string();
                                let description = item.description().unwrap_or("").to_string();
                                let pub_date = item.pub_date().map(|s| s.to_string());

                                if link.is_empty() || title.len() < 10 {
                                    continue;
                                }

                                // Generate URL hash
                                let mut hasher = Sha256::new();
                                hasher.update(link.as_bytes());
                                let url_hash = format!("{:x}", hasher.finalize())[..16].to_string();

                                // Skip if already processed
                                if self.processed_hashes.contains(&url_hash) {
                                    continue;
                                }

                                // Clean HTML tags
                                let clean_title = html_escape::decode_html_entities(&title)
                                    .to_string()
                                    .replace(|c: char| c == '<' || c == '>', "");
                                let clean_desc = html_escape::decode_html_entities(&description)
                                    .to_string()
                                    .replace(|c: char| c == '<' || c == '>', "");

                                let category = match feed.category {
                                    FeedCategory::Indonesian => "indonesian",
                                    FeedCategory::International => "international",
                                };

                                articles.push(CollectedArticle {
                                    title: clean_title,
                                    description: clean_desc,
                                    url: link,
                                    source: feed.name.to_string(),
                                    published_date: pub_date,
                                    collected_at: Utc::now(),
                                    url_hash,
                                    category: category.to_string(),
                                });
                            }
                        }
                    }
                    Err(e) => warn!("⚠️ {}: Parse error: {}", feed.name, e),
                }
            }
            Err(e) => warn!("⚠️ {}: {}", feed.name, e),
        }

        if !articles.is_empty() {
            info!("📰 {}: {} new articles", feed.name, articles.len());
        }

        articles
    }

    /// Store article in Qdrant with TEI embedding
    async fn store_article(&mut self, article: &CollectedArticle, collection: &str) -> Result<bool> {
        // Combine title + description for embedding
        let text = format!("{} {}", article.title, article.description);
        
        // Generate embedding via TEI
        let embedding = self.embed(&text).await?;

        // Check near-duplicate
        if self.is_near_duplicate(&embedding, collection).await? {
            self.stats.near_duplicates += 1;
            return Ok(false);
        }

        // Build payload
        use qdrant_client::qdrant::Value;
        use std::collections::HashMap;

        let str_val = |s: String| Value {
            kind: Some(qdrant_client::qdrant::value::Kind::StringValue(s)),
        };

        let mut payload: HashMap<String, Value> = HashMap::new();
        payload.insert("title".to_string(), str_val(article.title.clone()));
        payload.insert("description".to_string(), str_val(truncate_safe(&article.description, 500).to_string()));
        payload.insert("url".to_string(), str_val(article.url.clone()));
        payload.insert("source".to_string(), str_val(article.source.clone()));
        payload.insert("url_hash".to_string(), str_val(article.url_hash.clone()));
        payload.insert("category".to_string(), str_val(article.category.clone()));
        payload.insert("collected_at".to_string(), str_val(article.collected_at.to_rfc3339()));
        
        if let Some(ref pub_date) = article.published_date {
            payload.insert("published_date".to_string(), str_val(pub_date.clone()));
        }

        // Generate UUID from hash
        let point_id = format!("{:0>32}", article.url_hash);
        
        let point = PointStruct::new(
            point_id,
            embedding,
            payload,
        );

        self.qdrant_client
            .upsert_points(UpsertPointsBuilder::new(collection, vec![point]))
            .await?;

        // Mark as processed
        self.processed_hashes.insert(article.url_hash.clone());

        Ok(true)
    }

    /// Run one collection round
    pub async fn collect_round(&mut self, round: usize) -> Result<(usize, usize, usize, usize)> {
        info!("🔄 Round {} starting...", round);

        let mut indo_collected = 0;
        let mut indo_stored = 0;
        let mut intl_collected = 0;
        let mut intl_stored = 0;

        // Collect Indonesian feeds
        for feed in INDONESIAN_FEEDS {
            let articles = self.fetch_feed(feed).await;
            
            for article in &articles {
                indo_collected += 1;
                self.stats.indonesian_total += 1;

                match self.store_article(article, &self.indonesian_collection.clone()).await {
                    Ok(true) => {
                        indo_stored += 1;
                        self.stats.indonesian_stored += 1;
                    }
                    Ok(false) => {
                        self.stats.duplicates_skipped += 1;
                    }
                    Err(e) => {
                        self.stats.errors += 1;
                        warn!("⚠️ Store error: {}", e);
                    }
                }
            }

            // Rate limiting
            sleep(Duration::from_millis(500)).await;
        }

        // Collect international feeds (every 3rd round)
        if round % 3 == 0 {
            for feed in INTERNATIONAL_FEEDS {
                let articles = self.fetch_feed(feed).await;
                
                for article in &articles {
                    intl_collected += 1;
                    self.stats.international_total += 1;

                    match self.store_article(article, &self.international_collection.clone()).await {
                        Ok(true) => {
                            intl_stored += 1;
                            self.stats.international_stored += 1;
                        }
                        Ok(false) => {
                            self.stats.duplicates_skipped += 1;
                        }
                        Err(e) => {
                            self.stats.errors += 1;
                            warn!("⚠️ Store error: {}", e);
                        }
                    }
                }

                sleep(Duration::from_millis(500)).await;
            }
        }

        // Log round summary
        let runtime = Utc::now() - self.start_time;
        info!("📈 Round {} Summary:", round);
        info!("   Indonesian: {}/{} stored", indo_stored, indo_collected);
        info!("   International: {}/{} stored", intl_stored, intl_collected);
        info!("   Runtime: {}d {}h {}m", 
            runtime.num_days(), 
            runtime.num_hours() % 24, 
            runtime.num_minutes() % 60
        );
        info!("   Total Indonesian: {}", self.stats.indonesian_stored);
        info!("   Total International: {}", self.stats.international_stored);
        info!("   Duplicates skipped: {}", self.stats.duplicates_skipped);
        info!("   Near-duplicates: {}", self.stats.near_duplicates);

        Ok((indo_collected, indo_stored, intl_collected, intl_stored))
    }

    /// Run unlimited collection daemon
    pub async fn run_daemon(&mut self, interval_minutes: u64) -> Result<()> {
        info!("🚀 UNLIMITED NEWS COLLECTOR (Rust + TEI 768-dim)");
        info!("📰 Indonesian feeds: {}", INDONESIAN_FEEDS.len());
        info!("🌍 International feeds: {}", INTERNATIONAL_FEEDS.len());
        info!("⏱️ Interval: {} minutes", interval_minutes);
        info!("🔢 Embedding: 768-dim via TEI");

        let mut round = 1;

        loop {
            if let Err(e) = self.collect_round(round).await {
                error!("❌ Round {} error: {}", round, e);
                // Wait 1 minute on error, then continue
                sleep(Duration::from_secs(60)).await;
            }

            round += 1;

            info!("😴 Waiting {} minutes...", interval_minutes);
            sleep(Duration::from_secs(interval_minutes * 60)).await;
        }
    }

    /// Get current stats
    pub fn stats(&self) -> &UnlimitedStats {
        &self.stats
    }
}

/// Safely truncate string at char boundary
fn truncate_safe(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}
