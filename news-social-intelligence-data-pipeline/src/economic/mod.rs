//! Economic data collection module
//!
//! Collects commodities, crypto, interest rates, inflation data
//! and stores in ArangoDB for intelligence fusion.
//!
//! Storage functions for ArangoDB will be wired in when the
//! `arangodb` module is available (see Task 8).

pub mod bank_indonesia;
pub mod coingecko;
pub mod fred;
pub mod gdelt;
pub mod models;
pub mod yahoo_commodities;

pub use models::{EconomicIndicator, EconomicSource, EconomicStats, SignalSourceEdge};
pub use yahoo_commodities::YahooCommodityCollector;
