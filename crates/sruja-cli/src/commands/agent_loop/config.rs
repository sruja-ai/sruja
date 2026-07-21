//! Configuration types and constants for the agent loop.

/// Default shell commands the agent is allowed to execute when the user hasn't
/// configured an explicit `shell_allowlist` in `.sruja/loop.toml`.
pub(crate) const DEFAULT_SHELL_ALLOWLIST: &[&str] = &["cargo", "git"];

/// Maximum file size (bytes) for pre-loading into the comprehension prompt.
pub(crate) const PRELOAD_MAX_BYTES: usize = 50 * 1024;

/// Maximum tokens for architecture context injection.
pub(crate) const ARCH_CONTEXT_MAX_TOKENS: usize = 2000;

/// Options received from the CLI.
#[derive(Debug)]
pub struct AgentLoopOptions<'a> {
    pub repo: &'a str,
    pub goal: &'a str,
    pub max_iterations: Option<usize>,
    pub no_tdd: bool,
    pub dry_run: bool,
    pub model: Option<&'a str>,
    pub base_url: Option<&'a str>,
    pub spend_cap_usd: Option<f64>,
    pub no_oscillation_detection: bool,
    pub format: &'a str,
    pub force_proceed: bool,
    pub no_default_grader: bool,
    pub steer: bool,
    pub resume: bool,
    pub show_plan: bool,
    pub plan_only: bool,
    pub show_pipeline: bool,
    pub pipeline_override: Option<std::path::PathBuf>,
    pub checkpoint: bool,
    pub no_checkpoint: bool,
    pub changelog: bool,
    pub verbose: bool,
}
