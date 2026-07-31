//! Configuration management for Hermes services
//!
//! Provides centralized configuration loading and validation
//! for all Hermes pipeline services with environment variable consolidation.

use serde::{Deserialize, Serialize};
use std::env;
use anyhow::{anyhow, Result};

/// Comprehensive configuration for all Hermes services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesConfig {
    // Database configuration
    pub arango_url: String,
    pub arango_database: String,
    pub arango_username: Option<String>,
    pub arango_password: Option<String>,
    
    // Text Embedding Inference (TEI) configuration
    pub tei_url: Option<String>,
    
    // Labeler service configuration
    pub labeler_base_url: Option<String>,
    pub labeler_api_key: Option<String>,
    pub labeler_model: Option<String>,
    
    // Storage backend configuration
    pub storage_backend: StorageBackend,
    
    // Optional service-specific configurations
    pub collector_config: CollectorConfig,
    pub processor_config: ProcessorConfig,
    pub observability_config: ObservabilityConfig,
}

/// Storage backend options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackend {
    ArangoDB,
    PostgreSQL,
    SQLite,
    Memory,
}

/// Collector service specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    pub collection_interval_seconds: u64,
    pub max_concurrent_feeds: usize,
    pub circuit_breaker_failure_threshold: u32,
    pub circuit_breaker_timeout_seconds: u64,
}

/// Processor service specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    pub batch_size: usize,
    pub processing_timeout_seconds: u64,
    pub retry_attempts: u32,
    pub enable_embedding: bool,
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub log_level: String,
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub tracing_enabled: bool,
}

/// Legacy ServiceConfig for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub arango_url: String,
    pub arango_database: String,
    pub arango_username: Option<String>,
    pub arango_password: Option<String>,
}

impl HermesConfig {
    /// Load configuration from environment variables with validation
    pub fn from_env() -> Result<Self> {
        // Required database configuration
        let arango_url = env::var("ARANGO_URL")
            .unwrap_or_else(|_| "http://localhost:8529".to_string());
        
        let arango_database = env::var("ARANGO_DATABASE")
            .unwrap_or_else(|_| "intelligence".to_string());
            
        // Validate database URL format
        if !arango_url.starts_with("http://") && !arango_url.starts_with("https://") {
            return Err(anyhow!("ARANGO_URL must start with http:// or https://"));
        }
        
        // Optional database authentication
        let arango_username = env::var("ARANGO_USERNAME").ok();
        let arango_password = env::var("ARANGO_PASSWORD").ok();
        
        // Optional service URLs
        let tei_url = env::var("TEI_URL").ok();
        let labeler_base_url = env::var("LABELER_BASE_URL").ok();
        let labeler_api_key = env::var("LABELER_API_KEY").ok();
        let labeler_model = env::var("LABELER_MODEL").ok();
        
        // Storage backend with validation
        let storage_backend = match env::var("STORAGE_BACKEND")
            .unwrap_or_else(|_| "arangodb".to_string())
            .to_lowercase()
            .as_str() {
            "arangodb" => StorageBackend::ArangoDB,
            "postgresql" | "postgres" => StorageBackend::PostgreSQL,
            "sqlite" => StorageBackend::SQLite,
            "memory" => StorageBackend::Memory,
            invalid => return Err(anyhow!(
                "Invalid STORAGE_BACKEND '{}'. Valid options: arangodb, postgresql, sqlite, memory", 
                invalid
            )),
        };
        
        // Service-specific configurations with defaults
        let collector_config = CollectorConfig {
            collection_interval_seconds: env::var("COLLECTOR_INTERVAL_SECONDS")
                .unwrap_or_else(|_| "300".to_string()) // 5 minutes default
                .parse()
                .unwrap_or(300),
            max_concurrent_feeds: env::var("COLLECTOR_MAX_CONCURRENT")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            circuit_breaker_failure_threshold: env::var("COLLECTOR_CB_THRESHOLD")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            circuit_breaker_timeout_seconds: env::var("COLLECTOR_CB_TIMEOUT")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
        };
        
        let processor_config = ProcessorConfig {
            batch_size: env::var("PROCESSOR_BATCH_SIZE")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .unwrap_or(50),
            processing_timeout_seconds: env::var("PROCESSOR_TIMEOUT")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
            retry_attempts: env::var("PROCESSOR_RETRY_ATTEMPTS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            enable_embedding: env::var("PROCESSOR_ENABLE_EMBEDDING")
                .unwrap_or_else(|_| "true".to_string())
                .to_lowercase() == "true",
        };
        
        let observability_config = ObservabilityConfig {
            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()),
            metrics_enabled: env::var("METRICS_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .to_lowercase() == "true",
            metrics_port: env::var("METRICS_PORT")
                .unwrap_or_else(|_| "9090".to_string())
                .parse()
                .unwrap_or(9090),
            tracing_enabled: env::var("TRACING_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .to_lowercase() == "true",
        };
        
        Ok(HermesConfig {
            arango_url,
            arango_database,
            arango_username,
            arango_password,
            tei_url,
            labeler_base_url,
            labeler_api_key,
            labeler_model,
            storage_backend,
            collector_config,
            processor_config,
            observability_config,
        })
    }
    
    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        // Validate database configuration
        if self.arango_database.trim().is_empty() {
            return Err(anyhow!("Database name cannot be empty"));
        }
        
        // Validate URLs if provided
        if let Some(ref tei_url) = self.tei_url {
            if !tei_url.starts_with("http://") && !tei_url.starts_with("https://") {
                return Err(anyhow!("TEI_URL must start with http:// or https://"));
            }
        }
        
        if let Some(ref labeler_url) = self.labeler_base_url {
            if !labeler_url.starts_with("http://") && !labeler_url.starts_with("https://") {
                return Err(anyhow!("LABELER_BASE_URL must start with http:// or https://"));
            }
        }
        
        // Validate port ranges
        if self.observability_config.metrics_port < 1024 || self.observability_config.metrics_port > 65535 {
            return Err(anyhow!("METRICS_PORT must be between 1024 and 65535"));
        }
        
        // Validate log level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.observability_config.log_level.to_lowercase().as_str()) {
            return Err(anyhow!("LOG_LEVEL must be one of: trace, debug, info, warn, error"));
        }
        
        Ok(())
    }
    
