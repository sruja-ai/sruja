//! Declarative loop manifest (`.sruja/loop.toml`).
//!
//! The manifest is the **user-facing contract** for autonomous loop runs:
//! structured goal, budget, scope, and deterministic verification steps.
//! It is loaded by the CLI and resolved with the standard priority chain:
//! CLI flags > manifest > defaults.

use serde::{Deserialize, Serialize};

use crate::goal::GoalSpec;
use crate::verify::VerifyStep;

fn default_max_iterations() -> usize {
    3
}
fn default_max_tool_iterations() -> usize {
    8
}
fn default_true() -> bool {
    true
}
fn default_ten() -> u64 {
    10
}
fn default_sixty() -> u64 {
    60
}
fn default_fail_on() -> String {
    "cycles,layer-violations".to_string()
}

/// Transport type for an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpTransport {
    /// Child process with stdio (newline-delimited JSON-RPC).
    Stdio,
    /// HTTP/SSE endpoint with streamable transport.
    Http,
}

/// Mutation policy for MCP tools from a server.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum McpMutationPolicy {
    /// Infer from server `trusted` + tool `readOnlyHint`: trusted+readOnlyHint=false → mutating; otherwise conservatively mutating.
    #[default]
    Auto,
    /// Treat all tools as read-only (force non-mutating).
    Readonly,
    /// Treat all tools as mutating (force mutating).
    Mutating,
}

/// MCP server declaration from the manifest `[[mcp.servers]]` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerDecl {
    /// Unique identifier for this server (used in tool namespace: `mcp__{name}__{tool}`).
    pub name: String,

    /// Transport type (stdio or HTTP).
    pub transport: McpTransport,

    /// Whether this server is enabled at startup. Default true.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Whether server startup failure is fatal. Default false (non‑fatal degradation).
    #[serde(default)]
    pub required: bool,

    // stdio fields
    /// Command to spawn for stdio transport (e.g., "npx", "mcp-server-browser").
    #[serde(default)]
    pub command: Option<String>,

    /// Arguments to pass to the stdio command.
    #[serde(default)]
    pub args: Vec<String>,

    /// Working directory for the stdio child process. Default: repository root.
    #[serde(default)]
    pub cwd: Option<String>,

    // HTTP fields
    /// URL for HTTP transport (e.g., "http://localhost:3000/mcp").
    #[serde(default)]
    pub url: Option<String>,

    /// HTTP headers to send with each request.
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,

    /// Authorization header (e.g., "Bearer ${TOKEN}"). Supports ${VAR} expansion.
    #[serde(default)]
    pub auth: Option<String>,

    // security fields
    /// Explicit environment variables to pass to the stdio child process (env vars, not shell variables).
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,

    /// Allowlist of environment variable names to forward from the agent to the child. Default: empty (no forwarding).
    #[serde(default)]
    pub env_allow: Vec<String>,

    // lifecycle fields
    /// Timeout in seconds for the initialization handshake. Default 10.
    #[serde(default = "default_ten")]
    pub init_timeout_secs: u64,

    /// Timeout in seconds per tool call. Default 60.
    #[serde(default = "default_sixty")]
    pub tool_timeout_secs: u64,

    // mutation policy
    /// Whether this server is trusted (affects default mutating classification when policy=auto). Default false.
    #[serde(default)]
    pub trusted: bool,

    /// Mutation policy for tools from this server. Default Auto.
    #[serde(default)]
    pub mutation: McpMutationPolicy,

    // tool filtering
    /// Allowlist of tool names to enable (glob patterns). Default: all tools enabled.
    #[serde(default)]
    pub enabled_tools: Option<Vec<String>>,

    /// Blocklist of tool names to disable (glob patterns). Default: none disabled.
    #[serde(default)]
    pub disabled_tools: Option<Vec<String>>,
}

/// Wrapper for the `[mcp]` table in loop.toml.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpConfig {
    /// Server declarations from `[[mcp.servers]]`.
    #[serde(default)]
    pub servers: Vec<McpServerDecl>,

    /// Allowlist of tool name globs to enable from all MCP servers.
    /// If set, only matching tools are registered; others are silently skipped.
    #[serde(default)]
    pub allowlist: Option<Vec<String>>,
}

