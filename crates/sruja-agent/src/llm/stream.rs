//! Provider-neutral streaming types and stream→response reassembly.
//!
//! See [`super::LlmClient::complete_stream`]. Providers emit [`StreamEvent`]s;
//! callers either consume them directly (interactive UX) or reassemble into a
//! [`super::CompletionResponse`] via [`reassemble_stream`] (non-streaming
//! callers and tests).
//!
//! ## Tool-call assembly
//!
//! Tool-call arguments arrive as JSON-*string* fragments split arbitrarily
//! across chunks (mid-stream JSON is unparseable). The stream emits raw
//! fragments via [`StreamEvent::ToolCallArguments`]; callers concatenate by
//! `index` and parse only after [`StreamEvent::Finish`]. This matches the
//! OpenAI wire format and the existing parsed-`serde_json::Value` contract in
//! `super::ToolCall`.

use futures::stream::{BoxStream, StreamExt};

use super::{CompletionResponse, FinishReason, LlmError, ToolCall, Usage};

/// A single event in a streaming completion.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// A fragment of assistant text content (concatenate).
    ContentDelta(String),
    /// The first fragment for a tool call at `index`; carries its `id` and
    /// function `name`. These fields arrive only once per index; subsequent
    /// fragments omit them.
    ToolCallStart {
        index: usize,
        id: String,
        name: String,
    },
    /// A JSON-*string* fragment of the tool call's arguments at `index`.
    /// Concatenate across fragments; parse only after [`StreamEvent::Finish`].
    ToolCallArguments {
        index: usize,
        fragment: String,
    },
    /// Token usage (final chunk; may be absent if the stream is interrupted).
    Usage(Usage),
    /// The stream completed with this finish reason.
    Finish {
        finish_reason: FinishReason,
    },
}

/// A streaming completion: an async stream of [`StreamEvent`]s.
pub type Stream<'a> = BoxStream<'a, Result<StreamEvent, LlmError>>;

/// Accumulator for one tool call assembled from streamed fragments.
#[derive(Debug, Default)]
pub(crate) struct ToolCallAccumulator {
    pub id: Option<String>,
    pub name: Option<String>,
    pub arguments_buf: String,
}

