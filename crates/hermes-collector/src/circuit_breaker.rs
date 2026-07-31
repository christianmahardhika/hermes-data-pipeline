//! Circuit breaker implementation for RSS feed resilience

use std::time::{Duration, Instant};
use tracing::{warn, info, debug};

/// Circuit breaker states for RSS feed protection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Failing, blocking requests
    HalfOpen,  // Testing if service recovered
}

/// Circuit breaker for RSS feed collection resilience
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub name: String,
    pub state: CircuitState,
    pub failure_count: u32,
    pub failure_threshold: u32,
    pub timeout: Duration,
    pub last_failure_time: Option<Instant>,
    pub success_count_in_half_open: u32,
    pub required_successes: u32,
}

impl CircuitBreaker {
    /// Create a new circuit breaker for RSS feeds
    pub fn new(name: String, failure_threshold: u32, timeout_seconds: u64) -> Self {
        Self {
            name,
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            timeout: Duration::from_secs(timeout_seconds),
            last_failure_time: None,
            success_count_in_half_open: 0,
            required_successes: 3, // Need 3 successes to fully close
        }
    }

    /// Check if request should be allowed through circuit breaker
    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if self.should_attempt_reset() {
                    self.transition_to_half_open();
                    true
                } else {
                    debug!(
                        circuit_breaker = %self.name,
                        state = "open",
                        "🚫 Circuit breaker blocking request"
                    );
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record successful RSS feed collection
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                // Reset failure count on success
                if self.failure_count > 0 {
                    debug!(
                        circuit_breaker = %self.name,
                        previous_failures = self.failure_count,
                        "✅ Circuit breaker success - resetting failure count"
                    );
                    self.failure_count = 0;
                }
            }
            CircuitState::HalfOpen => {
                self.success_count_in_half_open += 1;
                info!(
                    circuit_breaker = %self.name,
                    successes = self.success_count_in_half_open,
                    required = self.required_successes,
                    "🔄 Circuit breaker half-open success"
                );
                
                if self.success_count_in_half_open >= self.required_successes {
                    self.transition_to_closed();
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
                warn!(
                    circuit_breaker = %self.name,
                    "⚠️ Unexpected success while circuit breaker is open"
                );
            }
        }
    }

    /// Record failed RSS feed collection
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        match self.state {
            CircuitState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    self.transition_to_open();
                } else {
                    warn!(
                        circuit_breaker = %self.name,
                        failures = self.failure_count,
                        threshold = self.failure_threshold,
                        "⚠️ Circuit breaker failure recorded"
                    );
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open goes back to open
                warn!(
                    circuit_breaker = %self.name,
                    "❌ Circuit breaker half-open test failed - reopening"
                );
                self.transition_to_open();
            }
            CircuitState::Open => {
                debug!(
                    circuit_breaker = %self.name,
                    "Circuit breaker already open - ignoring failure"
                );
            }
        }
    }

    /// Check if enough time has passed to attempt reset
    fn should_attempt_reset(&self) -> bool {
        if let Some(last_failure) = self.last_failure_time {
            last_failure.elapsed() >= self.timeout
        } else {
            false
        }
    }

    /// Transition to half-open state for testing
    fn transition_to_half_open(&mut self) {
        info!(
            circuit_breaker = %self.name,
            timeout_seconds = self.timeout.as_secs(),
            "🔄 Circuit breaker transitioning to half-open"
        );
        self.state = CircuitState::HalfOpen;
        self.success_count_in_half_open = 0;
    }

    /// Transition to closed state (fully recovered)
    fn transition_to_closed(&mut self) {
        info!(
            circuit_breaker = %self.name,
            "✅ Circuit breaker closed - service recovered"
        );
        self.state = CircuitState::Closed;
        self.failure_count = 0;
        self.success_count_in_half_open = 0;
    }

    /// Transition to open state (service failing)
    fn transition_to_open(&mut self) {
        warn!(
            circuit_breaker = %self.name,
            failures = self.failure_count,
            timeout_seconds = self.timeout.as_secs(),
            "🔴 Circuit breaker opened - blocking requests"
        );
        self.state = CircuitState::Open;
    }

    /// Get current circuit breaker metrics
    pub fn metrics(&self) -> CircuitBreakerMetrics {
        CircuitBreakerMetrics {
            name: self.name.clone(),
            state: self.state.clone(),
            failure_count: self.failure_count,
            failure_threshold: self.failure_threshold,
            timeout_seconds: self.timeout.as_secs(),
            time_since_last_failure_seconds: self.last_failure_time
                .map(|t| t.elapsed().as_secs())
                .unwrap_or(0),
        }
    }
}

