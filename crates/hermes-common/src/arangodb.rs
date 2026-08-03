//! ArangoDB client utilities and helpers
//!
//! Extracted from monolith services.rs - provides standardized ArangoDB 
//! connection and operation patterns for all Hermes services.
//! 
//! HTTP implementation using reqwest for real database operations.

use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info, warn};
use crate::config::ServiceConfig;

/// ArangoDB operations trait for dependency injection and testing
#[async_trait::async_trait]
pub trait ArangoOperations {
    async fn create_collection(&self, name: &str) -> Result<()>;
    async fn insert_document(&self, collection: &str, document: &Value) -> Result<String>;
    async fn query_documents(&self, aql: &str) -> Result<Vec<Value>>;
    async fn health_check(&self) -> Result<bool>;
}

/// ArangoDB HTTP API response wrappers
#[derive(Debug, Serialize, Deserialize)]
struct ArangoResponse<T> {
    error: bool,
    #[serde(rename = "errorMessage")]
    error_message: Option<String>,
    #[serde(rename = "errorNum")]
    error_num: Option<u16>,
    result: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateCollectionResult {
    name: String,
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct InsertDocumentResult {
    #[serde(rename = "_id")]
    id: String,
    #[serde(rename = "_key")]
    key: String,
    #[serde(rename = "_rev")]
    rev: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct QueryResult {
    result: Vec<Value>,
    has_more: bool,
    #[serde(rename = "cached")]
    cached: Option<bool>,
}

/// ArangoDB client wrapper with HTTP connection
#[derive(Debug, Clone)]
pub struct ArangoClient {
    base_url: String,
    database: String,
    username: String,
    password: String,
    client: Client,
}

impl ArangoClient {
    /// Create a new ArangoDB client from configuration
    pub fn new(config: ServiceConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            base_url: config.arango_url,
            database: config.arango_database,
            username: config.arango_username.unwrap_or_else(|| "root".to_string()),
            password: config.arango_password.unwrap_or_else(|| "".to_string()),
            client,
        })
    }

    /// Create ArangoDB client from environment variables (legacy compatibility)
    pub fn from_env() -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            base_url: std::env::var("ARANGO_URL").unwrap_or_else(|_| "http://localhost:8529".to_string()),
            database: std::env::var("ARANGO_DATABASE").unwrap_or_else(|_| "intelligence".to_string()),
            username: std::env::var("ARANGO_USERNAME").unwrap_or_else(|_| "root".to_string()),
            password: std::env::var("ARANGO_PASSWORD").unwrap_or_else(|_| "".to_string()),
            client,
        })
    }

    /// Get the API URL for a specific endpoint
    fn api_url(&self, endpoint: &str) -> String {
        format!("{}/_db/{}/{}", self.base_url, self.database, endpoint)
    }

    /// Initialize standard collections for Hermes intelligence pipeline
    pub async fn initialize_collections(&self) -> Result<()> {
        let collections = vec![
            "indonesian_stocks",
            "articles",
            "commodities",
            "geopolitical_signals",
            "economic_indicators",
        ];

        for collection in collections {
            match self.create_collection(collection).await {
                Ok(_) => info!("✅ Collection '{}' created", collection),
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("duplicate") || err_str.contains("1207") {
                        info!("📦 Collection '{}' already exists", collection);
                    } else {
                        warn!("⚠️  Failed to create collection '{}': {}", collection, e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Get database configuration for debugging
    pub fn get_config(&self) -> (&str, &str, &str) {
        (&self.base_url, &self.database, &self.username)
    }
}

#[async_trait::async_trait]
impl ArangoOperations for ArangoClient {
    /// Create a collection in ArangoDB via HTTP API
    async fn create_collection(&self, name: &str) -> Result<()> {
        let url = self.api_url("_api/collection");
        
        let body = serde_json::json!({
            "name": name,
            "type": 2,  // Document collection
        });

        debug!("Creating collection '{}' at {}", name, url);

        let response = self.client
            .post(&url)
            .basic_auth(&self.username, Some(&self.password))
            .json(&body)
            .send()
            .await
            .context("Failed to send create collection request")?;

        let _status = response.status();
        let body: ArangoResponse<CreateCollectionResult> = response
            .json()
            .await
            .context("Failed to parse create collection response")?;

        if body.error {
            if let Some(error_num) = body.error_num {
                // Error 1207 = duplicate collection (already exists)
                if error_num == 1207 {
                    return Err(anyhow!("Collection '{}' already exists", name));
                }
            }
            return Err(anyhow!("ArangoDB error: {}", body.error_message.unwrap_or_else(|| "Unknown error".to_string())));
        }

        info!("✅ Collection '{}' created successfully", name);
        Ok(())
    }

    /// Insert a document into a collection via HTTP API
    async fn insert_document(&self, collection: &str, document: &Value) -> Result<String> {
        let url = self.api_url(&format!("_api/document/{}", collection));

        debug!("Inserting document into collection '{}' at {}", collection, url);

        let response = self.client
            .post(&url)
            .basic_auth(&self.username, Some(&self.password))
            .json(document)
            .send()
            .await
            .context("Failed to send insert document request")?;

        let _status = response.status();
        let body: ArangoResponse<InsertDocumentResult> = response
            .json()
            .await
            .context("Failed to parse insert document response")?;

        if body.error {
            return Err(anyhow!("ArangoDB error: {}", body.error_message.unwrap_or_else(|| "Unknown error".to_string())));
        }

        let key = body.result
            .map(|r| r.key)
            .ok_or_else(|| anyhow!("No document key returned"))?;

        debug!("✅ Document inserted with key: {}", key);
        Ok(key)
    }

    /// Execute AQL query and return results via HTTP API
    async fn query_documents(&self, aql: &str) -> Result<Vec<Value>> {
        let url = self.api_url("_api/cursor");

        let body = serde_json::json!({
            "query": aql,
            "count": true,
        });

        debug!("Executing AQL query at {}", url);

        let response = self.client
            .post(&url)
            .basic_auth(&self.username, Some(&self.password))
            .json(&body)
            .send()
            .await
            .context("Failed to send query request")?;

        let _status = response.status();
        let body: ArangoResponse<QueryResult> = response
            .json()
            .await
            .context("Failed to parse query response")?;

        if body.error {
            return Err(anyhow!("ArangoDB error: {}", body.error_message.unwrap_or_else(|| "Unknown error".to_string())));
        }

        let results = body.result
            .map(|r| r.result)
            .unwrap_or_else(|| Vec::new());

        debug!("✅ Query returned {} documents", results.len());
        Ok(results)
    }

    /// Check ArangoDB health via HTTP API
    async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/_api/database/current", self.base_url);

        debug!("Checking ArangoDB health at {}", url);

        let response = self.client
            .get(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .context("Failed to send health check request")?;

        let is_healthy = response.status().is_success();

        if is_healthy {
            debug!("✅ ArangoDB health check passed");
        } else {
            warn!("⚠️  ArangoDB health check failed: {}", response.status());
        }

        Ok(is_healthy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arango_client_from_env() {
        // Test that client can be created from environment
        let client = ArangoClient::from_env();
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert_eq!(client.database, "intelligence");
        assert_eq!(client.username, "root");
    }

    #[tokio::test]
    async fn test_health_check() {
        // Integration test - requires local ArangoDB
        let client = ArangoClient::from_env().unwrap();
        
        // This will fail if ArangoDB is not running locally
        match client.health_check().await {
            Ok(healthy) => println!("ArangoDB healthy: {}", healthy),
            Err(e) => println!("Health check error (expected if no ArangoDB): {}", e),
        }
    }
}
