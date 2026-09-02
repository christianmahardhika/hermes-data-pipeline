use anyhow::{anyhow, Result};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Article {
    pub title: String,
    pub source: String,
    pub content: String,
    pub timestamp: String,
    pub category: String,
}

#[derive(Debug)]
pub struct StorageClient {
    client: Client,
    base_url: String,
    database: String,
    username: String,
    password: String,
}

impl StorageClient {
    /// Create new client from env vars (ARANGO_URL, ARANGO_DATABASE, ARANGO_USERNAME, ARANGO_PASSWORD)
    pub fn from_env() -> Result<Self> {
        let base_url = std::env::var("ARANGO_URL").unwrap_or_else(|_| "http://localhost:8529".to_string());
        let database = std::env::var("ARANGO_DATABASE").unwrap_or_else(|_| "news_analysis".to_string());
        let username = std::env::var("ARANGO_USERNAME").unwrap_or_else(|_| "root".to_string());
        let password = std::env::var("ARANGO_PASSWORD").unwrap_or_else(|_| "".to_string());

        let mut headers = header::HeaderMap::new();
        let auth = format!("Basic {}", base64::encode(format!("{}:{}", username, password)));
        headers.insert(header::AUTHORIZATION, auth.parse().unwrap());

        let client = Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self { client, base_url, database, username, password })
    }

    /// Create collection if not exists
    pub async fn ensure_collection(&self, name: &str) -> Result<()> {
        let url = format!("{}/_db/{}/_api/collection", self.base_url, self.database);
        let body = serde_json::json!({ "name": name });
        let resp = self.client.post(&url).json(&body).send().await?;
        if !resp.status().is_success() && resp.status() != 409 {
            let err = resp.text().await?;
            return Err(anyhow!("Failed to create collection {}: {}", name, err));
        }
        Ok(())
    }

    /// Store article to ArangoDB collection "articles" (upsert by _key = hash of title+timestamp)
    pub async fn store_article(&self, article: &Article) -> Result<()> {
        self.ensure_collection("articles").await?;

        let key = {
            let bytes = md5::compute(format!("{}{}", article.title, article.timestamp));
            bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()
        };
        let doc = serde_json::json!({
            "_key": key,
            "title": article.title,
            "source": article.source,
            "content": article.content,
            "timestamp": article.timestamp,
            "category": article.category,
            "stored_at": chrono::Utc::now().to_rfc3339()
        });

        let url = format!("{}/_db/{}/_api/document/articles", self.base_url, self.database);
        let resp = self.client.post(&url).json(&doc).send().await?;
        
        if !resp.status().is_success() && resp.status() != 409 {
            let err = resp.text().await?;
            return Err(anyhow!("Failed to store article: {}", err));
        }
        Ok(())
    }

    /// Bulk store articles
    pub async fn store_articles(&self, articles: &[Article]) -> Result<usize> {
        let mut count = 0;
        for article in articles {
            if self.store_article(article).await.is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }
}

// Need base64 and md5
mod base64 {
    pub fn encode(input: String) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let bytes = input.as_bytes();
        let mut result = String::new();
        for chunk in bytes.chunks(3) {
            let b0 = chunk[0];
            let b1 = chunk.get(1).copied().unwrap_or(0);
            let b2 = chunk.get(2).copied().unwrap_or(0);
            let buf = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
            result.push(CHARS[((buf >> 18) & 0x3F) as usize] as char);
            result.push(CHARS[((buf >> 12) & 0x3F) as usize] as char);
            if chunk.len() > 1 {
                result.push(CHARS[((buf >> 6) & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
            if chunk.len() > 2 {
                result.push(CHARS[(buf & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
        }
        result
    }
}

mod md5 {
    pub fn compute(input: String) -> [u8; 16] {
        // Simplified md5 for key generation - in production use proper crate
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        let hash = hasher.finish();
        let mut bytes = [0u8; 16];
        bytes[..8].copy_from_slice(&hash.to_le_bytes());
        bytes
    }
}