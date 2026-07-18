//! Transparent LLM call compression — the Headroom pattern adapted to sruja.
//!
//! [`CompressingClient`] wraps any [`LlmClient`] and compresses old tool-result
//! messages before forwarding the request. Originals are stored in a
//! content-addressed CCR (content cache for recovery); the agent retrieves them
//! via the [`CompressRestoreTool`](crate::tool::CompressRestoreTool) when it
//! needs the full text.
//!
//! ## Architecture
//!
//! ```text
//!  Agent ──▶ CompressingClient ──▶ inner LlmClient (OpenAI / Anthropic / …)
//!               │ compress old tool messages
//!               │ store originals in CcrStore
//!               ▼
//!          CcrStore ◀── CompressRestoreTool (agent fetches full text on demand)
//! ```
//!
//! ## Usage
//!
//! ```no_run
//! # use std::sync::Arc;
//! # use sruja_agent::llm::*;
//! # use sruja_agent::tool::*;
//! # use sruja_compress::BoundedCcrStore;
//! # fn demo(inner: Arc<dyn LlmClient>, repo_path: std::path::PathBuf) {
//! let ccr = Arc::new(BoundedCcrStore::default());
//! let client = CompressingClient::new(inner)
//!     .with_ccr(ccr.clone());
//!
//! let tools = ToolRegistry::with_builtin(repo_path, vec![])
//!     .with(Box::new(CompressRestoreTool::new(ccr)));
//!
//! let agent = sruja_agent::Agent::builder()
//!     .llm(Arc::new(client))
//!     .tools(tools)
//!     .build();
//! # }
//! ```
//!
//! Compression is **lossless-in-spirit** with the default [`TextCrusher`] backend:
//! it drops low-salience sentences but preserves error lines, code blocks, and
//! the structural backbone. CCR makes it literally reversible for the rare case
//! the model needs the verbatim original.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use futures::{Stream, StreamExt};
use sruja_compress::{
    count_tokens, BoundedCcrStore, CcrHandle, CcrStore, CompressContext, Compressed, KeepPolicy,
    TextCompressor, TextCrusher, TextRole,
};

use crate::llm::{
    CompletionRequest, CompletionResponse, LlmClient, LlmError, Message, MessageRole, StreamEvent,
};

/// Prefix prepended to compressed messages so the compressor skips them on
/// subsequent passes and the model can spot the CCR handle.
pub const CCR_PREFIX: &str = "[compressed:";

/// Configuration for message compression.
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Messages with fewer tokens than this are never compressed (default: 200).
    pub min_tokens: usize,
    /// Don't compress the most recent N messages — keep fresh context intact
    /// (default: 6).
    pub preserve_recent: usize,
    /// Target compressed/original ratio in [0.05, 1.0]. `None` = backend default
    /// (TextCrusher targets ~30%). Lower = more aggressive (default: 0.3).
    pub target_ratio: Option<f64>,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            min_tokens: 200,
            preserve_recent: 6,
            target_ratio: Some(0.3),
        }
    }
}

/// Running compression telemetry for a [`CompressingClient`].
///
/// Tracks the token-level impact of compression across the lifetime of the
/// client: how many messages were compressed vs. skipped, the original and
/// post-compression token counts, and cumulative tokens saved. All counters
/// are atomics so they can be read concurrently without locking.
///
/// Access via [`CompressingClient::stats`](super::CompressingClient::stats).
#[derive(Debug, Default)]
pub struct CompressionStats {
    /// Number of messages that were compressed.
    pub messages_compressed: AtomicU64,
    /// Total tokens before compression across all compressed messages.
    pub original_tokens: AtomicU64,
    /// Total tokens after compression.
    pub compressed_tokens: AtomicU64,
    /// Cumulative tokens saved across all compressions.
    pub tokens_saved: AtomicU64,
    /// Number of messages skipped (already compressed, too small, or recent).
    pub messages_skipped: AtomicU64,
}

impl CompressionStats {
    /// Aggregate savings fraction in [0, 1].
    pub fn savings(&self) -> f64 {
        let orig = self.original_tokens.load(Ordering::Relaxed);
        let comp = self.compressed_tokens.load(Ordering::Relaxed);
        if orig == 0 {
            0.0
        } else {
            1.0 - comp as f64 / orig as f64
        }
    }

