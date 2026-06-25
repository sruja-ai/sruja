//! @element Sruja.Agent.Chat
//! @layer Core Engine
//! @boundary Streaming turn logic lives here; terminal I/O and git stay in sruja-cli.
//!
//! Interactive chat turn: a single-pass streaming tool loop that emits
//! [`TurnEvent`]s to a caller-provided channel and returns a [`TurnResult`].
//!
//! This is the streaming analogue of [`crate::cognition::Agent::run_tool_loop`],
//! designed for the interactive `sruja agent chat` REPL. It does **not** embed
//! the comprehend → critique → replan cycle (that lives in `agent loop`).
//! The deterministic grader is available on demand via `/verify` in the host.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::mpsc;

use crate::llm::{
    CompletionRequest, CompletionResponse, FinishReason, Message, MessageRole, ToolCall,
    Usage,
};
use crate::llm::stream::StreamEvent;
use crate::tool::ToolSignal;
use crate::AgentError;

/// Events emitted during a streaming turn, sent to the host for rendering.
///
/// The host receives these via an [`mpsc::Sender`] and renders them
/// (streaming tokens inline, tool start/done as one-liners).
#[derive(Debug, Clone)]
pub enum TurnEvent {
    /// A fragment of assistant text (concatenate and print inline).
    ContentDelta(String),
    /// The model is about to dispatch a tool call.
    ToolStart {
        name: String,
    },
    /// A tool call finished.
    ToolDone {
        name: String,
        ok: bool,
        elapsed_ms: u64,
    },
    /// The model produced a final answer (no more tool calls).
    Done,
}

/// The result of a single streaming turn.
#[derive(Debug, Clone)]
pub struct TurnResult {
    /// The final assistant message (content + any tool calls in this turn).
    pub message: Message,
    /// All tool calls made during the turn.
    pub tool_calls: Vec<ToolCall>,
    /// Cumulative token usage across all LLM calls in this turn.
    pub usage: Usage,
    /// File paths mutated by tools during this turn (for selective git staging).
    pub mutated_paths: Vec<String>,
    /// Per-tool-call signals (for the deterministic grader).
    pub tool_signals: Vec<Signal>,
}

/// Simplified tool signal for chat (the full `ToolSignal` includes source
/// classification that the host doesn't need for rendering).
#[derive(Debug, Clone)]
pub struct Signal {
    pub tool: String,
    pub ok: bool,
    pub elapsed_ms: u64,
}

impl From<&ToolSignal> for Signal {
    fn from(s: &ToolSignal) -> Self {
        Self {
            tool: s.tool.clone(),
            ok: s.ok,
            elapsed_ms: s.elapsed_ms,
        }
    }
}

// ---------------------------------------------------------------------------
// Agent::run_streaming_turn
// ---------------------------------------------------------------------------

