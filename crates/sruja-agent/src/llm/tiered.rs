//! Multi-provider client that routes by model name.
//!
//! When different task tiers use different providers (e.g. cheap on z.ai,
//! premium on OpenRouter), a single [`LlmClient`] cannot serve all tiers —
//! the endpoint and API key differ per provider. [`TieredClient`] solves
//! this by holding a map of model-name → client and dispatching each
//! request to the correct provider.

use std::collections::HashMap;
use std::sync::Arc;

use super::{CompletionRequest, CompletionResponse, LlmClient, LlmError};

/// Routes requests to different LLM clients based on `req.model`.
///
/// Falls back to a default client when the model is unknown or unset.
pub struct TieredClient {
    default: Arc<dyn LlmClient>,
    by_model: HashMap<String, Arc<dyn LlmClient>>,
}

impl TieredClient {
    /// Create with a fallback client used when no model-specific route matches.
    pub fn new(default: Arc<dyn LlmClient>) -> Self {
        Self {
            default,
            by_model: HashMap::new(),
        }
    }

    /// Register a client for a specific model name.
    ///
    /// When a request has `model` matching this name, it is sent to `client`
    /// instead of the default. This lets each task tier hit its own provider.
    pub fn with_route(mut self, model: impl Into<String>, client: Arc<dyn LlmClient>) -> Self {
        self.by_model.insert(model.into(), client);
        self
    }
}

#[async_trait::async_trait]
impl LlmClient for TieredClient {
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        if let Some(model) = req.model.as_deref() {
            if let Some(client) = self.by_model.get(model) {
                return client.complete(req).await;
            }
        }
        self.default.complete(req).await
    }

    fn default_model(&self) -> &str {
        self.default.default_model()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{CompletionResponse, FinishReason, Usage};

    struct FakeClient {
        name: &'static str,
        model: &'static str,
    }

    #[async_trait::async_trait]
    impl LlmClient for FakeClient {
        async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
            Ok(CompletionResponse {
                content: self.name.into(),
                tool_calls: vec![],
                usage: Usage::default(),
                model: self.model.into(),
                finish_reason: FinishReason::Stop,
            })
        }
        fn default_model(&self) -> &str {
            self.model
        }
    }

    #[tokio::test]
    async fn routes_to_correct_client() {
        let default = Arc::new(FakeClient {
            name: "default",
            model: "gpt-4o-mini",
        });
        let premium = Arc::new(FakeClient {
            name: "premium",
            model: "claude-sonnet-4",
        });

        let tiered = TieredClient::new(default).with_route("claude-sonnet-4", premium);

        // Request with a routed model → hits the premium client
        let req = CompletionRequest::new(vec![]).with_model("claude-sonnet-4");
        let resp = tiered.complete(&req).await.unwrap();
        assert_eq!(resp.content, "premium");

        // Request with an unrouted model → hits the default client
        let req = CompletionRequest::new(vec![]).with_model("gpt-4o-mini");
        let resp = tiered.complete(&req).await.unwrap();
        assert_eq!(resp.content, "default");

        // Request with no model → hits the default client
        let req = CompletionRequest::new(vec![]);
        let resp = tiered.complete(&req).await.unwrap();
        assert_eq!(resp.content, "default");
    }
}
