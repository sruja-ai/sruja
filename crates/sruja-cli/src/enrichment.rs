//! Shared enrichment arguments for LLM-powered narrative output.

use clap::Args;

/// CLI arguments for optional LLM enrichment.
///
/// Used by commands that support `--enrich` to add an optional LLM-generated
/// narrative section grounded in deterministic Sruja output.
#[derive(Args, Clone, Debug)]
pub struct EnrichmentArgs {
    /// Add an LLM-enriched narrative section to the output
    #[arg(long)]
    pub enrich: bool,

    /// Enrichment provider: cmd (default) or openai. Also via SRUJA_ENRICH_PROVIDER.
    #[arg(long, alias = "llm-provider")]
    pub enrich_provider: Option<String>,

    /// External enrichment command (reads JSON from stdin; writes markdown to stdout)
    #[arg(long)]
    pub enrich_cmd: Option<String>,

    /// Model name (used for provider=openai). Also via SRUJA_ENRICH_MODEL.
    #[arg(long, alias = "llm-model")]
    pub enrich_model: Option<String>,

    /// Base URL (used for provider=openai). Also via SRUJA_ENRICH_BASE_URL.
    #[arg(long, alias = "llm-base-url")]
    pub enrich_base_url: Option<String>,

    /// Timeout for enrichment in milliseconds (default: 15000)
    #[arg(long, default_value_t = 15000)]
    pub enrich_timeout_ms: u64,

    /// Max bytes to read from enrichment stdout (default: 20000)
    #[arg(long, default_value_t = 20000)]
    pub enrich_max_bytes: usize,
}

/// Borrowed view of enrichment args for passing to handler functions.
///
/// This avoids passing 7 individual parameters and lets handlers access
/// enrichment config through a single reference.
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct EnrichmentRef<'a> {
    /// Whether enrichment is enabled
    pub enrich: bool,
    /// Enrichment provider name
    pub provider: Option<&'a str>,
    /// External enrichment command
    pub cmd: Option<&'a str>,
    /// Model name
    pub model: Option<&'a str>,
    /// Base URL for OpenAI-compatible servers
    pub base_url: Option<&'a str>,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Max bytes from enrichment stdout
    pub max_bytes: usize,
}

impl EnrichmentArgs {
    /// Create a borrowed [`EnrichmentRef`] from owned args.
    pub fn as_ref(&self) -> EnrichmentRef<'_> {
        EnrichmentRef {
            enrich: self.enrich,
            provider: self.enrich_provider.as_deref(),
            cmd: self.enrich_cmd.as_deref(),
            model: self.enrich_model.as_deref(),
            base_url: self.enrich_base_url.as_deref(),
            timeout_ms: self.enrich_timeout_ms,
            max_bytes: self.enrich_max_bytes,
        }
    }
}
