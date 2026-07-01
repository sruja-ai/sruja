//! Retry-capable client wrapper for transient LLM failures.

use super::{CompletionRequest, CompletionResponse, LlmClient, LlmError};
use std::sync::Arc;

/// Wraps any [`LlmClient`] with automatic retry logic for transient failures.
///
/// Retries on: `Network`, `RateLimited`, and 5xx `Api` errors.
/// Does **not** retry on: `BudgetExceeded`, `Auth`, `Deserialize`, or generic `Other`.
pub struct ResilientClient {
    inner: Arc<dyn LlmClient>,
    max_retries: u32,
}

impl ResilientClient {
    /// Create a new resilient wrapper.
    ///
    /// `max_retries` is the maximum number of *retries* (so total attempts
    /// = max_retries + 1).
    pub fn new(inner: Arc<dyn LlmClient>, max_retries: u32) -> Self {
        Self {
            inner,
            max_retries,
        }
    }
}

/// Determine whether an error is transient and worth retrying.
fn is_retryable(err: &LlmError) -> bool {
    match err {
        LlmError::Network(_) => true,
        LlmError::RateLimited { .. } => true,
        LlmError::Api { status, .. } => *status >= 500,
        // BudgetExceeded, Auth, Deserialize, Other → no retry
        _ => false,
    }
}

#[async_trait::async_trait]
impl LlmClient for ResilientClient {
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let mut last_err = None;
        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                // Exponential backoff: 100ms, 200ms, 400ms, …
                let delay_ms = 100 * 2u64.pow(attempt - 1);
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            }
            match self.inner.complete(req).await {
                Ok(resp) => return Ok(resp),
                Err(e) if is_retryable(&e) && attempt < self.max_retries => {
                    last_err = Some(e);
                }
                Err(e) => return Err(e),
            }
        }
        Err(last_err.unwrap_or_else(|| LlmError::Other("retry exhausted".into())))
    }

    fn default_model(&self) -> &str {
        self.inner.default_model()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FailingClient {
        fail_count: AtomicUsize,
        fail_until: usize,
    }

    #[async_trait]
    impl LlmClient for FailingClient {
        async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
            let attempt = self.fail_count.fetch_add(1, Ordering::SeqCst);
            if attempt < self.fail_until {
                Err(LlmError::Network("transient".into()))
            } else {
                Ok(CompletionResponse {
                    content: "ok".into(),
                    tool_calls: vec![],
                    usage: super::super::Usage {
                        prompt_tokens: 100,
                        completion_tokens: 50,
                        total_tokens: 150,
                    },
                    model: "test".into(),
                    finish_reason: super::super::FinishReason::Stop,
                })
            }
        }

        fn default_model(&self) -> &str {
            "test"
        }
    }

    #[tokio::test]
    async fn retries_on_network_error() {
        let client = FailingClient {
            fail_count: AtomicUsize::new(0),
            fail_until: 2, // Fail first 2 attempts, succeed on 3rd
        };
        let resilient = ResilientClient::new(Arc::new(client), 3);
        let req = CompletionRequest::new(vec![]);
        let resp = resilient.complete(&req).await.unwrap();
        assert_eq!(resp.content, "ok");
    }

    #[tokio::test]
    async fn does_not_retry_budget_exceeded() {
        struct BudgetClient;
        #[async_trait]
        impl LlmClient for BudgetClient {
            async fn complete(
                &self,
                _req: &CompletionRequest,
            ) -> Result<CompletionResponse, LlmError> {
                Err(LlmError::BudgetExceeded {
                    spent: 10.0,
                    cap: 5.0,
                })
            }
            fn default_model(&self) -> &str {
                "test"
            }
        }
        let resilient = ResilientClient::new(Arc::new(BudgetClient), 3);
        let req = CompletionRequest::new(vec![]);
        let err = resilient.complete(&req).await.unwrap_err();
        assert!(matches!(err, LlmError::BudgetExceeded { .. }));
    }

    #[tokio::test]
    async fn does_not_retry_auth_error() {
        struct AuthClient;
        #[async_trait]
        impl LlmClient for AuthClient {
            async fn complete(
                &self,
                _req: &CompletionRequest,
            ) -> Result<CompletionResponse, LlmError> {
                Err(LlmError::Auth)
            }
            fn default_model(&self) -> &str {
                "test"
            }
        }
        let resilient = ResilientClient::new(Arc::new(AuthClient), 3);
        let req = CompletionRequest::new(vec![]);
        let err = resilient.complete(&req).await.unwrap_err();
        assert!(matches!(err, LlmError::Auth));
    }

    #[tokio::test]
    async fn exhausts_retries() {
        struct AlwaysFail;
        #[async_trait]
        impl LlmClient for AlwaysFail {
            async fn complete(
                &self,
                _req: &CompletionRequest,
            ) -> Result<CompletionResponse, LlmError> {
                Err(LlmError::Network("down".into()))
            }
            fn default_model(&self) -> &str {
                "test"
            }
        }
        let resilient = ResilientClient::new(Arc::new(AlwaysFail), 2);
        let req = CompletionRequest::new(vec![]);
        let err = resilient.complete(&req).await.unwrap_err();
        assert!(matches!(err, LlmError::Network(_)));
    }
}