impl McpConfig {
    pub fn is_empty(&self) -> bool {
        self.servers.is_empty()
    }
}

impl std::ops::Deref for McpConfig {
    type Target = [McpServerDecl];
    fn deref(&self) -> &Self::Target {
        &self.servers
    }
}

/// Override for a critique persona's model. Matches the default persona by `id`
/// and replaces its model. If `id` doesn't match a default, a new persona is
/// created with the default prompt for that focus area.
///
/// ## Example
///
/// ```toml
/// [[critique.personas]]
/// id = "correctness"
/// model = "GLM-5.1"
///
/// [[critique.personas]]
/// id = "regression"
/// model = "mimo-v2.5-pro"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritiquePersonaOverride {
    /// Persona id to override (e.g. "correctness", "spec_coverage", "boundary",
    /// "regression", "adversarial_test").
    pub id: String,
    /// Model name to route this persona to (e.g. "GLM-5.1", "mimo-v2.5-pro").
    /// The TieredClient resolves this to the correct provider via exact match
    /// or provider prefix match.
    pub model: String,
}

/// Configuration for the critique persona ensemble.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CritiqueConfig {
    /// Per-persona model overrides. Each entry maps a default persona id to a
    /// model name. Unlisted personas keep the default `models.review` tier.
    #[serde(default)]
    pub personas: Vec<CritiquePersonaOverride>,
}

/// Declarative configuration for `sruja agent loop`, loaded from `.sruja/loop.toml`.
///
/// ## Example
///
/// ```toml
/// [goal]
/// statement = "Add JWT authentication to all /api/* endpoints"
/// acceptance_criteria = [
///   "all existing tests pass",
///   "new tests cover token validation",
/// ]
/// constraints = ["do not modify the public API", "no new dependencies"]
///
/// max_iterations = 5
/// spend_cap_usd = 2.0
/// shell_allowlist = ["cargo", "git"]
///
/// [[verify]]
/// id = "tests"
/// command = "cargo"
/// args = ["test", "--workspace"]
///
/// # Cross-model adversarial review: each persona runs on a different provider
/// [[critique.personas]]
/// id = "correctness"
/// model = "GLM-5.1"
///
/// [[critique.personas]]
/// id = "regression"
/// model = "mimo-v2.5-pro"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopManifest {
    /// Structured goal specification (statement + acceptance criteria + constraints).
    /// If `statement` is empty, the CLI `--goal` flag is required.
    #[serde(default)]
    pub goal: GoalSpec,

    /// Maximum plan→execute→critique iterations.
    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,

    /// Max tool-call iterations per LLM phase (comprehension, plan, execute, critique).
    /// Soft guard fires at half, hard guard at quarter. Default: 8.
    #[serde(default = "default_max_tool_iterations")]
    pub max_tool_iterations: usize,

    /// Write tests before implementation (TDD mode).
    #[serde(default = "default_true")]
    pub tdd: bool,

    /// Run the critic after every tool execution.
    #[serde(default = "default_true")]
    pub review_every_change: bool,

    /// Block all file mutations (dry-run mode).
    #[serde(default)]
    pub dry_run: bool,

    /// Shell commands the agent is allowed to execute.
    #[serde(default)]
    pub shell_allowlist: Vec<String>,

    /// USD spend cap for the entire loop run.
    #[serde(default)]
    pub spend_cap_usd: Option<f64>,

    /// Detect and terminate on repeated critique patterns (oscillation).
    #[serde(default = "default_true")]
    pub detect_oscillation: bool,

    /// Drift --fail-on criteria for the default grader.
    ///
    /// Only used when `default_grader` is enabled and no user `[[verify]]`
    /// steps are configured. Defaults to high-severity architectural violations.
    #[serde(default = "default_fail_on")]
    pub default_grader_fail_on: String,

    /// Deterministic verification steps run after the loop completes.
    ///
    /// These are the **independent grader** — the agent that writes code
    /// cannot fake a passing `cargo test`. If any step fails, the loop
    /// result reports verification failure regardless of LLM critique.
    #[serde(default, rename = "verify")]
    pub verify_steps: Vec<VerifyStep>,

    /// MCP server declarations (stdio + HTTP). Each declared server is
    /// connected at loop startup and its tools exposed as `mcp__{server}__{tool}`.
    #[serde(default)]
    pub mcp: McpConfig,

    /// Automatically consolidate memory at the end of each loop run.
    ///
    /// When enabled, stale entries are archived and low-utility entries
    /// are pruned after trajectory persistence. Invariant entries are
    /// never touched. Defaults to `true`.
    #[serde(default = "default_true")]
    pub auto_consolidate: bool,

    /// Critique persona ensemble configuration.
    ///
    /// Override the model used by each default critique persona. This enables
    /// cross-model adversarial review: different personas run on different
    /// providers so the Judge has independent perspectives.
    #[serde(default)]
    pub critique: CritiqueConfig,
}