impl crate::cognition::Agent {
    /// Run a single-pass streaming turn: send the history to the model, stream
    /// content deltas and dispatch tool calls (emitting events), until the model
    /// produces a final answer or the iteration limit is hit.
    ///
    /// **Events** are sent to `events` in order: `ContentDelta` fragments as the
    /// model generates text, then `ToolStart`/`ToolDone` around each dispatch,
    /// and a final `Done`. The method performs **no terminal I/O** — the host
    /// renders events (K6 boundary).
    ///
    /// **Cancellation:** set `cancel` to `true` to abort mid-turn. The method
    /// checks the flag between tool dispatches and between LLM calls. Partial
    /// output is preserved in the returned `TurnResult`.
    ///
    /// **Convergence:** bounded by `config.max_tool_iterations` with the same
    /// soft/hard "wrap up" pressure messages as [`run_tool_loop`].
    ///
    /// [`run_tool_loop`]: crate::cognition::Agent::run_tool_loop
    pub async fn run_streaming_turn(
        &self,
        mut req: CompletionRequest,
        events: &mpsc::Sender<TurnEvent>,
        cancel: &Arc<AtomicBool>,
    ) -> Result<TurnResult, AgentError> {
        let mut total_usage = Usage::default();
        let mut all_tool_calls: Vec<ToolCall> = Vec::new();
        let mut all_signals: Vec<Signal> = Vec::new();
        let mut mutated_paths: Vec<String> = Vec::new();
        let mut last_response: Option<CompletionResponse> = None;
        let mut soft_sent = false;
        let mut hard_sent = false;

        for iteration in 0..self.config.max_tool_iterations {
            if cancel.load(Ordering::Relaxed) {
                tracing::info!(iteration, "streaming_turn: cancelled");
                break;
            }

            // Stream the completion and reassemble while forwarding deltas.
            let stream = self.llm.complete_stream(&req);
            let response = {
                let event_proxy = async {
                    let mut s = stream;
                    use futures::StreamExt;
                    let mut acc_events: Vec<StreamEvent> = Vec::new();
                    while let Some(item) = s.next().await {
                        match item {
                            Ok(ev) => {
                                // Forward content deltas to the host immediately.
                                if let StreamEvent::ContentDelta(ref delta) = ev {
                                    let _ = events.send(TurnEvent::ContentDelta(delta.clone())).await;
                                }
                                acc_events.push(ev);
                            }
                            Err(e) => return Err(e),
                        }
                    }
                    Ok(acc_events)
                };
                let events_vec = event_proxy.await?;
                // Reassemble from the collected events.
                let dummy_model = req
                    .model
                    .as_deref()
                    .unwrap_or("unknown")
                    .to_string();
                reassemble_from_events(&events_vec, &dummy_model)?
            };

            total_usage.prompt_tokens += response.usage.prompt_tokens;
            total_usage.completion_tokens += response.usage.completion_tokens;
            total_usage.total_tokens += response.usage.total_tokens;

            let tool_names: Vec<&str> =
                response.tool_calls.iter().map(|c| c.name.as_str()).collect();
            let content_preview: String = response.content.chars().take(120).collect();
            tracing::info!(
                iteration,
                finish_reason = ?response.finish_reason,
                tool_calls = tool_names.len(),
                tool_names = ?tool_names,
                content_preview = %content_preview,
                "streaming_turn: LLM response"
            );

            // Termination: stop when the model stops requesting tools.
            if !response.wants_tools() || response.finish_reason == FinishReason::Stop {
                if response.wants_tools() {
                    tracing::warn!(
                        iteration,
                        tool_names = ?tool_names,
                        "streaming_turn: finish_reason=Stop but tool_calls present — \
                         treating content as final answer"
                    );
                    let mut response = response;
                    response.tool_calls.clear();
                    last_response = Some(response);
                    break;
                }
                last_response = Some(response);
                break;
            }

            last_response = Some(response.clone());

            // Push the assistant's tool-call message.
            req.messages.push(Message {
                role: MessageRole::Assistant,
                content: response.content.clone(),
                tool_calls: response.tool_calls.clone(),
                tool_call_id: None,
            });

            // Execute each requested tool and feed results back.
            for call in &response.tool_calls {
                if cancel.load(Ordering::Relaxed) {
                    tracing::info!(tool = %call.name, "streaming_turn: cancelled before tool dispatch");
                    break;
                }

                let _ = events
                    .send(TurnEvent::ToolStart {
                        name: call.name.clone(),
                    })
                    .await;

                tracing::debug!(
                    tool = %call.name,
                    args_preview = %call.arguments.to_string().chars().take(200).collect::<String>(),
                    "streaming_turn: dispatching tool"
                );

                let (result, record) = match self
                    .tools
                    .dispatch_record(&call.name, call.arguments.clone())
                    .await
                {
                    Ok(out) => out,
                    Err(e) => {
                        let record = crate::tool::ToolCallRecord {
                            ok: false,
                            empty: false,
                            elapsed_ms: 0,
                            source: crate::tool::ToolRegistry::classify_source(&call.name),
                            truncated: false,
                            payload: e.to_string(),
                        };
                        (format!("ERROR: {e}"), record)
                    }
                };

                let truncated_text = truncate(&result, 8_000);
                let was_truncated = result.len() > 8_000;
                let mut record = record;
                if was_truncated {
                    record.truncated = true;
                }

                all_tool_calls.push(call.clone());
                all_signals.push(Signal {
                    tool: call.name.clone(),
                    ok: record.ok,
                    elapsed_ms: record.elapsed_ms,
                });

                let signal = ToolSignal {
                    tool: call.name.clone(),
                    ok: record.ok,
                    empty: record.empty,
                    elapsed_ms: record.elapsed_ms,
                    source: record.source,
                };

                let _ = events
                    .send(TurnEvent::ToolDone {
                        name: call.name.clone(),
                        ok: record.ok,
                        elapsed_ms: record.elapsed_ms,
                    })
                    .await;

                // Collect mutated paths for selective git staging.
                let paths = self.tools.mutated_paths(&call.name, &call.arguments);
                for p in paths {
                    if !mutated_paths.contains(&p) {
                        mutated_paths.push(p);
                    }
                }

                let _ = signal; // ToolSignal collected via all_signals (simplified Signal)

                tracing::debug!(
                    tool = %call.name,
                    result_len = truncated_text.len(),
                    ok = record.ok,
                    "streaming_turn: tool result"
                );

                req.messages
                    .push(Message::tool_result(&call.id, truncated_text));
            }

            if cancel.load(Ordering::Relaxed) {
                break;
            }

            // ── Convergence pressure (same as run_tool_loop) ─────────────
            let remaining = self.config.max_tool_iterations - iteration - 1;
            let quarter = (self.config.max_tool_iterations / 4).max(1);
            let half = self.config.max_tool_iterations / 2;
            if remaining > 0 && remaining <= quarter && !hard_sent {
                hard_sent = true;
                tracing::warn!(
                    iteration,
                    remaining,
                    "streaming_turn: injecting hard convergence message"
                );
                req.messages.push(Message::user(
                    "CRITICAL: You have very few tool calls remaining. \
                     You MUST produce your final answer now as plain text. \
                     Do NOT call any more tools. Synthesize what you have \
                     learned and write your answer.",
                ));
            } else if remaining > 0 && remaining <= half && !soft_sent {
                soft_sent = true;
                tracing::info!(
                    iteration,
                    remaining,
                    "streaming_turn: injecting soft convergence reminder"
                );
                req.messages.push(Message::user(
                    "REMINDER: You have used more than half your tool budget. \
                     Wrap up exploration and produce your final answer soon.",
                ));
            }
        }

        // Graceful degradation (same as run_tool_loop).
        let mut final_response = last_response.unwrap_or_else(|| {
            CompletionResponse::text(
                "ERROR: streaming turn exhausted without any response from the model.",
            )
        });
        final_response.tool_calls.clear();
        final_response.finish_reason = FinishReason::Stop;
        if final_response.content.is_empty() {
            final_response.content =
                "Tool iterations exhausted. See tool results above for context.".into();
        }

        let message = Message {
            role: MessageRole::Assistant,
            content: final_response.content.clone(),
            tool_calls: Vec::new(),
            tool_call_id: None,
        };

        let _ = events.send(TurnEvent::Done).await;

        Ok(TurnResult {
            message,
            tool_calls: all_tool_calls,
            usage: total_usage,
            mutated_paths,
            tool_signals: all_signals,
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Reassemble a `CompletionResponse` from a collected vec of `StreamEvent`s.
///
/// This is a local helper that avoids the borrow issues of calling
/// `reassemble_stream` (which takes an `impl Stream`) when we already have
/// the events in a `Vec`.
fn reassemble_from_events(
    events: &[StreamEvent],
    model: &str,
) -> Result<CompletionResponse, crate::llm::LlmError> {
    let mut content = String::new();
    let mut tool_calls: Vec<ToolCall> = Vec::new();
    let mut usage = Usage::default();
    let mut finish_reason = FinishReason::Stop;

    // Track tool-call assembly by index.
    let mut acc: std::collections::BTreeMap<usize, (Option<String>, Option<String>, String)> =
        std::collections::BTreeMap::new();

    for event in events {
        match event {
            StreamEvent::ContentDelta(delta) => content.push_str(delta),
            StreamEvent::ToolCallStart { index, id, name } => {
                let entry = acc.entry(*index).or_default();
                entry.0 = Some(id.clone());
                entry.1 = Some(name.clone());
            }
            StreamEvent::ToolCallArguments { index, fragment } => {
                let entry = acc.entry(*index).or_default();
                entry.2.push_str(fragment);
            }
            StreamEvent::Usage(u) => {
                usage = u.clone();
            }
            StreamEvent::Finish {
                finish_reason: fr,
            } => {
                finish_reason = fr.clone();
            }
        }
    }

    for (index, (id, name, args_buf)) in acc {
        let id = id.unwrap_or_else(|| format!("call_{}", index));
        let name = name.unwrap_or_else(|| format!("tool_{}", index));
        let arguments = if args_buf.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&args_buf).unwrap_or(serde_json::json!({}))
        };
        tool_calls.push(ToolCall {
            id,
            name,
            arguments,
        });
    }

    Ok(CompletionResponse {
        content,
        tool_calls,
        usage,
        model: model.to_string(),
        finish_reason,
    })
}

/// Truncate tool output to a maximum length (matches `run_tool_loop` behavior).
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let truncated = &s[..max];
        format!("{truncated}\n... [truncated: {} bytes]", s.len() - max)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{
        CompletionRequest, CompletionResponse, FinishReason, LlmClient, LlmError,
        Message, Usage,
    };
    use crate::llm::stream::{Stream, StreamEvent};
    use crate::tool::{ToolRegistry, tools};
    use std::sync::Arc;

    /// A fake `LlmClient` that returns a scripted sequence of responses.
    /// Each call to `complete_stream` pops the next response from the queue.
    struct ScriptedClient {
        responses: std::sync::Mutex<Vec<Vec<StreamEvent>>>,
    }

    impl ScriptedClient {
        fn new(responses: Vec<Vec<StreamEvent>>) -> Self {
            Self {
                responses: std::sync::Mutex::new(responses),
            }
        }
    }

    #[async_trait::async_trait]
    impl LlmClient for ScriptedClient {
        async fn complete(
            &self,
            _req: &CompletionRequest,
        ) -> Result<CompletionResponse, LlmError> {
            // Not used by run_streaming_turn.
            Ok(CompletionResponse::text("(non-streaming)"))
        }

        fn complete_stream<'a>(
            &'a self,
            _req: &'a CompletionRequest,
        ) -> Stream<'a> {
            let mut queue = self.responses.lock().unwrap();
            if queue.is_empty() {
                return Box::pin(futures::stream::iter(vec![Err(LlmError::Network(
                    "no more scripted responses".into(),
                ))]));
            }
            let events = queue.remove(0);
            // Push a Finish event if the script didn't include one.
            let has_finish = events
                .iter()
                .any(|e| matches!(e, StreamEvent::Finish { .. }));
            let mut full_events = events;
            if !has_finish {
                full_events.push(StreamEvent::Finish {
                    finish_reason: FinishReason::Stop,
                });
            }
            Box::pin(futures::stream::iter(
                full_events.into_iter().map(Ok).collect::<Vec<_>>(),
            ))
        }

