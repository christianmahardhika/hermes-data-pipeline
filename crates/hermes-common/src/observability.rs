//! Observability setup for Hermes services
//!
//! Provides centralized logging, metrics, and tracing initialization
//! for all Hermes pipeline services with Indonesian market context.

use anyhow::Result;
use std::sync::Once;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static INIT: Once = Once::new();

/// Basic observability configuration
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    pub log_level: String,
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub tracing_enabled: bool,
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

/// Initialize observability stack for Hermes services
pub fn init_observability(config: &ObservabilityConfig, service_name: &str) -> Result<()> {
    INIT.call_once(|| {
        let log_level = parse_log_level(&config.log_level);
        
        // Create environment filter
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(format!("{}={}", service_name, log_level)));
        
        // Initialize tracing subscriber
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true)
                    .compact(),
            )
            .init();
    });
    
    info!(
        service = service_name,
        log_level = config.log_level,
        metrics_enabled = config.metrics_enabled,
        "🚀 Hermes observability initialized"
    );
    
    Ok(())
}

/// Parse log level string to tracing Level
fn parse_log_level(level_str: &str) -> &'static str {
    match level_str.to_lowercase().as_str() {
        "trace" => "trace",
        "debug" => "debug", 
        "info" => "info",
        "warn" => "warn",
        "error" => "error",
        _ => "info", // Default fallback
    }
}

/// Metrics collection utilities for Hermes services
pub struct HermesMetrics {
    pub service_name: String,
    pub enabled: bool,
}

impl HermesMetrics {
    /// Create new metrics collector
    pub fn new(service_name: String, enabled: bool) -> Self {
        Self {
            service_name,
            enabled,
        }
    }
    
    /// Record processing latency for Indonesian market operations
    pub fn record_processing_latency(&self, operation: &str, duration_ms: u64) {
        if !self.enabled {
            return;
        }
        
        tracing::info!(
            service = %self.service_name,
            operation = operation,
            duration_ms = duration_ms,
            "⏱️ Processing latency recorded"
        );
    }
    
    /// Record article processing count
    pub fn record_article_processed(&self, source: &str, status: &str) {
        if !self.enabled {
            return;
        }
        
        tracing::info!(
            service = %self.service_name,
            source = source,
            status = status,
            "📰 Article processing recorded"
        );
    }
    
    /// Record Indonesian stock data collection
    pub fn record_stock_collection(&self, symbol: &str, success: bool) {
        if !self.enabled {
            return;
        }
        
        let status = if success { "success" } else { "failed" };
        tracing::info!(
            service = %self.service_name,
            stock_symbol = symbol,
            status = status,
            "📈 Indonesian stock data collection recorded"
        );
    }
    
    /// Record database operation metrics
    pub fn record_db_operation(&self, operation: &str, collection: &str, duration_ms: u64) {
        if !self.enabled {
            return;
        }
        
        tracing::info!(
            service = %self.service_name,
            db_operation = operation,
            collection = collection,
            duration_ms = duration_ms,
            "🗄️ Database operation recorded"
        );
    }
    
    /// Record circuit breaker events
    pub fn record_circuit_breaker_event(&self, endpoint: &str, state: &str) {
        if !self.enabled {
            return;
        }
        
        tracing::warn!(
            service = %self.service_name,
            endpoint = endpoint,
            circuit_state = state,
            "⚡ Circuit breaker state change"
        );
    }
    
    /// Record Prof Jiang framework analysis metrics
    pub fn record_prof_jiang_analysis(&self, topic: &str, relevance_score: f64) {
        if !self.enabled {
            return;
        }
        
        tracing::info!(
            service = %self.service_name,
            analysis_topic = topic,
            prof_jiang_relevance = relevance_score,
            "🧠 Prof Jiang framework analysis recorded"
        );
    }
}

/// Health check utilities for Hermes services
pub struct HealthChecker {
    service_name: String,
}

