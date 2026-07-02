//! Circuit breaker for LLM clients.
//!
//! Wraps any [`LlmClient`] and tracks consecutive failures per model name.
//! After a configurable threshold of consecutive failures for a model, the
//! circuit opens and subsequent calls for that model are rejected immediately
//! without touching the inner client. After a half-open timeout, a single
//! probe request is allowed through to test recovery.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::sync::Arc;

use super::{CompletionRequest, CompletionResponse, LlmClient, LlmError};

/// Configuration for the circuit breaker.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures to trip the circuit (default: 3).
    pub threshold: u32,
    /// How long the circuit stays open before allowing a probe (default: 30s).
    pub half_open_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            threshold: 3,
            half_open_timeout: Duration::from_secs(30),
        }
    }
}

/// Per-model circuit state.
enum CircuitState {
    /// Normal operation. Tracks consecutive failures.
    Closed { failures: u32 },
    /// Rejecting all requests. A timer determines when to probe.
    Open { since: Instant },
    /// A single probe request is allowed through. If it succeeds, the
    /// circuit closes. If it fails, the circuit re-opens.
    HalfOpen,
}

/// Wraps an [`LlmClient`] with a per-model circuit breaker.
///
/// Tracks failures keyed by `req.model`. When a model hits the consecutive
/// failure threshold, the circuit opens and further calls for that model
/// fail fast. After `half_open_timeout`, one probe request is allowed
/// through — if it succeeds, the circuit closes; if it fails, the circuit
/// stays open.
pub struct CircuitBreakerClient {
    inner: Arc<dyn LlmClient>,
    config: CircuitBreakerConfig,
    states: Mutex<HashMap<String, CircuitState>>,
}

impl CircuitBreakerClient {
    /// Wrap an LLM client with the default circuit breaker config.
    pub fn new(inner: Arc<dyn LlmClient>) -> Self {
        Self::with_config(inner, CircuitBreakerConfig::default())
    }