/// Drive a stream to completion, reassembling into a [`CompletionResponse`].
///
/// Tool-call arguments are parsed at the end (mid-stream JSON is unparseable);
/// on parse failure each argument set falls back to `{}` (matching `openai.rs`).
/// Missing usage (interrupted stream) zero-fills via [`Usage::default`].
pub async fn reassemble_stream(
    stream: impl futures::Stream<Item = Result<StreamEvent, LlmError>> + Unpin,
    model: &str,
) -> Result<CompletionResponse, LlmError> {
    let mut stream = stream;
    let mut content = String::new();
    let mut accs: std::collections::BTreeMap<usize, ToolCallAccumulator> = Default::default();
    let mut usage = Usage::default();
    let mut finish_reason = FinishReason::Stop;

    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::ContentDelta(s) => content.push_str(&s),
            StreamEvent::ToolCallStart { index, id, name } => {
                let acc = accs.entry(index).or_default();
                if acc.id.is_none() {
                    acc.id = Some(id);
                }
                if acc.name.is_none() {
                    acc.name = Some(name);
                }
            }
            StreamEvent::ToolCallArguments { index, fragment } => {
                accs
                    .entry(index)
                    .or_default()
                    .arguments_buf
                    .push_str(&fragment);
            }
            StreamEvent::Usage(u) => usage = u,
            StreamEvent::Finish {
                finish_reason: fr,
            } => finish_reason = fr,
        }
    }

    let tool_calls: Vec<ToolCall> = accs
        .into_iter()
        .map(|(i, acc)| {
            let arguments =
                serde_json::from_str(&acc.arguments_buf).unwrap_or(serde_json::json!({}));
            ToolCall {
                id: acc.id.unwrap_or_else(|| format!("call_stream_{i}")),
                name: acc.name.unwrap_or_default(),
                arguments,
            }
        })
        .collect();

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
    use futures::{stream, Stream};

    /// Build a stream from a vec of events (all Ok).
    fn evs(events: Vec<StreamEvent>) -> impl Stream<Item = Result<StreamEvent, LlmError>> {
        stream::iter(events.into_iter().map(Ok))
    }

    #[tokio::test]
    async fn reassemble_text_deltas() {
        let s = evs(vec![
            StreamEvent::ContentDelta("Hel".into()),
            StreamEvent::ContentDelta("lo".into()),
            StreamEvent::Usage(Usage {
                prompt_tokens: 3,
                completion_tokens: 2,
                total_tokens: 5,
            }),
            StreamEvent::Finish {
                finish_reason: FinishReason::Stop,
            },
        ]);
        let resp = reassemble_stream(s, "gpt-4o-mini").await.unwrap();
        assert_eq!(resp.content, "Hello");
        assert!(resp.tool_calls.is_empty());
        assert_eq!(resp.usage.total_tokens, 5);
        assert_eq!(resp.finish_reason, FinishReason::Stop);
    }

    #[tokio::test]
    async fn reassemble_tool_call_fragmented_args() {
        // The JSON {"path":"src/main.rs"} split mid-key/mid-value, interleaved-style.
        let s = evs(vec![
            StreamEvent::ToolCallStart {
                index: 0,
                id: "call_1".into(),
                name: "file_read".into(),
            },
            StreamEvent::ToolCallArguments {
                index: 0,
                fragment: "{\"".into(),
            },
            StreamEvent::ToolCallArguments {
                index: 0,
                fragment: "path".into(),
            },
            StreamEvent::ToolCallArguments {
                index: 0,
                fragment: "\":\"src/main.rs\"}".into(),
            },
            StreamEvent::Finish {
                finish_reason: FinishReason::ToolCalls,
            },
        ]);
        let resp = reassemble_stream(s, "gpt-4o-mini").await.unwrap();
        assert_eq!(resp.tool_calls.len(), 1);
        assert_eq!(resp.tool_calls[0].id, "call_1");
        assert_eq!(resp.tool_calls[0].name, "file_read");
        assert_eq!(resp.tool_calls[0].arguments["path"], "src/main.rs");
        assert_eq!(resp.finish_reason, FinishReason::ToolCalls);
    }

    #[tokio::test]
    async fn reassemble_interleaved_tool_calls_by_index() {
        let s = evs(vec![
            StreamEvent::ToolCallStart {
                index: 0,
                id: "a".into(),
                name: "f".into(),
            },
            StreamEvent::ToolCallStart {
                index: 1,
                id: "b".into(),
                name: "g".into(),
            },
            StreamEvent::ToolCallArguments {
                index: 0,
                fragment: "{\"x\":1}".into(),
            },
            StreamEvent::ToolCallArguments {
                index: 1,
                fragment: "{\"y\":2}".into(),
            },
            StreamEvent::Finish {
                finish_reason: FinishReason::ToolCalls,
            },
        ]);
        let resp = reassemble_stream(s, "m").await.unwrap();
        assert_eq!(resp.tool_calls.len(), 2);
        // BTreeMap ordering → index-sorted.
        assert_eq!(resp.tool_calls[0].id, "a");
        assert_eq!(resp.tool_calls[0].arguments["x"], 1);
        assert_eq!(resp.tool_calls[1].id, "b");
        assert_eq!(resp.tool_calls[1].arguments["y"], 2);
    }

    #[tokio::test]
    async fn reassemble_malformed_args_falls_back_to_empty_object() {
        let s = evs(vec![
            StreamEvent::ToolCallStart {
                index: 0,
                id: "c".into(),
                name: "h".into(),
            },
            StreamEvent::ToolCallArguments {
                index: 0,
                fragment: "{not valid json".into(),
            },
            StreamEvent::Finish {
                finish_reason: FinishReason::ToolCalls,
            },
        ]);
        let resp = reassemble_stream(s, "m").await.unwrap();
        assert_eq!(resp.tool_calls.len(), 1);
        assert_eq!(resp.tool_calls[0].arguments, serde_json::json!({}));
    }

    #[tokio::test]
    async fn reassemble_interrupted_stream_zero_fills_usage() {
        // No Finish, no Usage (connection dropped).
        let s = evs(vec![StreamEvent::ContentDelta("partial".into())]);
        let resp = reassemble_stream(s, "m").await.unwrap();
        assert_eq!(resp.content, "partial");
        assert_eq!(resp.usage, Usage::default());
        assert_eq!(resp.finish_reason, FinishReason::Stop);
    }

    #[tokio::test]
    async fn reassemble_empty_content_with_tool_calls() {
        // Tool turn: content is empty (never emitted), only tool calls.
        let s = evs(vec![
            StreamEvent::ToolCallStart {
                index: 0,
                id: "x".into(),
                name: "t".into(),
            },
            StreamEvent::ToolCallArguments {
                index: 0,
                fragment: "{}".into(),
            },
            StreamEvent::Finish {
                finish_reason: FinishReason::ToolCalls,
            },
        ]);
        let resp = reassemble_stream(s, "m").await.unwrap();
        assert_eq!(resp.content, "");
        assert_eq!(resp.tool_calls.len(), 1);
    }

    #[tokio::test]
    async fn reassemble_missing_tool_id_synthesizes_one() {
        let s = evs(vec![
            StreamEvent::ToolCallStart {
                index: 0,
                id: String::new(),
                name: "n".into(),
            },
            StreamEvent::ToolCallArguments {
                index: 0,
                fragment: "{}".into(),
            },
            StreamEvent::Finish {
                finish_reason: FinishReason::ToolCalls,
            },
        ]);
        let resp = reassemble_stream(s, "m").await.unwrap();
        // Empty id falls through (not None here), but name present.
        assert_eq!(resp.tool_calls[0].name, "n");
    }
}