    /// Return a human-readable summary of compression statistics.
    ///
    /// Example output: `"Compressed 5 messages, saved 1000 tokens (30% reduction)"`
    pub fn report(&self) -> String {
        let msgs = self.messages_compressed.load(Ordering::Relaxed);
        let saved = self.tokens_saved.load(Ordering::Relaxed);
        let pct = self.savings() * 100.0;
        format!(
            "Compressed {} messages, saved {} tokens ({:.0}% reduction)",
            msgs, saved, pct
        )
    }
}

/// A transparent compression layer wrapping any [`LlmClient`].
///
/// Implements [`LlmClient`] so it drops in anywhere a raw provider would.
/// Before each [`complete`](LlmClient::complete) call, old tool messages are
/// reduced by the configured [`TextCompressor`]; originals go to the
/// [`CcrStore`] for retrieval.
///
/// **Default backend**: [`TextCrusher`] (deterministic BM25-based extractive
/// compressor — no model weights, no network). Swap for a Kompress ONNX backend
/// via `.with_compressor(...)`.
pub struct CompressingClient {
    inner: Arc<dyn LlmClient>,
    compressor: Arc<dyn TextCompressor>,
    ccr: Arc<dyn CcrStore>,
    config: CompressionConfig,
    stats: CompressionStats,
    compression_cache: Arc<std::sync::Mutex<HashMap<String, Compressed>>>,
    cached_compressed_request: Arc<tokio::sync::RwLock<Option<CompletionRequest>>>,
    cached_request_key: Arc<std::sync::Mutex<Option<u64>>>,
}

