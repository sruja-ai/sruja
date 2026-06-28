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
///
/// Two routing mechanisms:
/// - **Exact match** (`by_model`): model name → client. Used for tier models.
/// - **Name substring match** (`by_name_substring`): if the model name
///   *contains* a registered substring, route to that provider's client. This
///   lets persona model overrides (e.g. "mimo-v2.5") find the right provider
///   without registering every variant.
pub struct TieredClient {
    default: Arc<dyn LlmClient>,
    by_model: HashMap<String, Arc<dyn LlmClient>>,
    by_name_substring: Vec<(String, Arc<dyn LlmClient>)>,
}

impl TieredClient {
    /// Create with a fallback client used when no model-specific route matches.
    pub fn new(default: Arc<dyn LlmClient>) -> Self {
        Self {
            default,
            by_model: HashMap::new(),
            by_name_substring: Vec::new(),
        }
    }

    /// Register a client for a specific model name (exact match).
    ///
    /// When a request has `model` matching this name, it is sent to `client`
    /// instead of the default. This lets each task tier hit its own provider.
    pub fn with_route(mut self, model: impl Into<String>, client: Arc<dyn LlmClient>) -> Self {
        self.by_model.insert(model.into(), client);
        self
    }

    /// Register a client for a provider identified by a substring in model names.
    ///
    /// When a request has `model` *containing* `substring` (case-insensitive),
    /// it is routed to `client`. This lets persona model overrides like
    /// "mimo-v2.5" find the ximimo provider without registering every variant.
    pub fn with_provider_name_contains(
        mut self,
        substring: impl Into<String>,
        client: Arc<dyn LlmClient>,
    ) -> Self {
        self.by_name_substring.push((substring.into(), client));
        self
    }
}

#[async_trait::async_trait]
impl LlmClient for TieredClient {
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        if let Some(model) = req.model.as_deref() {
            // Exact match first (fast path — tier models).
            if let Some(client) = self.by_model.get(model) {
                return client.complete(req).await;
            }
            // Name substring match (for persona model overrides).
            let model_lower = model.to_lowercase();
            for (substring, client) in &self.by_name_substring {
                if model_lower.contains(&substring.to_lowercase()) {
                    return client.complete(req).await;
                }
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

    #[tokio::test]
    async fn model_name_contains_routes_to_correct_client() {
        let default = Arc::new(FakeClient {
            name: "zai",
            model: "GLM-5.2",
        });
        let ximimo = Arc::new(FakeClient {
            name: "ximimo",
            model: "mimo-v2.5-pro",
        });

        // "mimo" is registered as a name substring — matches "mimo-v2.5-pro",
        // "mimo-v2.5", etc. by containment.
        let tiered = TieredClient::new(default).with_provider_name_contains("mimo", ximimo);

        // Exact tier model "mimo-v2.5-pro" → hits ximimo via prefix
        let req = CompletionRequest::new(vec![]).with_model("mimo-v2.5-pro");
        let resp = tiered.complete(&req).await.unwrap();
        assert_eq!(resp.content, "ximimo");

        // Persona variant "mimo-v2.5" → also hits ximimo via prefix
        let req = CompletionRequest::new(vec![]).with_model("mimo-v2.5");
        let resp = tiered.complete(&req).await.unwrap();
        assert_eq!(resp.content, "ximimo");

        // GLM model → stays on default (no prefix match)
        let req = CompletionRequest::new(vec![]).with_model("GLM-5.1");
        let resp = tiered.complete(&req).await.unwrap();
        assert_eq!(resp.content, "zai");
    }

    #[tokio::test]
    async fn prefix_is_case_insensitive() {
        let default = Arc::new(FakeClient { name: "def", model: "x" });
        let other = Arc::new(FakeClient { name: "other", model: "y" });

        let tiered = TieredClient::new(default).with_provider_name_contains("GLM", other);

        // Lowercase model name should still match uppercase prefix
        let req = CompletionRequest::new(vec![]).with_model("glm-4.7");
        let resp = tiered.complete(&req).await.unwrap();
        assert_eq!(resp.content, "other");
    }

    #[tokio::test]
    async fn exact_route_beats_prefix() {
        let default = Arc::new(FakeClient { name: "def", model: "x" });
        let exact = Arc::new(FakeClient { name: "exact", model: "y" });
        let prefix = Arc::new(FakeClient { name: "prefix", model: "z" });

        let tiered = TieredClient::new(default)
            .with_route("mimo-v2.5-pro", exact.clone())
            .with_provider_name_contains("mimo", prefix);

        // Exact match wins over prefix
        let req = CompletionRequest::new(vec![]).with_model("mimo-v2.5-pro");
        let resp = tiered.complete(&req).await.unwrap();
        assert_eq!(resp.content, "exact");
    }
}