impl HealthChecker {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }
    
    /// Perform basic service health check
    pub fn health_check(&self) -> HealthStatus {
        // Basic health check - always healthy for now
        // In production, this would check database connectivity, 
        // external service availability, etc.
        
        tracing::debug!(
            service = %self.service_name,
            "🩺 Health check performed"
        );
        
        HealthStatus {
            service: self.service_name.clone(),
            healthy: true,
            checks: vec![
                HealthCheck {
                    name: "basic".to_string(),
                    status: "healthy".to_string(),
                    message: "Service operational".to_string(),
                },
            ],
        }
    }
    
    /// Check database connectivity health
    pub fn check_database_health(&self, connected: bool) -> HealthCheck {
        let (status, message) = if connected {
            ("healthy", "Database connection active")
        } else {
            ("unhealthy", "Database connection failed")
        };
        
        tracing::info!(
            service = %self.service_name,
            db_status = status,
            "🗄️ Database health check"
        );
        
        HealthCheck {
            name: "database".to_string(),
            status: status.to_string(),
            message: message.to_string(),
        }
    }
    
    /// Check Indonesian market data pipeline health
    pub fn check_indonesian_pipeline_health(&self, last_collection_age_minutes: Option<u64>) -> HealthCheck {
        let (status, message) = match last_collection_age_minutes {
            Some(age) if age < 60 => ("healthy", "Indonesian market data fresh".to_string()),
            Some(age) => ("degraded", format!("Indonesian market data {} minutes old", age)),
            None => ("unhealthy", "No Indonesian market data collected".to_string()),
        };
        
        tracing::info!(
            service = %self.service_name,
            pipeline_status = status,
            data_age_minutes = last_collection_age_minutes,
            "🇮🇩 Indonesian pipeline health check"
        );
        
        HealthCheck {
            name: "indonesian_pipeline".to_string(),
            status: status.to_string(),
            message: message.to_string(),
        }
    }
}

/// Health status representation
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub service: String,
    pub healthy: bool,
    pub checks: Vec<HealthCheck>,
}

/// Individual health check result
#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub name: String,
    pub status: String,
    pub message: String,
}

impl HealthStatus {
    /// Check if all health checks are passing
    pub fn is_healthy(&self) -> bool {
        self.healthy && self.checks.iter().all(|check| check.status == "healthy")
    }
    
    /// Get summary of health status
    pub fn summary(&self) -> String {
        let total_checks = self.checks.len();
        let healthy_checks = self.checks.iter().filter(|c| c.status == "healthy").count();
        
        format!(
            "{}: {}/{} checks healthy",
            self.service, healthy_checks, total_checks
        )
    }
}

/// Performance monitoring utilities
pub struct PerformanceMonitor {
    service_name: String,
}

