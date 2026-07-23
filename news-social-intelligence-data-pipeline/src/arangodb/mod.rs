//! ArangoDB Client Module
//!
//! Raw reqwest HTTP client for ArangoDB operations.
//! Matches the Python script pattern — no arangors crate.
//!
//! Config via env vars:
//! - ARANGO_URL (default: http://localhost:8529)
//! - ARANGO_DATABASE (default: hermes_intelligence)
//! - ARANGO_USERNAME (default: root)
//! - ARANGO_PASSWORD (default: hermes)

pub mod ingester;
pub mod schema;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::time::Duration;
use tracing::{info, warn, error};

/// Default timeout for ArangoDB HTTP requests
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// ArangoDB client using raw reqwest HTTP
pub struct ArangoClient {
    client: Client,
    base_url: String,
    database: String,
    username: String,
    password: String,
}

impl ArangoClient {
    /// Create a new ArangoClient from environment variables
    pub fn new() -> Result<Self> {
        let base_url = std::env::var("ARANGO_URL")
            .unwrap_or_else(|_| "http://localhost:8529".to_string());
        let database = std::env::var("ARANGO_DATABASE")
            .unwrap_or_else(|_| "hermes_intelligence".to_string());
        let username = std::env::var("ARANGO_USERNAME")
            .unwrap_or_else(|_| "root".to_string());
        let password = std::env::var("ARANGO_PASSWORD")
            .unwrap_or_else(|_| "hermes".to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .context("building ArangoDB HTTP client")?;

        info!("🗄️ ArangoDB client initialized: {}/{}", base_url, database);

        Ok(Self {
            client,
            base_url,
            database,
            username,
            password,
        })
    }

    /// Build the database API base URL
    fn db_url(&self, path: &str) -> String {
        format!("{}/_db/{}{}", self.base_url, self.database, path)
    }

    /// Build a server-level URL (no database prefix)
    pub(crate) fn server_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Build a database-level API URL (public for schema module)
    pub(crate) fn db_api_url(&self, path: &str) -> String {
        self.db_url(path)
    }

    /// Perform an authenticated GET request
    pub(crate) async fn http_get(&self, url: &str) -> Result<reqwest::Response> {
        self.client
            .get(url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .context(format!("HTTP GET {}", url))
    }

    /// Perform an authenticated POST request with JSON body
    pub(crate) async fn http_post(&self, url: &str, body: &serde_json::Value) -> Result<reqwest::Response> {
        self.client
            .post(url)
            .basic_auth(&self.username, Some(&self.password))
            .json(body)
            .send()
            .await
            .context(format!("HTTP POST {}", url))
    }

    /// Check ArangoDB server health
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/_api/version", self.base_url);

        let resp = self.client
            .get(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .context("ArangoDB health check request failed")?;

        if resp.status().is_success() {
            let body: serde_json::Value = resp.json().await
                .context("parsing ArangoDB version response")?;
            info!("🗄️ ArangoDB healthy: server={}, version={}",
                body.get("server").and_then(|v| v.as_str()).unwrap_or("unknown"),
                body.get("version").and_then(|v| v.as_str()).unwrap_or("unknown"),
            );
            Ok(true)
        } else {
            warn!("⚠️ ArangoDB health check failed: status={}", resp.status());
            Ok(false)
        }
    }

    /// Execute an AQL query and return deserialized results
    pub async fn query_aql<T: DeserializeOwned>(
        &self,
        query: &str,
        bind_vars: serde_json::Value,
    ) -> Result<Vec<T>> {
        let url = self.db_url("/_api/cursor");

        let payload = serde_json::json!({
            "query": query,
            "bindVars": bind_vars,
        });

        let resp = self.client
            .post(&url)
            .basic_auth(&self.username, Some(&self.password))
            .json(&payload)
            .send()
            .await
            .context("ArangoDB AQL query request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("❌ ArangoDB AQL error: status={}, body={}", status, body);
            anyhow::bail!("ArangoDB AQL query failed: status={}, body={}", status, body);
        }

        let body: serde_json::Value = resp.json().await
            .context("parsing ArangoDB cursor response")?;

        let results = body.get("result")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        let typed_results: Vec<T> = results
            .into_iter()
            .map(|v| serde_json::from_value(v))
            .collect::<std::result::Result<Vec<T>, _>>()
            .context("deserializing AQL query results")?;

        Ok(typed_results)
    }

    /// Insert a document into a collection, returns the _key
    pub async fn insert_document(
        &self,
        collection: &str,
        doc: &serde_json::Value,
    ) -> Result<String> {
        let url = self.db_url(&format!("/_api/document/{}", collection));

        let resp = self.client
            .post(&url)
            .basic_auth(&self.username, Some(&self.password))
            .json(doc)
            .send()
            .await
            .context("ArangoDB insert document request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("❌ ArangoDB insert error: collection={}, status={}, body={}",
                collection, status, body);
            anyhow::bail!(
                "ArangoDB insert_document failed: collection={}, status={}, body={}",
                collection, status, body
            );
        }

        let body: serde_json::Value = resp.json().await
            .context("parsing ArangoDB insert response")?;

        let key = body.get("_key")
            .and_then(|k| k.as_str())
            .map(|s| s.to_string())
            .context("missing _key in ArangoDB insert response")?;

        Ok(key)
    }

    /// Insert an edge document with _from, _to, and additional data fields
    pub async fn insert_edge(
        &self,
        collection: &str,
        from: &str,
        to: &str,
        data: &serde_json::Value,
    ) -> Result<String> {
        let mut edge_doc = data.clone();

        // Merge _from and _to into the edge document
        if let Some(obj) = edge_doc.as_object_mut() {
            obj.insert("_from".to_string(), serde_json::Value::String(from.to_string()));
            obj.insert("_to".to_string(), serde_json::Value::String(to.to_string()));
        } else {
            // If data is not an object, create one with _from, _to, and data
            edge_doc = serde_json::json!({
                "_from": from,
                "_to": to,
                "data": data,
            });
        }

        self.insert_document(collection, &edge_doc).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running ArangoDB at localhost:8529
    async fn test_arango_health_check() {
        let client = ArangoClient::new().expect("failed to create ArangoClient");
        let healthy = client.health_check().await.expect("health check failed");
        assert!(healthy, "ArangoDB should be healthy");
    }

    #[tokio::test]
    #[ignore] // Requires running ArangoDB at localhost:8529
    async fn test_arango_query_aql() {
        let client = ArangoClient::new().expect("failed to create ArangoClient");
        let results: Vec<serde_json::Value> = client
            .query_aql("RETURN 1", serde_json::json!({}))
            .await
            .expect("AQL query failed");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_arango_client_creation() {
        // Should succeed with default env vars
        let client = ArangoClient::new();
        assert!(client.is_ok(), "ArangoClient::new() should succeed with defaults");
    }
}
