//! Anthropic Messages API client (Claude family).
//!
//! Works with the Anthropic `/v1/messages` endpoint. Unlike the OpenAI client,
//! Anthropic separates the system prompt into a top-level field and uses
//! content-block arrays for rich message payloads.

use std::time::Duration;

use super::{
    CompletionRequest, CompletionResponse, FinishReason, FunctionSchema, LlmClient, LlmError,
    Message, MessageRole, ToolCall, Usage,
};

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com/v1";
const DEFAULT_MODEL: &str = "claude-sonnet-4-5-20250929";
const DEFAULT_TIMEOUT_SECS: u64 = 120;
const DEFAULT_MAX_TOKENS: usize = 4096;
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// An Anthropic Messages API client (Claude).
pub struct AnthropicClient {
    api_key: String,
    base_url: String,
    default_model: String,
    http: reqwest::Client,
}

impl AnthropicClient {
    /// Create with explicit configuration.
    pub fn new(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<Self, LlmError> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(|e| LlmError::Network(e.to_string()))?;

        Ok(Self {
            api_key: api_key.into(),
            base_url: base_url.into(),
            default_model: model.into(),
            http,
        })
    }

    /// Create from environment variables.
    ///
    /// Reads:
    /// - `ANTHROPIC_API_KEY` (or `SRUJA_AGENT_API_KEY`)
    /// - `ANTHROPIC_BASE_URL` (or `SRUJA_AGENT_BASE_URL`) — defaults to Anthropic
    /// - `ANTHROPIC_MODEL` (or `SRUJA_AGENT_MODEL`) — defaults to `claude-sonnet-4-5`
    pub fn from_env() -> Result<Self, LlmError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .or_else(|_| std::env::var("SRUJA_AGENT_API_KEY"))
            .map_err(|_| LlmError::Auth)?;

        let base_url = std::env::var("ANTHROPIC_BASE_URL")
            .or_else(|_| std::env::var("SRUJA_AGENT_BASE_URL"))
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

        let model = std::env::var("ANTHROPIC_MODEL")
            .or_else(|_| std::env::var("SRUJA_AGENT_MODEL"))
            .unwrap_or_else(|_| DEFAULT_MODEL.to_string());

        Self::new(api_key, base_url, model)
    }

    fn url(&self) -> String {
        let base = self.base_url.trim_end_matches('/');
        format!("{base}/messages")
    }

    fn build_body(&self, req: &CompletionRequest) -> serde_json::Value {
        let model = req.model.as_deref().unwrap_or(&self.default_model);

        // Anthropic separates system prompts from the messages array.
        let system: String = req
            .messages
            .iter()
            .filter(|m| m.role == MessageRole::System)
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        let messages: Vec<serde_json::Value> = req
            .messages
            .iter()
            .filter(|m| m.role != MessageRole::System)
            .map(message_to_json)
            .collect();

        let mut body = serde_json::json!({
            "model": model,
            "messages": messages,
            "max_tokens": req.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
        });

        if !system.is_empty() {
            body["system"] = serde_json::json!(system);
        }

        if let Some(temp) = req.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if !req.tools.is_empty() {
            body["tools"] =
                serde_json::json!(req.tools.iter().map(schema_to_tool).collect::<Vec<_>>());
        }

        body
    }
}

#[async_trait::async_trait]
impl LlmClient for AnthropicClient {
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let body = self.build_body(req);
        let model = body["model"]
            .as_str()
            .unwrap_or(&self.default_model)
            .to_string();

        let resp = self
            .http
            .post(self.url())
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        let status = resp.status().as_u16();

        if status == 401 || status == 403 {
            return Err(LlmError::Auth);
        }
        if status == 429 {
            let retry = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .map(|s| s * 1000);
            return Err(LlmError::RateLimited {
                retry_after_ms: retry,
            });
        }

        let json: serde_json::Value = if (200..300).contains(&status) {
            resp.json()
                .await
                .map_err(|e| LlmError::Deserialize(e.to_string()))?
        } else {
            let text = resp.text().await.unwrap_or_default();
            return Err(LlmError::Api { status, body: text });
        };

        parse_completion(&json, &model)
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }
}