impl Default for LoopManifest {
    fn default() -> Self {
        Self {
            goal: GoalSpec::default(),
            max_iterations: default_max_iterations(),
            max_tool_iterations: default_max_tool_iterations(),
            tdd: default_true(),
            review_every_change: default_true(),
            dry_run: false,
            shell_allowlist: Vec::new(),
            spend_cap_usd: None,
            detect_oscillation: default_true(),
            default_grader_fail_on: default_fail_on(),
            verify_steps: Vec::new(),
            mcp: McpConfig::default(),
            auto_consolidate: default_true(),
            critique: CritiqueConfig::default(),
        }
    }
}

impl LoopManifest {
    /// Load from a TOML string.
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Load from a `.sruja/loop.toml` file path. Returns `Default` if the
    /// file does not exist (non-fatal — the manifest is optional).
    /// Logs a warning if the file exists but cannot be parsed.
    pub fn load_from_path(repo: &std::path::Path) -> Self {
        let path = repo.join(".sruja/loop.toml");
        match std::fs::read_to_string(&path) {
            Ok(content) => match Self::from_toml_str(&content) {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse {}: {e}. Using default loop config.",
                        path.display()
                    );
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_manifest() {
        let toml_str = r#"
max_iterations = 5
spend_cap_usd = 1.5
"#;
        let m = LoopManifest::from_toml_str(toml_str).unwrap();
        assert_eq!(m.max_iterations, 5);
        assert_eq!(m.spend_cap_usd, Some(1.5));
        assert!(m.goal.statement.is_empty());
        assert!(m.tdd);
    }

    #[test]
    fn parse_max_tool_iterations() {
        let toml_str = r#"
max_iterations = 5
max_tool_iterations = 12
"#;
        let m = LoopManifest::from_toml_str(toml_str).unwrap();
        assert_eq!(m.max_iterations, 5);
        assert_eq!(m.max_tool_iterations, 12);
    }

    #[test]
    fn max_tool_iterations_defaults_to_eight() {
        let toml_str = r#"
max_iterations = 3
"#;
        let m = LoopManifest::from_toml_str(toml_str).unwrap();
        assert_eq!(m.max_tool_iterations, 8);
    }

    #[test]
    fn parse_full_manifest() {
        let toml_str = r#"
max_iterations = 3
tdd = true
shell_allowlist = ["cargo", "git"]

[goal]
statement = "Add JWT auth"
acceptance_criteria = ["tests pass", "tokens validated"]
constraints = ["no new deps"]

[[verify]]
id = "tests"
command = "cargo"
args = ["test", "--workspace"]

[[verify]]
id = "lint"
command = "cargo"
args = ["clippy", "--", "-D", "warnings"]
"#;
        let m = LoopManifest::from_toml_str(toml_str).unwrap();
        assert_eq!(m.goal.statement, "Add JWT auth");
        assert_eq!(m.goal.acceptance_criteria.len(), 2);
        assert_eq!(m.goal.constraints, vec!["no new deps"]);
        assert_eq!(m.verify_steps.len(), 2);
        assert_eq!(m.verify_steps[0].id, "tests");
        assert_eq!(m.verify_steps[1].args[2], "-D");
        assert_eq!(m.shell_allowlist, vec!["cargo", "git"]);
    }

    #[test]
    fn empty_file_gives_default() {
        let m = LoopManifest::from_toml_str("").unwrap();
        assert_eq!(m.max_iterations, 3);
        assert!(m.tdd);
        assert!(m.review_every_change);
        assert!(m.mcp.servers.is_empty());
    }

    #[test]
    fn missing_file_gives_default() {
        let m = LoopManifest::load_from_path(std::path::Path::new("/nonexistent"));
        assert_eq!(m.max_iterations, 3);
    }

    #[test]
    fn parse_mcp_servers_stdio() {
        let toml_str = r#"
[[mcp.servers]]
name = "browser"
transport = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-browser"]
enabled = true
required = false
"#;
        let m = LoopManifest::from_toml_str(toml_str).unwrap();
        assert_eq!(m.mcp.servers.len(), 1);
        let s = &m.mcp.servers[0];
        assert_eq!(s.name, "browser");
        assert!(matches!(s.transport, McpTransport::Stdio));
        assert_eq!(s.command.as_deref(), Some("npx"));
        assert_eq!(s.args, vec!["-y", "@modelcontextprotocol/server-browser"]);
        assert!(s.enabled);
        assert!(!s.required);
    }

    #[test]
    fn parse_mcp_servers_http() {
        let toml_str = r#"
[[mcp.servers]]
name = "my-http-server"
transport = "http"
url = "http://localhost:3000/mcp"
headers = { "X-Custom" = "value" }
auth = "Bearer ${TOKEN}"
trusted = true
mutation = "readonly"
init_timeout_secs = 15
"#;
        let m = LoopManifest::from_toml_str(toml_str).unwrap();
        assert_eq!(m.mcp.servers.len(), 1);
        let s = &m.mcp.servers[0];
        assert_eq!(s.name, "my-http-server");
        assert!(matches!(s.transport, McpTransport::Http));
        assert_eq!(s.url.as_deref(), Some("http://localhost:3000/mcp"));
        assert_eq!(s.headers.get("X-Custom"), Some(&"value".to_string()));
        assert_eq!(s.auth.as_deref(), Some("Bearer ${TOKEN}"));
        assert!(s.trusted);
        assert!(matches!(s.mutation, McpMutationPolicy::Readonly));
        assert_eq!(s.init_timeout_secs, 15);
    }

    #[test]
    fn parse_mcp_allowlist() {
        let toml_str = r#"
[mcp]
allowlist = ["browser__navigate", "db__query"]
"#;
        let m = LoopManifest::from_toml_str(toml_str).unwrap();
        assert_eq!(
            m.mcp.allowlist.as_deref(),
            Some(&["browser__navigate".to_string(), "db__query".to_string()][..])
        );
    }

    #[test]
    fn parse_minimal_manifest_with_mcp() {
        let toml_str = r#"
[[mcp.servers]]
name = "browser"
transport = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-browser"]
"#;
        let m = LoopManifest::from_toml_str(toml_str).unwrap();
        assert_eq!(m.mcp.servers.len(), 1);
        let s = &m.mcp.servers[0];
        assert_eq!(s.name, "browser");
        assert_eq!(s.command.as_deref(), Some("npx"));
        assert_eq!(s.init_timeout_secs, 10);
        assert_eq!(s.tool_timeout_secs, 60);
    }

    #[test]
    fn parse_critique_personas() {
        let toml_str = r#"
[[critique.personas]]
id = "correctness"
model = "GLM-5.1"

[[critique.personas]]
id = "regression"
model = "mimo-v2.5-pro"
"#;
        let m = LoopManifest::from_toml_str(toml_str).unwrap();
        assert_eq!(m.critique.personas.len(), 2);
        assert_eq!(m.critique.personas[0].id, "correctness");
        assert_eq!(m.critique.personas[0].model, "GLM-5.1");
        assert_eq!(m.critique.personas[1].id, "regression");
        assert_eq!(m.critique.personas[1].model, "mimo-v2.5-pro");
    }

    #[test]
    fn empty_critique_personas_gives_default() {
        let m = LoopManifest::from_toml_str("").unwrap();
        assert!(m.critique.personas.is_empty());
    }
}
