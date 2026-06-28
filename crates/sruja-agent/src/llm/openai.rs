//! OpenAI-compatible LLM client.
//!
//! Works with any endpoint that implements the `/chat/completions` schema:
//! OpenAI, Azure OpenAI, Ollama, vLLM, LiteLLM, OpenRouter, Groq, etc.

use std::time::Duration;

use super::{
    CompletionRequest, CompletionResponse, FinishReason, FunctionSchema, LlmClient, LlmError,
    Message, MessageRole, ToolCall, Usage, DEFAULT_MODEL,
};
use super::stream::{Stream, StreamEvent};

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
#[allow(dead_code)]
const DEFAULT_TIMEOUT_SECS: u64 = 120;
/// Max timeout we'll use for any single request (300s = 5 min).
const MAX_TIMEOUT_SECS: u64 = 300;

/// An OpenAI-compatible chat completions client.
pub struct OpenAiClient {
    api_key: String,
    base_url: String,
    default_model: String,
    http: reqwest::Client,
}

impl OpenAiClient {
    /// Compute an adaptive timeout based on request body size.
    ///
    /// Larger prompts (e.g. fixer reading full files) get more time.
    /// Formula: base 60s + (body_chars / 100) * 10s, capped at MAX_TIMEOUT_SECS.
    /// This avoids hard timeouts on complex tasks while keeping quick tasks fast.
    fn adaptive_timeout(&self, body: &serde_json::Value) -> Duration {
        let body_str = serde_json::to_string(body).unwrap_or_default();
        let chars = body_str.len().max(100);
        // Each 100 chars = 10 extra seconds, base 60s
        let secs = (60u64 + (chars as u64 / 100) * 10).min(MAX_TIMEOUT_SECS);
        Duration::from_secs(secs)
    }

    /// Create with explicit configuration.
    pub fn new(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<Self, LlmError> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(MAX_TIMEOUT_SECS))
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

    async fn send(&self, body: &serde_json::Value) -> Result<reqwest::Response, LlmError> {
        let resp = self
            .http
            .post(self.url())
            .bearer_auth(&self.api_key)
            .json(body)
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

        if !(200..300).contains(&status) {
            let text = resp.text().await.unwrap_or_default();
            return Err(LlmError::Api { status, body: text });
        }

        Ok(resp)
    }

    /// Request body for a streaming completion: the non-streaming body plus
    /// `stream: true` and `stream_options.include_usage` (required to receive
    /// a usage chunk; without it usage is silently absent).
    fn build_streaming_body(&self, req: &CompletionRequest) -> serde_json::Value {
        let mut body = self.build_body(req);
        body["stream"] = serde_json::json!(true);
        body["stream_options"] = serde_json::json!({ "include_usage": true });
        body
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

        // Adaptive timeout based on prompt size
        let timeout = self.adaptive_timeout(&body);
        let resp = tokio::time::timeout(timeout, self.send(&body))
            .await
            .map_err(|_| LlmError::Network(format!(
                "Request timed out after {timeout:?}. Prompt size: ~{} chars",
                serde_json::to_string(&body).unwrap_or_default().len()
            )))??;

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| LlmError::Deserialize(e.to_string()))?;

