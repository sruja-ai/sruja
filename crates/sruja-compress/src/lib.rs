//! sruja-compress: content-aware context compression for the agent's LLM calls.
//!
//! @element Sruja.Compress
//! @layer Core Engine
//! @boundary Compression is a read-only transform on message text; it must not
//!           depend on sruja-agent, sruja-cli, or any LLM provider crate. The
//!           agent layer wraps a [`TextCompressor`] around its LlmClient.
//!
//! ## Attribution
//!
//! This crate ports and adapts design patterns and algorithms from [Headroom](https://github.com/headroomlabs-ai/headroom)
//! by Tejas Chopra. The reversible CCR (content cache for recovery) approach,
//! extractive TextCrusher with BM25 recency/relevance scoring, near-duplicate
//! suppression via shingles, and the Kompress ONNX backend are inspired by and
//! derived from Headroom's architecture. Original Headroom is Apache-2.0 licensed.
//!
//! Sits *inside* the LLM call path — the agent owns its client directly, so
//! there is no proxy. A compressor transforms message text before it is sent,
//! and a reversible compressor hands the original to a content-addressed store
//! so the model can fetch it back via a `retrieve` tool.
//!
//! ## Why a dedicated small model, not the main LLM
//!
//! Compressing *input* with the expensive model spends the tokens you were
//! trying to save (a 50k-token tool output costs ~$0.90 to compress via Opus,
//! vs. free + 20ms locally). So:
//!
//! | Backend | When | Cost |
//! |---|---|---|
//! | [`extractive::TextCrusher`] (default) | high-volume tool output, logs, search dumps | free, local, deterministic |
//! | [`kompress::KompressBackend`] *(feature `kompress`)* | long prose / tool text needing ML salience | ~279MB, 13–29ms local |
//! | [`LlmSummarizerBackend`] | high-stakes, *reused* summaries (run/decision logs) | a paid LLM round-trip — opt-in only |
//!
//! The capable-model summarization path is what sruja already does in its
//! cognition `reflect` phase and `sruja-memory`; this crate fills the bulk-input
//! gap the main model is too expensive to service.

pub mod ccr;
pub mod extractive;
#[cfg(feature = "kompress")]
pub mod kompress;

pub use ccr::{BoundedCcrStore, CcrHandle, CcrStore, InMemoryCcrStore};
pub use extractive::{TextCrusher, TextCrusherConfig};

#[cfg(feature = "kompress")]
pub use kompress::{KompressBackend, KompressConfig, KompressVariant};

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// Compression never aborts the agent loop. On any error the caller falls back
/// to the original (uncompressed) text.
#[derive(Debug, thiserror::Error)]
pub enum CompressError {
    #[error("backend not available: {0}")]
    BackendUnavailable(String),
    #[error("model load failed: {0}")]
    Load(String),
    #[error("inference failed: {0}")]
    Inference(String),
    #[error("ccr store error: {0}")]
    Ccr(String),
    #[error("summarizer (LLM) error: {0}")]
    Summarizer(String),
}

/// Which message a piece of text came from. Tool outputs and logs compress
/// hardest and most safely; system prompts compress least.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Spans the compressor must never drop, applied as a post-pass restore.
/// Extractive ML classifiers ship with a trained must-keep overlay for
/// names/numbers/URLs/code identifiers; this policy covers the deterministic
/// layer and lets callers force-keep domain-critical tokens.
#[derive(Debug, Clone, Default)]
pub struct KeepPolicy {
    /// Keep fenced code blocks (` ``` ... ``` `) verbatim.
    pub keep_code_blocks: bool,
    /// Keep any line matching one of these (e.g. `^Error:`, `^\s*at\s`, `FATAL`).
    pub keep_line_patterns: Vec<regex::Regex>,
    /// Whole-word tokens to force-keep (identifiers, error codes).
    pub keep_words: Vec<String>,
}

impl KeepPolicy {
    /// Conservative defaults for tool output: never drop code or tracebacks.
    pub fn for_tool_output() -> Self {
        Self {
            keep_code_blocks: true,
            keep_line_patterns: vec![
                regex::Regex::new(r"(?i)(error|fatal|panic|exception|traceback)").unwrap(),
                regex::Regex::new(r"^\s*at\s").unwrap(),
                regex::Regex::new(r#"^\s*File ""#).unwrap(),
            ],
            keep_words: Vec::new(),
        }
    }
}

/// Hints passed to the compressor. All optional; backends use what they support.
#[derive(Debug, Clone, Default)]
pub struct CompressContext<'a> {
    /// Relevance query (e.g. the user's last message). BM25-style extractive
    /// compressors score segments against this.
    pub query: Option<&'a str>,
    pub role: Option<TextRole>,
    /// Target compressed/original ratio in [0.05, 1.0]. None = backend default.
    pub target_ratio: Option<f64>,
    pub keep: KeepPolicy,
}

/// Which compressor produced an output — surfaced in stats/tracing so savings
/// can be attributed per backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendId {
    Passthrough,
    TextCrusher,
    KompressSmall,
    KompressV2Base,
    LlmSummarizer,
}