/// Circuit breaker metrics for monitoring
#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    pub name: String,
    pub state: CircuitState,
    pub failure_count: u32,
    pub failure_threshold: u32,
    pub timeout_seconds: u64,
    pub time_since_last_failure_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_circuit_breaker_creation() {
        let mut cb = CircuitBreaker::new("test_feed".to_string(), 3, 60);
        assert_eq!(cb.name, "test_feed");
        assert_eq!(cb.state, CircuitState::Closed);
        assert_eq!(cb.failure_threshold, 3);
        assert_eq!(cb.timeout, Duration::from_secs(60));
        assert!(cb.can_execute());
    }

    #[test]
    fn test_circuit_breaker_failure_threshold() {
        let mut cb = CircuitBreaker::new("test_feed".to_string(), 2, 60);
        
        // Initially closed, allows execution
        assert_eq!(cb.state, CircuitState::Closed);
        assert!(cb.can_execute());
        
        // First failure - still closed
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Closed);
        assert!(cb.can_execute());
        
        // Second failure - opens circuit
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Open);
        assert!(!cb.can_execute());
    }

    #[test]
    fn test_circuit_breaker_success_reset() {
        let mut cb = CircuitBreaker::new("test_feed".to_string(), 3, 60);
        
        // Record failures but not enough to open
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.failure_count, 2);
        assert_eq!(cb.state, CircuitState::Closed);
        
        // Success should reset failure count
        cb.record_success();
        assert_eq!(cb.failure_count, 0);
        assert_eq!(cb.state, CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_half_open_success() {
        let mut cb = CircuitBreaker::new("test_feed".to_string(), 1, 1); // Fast timeout
        
        // Trip circuit breaker
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Open);
        
        // Wait for timeout and transition to half-open
        sleep(Duration::from_millis(1100));
        assert!(cb.can_execute()); // This should transition to half-open
        assert_eq!(cb.state, CircuitState::HalfOpen);
        
        // Need 3 successes to close
        cb.record_success();
        assert_eq!(cb.state, CircuitState::HalfOpen);
        cb.record_success();
        assert_eq!(cb.state, CircuitState::HalfOpen);
        cb.record_success();
        assert_eq!(cb.state, CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_half_open_failure() {
        let mut cb = CircuitBreaker::new("test_feed".to_string(), 1, 1);
        
        // Trip circuit breaker
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Open);
        
        // Wait and transition to half-open
        sleep(Duration::from_millis(1100));
        assert!(cb.can_execute());
        assert_eq!(cb.state, CircuitState::HalfOpen);
        
        // Failure in half-open should reopen circuit
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Open);
        assert!(!cb.can_execute());
    }

    #[test]
    fn test_circuit_breaker_metrics() {
        let mut cb = CircuitBreaker::new("kompas_rss".to_string(), 3, 120);
        cb.record_failure();
        
        let metrics = cb.metrics();
        assert_eq!(metrics.name, "kompas_rss");
        assert_eq!(metrics.state, CircuitState::Closed);
        assert_eq!(metrics.failure_count, 1);
        assert_eq!(metrics.failure_threshold, 3);
        assert_eq!(metrics.timeout_seconds, 120);
        assert!(metrics.time_since_last_failure_seconds < 5); // Should be very recent
    }
}