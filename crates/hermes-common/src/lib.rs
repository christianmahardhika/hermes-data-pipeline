//! Hermes Common Library
//! 
//! Shared components for Hermes Data Pipeline services including:
//! - Configuration management
//! - Domain types and models  
//! - ArangoDB client utilities
//! - Observability primitives
//! - Error handling types

pub mod config;
pub mod types;
pub mod arangodb;
pub mod observability;

// Re-export commonly used types
pub use anyhow::{Error, Result};
pub use uuid::Uuid;