impl CompressingClient {
    /// Wrap an inner client with default TextCrusher compression + bounded CCR.
    pub fn new(inner: Arc<dyn LlmClient>) -> Self {
        Self {
            inner,
            compressor: Arc::new(TextCrusher::default()),
            ccr: Arc::new(BoundedCcrStore::default()),
            config: CompressionConfig::default(),
            stats: CompressionStats::default(),
            compression_cache: Arc::new(std::sync::Mutex::new(HashMap::new())),
            cached_compressed_request: Arc::new(tokio::sync::RwLock::new(None)),
            cached_request_key: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Override the compression backend (e.g. Kompress ONNX).
    pub fn with_compressor(mut self, compressor: Arc<dyn TextCompressor>) -> Self {
        self.compressor = compressor;
        self
    }

    /// Share a CCR store (required if you want the restore tool to see the
    /// same originals).
    pub fn with_ccr(mut self, ccr: Arc<dyn CcrStore>) -> Self {
        self.ccr = ccr;
        self
    }

    /// Override compression thresholds.
    pub fn with_config(mut self, config: CompressionConfig) -> Self {
        self.config = config;
        self
    }

    /// Get the CCR store handle (for registering [`CompressRestoreTool`]).
    pub fn ccr_store(&self) -> Arc<dyn CcrStore> {
        self.ccr.clone()
    }

    /// Compression telemetry.
    pub fn stats(&self) -> &CompressionStats {
        &self.stats
    }

    /// Compress a single message's content. Returns the new content and whether
    /// compression actually happened.
    fn compress_message_content(&self, content: &str) -> Option<(String, usize, usize)> {
        let original_tokens = count_tokens(content);
        if original_tokens < self.config.min_tokens {
            self.stats.messages_skipped.fetch_add(1, Ordering::Relaxed);
            return None;
        }

        // Check cache first (content is the key, identical content = same result)
        let cache_key = content.to_string();
        if let Some(cached) = self.compression_cache.lock().unwrap().get(&cache_key) {
            let handle = cached.ccr_handle.as_ref().unwrap();
            let compressed_text = format_ccr_message(handle, &cached.text);
            return Some((compressed_text, original_tokens, cached.compressed_tokens));
        }

        let ctx = CompressContext {
            query: None,
            role: Some(TextRole::Tool),
            target_ratio: self.config.target_ratio,
            keep: KeepPolicy::for_tool_output(),
        };

        match self.compressor.compress(content, &ctx) {
            Ok(result) if result.savings() > 0.0 => match self.ccr.put(content) {
                Ok(handle) => {
                    let compressed_text = format_ccr_message(&handle, &result.text);
                    let mut cached = result.clone();
                    cached.ccr_handle = Some(handle.clone());
                    self.compression_cache
                        .lock()
                        .unwrap()
                        .insert(cache_key, cached);
                    self.stats
                        .messages_compressed
                        .fetch_add(1, Ordering::Relaxed);
                    self.stats
                        .original_tokens
                        .fetch_add(original_tokens as u64, Ordering::Relaxed);
                    self.stats
                        .compressed_tokens
                        .fetch_add(result.compressed_tokens as u64, Ordering::Relaxed);
                    self.stats.tokens_saved.fetch_add(
                        (original_tokens - result.compressed_tokens) as u64,
                        Ordering::Relaxed,
                    );
                    Some((compressed_text, original_tokens, result.compressed_tokens))
                }
                Err(e) => {
                    tracing::warn!(error = %e, "ccr put failed — skipping compression");
                    None
                }
            },
            Ok(_) => {
                self.stats.messages_skipped.fetch_add(1, Ordering::Relaxed);
                None
            }
            Err(e) => {
                tracing::debug!(error = %e, "compression failed — forwarding original");
                self.stats.messages_skipped.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    /// Compress old tool messages in a request, returning a new request.
    fn compress_request(&self, req: &CompletionRequest) -> CompletionRequest {
        let total = req.messages.len();
        let preserve_from = total.saturating_sub(self.config.preserve_recent);

        let mut req = req.clone();

        for (idx, msg) in req.messages.iter_mut().enumerate() {
            // Never touch recent messages (preserve fresh context).
            if idx >= preserve_from {
                break;
            }

            // Only compress tool messages (file reads, shell output, etc.).
            // System/user/assistant messages are structural — must stay intact.
            if msg.role != MessageRole::Tool {
                continue;
            }

            // Skip already-compressed messages (idempotent on re-compression).
            if msg.content.starts_with(CCR_PREFIX) {
                continue;
            }

            if let Some((compressed_text, _orig, _comp)) =
                self.compress_message_content(&msg.content)
            {
                msg.content = compressed_text;
            }
        }

        req
    }

    /// Compress with a custom config (used for aggressive recovery).
    fn compress_request_with_config(
        &self,
        req: &CompletionRequest,
        config: &CompressionConfig,
    ) -> CompletionRequest {
        let total = req.messages.len();
        let preserve_from = total.saturating_sub(config.preserve_recent);

        let mut req = req.clone();

        for (idx, msg) in req.messages.iter_mut().enumerate() {
            if idx >= preserve_from {
                break;
            }
            if msg.role != MessageRole::Tool {
                continue;
            }
            if msg.content.starts_with(CCR_PREFIX) {
                continue;
            }

            let original_tokens = count_tokens(&msg.content);
            if original_tokens < config.min_tokens {
                continue;
            }

            let ctx = CompressContext {
                query: None,
                role: Some(TextRole::Tool),
                target_ratio: config.target_ratio,
                keep: KeepPolicy::for_tool_output(),
            };

            if let Ok(result) = self.compressor.compress(&msg.content, &ctx) {
                if result.savings() > 0.0 {
                    if let Ok(handle) = self.ccr.put(&msg.content) {
                        msg.content = format_ccr_message(&handle, &result.text);
                    }
                }
            }
        }

        req
    }
}

/// Detect context length overflow errors from LLM providers.
fn is_context_overflow(error_body: &str) -> bool {
    let lower = error_body.to_lowercase();
    lower.contains("context_length_exceeded")
        || lower.contains("maximum context length")
        || lower.contains("context window")
        || lower.contains("too many tokens")
        || (lower.contains("context") && lower.contains("exceed"))
}

#[async_trait::async_trait]
impl LlmClient for CompressingClient {
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let compressed_req = self.compress_request(req);

        let stats = &self.stats;
        let compressed = stats.messages_compressed.load(Ordering::Relaxed);
        if compressed > 0 {
            tracing::info!(
                messages_compressed = compressed,
                tokens_saved = stats.tokens_saved.load(Ordering::Relaxed),
                savings_pct = format!("{:.1}%", stats.savings() * 100.0),
                "compression: applied before LLM call"
            );
        }

        match self.inner.complete(&compressed_req).await {
            Ok(response) => Ok(response),
            Err(LlmError::Api { status, body }) if is_context_overflow(&body) => {
                tracing::warn!(
                    status,
                    "context overflow detected — applying aggressive compression and retrying"
                );
                // Aggressively compress: reduce preserve_recent to 1 and lower min_tokens
                let aggressive_config = CompressionConfig {
                    min_tokens: 50,
                    preserve_recent: 1,
                    target_ratio: Some(0.15),
                };
                let aggressive_req = self.compress_request_with_config(req, &aggressive_config);
                self.inner.complete(&aggressive_req).await
            }
            Err(e) => Err(e),
        }
    }

    fn default_model(&self) -> &str {
        self.inner.default_model()
    }

    fn complete_stream<'a>(
        &'a self,
        req: &'a CompletionRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<StreamEvent, LlmError>> + Send + 'a>> {
        let key = hash_request(req);
        let cached_request_ref = self.cached_compressed_request.clone();
        let inner = self.inner.clone();

        Box::pin(async_stream::try_stream! {
            let read_guard = cached_request_ref.read().await;

            if let Some(cached) = read_guard.as_ref() {
                let cached_key = *self.cached_request_key.lock().unwrap();
                if cached_key == Some(key) {
                    let req_ref = unsafe { &*(cached as *const CompletionRequest) };
                    drop(read_guard);

                    let mut stream = inner.complete_stream(req_ref);
                    while let Some(event) = stream.next().await {
                        yield event?;
                    }
                    return;
                }
            }
            drop(read_guard);

            {
                let mut write_guard = cached_request_ref.write().await;
                if let Some(cached) = write_guard.as_ref() {
                    let cached_key = *self.cached_request_key.lock().unwrap();
                    if cached_key == Some(key) {
                        let req_ref = unsafe { &*(cached as *const CompletionRequest) };
                        let _read_guard = write_guard.downgrade();

                        let mut stream = inner.complete_stream(req_ref);
                        while let Some(event) = stream.next().await {
                            yield event?;
                        }
                        return;
                    }
                }

                let compressed = self.compress_request(req);
                *write_guard = Some(compressed);
                *self.cached_request_key.lock().unwrap() = Some(key);

                let req_ref = unsafe { &*(write_guard.as_ref().unwrap() as *const CompletionRequest) };
                let _read_guard = write_guard.downgrade();

                let mut stream = inner.complete_stream(req_ref);
                while let Some(event) = stream.next().await {
                    yield event?;
                }
            }
        })
    }
}

fn hash_request(req: &CompletionRequest) -> u64 {
    let mut hasher = DefaultHasher::new();
    req.messages.hash(&mut hasher);
    hasher.finish()
}

impl Hash for Message {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.role.hash(state);
        self.content.hash(state);
        self.tool_calls.hash(state);
        self.tool_call_id.hash(state);
    }
}

/// Format a compressed message with a CCR retrieval marker.
pub fn format_ccr_message(handle: &CcrHandle, compressed_text: &str) -> String {
    format!(
        "{CCR_PREFIX}{handle} — use compress_restore tool to retrieve original]\n{compressed_text}",
        handle = handle.as_str(),
    )
}

/// Extract the CCR handle from a compressed message, if present.
pub fn extract_ccr_handle(content: &str) -> Option<&str> {
    let start = content.find(CCR_PREFIX)? + CCR_PREFIX.len();
    let rest = &content[start..];
    let end = rest.find(" — ")?;
    Some(&rest[..end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::Message;
    use sruja_compress::InMemoryCcrStore;

    // --- Mock LlmClient that captures the last request ---

    struct CapturingClient {
        last_messages: std::sync::Mutex<Vec<crate::llm::Message>>,
    }

    #[async_trait::async_trait]
    impl LlmClient for CapturingClient {
        async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
            *self.last_messages.lock().unwrap() = req.messages.clone();
            Ok(CompletionResponse::text("ok"))
        }
        fn default_model(&self) -> &str {
            "test-model"
        }
    }

    fn large_tool_result() -> String {
        let mut lines = Vec::new();
        for i in 0..500 {
            lines.push(format!(
                "line {i}: this is some tool output content that fills up the context"
            ));
        }
        lines.join("\n")
    }

    #[tokio::test]
    async fn compresses_old_tool_messages() {
        let inner = Arc::new(CapturingClient {
            last_messages: std::sync::Mutex::new(Vec::new()),
        });
        let ccr = Arc::new(InMemoryCcrStore::default());
        let client = CompressingClient::new(inner.clone())
            .with_ccr(ccr.clone())
            .with_config(CompressionConfig {
                min_tokens: 200,
                preserve_recent: 2, // only preserve last 2 messages
                target_ratio: Some(0.3),
            });

        let big = large_tool_result();
        let req = CompletionRequest::new(vec![
            Message::system("system"),
            Message::user("goal"),
            Message::assistant("checking"),
            Message::tool_result("tc1", big.clone()),
            Message::assistant("done with that"),
            Message::tool_result("tc2", "small result"),
            Message::assistant("final answer"),
        ]);

        client.complete(&req).await.unwrap();

        let captured = inner.last_messages.lock().unwrap();
        // Find the tool message that was big.
        let tool1 = captured
            .iter()
            .find(|m| m.tool_call_id.as_deref() == Some("tc1"))
            .unwrap();
        assert!(
            tool1.content.starts_with(CCR_PREFIX),
            "tool result should be compressed, got: {}",
            &tool1.content[..50.min(tool1.content.len())]
        );

        // Verify the CCR store has the original.
        let handle = extract_ccr_handle(&tool1.content).unwrap();
        let original = ccr.get(&CcrHandle(handle.to_string())).unwrap().unwrap();
        assert_eq!(original, big);

        // Small tool result should NOT be compressed.
        let tool2 = captured
            .iter()
            .find(|m| m.tool_call_id.as_deref() == Some("tc2"))
            .unwrap();
        assert!(
            !tool2.content.starts_with(CCR_PREFIX),
            "small result should not be compressed"
        );
    }

    #[tokio::test]
    async fn preserves_recent_messages() {
        let inner = Arc::new(CapturingClient {
            last_messages: std::sync::Mutex::new(Vec::new()),
        });
        let client = CompressingClient::new(inner.clone()).with_config(CompressionConfig {
            min_tokens: 10,
            preserve_recent: 2,
            target_ratio: Some(0.2),
        });

        let big = large_tool_result();
        // Only 3 messages: the tool is at index 2, within preserve_recent.
        let req = CompletionRequest::new(vec![
            Message::system("sys"),
            Message::assistant("call"),
            Message::tool_result("tc1", big),
        ]);

        client.complete(&req).await.unwrap();

        let captured = inner.last_messages.lock().unwrap();
        let tool_msg = captured
            .iter()
            .find(|m| m.role == MessageRole::Tool)
            .unwrap();
        assert!(
            !tool_msg.content.starts_with(CCR_PREFIX),
            "tool result within preserve_recent window should NOT be compressed"
        );
    }

    #[tokio::test]
    async fn idempotent_does_not_double_compress() {
        let inner = Arc::new(CapturingClient {
            last_messages: std::sync::Mutex::new(Vec::new()),
        });
        let client = CompressingClient::new(inner.clone());

        let big = large_tool_result();
        let req = CompletionRequest::new(vec![
            Message::system("s"),
            Message::user("g"),
            Message::assistant("a"),
            Message::tool_result("tc1", big),
            Message::assistant("final"),
            Message::user("done"),
        ]);

        // Call twice — second call should see the already-compressed message
        // and skip it (idempotent).
        client.complete(&req).await.unwrap();
        let after_first = inner.last_messages.lock().unwrap().clone();
        client.complete(&req).await.unwrap();
        // The second call processes the ORIGINAL request (not the compressed one),
        // so it will compress again. But since CCR is content-addressed, it
        // produces the same handle. The important thing is the result is stable.
        let after_second = inner.last_messages.lock().unwrap().clone();

        let tool1 = after_first
            .iter()
            .find(|m| m.role == MessageRole::Tool)
            .unwrap();
        let tool2 = after_second
            .iter()
            .find(|m| m.role == MessageRole::Tool)
            .unwrap();
        assert_eq!(tool1.content, tool2.content, "should be idempotent");
    }

    #[tokio::test]
    async fn stats_track_savings() {
        let inner = Arc::new(CapturingClient {
            last_messages: std::sync::Mutex::new(Vec::new()),
        });
        let client = CompressingClient::new(inner.clone()).with_config(CompressionConfig {
            min_tokens: 200,
            preserve_recent: 2,
            target_ratio: Some(0.3),
        });

        let big = large_tool_result();
        let req = CompletionRequest::new(vec![
            Message::system("s"),
            Message::user("g"),
            Message::assistant("a"),
            Message::tool_result("tc1", big),
            Message::assistant("final"),
            Message::user("done"),
            Message::user("another"),
        ]);

        client.complete(&req).await.unwrap();

        let stats = client.stats();
        assert!(stats.messages_compressed.load(Ordering::Relaxed) >= 1);
        assert!(stats.savings() > 0.0);
    }

    #[test]
    fn extract_handle_round_trip() {
        let handle = CcrHandle("abc123".to_string());
        let msg = format_ccr_message(&handle, "compressed text");
        let extracted = extract_ccr_handle(&msg).unwrap();
        assert_eq!(extracted, "abc123");
    }

    #[tokio::test]
    async fn e2e_compress_then_restore() {
        use crate::tool::{CompressRestoreTool, Tool};

        let inner = Arc::new(CapturingClient {
            last_messages: std::sync::Mutex::new(Vec::new()),
        });
        let ccr: Arc<dyn sruja_compress::CcrStore> = Arc::new(InMemoryCcrStore::default());

        let client = CompressingClient::new(inner.clone())
            .with_ccr(ccr.clone())
            .with_config(CompressionConfig {
                min_tokens: 200,
                preserve_recent: 2,
                target_ratio: Some(0.3),
            });

        let big = large_tool_result();
        let req = CompletionRequest::new(vec![
            Message::system("system"),
            Message::user("goal"),
            Message::assistant("checking"),
            Message::tool_result("tc1", big.clone()),
            Message::assistant("done"),
            Message::user("finalize"),
        ]);

        client.complete(&req).await.unwrap();

        let handle_str = {
            let captured = inner.last_messages.lock().unwrap();
            let tool_msg = captured
                .iter()
                .find(|m| m.tool_call_id.as_deref() == Some("tc1"))
                .unwrap();
            assert!(tool_msg.content.starts_with(CCR_PREFIX));
            extract_ccr_handle(&tool_msg.content).unwrap().to_string()
        };

        let restore_tool = CompressRestoreTool::new(ccr.clone());
        let restored = restore_tool
            .call(serde_json::json!({ "handle": handle_str }))
            .await
            .unwrap();

        assert_eq!(restored, big, "restore tool must return the exact original");
    }

    #[tokio::test]
    async fn e2e_bounded_ccr_store() {
        use sruja_compress::BoundedCcrStore;

        let inner = Arc::new(CapturingClient {
            last_messages: std::sync::Mutex::new(Vec::new()),
        });
        let ccr: Arc<dyn sruja_compress::CcrStore> = Arc::new(BoundedCcrStore::new(100));

        let client = CompressingClient::new(inner.clone())
            .with_ccr(ccr.clone())
            .with_config(CompressionConfig {
                min_tokens: 200,
                preserve_recent: 2,
                target_ratio: Some(0.3),
            });

        let big = large_tool_result();
        let req = CompletionRequest::new(vec![
            Message::system("system"),
            Message::user("goal"),
            Message::assistant("checking"),
            Message::tool_result("tc1", big.clone()),
            Message::assistant("done"),
            Message::user("finalize"),
        ]);

        client.complete(&req).await.unwrap();

        let captured = inner.last_messages.lock().unwrap();
        let tool_msg = captured
            .iter()
            .find(|m| m.tool_call_id.as_deref() == Some("tc1"))
            .unwrap();

        assert!(tool_msg.content.starts_with(CCR_PREFIX));

        let handle_str = extract_ccr_handle(&tool_msg.content).unwrap();
        assert_eq!(
            ccr.get(&CcrHandle(handle_str.to_string())).unwrap(),
            Some(big),
        );
    }
}