        parse_completion(&json, &model)
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    /// Real streaming over the OpenAI-compatible SSE endpoint.
    ///
    /// Reads `bytes_stream()`, delimits SSE frames on `\n\n`, and decodes each
    /// whole frame (UTF-8-safe: `\n\n` is ASCII, so multi-byte chars are never
    /// split across frames). Tool-call arguments are emitted as raw JSON-string
    /// fragments keyed by `index`; the caller concatenates and parses at the end.
    fn complete_stream<'a>(&'a self, req: &'a CompletionRequest) -> Stream<'a> {
        let http = self.http.clone();
        let body = self.build_streaming_body(req);
        let url = self.url();
        let key = self.api_key.clone();

        let s = async_stream::try_stream! {
            let resp = http
                .post(&url)
                .bearer_auth(&key)
                .json(&body)
                .send()
                .await
                .map_err(|e| LlmError::Network(e.to_string()))?;

            let status = resp.status().as_u16();
            if status == 401 || status == 403 {
                Err(LlmError::Auth)?;
            }
            if status == 429 {
                let retry = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(|s| s * 1000);
                Err(LlmError::RateLimited {
                    retry_after_ms: retry,
                })?;
            }
            let mut byte_stream = resp.bytes_stream();
            let mut buf: Vec<u8> = Vec::new();
            let mut started: std::collections::HashSet<usize> = std::collections::HashSet::new();

            if !(200..300).contains(&status) {
                use futures::StreamExt;
                while let Some(chunk) = byte_stream.next().await {
                    match chunk {
                        Ok(b) => buf.extend_from_slice(&b),
                        Err(e) => Err(LlmError::Network(e.to_string()))?,
                    }
                }
                let text = String::from_utf8_lossy(&buf).to_string();
                Err(LlmError::Api { status, body: text })?;
            }

            use futures::StreamExt;
            loop {
                while let Some((content_end, drain_end)) = frame_split_pos(&buf) {
                    let frame_bytes: Vec<u8> = buf.drain(..drain_end).collect();
                    for event in parse_frame_events(&frame_bytes[..content_end], &mut started) {
                        yield event;
                    }
                }
                match byte_stream.next().await {
                    Some(Ok(chunk)) => buf.extend_from_slice(&chunk),
                    Some(Err(e)) => Err(LlmError::Network(e.to_string()))?,
                    None => break,
                }
            }
            if !buf.is_empty() {
                for event in parse_frame_events(&buf, &mut started) {
                    yield event;
                }
            }
        };

        Box::pin(s)
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

// ---------------------------------------------------------------------------
// Streaming SSE parsing
// ---------------------------------------------------------------------------

/// Map an OpenAI finish-reason string to the provider-neutral enum.
fn map_finish_reason(s: &str) -> FinishReason {
    match s {
        "stop" => FinishReason::Stop,
        "length" => FinishReason::Length,
        "tool_calls" => FinishReason::ToolCalls,
        "content_filter" => FinishReason::ContentFilter,
        other => FinishReason::Other(other.to_string()),
    }
}

/// Find the next complete SSE frame in `buf`.
///
/// Returns `(content_end, drain_end)` where `buf[0..content_end]` is the frame
/// text and draining `buf[..drain_end]` consumes the frame plus its `\n\n` (or
/// `\r\n\r\n`) terminator. `None` when no complete frame is buffered yet.
fn frame_split_pos(buf: &[u8]) -> Option<(usize, usize)> {
    let mut i = 0;
    while i + 1 < buf.len() {
        if buf[i] == b'\n' && buf[i + 1] == b'\n' {
            return Some((i, i + 2));
        }
        // CRLF framing: \r\n\r\n
        if i + 3 < buf.len()
            && buf[i] == b'\r'
            && buf[i + 1] == b'\n'
            && buf[i + 2] == b'\r'
            && buf[i + 3] == b'\n'
        {
            return Some((i, i + 4));
        }
        i += 1;
    }
    None
}

/// Deserialize the parts of a `chat.completion.chunk` we consume.
#[derive(serde::Deserialize)]
struct StreamChunk {
    #[serde(default)]
    choices: Vec<StreamChoice>,
    #[serde(default)]
    usage: Option<StreamUsage>,
}

#[derive(serde::Deserialize)]
struct StreamChoice {
    #[serde(default)]
    delta: StreamDelta,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(serde::Deserialize, Default)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<StreamToolCallDelta>,
}

#[derive(serde::Deserialize)]
struct StreamToolCallDelta {
    #[serde(default)]
    index: usize,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    function: Option<StreamFunctionDelta>,
}

#[derive(serde::Deserialize, Default)]
struct StreamFunctionDelta {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

#[derive(serde::Deserialize)]
struct StreamUsage {
    #[serde(default)]
    prompt_tokens: u64,
    #[serde(default)]
    completion_tokens: u64,
    #[serde(default)]
    total_tokens: u64,
}

/// Translate one SSE frame's bytes into [`StreamEvent`]s.
///
/// `frame` excludes the `\n\n` terminator. It is decoded whole (UTF-8-safe).
/// `started` tracks which tool-call indices have already emitted Start, so the
/// id/name (which arrive only on the first delta for an index) are emitted once.
fn parse_frame_events(
    frame: &[u8],
    started: &mut std::collections::HashSet<usize>,
) -> Vec<StreamEvent> {
    let mut events = Vec::new();
    // `\n\n` is ASCII, so splitting never breaks a multi-byte UTF-8 char.
    let text = std::str::from_utf8(frame).unwrap_or("");
    for raw_line in text.split('\n') {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line).trim_start();
        if line.is_empty() || line.starts_with(':') {
            continue; // keep-alive comment / blank
        }
        let payload = match line.strip_prefix("data:") {
            Some(p) => p.trim_start_matches(' '),
            None => continue, // event:/id:/retry: not used by OpenAI data-only SSE
        };
        if payload == "[DONE]" {
            // Server closes the stream after [DONE]; let the byte loop drain.
            continue;
        }
        let chunk: StreamChunk = match serde_json::from_str(payload) {
            Ok(c) => c,
            Err(_) => continue, // tolerate stray unparseable lines
        };
        for choice in &chunk.choices {
            if let Some(content) = choice.delta.content.as_deref() {
                if !content.is_empty() {
                    events.push(StreamEvent::ContentDelta(content.to_string()));
                }
            }
            for tc in &choice.delta.tool_calls {
                let idx = tc.index;
                if started.insert(idx) {
                    events.push(StreamEvent::ToolCallStart {
                        index: idx,
                        id: tc.id.clone().unwrap_or_default(),
                        name: tc
                            .function
                            .as_ref()
                            .and_then(|f| f.name.clone())
                            .unwrap_or_default(),
                    });
                }
                if let Some(args) = tc.function.as_ref().and_then(|f| f.arguments.as_deref()) {
                    if !args.is_empty() {
                        events.push(StreamEvent::ToolCallArguments {
                            index: idx,
                            fragment: args.to_string(),
                        });
                    }
                }
            }
            if let Some(fr) = choice.finish_reason.as_deref() {
                events.push(StreamEvent::Finish {
                    finish_reason: map_finish_reason(fr),
                });
            }
        }
        if let Some(u) = chunk.usage {
            events.push(StreamEvent::Usage(Usage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }));
        }
    }
    events
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

    // ── Streaming SSE parsing ────────────────────────────────────────────

    /// Build one SSE frame (`data: <json>\n\n`) from a JSON value.
    fn data_frame(json: serde_json::Value) -> Vec<u8> {
        format!("data: {}\n\n", json).into_bytes()
    }

    /// The `delta` object of a content chunk.
    fn content_delta(s: &str) -> serde_json::Value {
        serde_json::json!({ "content": s })
    }

    /// A tool-call delta fragment. `args` is the raw JSON-string fragment.
    fn tc_delta(index: usize, id: Option<&str>, name: Option<&str>, args: Option<&str>) -> serde_json::Value {
        let mut func = serde_json::Map::new();
        if let Some(n) = name { func.insert("name".into(), n.into()); }
        if let Some(a) = args { func.insert("arguments".into(), a.into()); }
        let mut tc = serde_json::Map::new();
        tc.insert("index".into(), index.into());
        if let Some(i) = id { tc.insert("id".into(), i.into()); }
        tc.insert("function".into(), serde_json::Value::Object(func));
        serde_json::json!({ "tool_calls": [serde_json::Value::Object(tc)] })
    }

    fn chunk(delta: serde_json::Value, finish: Option<&str>) -> serde_json::Value {
        let mut choice = serde_json::Map::new();
        choice.insert("index".into(), 0.into());
        choice.insert("delta".into(), delta);
        choice.insert("finish_reason".into(), finish.into());
        serde_json::json!({ "choices": [serde_json::Value::Object(choice)] })
    }

    #[test]
    fn frame_split_finds_double_newline() {
        let buf = data_frame(serde_json::json!({"a":1}));
        let (c, d) = frame_split_pos(&buf).unwrap();
        // content is the `data: {...}` line without the trailing \n\n.
        let content = &buf[..c];
        assert!(content.starts_with(b"data: {"));
        assert_eq!(d, c + 2);
    }

    #[test]
    fn frame_split_returns_none_when_incomplete() {
        assert!(frame_split_pos(b"data: partial").is_none());
        assert!(frame_split_pos(b"data: no newline yet\n").is_none());
    }

    #[test]
    fn parse_text_delta_frame() {
        let frame = data_frame(chunk(content_delta("Hello"), None));
        let mut started = std::collections::HashSet::new();
        let evs = parse_frame_events(&frame, &mut started);
        assert_eq!(evs.len(), 1);
        match &evs[0] {
            StreamEvent::ContentDelta(s) => assert_eq!(s, "Hello"),
            other => panic!("expected ContentDelta, got {other:?}"),
        }
    }

    #[test]
    fn parse_fragmented_tool_call_assembles_by_index() {
        // A single get_weather call split across chunks, exactly like the
        // OpenAI wire format: first carries id+name, second carries arg fragments.
        let frames = [
            data_frame(chunk(tc_delta(0, Some("call_x"), Some("get_weather"), Some("")), None)),
            data_frame(chunk(tc_delta(0, None, None, Some("{\"city\":\"Paris\"}")), None)),
            data_frame(chunk(serde_json::json!({}), Some("tool_calls"))),
        ];
        let mut started = std::collections::HashSet::new();
        let mut all = Vec::new();
        for f in &frames {
            all.extend(parse_frame_events(f, &mut started));
        }
        assert!(matches!(
            all[0],
            StreamEvent::ToolCallStart { index: 0, ref id, ref name }
                if id == "call_x" && name == "get_weather"
        ));
        let frag = all
            .iter()
            .find_map(|e| match e {
                StreamEvent::ToolCallArguments { index: 0, fragment } => Some(fragment.clone()),
                _ => None,
            })
            .unwrap();
        assert_eq!(frag, "{\"city\":\"Paris\"}");
        assert!(matches!(
            all.last().unwrap(),
            StreamEvent::Finish { finish_reason: super::FinishReason::ToolCalls }
        ));
    }

    #[test]
    fn parse_usage_chunk_with_empty_choices() {
        // The usage-only final frame: choices is [] and usage is populated.
        let frame = data_frame(serde_json::json!({
            "choices": [],
            "usage": { "prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15 }
        }));
        let mut started = std::collections::HashSet::new();
        let evs = parse_frame_events(&frame, &mut started);
        assert_eq!(evs.len(), 1);
        match &evs[0] {
            StreamEvent::Usage(u) => assert_eq!(u.total_tokens, 15),
            other => panic!("expected Usage, got {other:?}"),
        }
    }

    #[test]
    fn parse_done_literal_does_not_error() {
        let frame = b"data: [DONE]\n\n".to_vec();
        let mut started = std::collections::HashSet::new();
        let evs = parse_frame_events(&frame, &mut started);
        assert!(evs.is_empty(), "[DONE] must be silently skipped");
    }

    #[test]
    fn parse_null_content_in_tool_turn_is_noop() {
        // delta.content is null while tool_calls present — must not panic.
        let frame = data_frame(serde_json::json!({
            "choices": [{
                "index": 0,
                "delta": {
                    "content": null,
                    "tool_calls": [{
                        "index": 0,
                        "id": "c",
                        "function": { "name": "t", "arguments": "{}" }
                    }]
                },
                "finish_reason": null
            }]
        }));
        let mut started = std::collections::HashSet::new();
        let evs = parse_frame_events(&frame, &mut started);
        // Start + Arguments fragment, no ContentDelta.
        assert!(evs.iter().all(|e| !matches!(e, StreamEvent::ContentDelta(_))));
        assert_eq!(evs.len(), 2);
    }

    #[tokio::test]
    async fn parse_interleaved_tool_calls_reassemble() {
        // Two tool calls interleaved across chunks, reassembled by index.
        let frames = [
            data_frame(chunk(tc_delta(0, Some("a"), Some("f0"), Some("{\"x\":")), None)),
            data_frame(chunk(tc_delta(1, Some("b"), Some("f1"), Some("{\"y\":")), None)),
            data_frame(chunk(
                serde_json::json!({
                    "tool_calls": [
                        { "index": 0, "function": { "arguments": "1}" } },
                        { "index": 1, "function": { "arguments": "2}" } }
                    ]
                }),
                None,
            )),
            data_frame(chunk(serde_json::json!({}), Some("tool_calls"))),
        ];
        let mut started = std::collections::HashSet::new();
        let mut all: Vec<StreamEvent> = Vec::new();
        for f in &frames {
            all.extend(parse_frame_events(f, &mut started));
        }
        let stream = futures::stream::iter(all.into_iter().map(Ok));
        let resp = super::super::stream::reassemble_stream(stream, "m").await.unwrap();
        assert_eq!(resp.tool_calls.len(), 2);
        assert_eq!(resp.tool_calls[0].id, "a");
        assert_eq!(resp.tool_calls[0].arguments["x"], 1);
        assert_eq!(resp.tool_calls[1].id, "b");
        assert_eq!(resp.tool_calls[1].arguments["y"], 2);
        assert_eq!(resp.finish_reason, super::FinishReason::ToolCalls);
    }
}
