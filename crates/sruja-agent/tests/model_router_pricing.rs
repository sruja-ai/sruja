//! Tests for ModelRouter unknown-model pricing warning and fallback_pricing.
//!
//! These tests exercise:
//! - Unknown model → warning logged, cost = 0.0 (no panic)
//! - fallback_pricing set → unknown model uses fallback rates
//! - Known model still uses exact pricing table entry

use std::sync::Arc;

use async_trait::async_trait;

use sruja_agent::llm::router::{ModelRouter, Pricing, RouterConfig};
use sruja_agent::llm::{CompletionRequest, CompletionResponse, LlmClient, LlmError, Usage};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

use sruja_agent::llm::FinishReason;
use sruja_agent::llm::DEFAULT_MODEL;

/// A fake client that always returns DEFAULT_MODEL.
struct FakeClient;

#[async_trait]
impl LlmClient for FakeClient {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        Ok(CompletionResponse {
            content: "ok".into(),
            tool_calls: vec![],
            usage: Usage {
                prompt_tokens: 1_000_000,
                completion_tokens: 500_000,
                total_tokens: 1_500_000,
            },
            model: DEFAULT_MODEL.into(),
            finish_reason: FinishReason::Stop,
        })
    }
    fn default_model(&self) -> &str {
        DEFAULT_MODEL
    }
}

/// A fake client that returns a caller-chosen model name in its response,
/// so `record_cost` sees a specific model string.
struct FakeClientWithModel(String);

#[async_trait]
impl LlmClient for FakeClientWithModel {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        Ok(CompletionResponse {
            content: "ok".into(),
            tool_calls: vec![],
            usage: Usage {
                prompt_tokens: 1_000_000,
                completion_tokens: 500_000,
                total_tokens: 1_500_000,
            },
            model: self.0.clone(),
            finish_reason: FinishReason::Stop,
        })
    }
    fn default_model(&self) -> &str {
        &self.0
    }
}

// ---------------------------------------------------------------------------
// Existing tests (preserved)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// NEW: unknown-model pricing warning tests
// ---------------------------------------------------------------------------

/// Unknown model must not panic and must record zero cost.
/// The `record_cost` implementation should log a warning for the missing
/// model and return 0.0 as the cost (via `unwrap_or(0.0)` or equivalent).
#[tokio::test]
async fn unknown_model_records_zero_cost_and_does_not_panic() {
    let unknown = "some-future-model-v99";
    let router = ModelRouter::new(Arc::new(FakeClientWithModel(unknown.to_string())));
    let req = CompletionRequest::new(vec![]);

    // Must not panic; must return Ok.
    let resp = router
        .complete(&req)
        .await
        .expect("must not panic on unknown model");
    assert_eq!(resp.model, unknown);

    // Cost for an unknown model with no fallback = 0.0.
    assert!(
        router.spent_usd().abs() < 0.001,
        "unknown model without fallback should record zero cost, got {}",
        router.spent_usd()
    );
}

/// When `fallback_pricing` is set on the config, an unknown model should
/// use those fallback rates instead of returning 0.0.
#[tokio::test]
async fn unknown_model_with_fallback_pricing_uses_fallback_rates() {
    let unknown = "some-future-model-v99";
    let fallback = Pricing {
        input_per_1m: 1.00,
        output_per_1m: 2.00,
    };
    let config = RouterConfig {
        fallback_pricing: Some(fallback),
        ..Default::default()
    };
    let router =
        ModelRouter::with_config(Arc::new(FakeClientWithModel(unknown.to_string())), config);
    let req = CompletionRequest::new(vec![]);
    router.complete(&req).await.unwrap();

    // 1M input @ $1.00 + 0.5M output @ $2.00 = $1.00 + $1.00 = $2.00
    assert!(
        (router.spent_usd() - 2.00).abs() < 0.001,
        "fallback pricing should yield $2.00, got {}",
        router.spent_usd()
    );
}

/// A known model must use the exact pricing table entry, even when a
/// `fallback_pricing` is configured.
#[tokio::test]
async fn known_model_uses_exact_pricing_table_entry() {
    // "gpt-4o-mini" is in the default pricing table at $0.15/$0.60.
    let router = ModelRouter::new(Arc::new(FakeClientWithModel(DEFAULT_MODEL.to_string())));
    let req = CompletionRequest::new(vec![]);
    router.complete(&req).await.unwrap();

    // 1M input @ $0.15 + 0.5M output @ $0.60 = $0.45
    assert!(
        (router.spent_usd() - 0.45).abs() < 0.001,
        "known model should use exact table pricing, got {}",
        router.spent_usd()
    );

    // Even if a fallback is configured, the exact entry must take precedence.
    let fallback = Pricing {
        input_per_1m: 99.0,
        output_per_1m: 99.0,
    };
    let config = RouterConfig {
        fallback_pricing: Some(fallback),
        ..Default::default()
    };
    let router2 = ModelRouter::with_config(
        Arc::new(FakeClientWithModel(DEFAULT_MODEL.to_string())),
        config,
    );
    let req2 = CompletionRequest::new(vec![]);
    router2.complete(&req2).await.unwrap();

    assert!(
        (router2.spent_usd() - 0.45).abs() < 0.001,
        "known model must use exact pricing, not fallback; got {}",
        router2.spent_usd()
    );
}
