//! Observability primitives for Hermes services
//!
//! Provides standardized logging, metrics, and tracing setup
//! for all services in the pipeline.

use tracing::{info, warn};

/// Initialize observability for a service
pub fn init_observability(service_name: &str) {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();
    
    info!("Observability initialized for service: {}", service_name);
}

/// Log a warning with service context
pub fn log_warning(service: &str, message: &str) {
    warn!(service = service, "{}", message);
}