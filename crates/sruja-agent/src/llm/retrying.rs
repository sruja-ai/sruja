//! Retry wrapper for LLM clients.
//!
//! Wraps any [`LlmClient`] and retries transient failures (rate limits,
//! server errors, network timeouts) with exponential backoff. Non-transient
//! errors (auth, budget) fail immediately.

use std::sync::Arc;
use std::time::Duration;

use super::{CompletionRequest, CompletionResponse, LlmClient, LlmError, Stream};

/// Configuration for the retry client.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries (default: 3).
    pub max_retries: u32,
    /// Base delay between retries (default: 1s). Doubles each attempt.
    pub base_delay: Duration,
    /// Maximum delay cap (default: 30s).
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    /// Returns a [`RetryConfig`] with 3 retries, 1 s base delay, and 30 s max delay.
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
        }
    }
}

/// Wraps an [`LlmClient`] with automatic retry on transient errors.
///
/// Transient errors that trigger retries:
/// - `RateLimited` (429)
/// - `Api` with status 500, 502, 503, 504
/// - `Network` errors (timeouts, connection refused)
///
/// Non-transient errors fail immediately:
/// - `Auth` (401/403)
/// - `BudgetExceeded`
/// - `Deserialize` errors
pub struct RetryingClient {
    inner: Arc<dyn LlmClient>,
    config: RetryConfig,
}

impl RetryingClient {
    /// Wrap an LLM client with default retry config.
    pub fn new(inner: Arc<dyn LlmClient>) -> Self {
        Self::with_config(inner, RetryConfig::default())
    }

    /// Wrap an LLM client with custom retry config.
    pub fn with_config(inner: Arc<dyn LlmClient>, config: RetryConfig) -> Self {
        Self { inner, config }
    }

    /// Check if an error is transient and should be retried.
    fn is_transient(err: &LlmError) -> bool {
        match err {
            LlmError::RateLimited { .. } => true,
            LlmError::Api { status, .. } => matches!(status, 500 | 502 | 503 | 504),
            LlmError::Network(_) => true,
            _ => false,
        }
    }

    /// Compute the delay for a given attempt (exponential backoff with jitter).
    fn retry_delay(&self, attempt: u32) -> Duration {
        let base = self.config.base_delay.as_millis() as u64;
        let delay_ms = base.saturating_mul(1u64 << attempt.min(5));
        let capped = delay_ms.min(self.config.max_delay.as_millis() as u64);
        // Add small jitter (0-250ms) to avoid thundering herd
        let jitter = (attempt as u64 * 73) % 250;
        Duration::from_millis(capped + jitter)
    }

    /// Extract retry-after hint from RateLimited error.
    fn retry_after_ms(err: &LlmError) -> Option<u64> {
        match err {
            LlmError::RateLimited { retry_after_ms } => *retry_after_ms,
            _ => None,
        }
    }

    /// Execute a completion request with automatic retry on transient errors.
    ///
    /// Retries up to [`RetryConfig::max_retries`] times with exponential backoff,
    /// respecting server-provided `retry-after` hints when present.
    async fn complete_with_retry(
        &self,
        req: &CompletionRequest,
    ) -> Result<CompletionResponse, LlmError> {
        let mut last_err = None;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let delay = Self::retry_delay(self, attempt - 1);
                // Use server-provided retry-after if available, otherwise use backoff
                let actual_delay = last_err
                    .as_ref()
                    .and_then(Self::retry_after_ms)
                    .map(Duration::from_millis)
                    .unwrap_or(delay);

                tracing::warn!(
                    attempt,
                    delay_ms = actual_delay.as_millis() as u64,
                    error = last_err.as_ref().map(|e| e.to_string()).unwrap_or_default(),
                    "retrying LLM request after transient error"
                );
                tokio::time::sleep(actual_delay).await;
            }

            match self.inner.complete(req).await {
                Ok(resp) => {
                    if attempt > 0 {
                        tracing::info!(attempt, "LLM request succeeded after retry");
                    }
                    return Ok(resp);
                }
                Err(err) => {
                    if Self::is_transient(&err) && attempt < self.config.max_retries {
                        last_err = Some(err);
                        continue;
                    }
                    return Err(err);
                }
            }
        }

        Err(last_err
            .unwrap_or_else(|| LlmError::Other("retry loop exhausted without error".into())))
    }
}

#[async_trait::async_trait]
impl LlmClient for RetryingClient {
    /// Delegate to [`RetryingClient::complete_with_retry`], which retries
    /// transient failures with exponential backoff before surfacing an error.
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        self.complete_with_retry(req).await
    }

    /// Return the default model identifier from the wrapped inner client.
    fn default_model(&self) -> &str {
        self.inner.default_model()
    }

    /// Stream a completion from the inner client without retry.
    ///
    /// Streaming responses are not retried because the caller owns the stream
    /// lifecycle and must handle mid-stream failures itself.
    fn complete_stream<'a>(&'a self, req: &'a CompletionRequest) -> Stream<'a> {
        // Streaming doesn't retry — the caller manages the stream.
        // Fall back to the inner client's streaming implementation.
        self.inner.complete_stream(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{FinishReason, Usage};

    struct TransientFailClient {
        fail_count: std::sync::atomic::AtomicU32,
    }

    #[async_trait::async_trait]
    impl LlmClient for TransientFailClient {
        async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
            let fails = self
                .fail_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if fails < 2 {
                Err(LlmError::RateLimited {
                    retry_after_ms: Some(10), // very short for tests
                })
            } else {
                Ok(CompletionResponse {
                    content: "ok".into(),
                    tool_calls: vec![],
                    usage: Usage::default(),
                    model: "test".into(),
                    finish_reason: FinishReason::Stop,
                })
            }
        }

        fn default_model(&self) -> &str {
            "test"
        }
    }

    struct NonTransientFailClient;

    #[async_trait::async_trait]
    impl LlmClient for NonTransientFailClient {
        async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
            Err(LlmError::Auth)
        }

        fn default_model(&self) -> &str {
            "test"
        }
    }

    #[tokio::test]
    async fn retries_transient_errors() {
        let inner = Arc::new(TransientFailClient {
            fail_count: std::sync::atomic::AtomicU32::new(0),
        });
        let client = RetryingClient::with_config(
            inner,
            RetryConfig {
                max_retries: 3,
                base_delay: Duration::from_millis(10),
                max_delay: Duration::from_millis(50),
            },
        );

        let req = CompletionRequest::new(vec![]);
        let result = client.complete(&req).await;
        assert!(result.is_ok(), "should succeed after retries");
    }

    #[tokio::test]
    async fn does_not_retry_auth_errors() {
        let inner = Arc::new(NonTransientFailClient);
        let client = RetryingClient::with_config(
            inner,
            RetryConfig {
                max_retries: 3,
                base_delay: Duration::from_millis(10),
                max_delay: Duration::from_millis(50),
            },
        );

        let req = CompletionRequest::new(vec![]);
        let result = client.complete(&req).await;
        assert!(matches!(result, Err(LlmError::Auth)));
    }

    #[test]
    fn default_retry_config_values() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay, Duration::from_secs(1));
        assert_eq!(config.max_delay, Duration::from_secs(30));
    }
}