impl PerformanceMonitor {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }
    
    /// Monitor Indonesian stock processing performance
    pub fn monitor_stock_processing(&self, stocks_processed: usize, duration_ms: u64) {
        let throughput = if duration_ms > 0 {
            (stocks_processed as f64 / duration_ms as f64) * 1000.0
        } else {
            0.0
        };
        
        tracing::info!(
            service = %self.service_name,
            stocks_processed = stocks_processed,
            duration_ms = duration_ms,
            throughput_per_second = %format!("{:.2}", throughput),
            "📊 Indonesian stock processing performance"
        );
    }
    
    /// Monitor news collection performance
    pub fn monitor_news_collection(&self, articles_collected: usize, sources: usize, duration_ms: u64) {
        let articles_per_second = if duration_ms > 0 {
            (articles_collected as f64 / duration_ms as f64) * 1000.0
        } else {
            0.0
        };
        
        tracing::info!(
            service = %self.service_name,
            articles_collected = articles_collected,
            sources_processed = sources,
            duration_ms = duration_ms,
            articles_per_second = %format!("{:.2}", articles_per_second),
            "📰 News collection performance"
        );
    }
    
    /// Monitor database operation performance
    pub fn monitor_db_performance(&self, operation: &str, records: usize, duration_ms: u64) {
        let records_per_second = if duration_ms > 0 {
            (records as f64 / duration_ms as f64) * 1000.0
        } else {
            0.0
        };
        
        tracing::info!(
            service = %self.service_name,
            db_operation = operation,
            records_processed = records,
            duration_ms = duration_ms,
            records_per_second = %format!("{:.2}", records_per_second),
            "🗄️ Database performance"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ObservabilityConfig;

    #[test]
    fn test_parse_log_level() {
        assert_eq!(parse_log_level("trace"), "trace");
        assert_eq!(parse_log_level("debug"), "debug");
        assert_eq!(parse_log_level("info"), "info");
        assert_eq!(parse_log_level("warn"), "warn");
        assert_eq!(parse_log_level("error"), "error");
        assert_eq!(parse_log_level("invalid"), "info"); // Default fallback
        assert_eq!(parse_log_level("DEBUG"), "debug"); // Case insensitive
    }

    #[test]
    fn test_hermes_metrics_creation() {
        let metrics = HermesMetrics::new("test-service".to_string(), true);
        assert_eq!(metrics.service_name, "test-service");
        assert!(metrics.enabled);
        
        let disabled_metrics = HermesMetrics::new("test-service".to_string(), false);
        assert!(!disabled_metrics.enabled);
    }

    #[test]
    fn test_health_checker_basic() {
        let checker = HealthChecker::new("test-service".to_string());
        let health = checker.health_check();
        
        assert_eq!(health.service, "test-service");
        assert!(health.healthy);
        assert!(!health.checks.is_empty());
        assert!(health.is_healthy());
    }

    #[test]
    fn test_database_health_check() {
        let checker = HealthChecker::new("test-service".to_string());
        
        let healthy_db = checker.check_database_health(true);
        assert_eq!(healthy_db.name, "database");
        assert_eq!(healthy_db.status, "healthy");
        
        let unhealthy_db = checker.check_database_health(false);
        assert_eq!(unhealthy_db.status, "unhealthy");
    }

    #[test]
    fn test_indonesian_pipeline_health() {
        let checker = HealthChecker::new("test-service".to_string());
        
        // Fresh data
        let fresh = checker.check_indonesian_pipeline_health(Some(30));
        assert_eq!(fresh.status, "healthy");
        
        // Stale data
        let stale = checker.check_indonesian_pipeline_health(Some(90));
        assert_eq!(stale.status, "degraded");
        
        // No data
        let no_data = checker.check_indonesian_pipeline_health(None);
        assert_eq!(no_data.status, "unhealthy");
    }

    #[test]
    fn test_health_status_summary() {
        let health_status = HealthStatus {
            service: "test-service".to_string(),
            healthy: true,
            checks: vec![
                HealthCheck {
                    name: "test1".to_string(),
                    status: "healthy".to_string(),
                    message: "OK".to_string(),
                },
                HealthCheck {
                    name: "test2".to_string(),
                    status: "healthy".to_string(),
                    message: "OK".to_string(),
                },
            ],
        };
        
        assert!(health_status.is_healthy());
        assert!(health_status.summary().contains("2/2 checks healthy"));
    }

    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new("test-service".to_string());
        
        // These methods primarily log, so we just test they don't panic
        monitor.monitor_stock_processing(10, 1000);
        monitor.monitor_news_collection(100, 5, 2000);
        monitor.monitor_db_performance("insert", 50, 500);
    }

    #[test]
    fn test_observability_config_integration() {
        let config = ObservabilityConfig {
            log_level: "debug".to_string(),
            metrics_enabled: true,
            metrics_port: 9090,
            tracing_enabled: true,
        };
        
        // Test initialization - this should not panic
        let result = init_observability(&config, "test-service");
        assert!(result.is_ok());
    }
}