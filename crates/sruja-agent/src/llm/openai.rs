//! OpenAI-compatible LLM client.
//!
//! Works with any endpoint that implements the `/chat/completions` schema:
//! OpenAI, Azure OpenAI, Ollama, vLLM, LiteLLM, OpenRouter, Groq, etc.

use std::time::Duration;

use super::{
    CompletionRequest, CompletionResponse, FinishReason, FunctionSchema, LlmClient, LlmError,
    Message, MessageRole, ToolCall, Usage,
};

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_MODEL: &str = "gpt-4o-mini";
const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// An OpenAI-compatible chat completions client.
pub struct OpenAiClient {
    api_key: String,
    base_url: String,
    default_model: String,
    http: reqwest::Client,
}

impl OpenAiClient {
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
    /// - `OPENAI_API_KEY` (or `SRUJA_ENRICH_API_KEY`)
    /// - `OPENAI_BASE_URL` (or `SRUJA_ENRICH_BASE_URL`) — defaults to OpenAI
    /// - `OPENAI_MODEL` (or `SRUJA_ENRICH_MODEL`) — defaults to `gpt-4o-mini`
    pub fn from_env() -> Result<Self, LlmError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .or_else(|_| std::env::var("SRUJA_ENRICH_API_KEY"))
            .map_err(|_| LlmError::Auth)?;

        let base_url = std::env::var("OPENAI_BASE_URL")
            .or_else(|_| std::env::var("SRUJA_ENRICH_BASE_URL"))
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

        let model = std::env::var("OPENAI_MODEL")
            .or_else(|_| std::env::var("SRUJA_ENRICH_MODEL"))
            .unwrap_or_else(|_| DEFAULT_MODEL.to_string());

        Self::new(api_key, base_url, model)
    }

    fn url(&self) -> String {
        let base = self.base_url.trim_end_matches('/');
        format!("{base}/chat/completions")
    }

    fn build_body(&self, req: &CompletionRequest) -> serde_json::Value {
        let model = req.model.as_deref().unwrap_or(&self.default_model);

        let messages: Vec<serde_json::Value> = req.messages.iter().map(message_to_json).collect();

        let mut body = serde_json::json!({
            "model": model,
            "messages": messages,
        });

        if let Some(temp) = req.temperature {
            body["temperature"] = serde_json::json!(temp);
        }
        if let Some(max) = req.max_tokens {
            body["max_tokens"] = serde_json::json!(max);
        }
        if !req.tools.is_empty() {
            body["tools"] =
                serde_json::json!(req.tools.iter().map(schema_to_tool).collect::<Vec<_>>());
        }

        body["response_format"] = match &req.response_format {
            super::ResponseFormat::Text => serde_json::json!({ "type": "text" }),
            super::ResponseFormat::JsonObject => serde_json::json!({ "type": "json_object" }),
            super::ResponseFormat::JsonSchema(schema) => {
                serde_json::json!({ "type": "json_schema", "json_schema": schema })
            }
        };

        body
    }
}

#[async_trait::async_trait]
impl LlmClient for OpenAiClient {
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let body = self.build_body(req);
        let model = body["model"]
            .as_str()
            .unwrap_or(&self.default_model)
            .to_string();

        let resp = self
            .http
            .post(self.url())
            .bearer_auth(&self.api_key)
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
        MessageRole::Tool => serde_json::json!({
            "role": "tool",
            "tool_call_id": msg.tool_call_id,
            "content": msg.content,
        }),
        MessageRole::Assistant if !msg.tool_calls.is_empty() => {
            let calls: Vec<serde_json::Value> = msg
                .tool_calls
                .iter()
                .map(|c| {
                    serde_json::json!({
                        "id": c.id,
                        "type": "function",
                        "function": {
                            "name": c.name,
                            "arguments": c.arguments.to_string(),
                        }
                    })
                })
                .collect();
            serde_json::json!({
                "role": "assistant",
                "content": msg.content,
                "tool_calls": calls,
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
        "type": "function",
        "function": {
            "name": s.name,
            "description": s.description,
            "parameters": s.parameters,
        }
    })
}

fn parse_completion(json: &serde_json::Value, model: &str) -> Result<CompletionResponse, LlmError> {
    let choice = json
        .get("choices")
        .and_then(|c| c.get(0))
        .ok_or_else(|| LlmError::Deserialize("missing choices[0]".into()))?;

    let message = choice
        .get("message")
        .ok_or_else(|| LlmError::Deserialize("missing message".into()))?;

    let content = message
        .get("content")
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    let tool_calls: Vec<ToolCall> = message
        .get("tool_calls")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|tc| {
                    let id = tc.get("id")?.as_str()?.to_string();
                    let func = tc.get("function")?;
                    let name = func.get("name")?.as_str()?.to_string();
                    let args_str = func
                        .get("arguments")
                        .and_then(|a| a.as_str())
                        .unwrap_or("{}");
                    let arguments = serde_json::from_str(args_str).unwrap_or(serde_json::json!({}));
                    Some(ToolCall {
                        id,
                        name,
                        arguments,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let finish_reason = choice
        .get("finish_reason")
        .and_then(|f| f.as_str())
        .map(|s| match s {
            "stop" => FinishReason::Stop,
            "length" => FinishReason::Length,
            "tool_calls" => FinishReason::ToolCalls,
            "content_filter" => FinishReason::ContentFilter,
            other => FinishReason::Other(other.to_string()),
        })
        .unwrap_or(FinishReason::Stop);

    let usage = json
        .get("usage")
        .map(|u| Usage {
            prompt_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
            completion_tokens: u
                .get("completion_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
        })
        .unwrap_or_default();

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
            "choices": [{
                "message": { "role": "assistant", "content": "hello" },
                "finish_reason": "stop"
            }],
            "usage": { "prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15 }
        });

        let resp = parse_completion(&json, "gpt-4o-mini").unwrap();
        assert_eq!(resp.content, "hello");
        assert_eq!(resp.usage.total_tokens, 15);
        assert!(resp.tool_calls.is_empty());
    }

    #[test]
    fn parse_tool_call() {
        let json = serde_json::json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {
                            "name": "file_read",
                            "arguments": "{\"path\":\"src/main.rs\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }]
        });

        let resp = parse_completion(&json, "gpt-4o-mini").unwrap();
        assert_eq!(resp.tool_calls.len(), 1);
        assert_eq!(resp.tool_calls[0].name, "file_read");
        assert_eq!(resp.tool_calls[0].arguments["path"], "src/main.rs");
        assert!(resp.wants_tools());
    }
}