fn message_to_json(msg: &Message) -> serde_json::Value {
    match msg.role {
        // Anthropic represents tool results as user messages with tool_result blocks.
        MessageRole::Tool => serde_json::json!({
            "role": "user",
            "content": [{
                "type": "tool_result",
                "tool_use_id": msg.tool_call_id.as_deref().unwrap_or(""),
                "content": msg.content,
            }]
        }),
        // Assistant tool calls become content blocks, not a separate field.
        MessageRole::Assistant if !msg.tool_calls.is_empty() => {
            let mut content: Vec<serde_json::Value> = Vec::new();
            if !msg.content.is_empty() {
                content.push(serde_json::json!({"type": "text", "text": msg.content}));
            }
            for c in &msg.tool_calls {
                content.push(serde_json::json!({
                    "type": "tool_use",
                    "id": c.id,
                    "name": c.name,
                    "input": c.arguments,
                }));
            }
            serde_json::json!({
                "role": "assistant",
                "content": content,
            })
        }
        _ => serde_json::json!({
            "role": msg.role.to_string(),
            "content": msg.content,
        }),
    }
}

fn schema_to_tool(s: &FunctionSchema) -> serde_json::Value {
    serde_json::json!({
        "name": s.name,
        "description": s.description,
        "input_schema": s.parameters,
    })
}

fn parse_completion(json: &serde_json::Value, model: &str) -> Result<CompletionResponse, LlmError> {
    let content_arr = json
        .get("content")
        .and_then(|c| c.as_array())
        .ok_or_else(|| LlmError::Deserialize("missing content array".into()))?;

    let mut text_parts: Vec<String> = Vec::new();
    let mut tool_calls: Vec<ToolCall> = Vec::new();

    for block in content_arr {
        match block.get("type").and_then(|t| t.as_str()) {
            Some("text") => {
                if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                    text_parts.push(t.to_string());
                }
            }
            Some("tool_use") => {
                let id = block
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = block
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let input = block.get("input").cloned().unwrap_or(serde_json::json!({}));
                tool_calls.push(ToolCall {
                    id,
                    name,
                    arguments: input,
                });
            }
            _ => {}
        }
    }

    let content = text_parts.join("\n");

    let finish_reason = json
        .get("stop_reason")
        .and_then(|f| f.as_str())
        .map(|s| match s {
            "end_turn" => FinishReason::Stop,
            "max_tokens" => FinishReason::Length,
            "tool_use" => FinishReason::ToolCalls,
            "stop_sequence" => FinishReason::Stop,
            other => FinishReason::Other(other.to_string()),
        })
        .unwrap_or(FinishReason::Stop);

    let input_tokens = json
        .get("usage")
        .and_then(|u| u.get("input_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let output_tokens = json
        .get("usage")
        .and_then(|u| u.get("output_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let usage = Usage {
        prompt_tokens: input_tokens,
        completion_tokens: output_tokens,
        total_tokens: input_tokens + output_tokens,
    };

    Ok(CompletionResponse {
        content,
        tool_calls,
        usage,
        model: model.to_string(),
        finish_reason,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_completion() {
        let json = serde_json::json!({
            "content": [{"type": "text", "text": "hello"}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        });

        let resp = parse_completion(&json, "claude-sonnet-4-5-20250929").unwrap();
        assert_eq!(resp.content, "hello");
        assert_eq!(resp.usage.prompt_tokens, 10);
        assert_eq!(resp.usage.completion_tokens, 5);
        assert_eq!(resp.usage.total_tokens, 15);
        assert!(resp.tool_calls.is_empty());
    }

    #[test]
    fn parse_tool_use() {
        let json = serde_json::json!({
            "content": [
                {"type": "text", "text": "Let me check."},
                {"type": "tool_use", "id": "toolu_1", "name": "file_read", "input": {"path": "src/main.rs"}}
            ],
            "stop_reason": "tool_use",
            "usage": {"input_tokens": 10, "output_tokens": 20}
        });

        let resp = parse_completion(&json, "claude-sonnet-4-5-20250929").unwrap();
        assert_eq!(resp.content, "Let me check.");
        assert_eq!(resp.tool_calls.len(), 1);
        assert_eq!(resp.tool_calls[0].id, "toolu_1");
        assert_eq!(resp.tool_calls[0].name, "file_read");
        assert_eq!(resp.tool_calls[0].arguments["path"], "src/main.rs");
        assert!(resp.wants_tools());
    }

    #[test]
    fn parse_empty_content_array() {
        let json = serde_json::json!({
            "content": [],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 5, "output_tokens": 0}
        });

        let resp = parse_completion(&json, "claude-sonnet-4-5-20250929").unwrap();
        assert_eq!(resp.content, "");
        assert!(resp.tool_calls.is_empty());
    }
}
