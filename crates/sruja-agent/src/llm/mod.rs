//! LLM client abstraction — the agent's brain.
//!
//! Any provider implements [`LlmClient`]. The framework ships an
//! OpenAI-compatible client ([`OpenAiClient`]), an Anthropic Messages API
//! client ([`AnthropicClient`]), and a cost-aware [`ModelRouter`] that tiers
//! requests by task complexity.
//!
//! ## Custom provider
//!
//! ```no_run
//! use async_trait::async_trait;
//! use sruja_agent::llm::*;
//!
//! struct MyClient;
//!
//! #[async_trait]
//! impl LlmClient for MyClient {
//!     async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
//!         // call your backend...
//!         Ok(CompletionResponse::text("hello"))
//!     }
//!     fn default_model(&self) -> &str { "my-model" }
//! }
//! ```

pub mod router;
pub mod stream;

#[cfg(feature = "llm-openai")]
pub mod openai;

#[cfg(feature = "llm-anthropic")]
pub mod anthropic;

pub mod tiered;

#[cfg(feature = "compression")]
pub mod compression;

pub mod constants;

use serde::{Deserialize, Serialize};

pub use constants::{DEFAULT_MODEL, PREMIUM_MODEL};

#[cfg(feature = "llm-anthropic")]
pub use anthropic::AnthropicClient;
#[cfg(feature = "llm-openai")]
pub use openai::OpenAiClient;
pub use router::{ModelRouter, TaskTier};
pub use stream::{reassemble_stream, Stream, StreamEvent};
pub use tiered::TieredClient;

#[cfg(feature = "compression")]
pub use compression::{CompressingClient, CompressionConfig, CompressionStats};

/// Error from an LLM provider call.
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("network error: {0}")]
    Network(String),
    #[error("API error ({status}): {body}")]
    Api { status: u16, body: String },
    #[error("authentication missing or invalid")]
    Auth,
    #[error("rate limited; retry after {retry_after_ms:?}ms")]
    RateLimited { retry_after_ms: Option<u64> },
    #[error("deserialization error: {0}")]
    Deserialize(String),
    #[error("budget exceeded: spent ${spent:.4} of ${cap:.4} cap")]
    BudgetExceeded { spent: f64, cap: f64 },
    #[error("{0}")]
    Other(String),
}

/// Role of a chat message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::User => write!(f, "user"),
            Self::Assistant => write!(f, "assistant"),
            Self::Tool => write!(f, "tool"),
        }
    }
}

/// A single chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_call_id: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_call_id: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_call_id: None,
        }
    }

    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

/// A function/tool the model may invoke.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSchema {
    pub name: String,
    pub description: String,
    /// JSON Schema describing the parameters.
    pub parameters: serde_json::Value,
}

/// A tool call requested by the model.
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    /// Parsed JSON arguments.
    pub arguments: serde_json::Value,
}

/// How the model should format its response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ResponseFormat {
    #[default]
    Text,
    JsonObject,
    JsonSchema(serde_json::Value),
}

/// Token usage for a completion.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

impl Usage {
    /// Add another usage record into this one in place.
    pub fn accumulate(&mut self, other: &Usage) {
        self.prompt_tokens += other.prompt_tokens;
        self.completion_tokens += other.completion_tokens;
        self.total_tokens += other.total_tokens;
    }

    /// Rough cost estimate in USD using conservative default rates
    /// (gpt-4o-mini pricing: $0.15/1M input, $0.60/1M output).
    ///
    /// For accurate per-model attribution, use [`crate::llm::ModelRouter`] instead.
    pub fn estimated_cost_usd(&self) -> f64 {
        self.prompt_tokens as f64 * 0.15 / 1_000_000.0
            + self.completion_tokens as f64 * 0.60 / 1_000_000.0
    }
}

/// Why the model stopped generating.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Other(String),
}

/// A request to an LLM provider.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub tools: Vec<FunctionSchema>,
    pub response_format: ResponseFormat,
}

impl CompletionRequest {
    /// Build a request from a message list.
    pub fn new(messages: Vec<Message>) -> Self {
        Self {
            messages,
            model: None,
            temperature: None,
            max_tokens: None,
            tools: Vec::new(),
            response_format: ResponseFormat::Text,
        }
    }

    /// Single-turn convenience: system prompt + user message.
    pub fn prompt(system: &str, user: impl Into<String>) -> Self {
        Self::new(vec![Message::system(system), Message::user(user)])
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    pub fn with_tools(mut self, tools: Vec<FunctionSchema>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_json(mut self) -> Self {
        self.response_format = ResponseFormat::JsonObject;
        self
    }
}

/// A response from an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    pub usage: Usage,
    pub model: String,
    pub finish_reason: FinishReason,
}

impl CompletionResponse {
    /// Convenience: plain-text response with zero usage.
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            tool_calls: Vec::new(),
            usage: Usage::default(),
            model: String::new(),
            finish_reason: FinishReason::Stop,
        }
    }

    /// Did the model request tool calls?
    pub fn wants_tools(&self) -> bool {
        !self.tool_calls.is_empty()
    }
}

/// The core trait every LLM provider implements.
#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    /// Perform a completion.
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError>;

    /// The default model this client uses.
    fn default_model(&self) -> &str;

    /// Streaming variant of [`complete`](Self::complete). Returns a stream of
    /// [`StreamEvent`]s that an interactive caller can render incrementally.
    ///
    /// The default implementation buffers [`complete`](Self::complete) and emits
    /// the whole result as a burst of events (correct, but not token-streamed).
    /// Providers with a real streaming endpoint override this for true streaming.
    fn complete_stream<'a>(&'a self, req: &'a CompletionRequest) -> Stream<'a> {
        // Drive the non-streaming call, then translate its result into events.
        // Tool calls are emitted as Start + a single Arguments fragment (the full
        // serialized JSON); content as one delta; then Usage and Finish.
        let fut = self.complete(req);
        Box::pin(async_stream::stream! {
            match fut.await {
                Ok(resp) => {
                    for (i, tc) in resp.tool_calls.iter().enumerate() {
                        yield Ok(StreamEvent::ToolCallStart {
                            index: i,
                            id: tc.id.clone(),
                            name: tc.name.clone(),
                        });
                        yield Ok(StreamEvent::ToolCallArguments {
                            index: i,
                            fragment: tc.arguments.to_string(),
                        });
                    }
                    if !resp.content.is_empty() {
                        yield Ok(StreamEvent::ContentDelta(resp.content));
                    }
                    yield Ok(StreamEvent::Usage(resp.usage));
                    yield Ok(StreamEvent::Finish {
                        finish_reason: resp.finish_reason,
                    });
                }
                Err(e) => yield Err(e),
            }
        })
    }
}
