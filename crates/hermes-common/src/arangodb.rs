//! ArangoDB client utilities and helpers
//!
//! Extracted from monolith services.rs - provides standardized ArangoDB 
//! connection and operation patterns for all Hermes services.
//! 
//! Uses trait-based design for testability and future HTTP implementation.

use anyhow::{anyhow, Result};
use serde_json::Value;
use tracing::{info, warn};
use crate::config::ServiceConfig;

/// ArangoDB operations trait for dependency injection and testing
#[async_trait::async_trait]
pub trait ArangoOperations {
    async fn create_collection(&self, name: &str) -> Result<()>;
    async fn insert_document(&self, collection: &str, document: &Value) -> Result<()>;
    async fn query_documents(&self, aql: &str) -> Result<Vec<Value>>;
    async fn health_check(&self) -> Result<bool>;
}

/// ArangoDB client wrapper with connection pooling
#[derive(Debug, Clone)]
pub struct ArangoClient {
    base_url: String,
    database: String,
    username: String,
    password: String,
}

impl ArangoClient {
    /// Create a new ArangoDB client from configuration
    pub fn new(config: ServiceConfig) -> Self {
        Self {
            base_url: config.arango_url,
            database: config.arango_database,
            username: config.arango_username.unwrap_or_else(|| "root".to_string()),
            password: config.arango_password.unwrap_or_else(|| "".to_string()),
        }
    }

    /// Create ArangoDB client from environment variables (legacy compatibility)
    pub fn from_env() -> Self {
        Self {
            // Use canonical env var names matching project standard
            base_url: std::env::var("ARANGO_URL").unwrap_or_else(|_| "http://localhost:8529".to_string()),
            database: std::env::var("ARANGO_DATABASE").unwrap_or_else(|_| "intelligence".to_string()),
            username: std::env::var("ARANGO_USERNAME").unwrap_or_else(|_| "root".to_string()),
            password: std::env::var("ARANGO_PASSWORD").unwrap_or_else(|_| "".to_string()),
        }
    }

    /// Initialize standard collections for Hermes intelligence pipeline
    pub async fn initialize_collections(&self) -> Result<()> {
        let collections = vec![
            "indonesian_stocks",
            "articles", 
            "commodities",
            "geopolitical_signals",
        ];

        for collection in collections {
            match self.create_collection(collection).await {
                Ok(_) => info!("✅ Collection '{}' ready", collection),
                Err(e) => {
                    if e.to_string().contains("duplicate") || e.to_string().contains("1207") {
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
    /// Create a collection in ArangoDB
    async fn create_collection(&self, name: &str) -> Result<()> {
        // TODO: Implement actual HTTP call when reqwest/OpenSSL is available
        // For now, mock implementation for TDD structure
        info!("Mock: Creating collection '{}'", name);
        Ok(())
    }

    /// Insert a document into a collection
    async fn insert_document(&self, collection: &str, _document: &Value) -> Result<()> {
        // TODO: Implement actual HTTP call when reqwest/OpenSSL is available
        info!("Mock: Inserting document into collection '{}'", collection);
        Ok(())
    }

    /// Execute AQL query and return results
    async fn query_documents(&self, aql: &str) -> Result<Vec<Value>> {
        // TODO: Implement actual HTTP call when reqwest/OpenSSL is available
        info!("Mock: Executing AQL query: {}", aql);
        Ok(vec![])
    }

    /// Health check for database connectivity
    async fn health_check(&self) -> Result<bool> {
        // TODO: Implement actual HTTP call when reqwest/OpenSSL is available
        info!("Mock: Health check for {}/{}", self.base_url, self.database);
        Ok(true)
    }
}

/// Mock implementation for testing
pub struct MockArangoClient {
    should_fail: bool,
}

impl MockArangoClient {
    pub fn new() -> Self {
        Self { should_fail: false }
    }

    pub fn with_failure() -> Self {
        Self { should_fail: true }
    }
}

#[async_trait::async_trait]
impl ArangoOperations for MockArangoClient {
    async fn create_collection(&self, _name: &str) -> Result<()> {
        if self.should_fail {
            Err(anyhow!("Mock: Collection creation failed"))
        } else {
            Ok(())
        }
    }

    async fn insert_document(&self, _collection: &str, _document: &Value) -> Result<()> {
        if self.should_fail {
            Err(anyhow!("Mock: Document insertion failed"))
        } else {
            Ok(())
        }
    }

    async fn query_documents(&self, _aql: &str) -> Result<Vec<Value>> {
        if self.should_fail {
            Err(anyhow!("Mock: Query failed"))
        } else {
            Ok(vec![serde_json::json!({"test": "data"})])
        }
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(!self.should_fail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_arango_client_creation() {
        let config = ServiceConfig::default();
        let client = ArangoClient::new(config);
        
        let (url, db, user) = client.get_config();
        assert_eq!(url, "http://localhost:8529");
        assert_eq!(db, "intelligence");
        assert_eq!(user, "root");
    }

    #[tokio::test]  
    async fn test_arango_client_from_env() {
        let client = ArangoClient::from_env();
        
        let (url, db, user) = client.get_config();
        assert_eq!(url, "http://localhost:8529");
        assert_eq!(db, "intelligence");
        assert_eq!(user, "root");
    }

    #[tokio::test]
    async fn test_mock_client_success() {
        let client = MockArangoClient::new();
        
        assert!(client.health_check().await.is_ok());
        assert!(client.create_collection("test").await.is_ok());
        assert!(client.insert_document("test", &serde_json::json!({})).await.is_ok());
        
        let results = client.query_documents("FOR doc IN test RETURN doc").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_mock_client_failure() {
        let client = MockArangoClient::with_failure();
        
        assert!(!client.health_check().await.unwrap());
        assert!(client.create_collection("test").await.is_err());
        assert!(client.insert_document("test", &serde_json::json!({})).await.is_err());
        assert!(client.query_documents("FOR doc IN test RETURN doc").await.is_err());
    }

    #[tokio::test]
    async fn test_initialize_collections() {
        let config = ServiceConfig::default();
        let client = ArangoClient::new(config);
        
        // Should not fail with mock implementation
        assert!(client.initialize_collections().await.is_ok());
    }
}