/// The result of compressing one text block.
#[derive(Debug, Clone)]
pub struct Compressed {
    pub text: String,
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub backend: BackendId,
    /// If the original was stored for retrieval, the key the model uses to fetch
    /// it back. None when no store is configured (extractive = lossless-in-spirit).
    pub ccr_handle: Option<CcrHandle>,
}

impl Compressed {
    /// Savings fraction in [0, 1]. 0.0 = no reduction, 0.9 = 90% fewer tokens.
    pub fn savings(&self) -> f64 {
        if self.original_tokens == 0 {
            0.0
        } else {
            1.0 - (self.compressed_tokens as f64 / self.original_tokens as f64)
        }
    }
}

/// The core seam. Implementations are CPU-bound (deterministic extractive) or
/// local inference (ONNX); both sync. The LLM summarizer backend wraps its
/// (possibly async) call in a blocking join.
///
/// Implementations MUST be total: on any error return [`CompressError`] so the
/// caller falls back to the original text. Never panic on malformed input.
pub trait TextCompressor: Send + Sync {
    fn backend(&self) -> BackendId;
    fn compress(
        &self,
        content: &str,
        ctx: &CompressContext<'_>,
    ) -> Result<Compressed, CompressError>;
}

/// A compressor that also owns a reversible store. When `ccr_handle` is `Some`,
/// a `retrieve` tool can fetch the original via [`CcrStore::get`].
pub trait ReversibleCompressor: TextCompressor {
    fn store(&self) -> &dyn CcrStore;
}

/// Delegates compression to a capable LLM. Implementations live in the agent
/// layer (wrapping its `LlmClient`) to keep this crate provider-agnostic. Use
/// only for high-stakes, *reused* summaries — a paid round-trip per tool output
/// is a net loss (see crate docs).
pub trait LlmSummarizer: Send + Sync {
    fn summarize(&self, content: &str, ctx: &CompressContext<'_>) -> Result<String, CompressError>;
}

/// Adapter turning any [`LlmSummarizer`] into a [`TextCompressor`].
pub struct LlmSummarizerBackend<S: LlmSummarizer> {
    summarizer: S,
}

impl<S: LlmSummarizer> LlmSummarizerBackend<S> {
    pub fn new(summarizer: S) -> Self {
        Self { summarizer }
    }
}

impl<S: LlmSummarizer> TextCompressor for LlmSummarizerBackend<S> {
    fn backend(&self) -> BackendId {
        BackendId::LlmSummarizer
    }

    fn compress(
        &self,
        content: &str,
        ctx: &CompressContext<'_>,
    ) -> Result<Compressed, CompressError> {
        let original_tokens = count_tokens(content);
        let text = self.summarizer.summarize(content, ctx)?;
        let compressed_tokens = count_tokens(&text);
        Ok(Compressed {
            text,
            original_tokens,
            compressed_tokens,
            backend: BackendId::LlmSummarizer,
            // Abstractive summaries are not reversible: the model rephrased, so
            // storing the original and letting it "retrieve" would contradict
            // its own summary. Keep CCR for extractive backends only.
            ccr_handle: None,
        })
    }
}

/// Rough whitespace token count. Good enough for ratio math; providers differ
/// but this is consistent within a message. Swap for a real tokenizer at the
/// agent layer when available.
pub fn count_tokens(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Post-pass restore for [`KeepPolicy`]. Ensures force-kept lines (matching a
/// pattern, inside a fenced code block, or containing a keep-word) survive
/// compression even when the compressor drops them. Shared by all backends.
pub(crate) fn restore_kept(original: &str, compressed: &str, policy: &KeepPolicy) -> String {
    if policy.keep_line_patterns.is_empty() && !policy.keep_code_blocks && policy.keep_words.is_empty() {
        return compressed.to_string();
    }
    let must = must_keep_lines(original, policy);
    if must.is_empty() {
        return compressed.to_string();
    }
    let comp_lines: HashSet<&str> = compressed.lines().collect();
    let missing: Vec<String> = must
        .into_iter()
        .filter(|l| !comp_lines.contains(l.as_str()))
        .collect();
    if missing.is_empty() {
        compressed.to_string()
    } else {
        let mut out = String::with_capacity(
            compressed.len() + missing.iter().map(|l| l.len() + 1).sum::<usize>(),
        );
        out.push_str(compressed);
        out.push_str("\n\n[kept]\n");
        out.push_str(&missing.join("\n"));
        out
    }
}

fn must_keep_lines(original: &str, policy: &KeepPolicy) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_fence = false;
    let keep_words_set: HashSet<&str> = policy.keep_words.iter().map(|s| s.as_str()).collect();
    for line in original.lines() {
        let trimmed = line.trim();
        if policy.keep_code_blocks {
            if trimmed.starts_with("```") {
                in_fence = !in_fence;
                out.push(line.to_string());
                continue;
            }
            if in_fence {
                out.push(line.to_string());
                continue;
            }
        }
        // Check if line contains any keep words (whole-word match)
        if !keep_words_set.is_empty() {
            let words: Vec<&str> = line.split_whitespace().collect();
            if words.iter().any(|w| keep_words_set.contains(w)) {
                out.push(line.to_string());
                continue;
            }
        }
        for re in &policy.keep_line_patterns {
            if re.is_match(line) {
                out.push(line.to_string());
                break;
            }
        }
    }
    out
}