        fn default_model(&self) -> &str {
            "test-model"
        }
    }

    fn make_agent(client: ScriptedClient, tools: ToolRegistry) -> crate::cognition::Agent {
        crate::cognition::Agent::builder()
            .llm(Arc::new(client))
            .tools(tools)
            .build()
            .unwrap()
    }

    fn cancel_token() -> Arc<AtomicBool> {
        Arc::new(AtomicBool::new(false))
    }

    #[tokio::test]
    async fn happy_path_text_only() {
        let client = ScriptedClient::new(vec![vec![
            StreamEvent::ContentDelta("Hello, world!".into()),
        ]]);
        let tools = ToolRegistry::with_builtin("/tmp/test", vec![]);
        let agent = make_agent(client, tools);

        let (tx, mut rx) = mpsc::channel(64);
        let cancel = cancel_token();
        let req = CompletionRequest::new(vec![Message::user("hi")]);

        let result = agent
            .run_streaming_turn(req, &tx, &cancel)
            .await
            .unwrap();

        assert_eq!(result.message.content, "Hello, world!");
        assert!(result.tool_calls.is_empty());
        assert!(result.mutated_paths.is_empty());

        // Check events.
        let mut event_count = 0;
        while let Ok(ev) = rx.try_recv() {
            event_count += 1;
            if event_count == 1 {
                assert!(matches!(ev, TurnEvent::ContentDelta(_)));
            }
            if event_count == 2 {
                assert!(matches!(ev, TurnEvent::Done));
            }
        }
        assert_eq!(event_count, 2);
    }

    #[tokio::test]
    async fn happy_path_one_tool_then_answer() {
        let client = ScriptedClient::new(vec![
            // First call: model requests file_read.
            vec![
                StreamEvent::ToolCallStart {
                    index: 0,
                    id: "call_1".into(),
                    name: "file_read".into(),
                },
                StreamEvent::ToolCallArguments {
                    index: 0,
                    fragment: r#"{"path":"/tmp/test/hello.txt"}"#.into(),
                },
                StreamEvent::Finish {
                    finish_reason: FinishReason::ToolCalls,
                },
            ],
            // Second call: model gives final answer.
            vec![StreamEvent::ContentDelta("The file says hello".into())],
        ]);

        // Create a temp file to read.
        let dir = std::env::temp_dir();
        let test_file = dir.join("sruja_chat_test.txt");
        std::fs::write(&test_file, "hello").unwrap();

        let tools = ToolRegistry::with_builtin(&dir, vec![]);
        let agent = make_agent(client, tools);

        let (tx, _rx) = mpsc::channel(64);
        let cancel = cancel_token();
        let req = CompletionRequest::new(vec![Message::user("read the file")]);

        let result = agent
            .run_streaming_turn(req, &tx, &cancel)
            .await
            .unwrap();

        assert_eq!(result.message.content, "The file says hello");
        assert_eq!(result.tool_calls.len(), 1);
        assert_eq!(result.tool_calls[0].name, "file_read");
        // file_read is not mutating.
        assert!(result.mutated_paths.is_empty());
    }

    #[tokio::test]
    async fn mutated_paths_collected_from_write_tool() {
        let dir = std::env::temp_dir().join("sruja_chat_write_test");
        std::fs::create_dir_all(&dir).unwrap();

        let client = ScriptedClient::new(vec![
            vec![
                StreamEvent::ToolCallStart {
                    index: 0,
                    id: "call_1".into(),
                    name: "file_write".into(),
                },
                StreamEvent::ToolCallArguments {
                    index: 0,
                    fragment: serde_json::json!({
                        "path": "output.txt",
                        "content": "test"
                    })
                    .to_string(),
                },
                StreamEvent::Finish {
                    finish_reason: FinishReason::ToolCalls,
                },
            ],
            vec![StreamEvent::ContentDelta("File written".into())],
        ]);

        let tools = ToolRegistry::with_builtin(&dir, vec![]);
        let agent = make_agent(client, tools);

        let (tx, _rx) = mpsc::channel(64);
        let cancel = cancel_token();
        let req = CompletionRequest::new(vec![Message::user("write the file")]);

        let result = agent
            .run_streaming_turn(req, &tx, &cancel)
            .await
            .unwrap();

        assert_eq!(result.message.content, "File written");
        assert_eq!(result.tool_calls.len(), 1);
        assert_eq!(result.tool_calls[0].name, "file_write");
        assert!(
            !result.mutated_paths.is_empty(),
            "mutated_paths should contain the written file"
        );
    }

    #[tokio::test]
    async fn max_iterations_graceful_fallback() {
        // A client that always requests a non-existent tool, forcing max iters.
        let max = 3;
        let mut scripts = Vec::new();
        for _ in 0..max {
            scripts.push(vec![
                StreamEvent::ToolCallStart {
                    index: 0,
                    id: "call_x".into(),
                    name: "nonexistent_tool".into(),
                },
                StreamEvent::ToolCallArguments {
                    index: 0,
                    fragment: "{}".into(),
                },
                StreamEvent::Finish {
                    finish_reason: FinishReason::ToolCalls,
                },
            ]);
        }

        let client = ScriptedClient::new(scripts);
        let tools = ToolRegistry::with_builtin("/tmp/test", vec![]);
        let agent = crate::cognition::Agent::builder()
            .llm(Arc::new(client))
            .tools(tools)
            .config(crate::cognition::AgentConfig {
                max_tool_iterations: max,
                ..Default::default()
            })
            .build()
            .unwrap();

        let (tx, _rx) = mpsc::channel(64);
        let cancel = cancel_token();
        let req = CompletionRequest::new(vec![Message::user("loop forever")]);

        let result = agent
            .run_streaming_turn(req, &tx, &cancel)
            .await
            .unwrap();

        // Should not panic; should return a fallback message.
        assert!(!result.message.content.is_empty());
    }

    #[tokio::test]
    async fn cancellation_aborts_mid_turn() {
        let client = ScriptedClient::new(vec![
            vec![
                StreamEvent::ToolCallStart {
                    index: 0,
                    id: "call_1".into(),
                    name: "nonexistent_tool".into(),
                },
                StreamEvent::ToolCallArguments {
                    index: 0,
                    fragment: "{}".into(),
                },
                StreamEvent::Finish {
                    finish_reason: FinishReason::ToolCalls,
                },
            ],
            // This response should never be consumed because we cancel.
            vec![StreamEvent::ContentDelta("unreached".into())],
        ]);

        let tools = ToolRegistry::with_builtin("/tmp/test", vec![]);
        let agent = make_agent(client, tools);

        let (tx, _rx) = mpsc::channel(64);
        let cancel = Arc::new(AtomicBool::new(true)); // pre-cancelled
        let req = CompletionRequest::new(vec![Message::user("test")]);

        let result = agent
            .run_streaming_turn(req, &tx, &cancel)
            .await
            .unwrap();

        // Should return immediately with a fallback (cancelled before first call).
        assert!(!result.message.content.is_empty());
    }

    #[tokio::test]
    async fn multi_tool_turn_both_dispatched() {
        let client = ScriptedClient::new(vec![
            vec![
                StreamEvent::ToolCallStart {
                    index: 0,
                    id: "call_a".into(),
                    name: "file_read".into(),
                },
                StreamEvent::ToolCallArguments {
                    index: 0,
                    fragment: r#"{"path":"/tmp/test/a.txt"}"#.into(),
                },
                StreamEvent::ToolCallStart {
                    index: 1,
                    id: "call_b".into(),
                    name: "file_read".into(),
                },
                StreamEvent::ToolCallArguments {
                    index: 1,
                    fragment: r#"{"path":"/tmp/test/b.txt"}"#.into(),
                },
                StreamEvent::Finish {
                    finish_reason: FinishReason::ToolCalls,
                },
            ],
            vec![StreamEvent::ContentDelta("Both files read".into())],
        ]);

        let dir = std::env::temp_dir();
        let tools = ToolRegistry::with_builtin(&dir, vec![]);
        let agent = make_agent(client, tools);

        let (tx, _rx) = mpsc::channel(64);
        let cancel = cancel_token();
        let req = CompletionRequest::new(vec![Message::user("read both")]);

        let result = agent
            .run_streaming_turn(req, &tx, &cancel)
            .await
            .unwrap();

        assert_eq!(result.message.content, "Both files read");
        assert_eq!(result.tool_calls.len(), 2);
        assert_eq!(result.tool_calls[0].name, "file_read");
        assert_eq!(result.tool_calls[1].name, "file_read");
    }

    #[test]
    fn reassemble_from_events_text() {
        let events = vec![
            StreamEvent::ContentDelta("Hello ".into()),
            StreamEvent::ContentDelta("world!".into()),
            StreamEvent::Usage(Usage {
                prompt_tokens: 3,
                completion_tokens: 2,
                total_tokens: 5,
            }),
            StreamEvent::Finish {
                finish_reason: FinishReason::Stop,
            },
        ];

        let resp = reassemble_from_events(&events, "test").unwrap();
        assert_eq!(resp.content, "Hello world!");
        assert_eq!(resp.usage.total_tokens, 5);
        assert_eq!(resp.finish_reason, FinishReason::Stop);
    }

    #[test]
    fn reassemble_from_events_tool_call() {
        let events = vec![
            StreamEvent::ToolCallStart {
                index: 0,
                id: "call_1".into(),
                name: "file_read".into(),
            },
            StreamEvent::ToolCallArguments {
                index: 0,
                fragment: r#"{"path":"# .into(),
            },
            StreamEvent::ToolCallArguments {
                index: 0,
                fragment: r#""test.rs"}"#.into(),
            },
            StreamEvent::Finish {
                finish_reason: FinishReason::ToolCalls,
            },
        ];

        let resp = reassemble_from_events(&events, "test").unwrap();
        assert_eq!(resp.tool_calls.len(), 1);
        assert_eq!(resp.tool_calls[0].name, "file_read");
        assert_eq!(
            resp.tool_calls[0].arguments,
            serde_json::json!({"path": "test.rs"})
        );
    }

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate("hello", 100), "hello");
    }

    #[test]
    fn truncate_long_string_cut() {
        let long = "x".repeat(200);
        let t = truncate(&long, 50);
        assert!(t.starts_with(&"x".repeat(50)));
        assert!(t.contains("truncated"));
    }
}