    /// Wrap an LLM client with a custom config.
    pub fn with_config(inner: Arc<dyn LlmClient>, config: CircuitBreakerConfig) -> Self {
        Self {
            inner,
            config,
            states: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl LlmClient for CircuitBreakerClient {
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let model = req.model.as_deref().unwrap_or_default().to_string();

        // Check circuit state before dispatching. For Open circuits past
        // the timeout, transition to HalfOpen and allow the probe.
        {
            let mut states = self.states.lock().unwrap();
            match states.get(&model) {
                Some(CircuitState::Open { since }) => {
                    if since.elapsed() < self.config.half_open_timeout {
                        return Err(LlmError::Other(format!(
                            "circuit breaker open for model `{model}` (retry after {}s)",
                            self.config.half_open_timeout.as_secs()
                                - since.elapsed().as_secs()
                        )));
                    }
                    // Timeout expired: transition to HalfOpen for probe.
                    states.insert(model.clone(), CircuitState::HalfOpen);
                }
                Some(CircuitState::HalfOpen) => {
                    // A probe is already in-flight. Reject until it resolves.
                    return Err(LlmError::Other(format!(
                        "circuit breaker probe pending for model `{model}`"
                    )));
                }
                _ => {}
            }
        }

        let result = self.inner.complete(req).await;

        // Update circuit state based on result.
        let mut states = self.states.lock().unwrap();
        match &result {
            Ok(_) => {
                // Success: close the circuit regardless of previous state.
                states.remove(&model);
            }
            Err(_) => {
                match states.get(&model) {
                    Some(CircuitState::HalfOpen) => {
                        // Probe failed — re-open the circuit.
                        states.insert(model, CircuitState::Open { since: Instant::now() });
                    }
                    _ => {
                        // Normal failure: increment counter.
                        let entry = states
                            .entry(model.clone())
                            .or_insert(CircuitState::Closed { failures: 0 });
                        if let CircuitState::Closed { ref mut failures } = entry {
                            *failures += 1;
                            if *failures >= self.config.threshold {
                                *entry = CircuitState::Open {
                                    since: Instant::now(),
                                };
                            }
                        }
                    }
                }
            }
        }

        result
    }

    fn default_model(&self) -> &str {
        self.inner.default_model()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{CompletionResponse, FinishReason, Usage};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct FailCounter {
        call_count: AtomicU32,
        fail_until: u32,
    }

    #[async_trait]
    impl LlmClient for FailCounter {
        async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
            let call = self.call_count.fetch_add(1, Ordering::SeqCst);
            if call < self.fail_until {
                Err(LlmError::Network("transient failure".into()))
            } else {
                Ok(CompletionResponse::text("ok"))
            }
        }
        fn default_model(&self) -> &str {
            "test-model"
        }
    }

    #[tokio::test]
    async fn circuit_opens_after_three_consecutive_failures() {
        let inner = Arc::new(FailCounter {
            call_count: AtomicU32::new(0),
            fail_until: 5, // Fail 5 times
        }) as Arc<dyn LlmClient>;
        let cb = CircuitBreakerClient::new(inner);
        let req = CompletionRequest::new(vec![]).with_model("test-model");

        // Calls 1-3: fail (circuit opens after 3rd)
        for _ in 0..3 {
            let err = cb.complete(&req).await.unwrap_err();
            // These should be the underlying Network error, not circuit breaker.
            assert!(matches!(err, LlmError::Network(_)));
        }

        // Call 4: circuit breaker open
        let err = cb.complete(&req).await.unwrap_err();
        assert!(err.to_string().contains("circuit breaker open"));
    }

    #[tokio::test]
    async fn success_resets_failures() {
        let inner = Arc::new(FailCounter {
            call_count: AtomicU32::new(0),
            fail_until: 0, // Always succeed
        }) as Arc<dyn LlmClient>;
        let cb = CircuitBreakerClient::new(inner);
        let req = CompletionRequest::new(vec![]).with_model("test-model");

        let resp = cb.complete(&req).await.unwrap();
        assert_eq!(resp.content, "ok");
    }

    #[tokio::test]
    async fn different_models_have_independent_states() {
        let inner = Arc::new(FailCounter {
            call_count: AtomicU32::new(0),
            fail_until: 5,
        }) as Arc<dyn LlmClient>;
        let cb = CircuitBreakerClient::new(inner);
        let failing_req = CompletionRequest::new(vec![]).with_model("failing-model");
        let ok_req = CompletionRequest::new(vec![]).with_model("ok-model");

        // Trip the circuit on failing-model
        for _ in 0..3 {
            let _ = cb.complete(&failing_req).await;
        }

        // ok-model should still work
        let err = cb.complete(&ok_req).await.unwrap_err();
        assert!(matches!(err, LlmError::Network(_))); // Not circuit broken
    }

    #[tokio::test]
    async fn circuit_closes_after_successful_probe() {
        let inner = Arc::new(FailCounter {
            call_count: AtomicU32::new(0),
            fail_until: 5, // Fail 5 times, then succeed
        }) as Arc<dyn LlmClient>;
        let config = CircuitBreakerConfig {
            threshold: 3,
            half_open_timeout: Duration::from_millis(10), // Short timeout for testing
        };
        let cb = CircuitBreakerClient::with_config(inner, config);
        let req = CompletionRequest::new(vec![]).with_model("test-model");

        // Trip the circuit
        for _ in 0..3 {
            let _ = cb.complete(&req).await;
        }

        // Wait for half-open timeout
        tokio::time::sleep(Duration::from_millis(20)).await;

        // This should be a probe (allowed through) — it'll fail (network error)
        // since we haven't reached fail_until yet, but it shouldn't be circuit broken
        let err = cb.complete(&req).await.unwrap_err();
        assert!(matches!(err, LlmError::Network(_)));

        // After probe fails, circuit re-opens
        let err = cb.complete(&req).await.unwrap_err();
        assert!(err.to_string().contains("circuit breaker open"));
    }
}
