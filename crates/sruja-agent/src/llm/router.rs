//! Cost-aware model router.
//!
//! Wraps any [`LlmClient`] and adds:
//! - Tiered model selection (cheap / mid / premium) per subtask.
//! - Cumulative spend tracking with optional USD cap.
//! - Per-model pricing for accurate cost attribution.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::{CompletionRequest, CompletionResponse, LlmClient, LlmError};

/// Complexity tier for routing a subtask to the right model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum TaskTier {
    /// Classification, summarisation, simple extraction.
    /// e.g. Haiku, GPT-4o-mini, Flash, local Ollama.
    Cheap,
    /// Standard coding and analysis.
    /// e.g. Sonnet, GPT-4.1-mini.
    #[default]
    Mid,
    /// Hard architectural reasoning.
    /// e.g. Opus, o3, GPT-4.1.
    Premium,
}

/// USD pricing per 1M tokens for a model.
#[derive(Debug, Clone, Copy)]
pub struct Pricing {
    pub input_per_1m: f64,
    pub output_per_1m: f64,
}

impl Pricing {
    pub fn free() -> Self {
        Self {
            input_per_1m: 0.0,
            output_per_1m: 0.0,
        }
    }

    pub fn cost(&self, input_tokens: u64, output_tokens: u64) -> f64 {
        (input_tokens as f64 / 1_000_000.0) * self.input_per_1m
            + (output_tokens as f64 / 1_000_000.0) * self.output_per_1m
    }
}

/// Router configuration: tier → model, model → pricing.
#[derive(Debug, Clone)]
pub struct RouterConfig {
    pub tiers: HashMap<TaskTier, String>,
    pub pricing: HashMap<String, Pricing>,
    pub spend_cap_usd: Option<f64>,
    /// Fallback pricing used when a model is not found in the primary pricing map.
    pub fallback_pricing: Option<Pricing>,
}

impl Default for RouterConfig {
    fn default() -> Self {
        use super::DEFAULT_MODEL;
        use super::PREMIUM_MODEL;

        let mut tiers = HashMap::new();
        tiers.insert(TaskTier::Cheap, DEFAULT_MODEL.to_string());
        tiers.insert(TaskTier::Mid, DEFAULT_MODEL.to_string());
        tiers.insert(TaskTier::Premium, PREMIUM_MODEL.to_string());

        let mut pricing = HashMap::new();
        pricing.insert(
            DEFAULT_MODEL.to_string(),
            Pricing {
                input_per_1m: 0.15,
                output_per_1m: 0.60,
            },
        );
        pricing.insert(
            PREMIUM_MODEL.to_string(),
            Pricing {
                input_per_1m: 2.50,
                output_per_1m: 10.00,
            },
        );
        pricing.insert(
            "claude-sonnet-4-5-20250929".to_string(),
            Pricing {
                input_per_1m: 3.00,
                output_per_1m: 15.00,
            },
        );
        pricing.insert(
            "claude-opus-4-1-20250805".to_string(),
            Pricing {
                input_per_1m: 15.00,
                output_per_1m: 75.00,
            },
        );

        Self {
            tiers,
            pricing,
            spend_cap_usd: None,
            fallback_pricing: None,
        }
    }
}

/// A cost-aware wrapper around any [`LlmClient`].
///
/// Implements [`LlmClient`] so it's a transparent drop-in, but also exposes
/// [`complete_tiered`](Self::complete_tiered) for explicit per-subtask routing.
pub struct ModelRouter {
    client: Arc<dyn LlmClient>,
    config: RouterConfig,
    spent: Mutex<f64>,
}

impl ModelRouter {
    /// Wrap a client with default routing config.
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self {
            client,
            config: RouterConfig::default(),
            spent: Mutex::new(0.0),
        }
    }

    /// Wrap a client with custom routing config.
    pub fn with_config(client: Arc<dyn LlmClient>, config: RouterConfig) -> Self {
        Self {
            client,
            config,
            spent: Mutex::new(0.0),
        }
    }

    /// Complete a request using a specific task tier's model.
    pub async fn complete_tiered(
        &self,
        tier: TaskTier,
        mut req: CompletionRequest,
    ) -> Result<CompletionResponse, LlmError> {
        if req.model.is_none() {
            if let Some(model) = self.config.tiers.get(&tier) {
                req.model = Some(model.clone());
            }
        }
        self.complete(&req).await
    }

    /// Total USD spent so far.
    pub fn spent_usd(&self) -> f64 {
        *self.spent.lock().unwrap()
    }

    /// Remaining budget before the cap is hit (`None` if no cap).
    pub fn remaining_usd(&self) -> Option<f64> {
        self.config.spend_cap_usd.map(|cap| {
            let spent = *self.spent.lock().unwrap();
            (cap - spent).max(0.0)
        })
    }

    fn check_budget(&self) -> Result<(), LlmError> {
        if let Some(cap) = self.config.spend_cap_usd {
            let spent = *self.spent.lock().unwrap();
            if spent >= cap {
                return Err(LlmError::BudgetExceeded { spent, cap });
            }
        }
        Ok(())
    }

    fn record_cost(&self, model: &str, usage: &super::Usage) {
        let cost = self
            .config
            .pricing
            .get(model)
            .map(|p| p.cost(usage.prompt_tokens, usage.completion_tokens))
            .unwrap_or(0.0);
        *self.spent.lock().unwrap() += cost;
    }
}

#[async_trait::async_trait]
impl LlmClient for ModelRouter {
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        self.check_budget()?;
        let resp = self.client.complete(req).await?;
        self.record_cost(&resp.model, &resp.usage);
        Ok(resp)
    }

    fn default_model(&self) -> &str {
        self.client.default_model()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct FakeClient;
    #[async_trait]
    impl LlmClient for FakeClient {
        async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
            Ok(CompletionResponse {
                content: "ok".into(),
                tool_calls: vec![],
                usage: super::super::Usage {
                    prompt_tokens: 1_000_000,
                    completion_tokens: 500_000,
                    total_tokens: 1_500_000,
                },
                model: crate::llm::DEFAULT_MODEL.into(),
                finish_reason: super::super::FinishReason::Stop,
            })
        }
        fn default_model(&self) -> &str {
            crate::llm::DEFAULT_MODEL
        }
    }

    #[tokio::test]
    async fn tracks_cost() {
        let router = ModelRouter::new(Arc::new(FakeClient));
        let req = CompletionRequest::new(vec![]);
        router.complete(&req).await.unwrap();
        // 1M input @ $0.15 + 0.5M output @ $0.60 = $0.15 + $0.30 = $0.45
        assert!((router.spent_usd() - 0.45).abs() < 0.001);
    }

    #[tokio::test]
    async fn enforces_cap() {
        let config = RouterConfig {
            spend_cap_usd: Some(0.10),
            ..Default::default()
        };
        let router = ModelRouter::with_config(Arc::new(FakeClient), config);
        let req = CompletionRequest::new(vec![]);
        // First call succeeds (spent=0 < cap=0.10), but records $0.45.
        router.complete(&req).await.unwrap();
        assert!((router.spent_usd() - 0.45).abs() < 0.001);
        // Second call fails (spent=0.45 >= cap=0.10).
        let err = router.complete(&req).await.unwrap_err();
        assert!(matches!(err, LlmError::BudgetExceeded { .. }));
    }
}
