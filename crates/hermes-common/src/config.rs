//! Configuration management for Hermes services
//!
//! Provides centralized configuration loading and validation
//! for all Hermes pipeline services.

use serde::{Deserialize, Serialize};

/// Base configuration shared across all services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub arango_url: String,
    pub arango_database: String,
    pub arango_username: Option<String>,
    pub arango_password: Option<String>,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            arango_url: "http://localhost:8529".to_string(),
            arango_database: "intelligence".to_string(),
            arango_username: None,
            arango_password: None,
        }
    }
}