    /// Get database configuration as ServiceConfig (backward compatibility)
    pub fn to_service_config(&self) -> ServiceConfig {
        ServiceConfig {
            arango_url: self.arango_url.clone(),
            arango_database: self.arango_database.clone(),
            arango_username: self.arango_username.clone(),
            arango_password: self.arango_password.clone(),
        }
    }
    
    /// Check if Indonesian market intelligence features are enabled
    pub fn is_indonesian_intelligence_enabled(&self) -> bool {
        // Enable if we have TEI for embeddings or labeler for analysis
        self.tei_url.is_some() || self.labeler_base_url.is_some()
    }
    
    /// Get comprehensive configuration summary for logging
    pub fn summary(&self) -> String {
        format!(
            "Hermes Config: DB={}/{}, Storage={:?}, TEI={}, Labeler={}, Metrics={}:{}",
            self.arango_url,
            self.arango_database,
            self.storage_backend,
            self.tei_url.is_some(),
            self.labeler_base_url.is_some(),
            self.observability_config.metrics_enabled,
            self.observability_config.metrics_port
        )
    }
}

impl Default for HermesConfig {
    fn default() -> Self {
        Self {
            arango_url: "http://localhost:8529".to_string(),
            arango_database: "intelligence".to_string(),
            arango_username: None,
            arango_password: None,
            tei_url: None,
            labeler_base_url: None,
            labeler_api_key: None,
            labeler_model: None,
            storage_backend: StorageBackend::ArangoDB,
            collector_config: CollectorConfig::default(),
            processor_config: ProcessorConfig::default(),
            observability_config: ObservabilityConfig::default(),
        }
    }
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            collection_interval_seconds: 300, // 5 minutes
            max_concurrent_feeds: 10,
            circuit_breaker_failure_threshold: 5,
            circuit_breaker_timeout_seconds: 60,
        }
    }
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            batch_size: 50,
            processing_timeout_seconds: 300, // 5 minutes
            retry_attempts: 3,
            enable_embedding: true,
        }
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            metrics_enabled: true,
            metrics_port: 9090,
            tracing_enabled: false,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_hermes_config_default() {
        let config = HermesConfig::default();
        
        assert_eq!(config.arango_url, "http://localhost:8529");
        assert_eq!(config.arango_database, "intelligence");
        assert!(matches!(config.storage_backend, StorageBackend::ArangoDB));
        assert_eq!(config.collector_config.collection_interval_seconds, 300);
        assert_eq!(config.observability_config.log_level, "info");
    }

    #[test]
    fn test_hermes_config_from_env_defaults() {
        // Clear environment to test defaults
        let _clean_env = CleanupEnv::new();
        
        // Explicitly set valid defaults to avoid validation errors
        env::set_var("ARANGO_URL", "http://localhost:8529");
        env::set_var("ARANGO_DATABASE", "intelligence");
        
        let config = HermesConfig::from_env().unwrap();
        
        assert_eq!(config.arango_url, "http://localhost:8529");
        assert_eq!(config.arango_database, "intelligence");
        assert!(matches!(config.storage_backend, StorageBackend::ArangoDB));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_hermes_config_from_env_custom() {
        let _clean_env = CleanupEnv::new();
        
        // Set all required environment variables for this test
        env::set_var("ARANGO_URL", "https://prod.arangodb.com:8529");
        env::set_var("ARANGO_DATABASE", "hermes_prod");
        env::set_var("ARANGO_USERNAME", "admin");
        env::set_var("ARANGO_PASSWORD", "secret");
        env::set_var("STORAGE_BACKEND", "postgresql");
        env::set_var("TEI_URL", "http://tei-service:8080");
        env::set_var("LABELER_BASE_URL", "http://labeler:3000");
        env::set_var("LOG_LEVEL", "debug");
        
        let config = HermesConfig::from_env().unwrap();
        
        assert_eq!(config.arango_url, "https://prod.arangodb.com:8529");
        assert_eq!(config.arango_database, "hermes_prod");
        assert_eq!(config.arango_username, Some("admin".to_string()));
        assert_eq!(config.arango_password, Some("secret".to_string()));
        assert!(matches!(config.storage_backend, StorageBackend::PostgreSQL));
        assert_eq!(config.tei_url, Some("http://tei-service:8080".to_string()));
        assert_eq!(
            config.labeler_base_url,
            Some("http://labeler:3000".to_string())
        );
        assert_eq!(config.observability_config.log_level, "debug");
        
        assert!(config.validate().is_ok());
    }
        let _clean_env = CleanupEnv::new();
        
        env::set_var("ARANGO_URL", "invalid-url");
        
        let result = HermesConfig::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must start with http://"));
    }

    #[test]
    fn test_invalid_storage_backend() {
        let _clean_env = CleanupEnv::new();
        
        env::set_var("STORAGE_BACKEND", "invalid_backend");
        
        let result = HermesConfig::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid STORAGE_BACKEND"));
    }

    #[test]
    fn test_config_validation() {
        let mut config = HermesConfig::default();
        
        // Valid config should pass
        assert!(config.validate().is_ok());
        
        // Empty database name should fail
        config.arango_database = "".to_string();
        assert!(config.validate().is_err());
        
        // Invalid TEI URL should fail
        config.arango_database = "test".to_string();
        config.tei_url = Some("invalid-url".to_string());
        assert!(config.validate().is_err());
        
        // Invalid metrics port should fail
        config.tei_url = None;
        config.observability_config.metrics_port = 80; // Below 1024
        assert!(config.validate().is_err());
        
        // Invalid log level should fail
        config.observability_config.metrics_port = 9090;
        config.observability_config.log_level = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_service_config_conversion() {
        let hermes_config = HermesConfig {
            arango_url: "https://test.com".to_string(),
            arango_database: "test_db".to_string(),
            arango_username: Some("user".to_string()),
            arango_password: Some("pass".to_string()),
            ..Default::default()
        };
        
        let service_config = hermes_config.to_service_config();
        
        assert_eq!(service_config.arango_url, "https://test.com");
        assert_eq!(service_config.arango_database, "test_db");
        assert_eq!(service_config.arango_username, Some("user".to_string()));
        assert_eq!(service_config.arango_password, Some("pass".to_string()));
    }

    #[test]
    fn test_indonesian_intelligence_enabled() {
        let mut config = HermesConfig::default();
        
        // Disabled by default (no TEI or labeler)
        assert!(!config.is_indonesian_intelligence_enabled());
        
        // Enabled with TEI
        config.tei_url = Some("http://tei:8080".to_string());
        assert!(config.is_indonesian_intelligence_enabled());
        
        // Enabled with labeler
        config.tei_url = None;
        config.labeler_base_url = Some("http://labeler:3000".to_string());
        assert!(config.is_indonesian_intelligence_enabled());
    }

    #[test]
    fn test_config_summary() {
        let config = HermesConfig::default();
        let summary = config.summary();
        
        assert!(summary.contains("http://localhost:8529"));
        assert!(summary.contains("intelligence"));
        assert!(summary.contains("ArangoDB"));
        assert!(summary.contains("9090"));
    }

    // Helper struct to clean up environment variables after tests
    struct CleanupEnv {
        vars_to_restore: Vec<(String, Option<String>)>,
    }

    impl CleanupEnv {
        fn new() -> Self {
            let vars_to_clean = [
                "ARANGO_URL", "ARANGO_DATABASE", "ARANGO_USERNAME", "ARANGO_PASSWORD",
                "TEI_URL", "LABELER_BASE_URL", "LABELER_API_KEY", "LABELER_MODEL",
                "STORAGE_BACKEND", "LOG_LEVEL", "METRICS_PORT", "METRICS_ENABLED",
                "COLLECTOR_INTERVAL_SECONDS", "PROCESSOR_BATCH_SIZE",
            ];
            
            let vars_to_restore: Vec<(String, Option<String>)> = vars_to_clean
                .iter()
                .map(|&var| (var.to_string(), env::var(var).ok()))
                .collect();
            
            // Clear all test-related environment variables
            for var in &vars_to_clean {
                env::remove_var(var);
            }
            
            Self { vars_to_restore }
        }
    }

    impl Drop for CleanupEnv {
        fn drop(&mut self) {
            // Restore original environment variables
            for (var, value) in &self.vars_to_restore {
                match value {
                    Some(val) => env::set_var(var, val),
                    None => env::remove_var(var),
                }
            }
        }
    }
}