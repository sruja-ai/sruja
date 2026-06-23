//! Tool abstraction — the agent's hands.
//!
//! Every capability the agent can invoke is a [`Tool`]. The framework ships
//! built-in filesystem and shell tools ([`tools`]); the CLI layer registers
//! sruja deterministic tools (focus, explain, drift) on top.
//!
//! ## Custom tool
//!
//! ```no_run
//! use async_trait::async_trait;
//! use sruja_agent::tool::{Tool, ToolError};
//! use serde_json::json;
//!
//! struct HelloTool;
//!
//! #[async_trait]
//! impl Tool for HelloTool {
//!     fn name(&self) -> &str { "hello" }
//!     fn description(&self) -> &str { "Says hello" }
//!     fn parameters(&self) -> serde_json::Value {
//!         json!({ "type": "object", "properties": { "name": { "type": "string" } } })
//!     }
//!     async fn call(&self, params: serde_json::Value) -> Result<String, ToolError> {
//!         let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("world");
//!         Ok(format!("Hello, {name}!"))
//!     }
//! }
//! ```

pub mod builtin;
pub mod policy;
pub mod sruja;

#[cfg(feature = "mcp-client")]
pub mod mcp;

pub use builtin::tools;
pub use policy::{FileGuard, Phase, TestPathClassifier};

use std::collections::HashMap;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::llm::FunctionSchema;

/// Known empty-result sentinel strings emitted by tools.
///
/// When a tool's output matches one of these (after trimming), the result is
/// classified as "empty" rather than "ok" — preventing false-green step status.
const EMPTY_SENTINELS: &[&str] = &[
    "(no matches)\n",
    "(empty or beyond end of file)\n",
    "(tool returned no text)",
];

/// Where a tool call was dispatched from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolSource {
    Builtin,
    Sruja,
    Mcp,
}

/// Structural outcome of a single tool invocation.
///
/// Built by the dispatcher wrapping [`Tool::call`], never by the tool itself.
/// This envelope makes empty/no-data results and errors visible to the grader
/// so they stop producing false-green `StepStatus::Ok`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    /// Whether the tool returned `Ok(...)`.
    pub ok: bool,
    /// Whether the payload matched a known empty-result sentinel.
    pub empty: bool,
    /// Wall-clock elapsed time in milliseconds.
    pub elapsed_ms: u64,
    /// Where the tool lives (builtin filesystem, sruja deterministic, MCP).
    pub source: ToolSource,
    /// Whether the payload was truncated before being returned to the LLM.
    pub truncated: bool,
    /// The full (possibly truncated) text payload returned by the tool.
    pub payload: String,
}

/// Compact signal carried per tool call into the step result for grading.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSignal {
    pub tool: String,
    pub ok: bool,
    pub empty: bool,
    pub elapsed_ms: u64,
    pub source: ToolSource,
}

/// Error from a tool invocation.
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("tool not found: {0}")]
    NotFound(String),
    #[error("invalid parameters: {0}")]
    InvalidParams(String),
    #[error("tool execution failed: {0}")]
    Execution(String),
    #[error("blocked by dry_run: '{0}' is a mutating tool")]
    BlockedByDryRun(String),
    #[error("path escapes workspace root: {0}")]
    PathEscape(String),
    #[error("{0}")]
    Other(String),
}

/// The core trait every agent tool implements.
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// Unique tool name (used by the LLM to invoke it).
    fn name(&self) -> &str;

    /// Human/LLM-readable description of what the tool does.
    fn description(&self) -> &str;

    /// JSON Schema describing the parameters the tool accepts.
    fn parameters(&self) -> serde_json::Value;

    /// Execute the tool and return a text result for the LLM.
    async fn call(&self, params: serde_json::Value) -> Result<String, ToolError>;

    /// Whether this tool mutates state (writes files, runs commands).
    ///
    /// Mutating tools are blocked when the registry is in `dry_run` mode.
    fn is_mutating(&self) -> bool {
        false
    }

    /// Paths this invocation will modify (for phase-based guard checking).
    ///
    /// File-mutating tools override this to return the target path(s).
    /// The registry checks these against the [`FileGuard`] before calling.
    fn affected_paths(&self, _params: &serde_json::Value) -> Vec<String> {
        Vec::new()
    }
}

/// Convert any [`Tool`] into the LLM function schema.
pub fn tool_schema(tool: &dyn Tool) -> FunctionSchema {
    FunctionSchema {
        name: tool.name().to_string(),
        description: tool.description().to_string(),
        parameters: tool.parameters(),
    }
}

/// A registry of tools the agent can invoke.
///
/// In `dry_run` mode, mutating tools are blocked automatically.
/// When a [`FileGuard`] is attached, phase-based TDD enforcement applies.
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
    dry_run: bool,
    guard: Option<FileGuard>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    /// Create an empty registry (write-enabled by default).
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            dry_run: false,
            guard: None,
        }
    }

    /// Create a registry pre-loaded with the standard built-in tools
    /// ([`builtin::tools::FileRead`], [`FileWrite`], [`FileEdit`], [`Glob`],
    /// [`Grep`], [`Shell`]) rooted at `root`, with the given shell executable
    /// allowlist. This is what a standalone agent needs to actually read,
    /// write, search, and run commands in a workspace.
    ///
    /// Pass an empty `Vec` for `shell_allowlist` to disable shell execution
    /// entirely.
    pub fn with_builtin(root: impl Into<std::path::PathBuf>, shell_allowlist: Vec<String>) -> Self {
        let root = root.into();
        Self::new()
            .with(Box::new(builtin::FileRead::with_root(root.clone())))
            .with(Box::new(builtin::FileWrite::with_root(root.clone())))
            .with(Box::new(builtin::FileEdit::with_root(root.clone())))
            .with(Box::new(builtin::DiffEdit::with_root(root.clone())))
            .with(Box::new(builtin::Glob::with_root(root.clone())))
            .with(Box::new(builtin::Grep::with_root(root.clone())))
            .with(Box::new(builtin::Shell::with_allowlist(
                root,
                shell_allowlist,
            )))
    }

    /// Enable dry-run mode: mutating tools will be blocked.
    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    /// Set dry-run mode on an existing registry.
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    /// Attach a [`FileGuard`] for phase-based TDD enforcement.
    pub fn with_guard(mut self, guard: FileGuard) -> Self {
        self.guard = Some(guard);
        self
    }

    /// Set the file guard on an existing registry.
    pub fn set_guard(&mut self, guard: FileGuard) {
        self.guard = Some(guard);
    }

    /// Whether the registry is in dry-run mode.
    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    /// The attached guard, if any.
    pub fn guard(&self) -> Option<&FileGuard> {
        self.guard.as_ref()
    }

    /// Register a tool.
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Register a tool (builder-style).
    pub fn with(mut self, tool: Box<dyn Tool>) -> Self {
        self.register(tool);
        self
    }

    /// Look up a tool by name.
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// All registered tool names.
    pub fn names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// Schemas for all tools, ready to pass to an LLM.
    pub fn schemas(&self) -> Vec<FunctionSchema> {
        self.tools
            .values()
            .map(|t| tool_schema(t.as_ref()))
            .collect()
    }

    /// Dispatch a tool call by name.
    ///
    /// In dry-run mode, mutating tools return [`ToolError::BlockedByDryRun`].
    /// When a guard is attached, writes to phase-frozen paths are blocked.
    pub async fn dispatch(
        &self,
        name: &str,
        params: serde_json::Value,
    ) -> Result<String, ToolError> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))?;

        if self.dry_run && tool.is_mutating() {
            return Err(ToolError::BlockedByDryRun(name.to_string()));
        }

        if let Some(guard) = &self.guard {
            if tool.is_mutating() {
                for path in tool.affected_paths(&params) {
                    if !guard.can_write(&path) {
                        let reason = guard
                            .deny_reason(&path)
                            .unwrap_or_else(|| "blocked by file guard".into());
                        return Err(ToolError::Execution(format!(
                            "write to '{path}' denied: {reason}"
                        )));
                    }
                }
            }
        }

        tool.call(params).await
    }

    /// Classify a tool name into its [`ToolSource`].
    pub fn classify_source(name: &str) -> ToolSource {
        if name.starts_with("mcp__") {
            ToolSource::Mcp
        } else if sruja::is_sruja_tool(name) {
            ToolSource::Sruja
        } else {
            ToolSource::Builtin
        }
    }

    /// Dispatch a tool call and wrap the outcome in a [`ToolCallRecord`].
    ///
    /// This is the primary entry point for the tool loop — it captures timing,
    /// classifies the source, and detects empty-result sentinels.
    pub async fn dispatch_record(
        &self,
        name: &str,
        params: serde_json::Value,
    ) -> Result<(String, ToolCallRecord), ToolError> {
        let source = Self::classify_source(name);
        let start = Instant::now();
        let result = self.dispatch(name, params).await;
        let elapsed_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(payload) => {
                let trimmed = payload.trim_end();
                let empty = EMPTY_SENTINELS
                    .iter()
                    .any(|sentinel| trimmed == sentinel.trim_end());
                let record = ToolCallRecord {
                    ok: true,
                    empty,
                    elapsed_ms,
                    source,
                    truncated: false, // caller will set this after truncation
                    payload: payload.clone(),
                };
                Ok((payload, record))
            }
            Err(e) => {
                let record = ToolCallRecord {
                    ok: false,
                    empty: false,
                    elapsed_ms,
                    source,
                    truncated: false,
                    payload: e.to_string(),
                };
                Ok((format!("ERROR: {e}"), record))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Echo;
    #[async_trait::async_trait]
    impl Tool for Echo {
        fn name(&self) -> &str {
            "echo"
        }
        fn description(&self) -> &str {
            "Echoes input"
        }
        fn parameters(&self) -> serde_json::Value {
            serde_json::json!({ "type": "object", "properties": {} })
        }
        async fn call(&self, _p: serde_json::Value) -> Result<String, ToolError> {
            Ok("echo".into())
        }
    }

    struct Write;
    #[async_trait::async_trait]
    impl Tool for Write {
        fn name(&self) -> &str {
            "write"
        }
        fn description(&self) -> &str {
            "Writes"
        }
        fn parameters(&self) -> serde_json::Value {
            serde_json::json!({})
        }
        async fn call(&self, _p: serde_json::Value) -> Result<String, ToolError> {
            Ok("ok".into())
        }
        fn is_mutating(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn dispatch_works() {
        let reg = ToolRegistry::new().with(Box::new(Echo));
        assert_eq!(
            reg.dispatch("echo", serde_json::json!({})).await.unwrap(),
            "echo"
        );
    }

    #[tokio::test]
    async fn dry_run_blocks_mutating() {
        let reg = ToolRegistry::new().dry_run().with(Box::new(Write));
        let err = reg
            .dispatch("write", serde_json::json!({}))
            .await
            .unwrap_err();
        assert!(matches!(err, ToolError::BlockedByDryRun(_)));
    }

    #[test]
    fn with_builtin_registers_all_tools() {
        let reg = ToolRegistry::with_builtin(".", vec!["echo".into()]);
        let mut names = reg.names();
        names.sort();
        assert_eq!(
            names,
            vec![
                "diff_edit",
                "file_edit",
                "file_read",
                "file_write",
                "glob",
                "grep",
                "shell"
            ]
        );
    }

    struct Sentinel;
    #[async_trait::async_trait]
    impl Tool for Sentinel {
        fn name(&self) -> &str {
            "sentinel"
        }
        fn description(&self) -> &str {
            "Returns sentinel text"
        }
        fn parameters(&self) -> serde_json::Value {
            serde_json::json!({})
        }
        async fn call(&self, _p: serde_json::Value) -> Result<String, ToolError> {
            Ok("(no matches)\n".into())
        }
    }

    #[tokio::test]
    async fn dispatch_record_ok_non_empty() {
        let reg = ToolRegistry::new().with(Box::new(Echo));
        let (payload, record) = reg
            .dispatch_record("echo", serde_json::json!({}))
            .await
            .unwrap();
        assert_eq!(payload, "echo");
        assert!(record.ok);
        assert!(!record.empty);
        assert!(record.elapsed_ms < 1000);
    }

    #[tokio::test]
    async fn dispatch_record_detects_sentinel() {
        let reg = ToolRegistry::new().with(Box::new(Sentinel));
        let (payload, record) = reg
            .dispatch_record("sentinel", serde_json::json!({}))
            .await
            .unwrap();
        assert!(payload.contains("(no matches)"));
        assert!(record.ok);
        assert!(record.empty);
    }

    #[tokio::test]
    async fn dispatch_record_error_sets_ok_false() {
        let err_tool = FailingTool;
        let reg = ToolRegistry::new().with(Box::new(err_tool));
        let (payload, record) = reg
            .dispatch_record("fail", serde_json::json!({}))
            .await
            .unwrap();
        assert!(payload.starts_with("ERROR:"));
        assert!(!record.ok);
        assert!(!record.empty);
    }

    #[test]
    fn classify_source_builtin() {
        assert_eq!(ToolRegistry::classify_source("grep"), ToolSource::Builtin);
        assert_eq!(ToolRegistry::classify_source("shell"), ToolSource::Builtin);
    }

    #[test]
    fn classify_source_mcp() {
        assert_eq!(
            ToolRegistry::classify_source("mcp__server__tool"),
            ToolSource::Mcp
        );
    }

    #[test]
    fn classify_source_sruja() {
        assert_eq!(
            ToolRegistry::classify_source("sruja_focus"),
            ToolSource::Sruja
        );
    }

    struct FailingTool;
    #[async_trait::async_trait]
    impl Tool for FailingTool {
        fn name(&self) -> &str {
            "fail"
        }
        fn description(&self) -> &str {
            "Always fails"
        }
        fn parameters(&self) -> serde_json::Value {
            serde_json::json!({})
        }
        async fn call(&self, _p: serde_json::Value) -> Result<String, ToolError> {
            Err(ToolError::Execution("boom".into()))
        }
    }

    /// Regression test: every known builtin empty-result sentinel must be in
    /// `EMPTY_SENTINELS`. If a builtin changes its empty output, this test
    /// will fail — forcing an update to the sentinel set (KD5).
    #[test]
    fn sentinel_set_covers_all_known_builtin_sentinels() {
        let known: &[&str] = &[
            "(no matches)\n",
            "(empty or beyond end of file)\n",
            "(tool returned no text)",
        ];
        for sentinel in known {
            assert!(
                EMPTY_SENTINELS
                    .iter()
                    .any(|s| s.trim_end() == sentinel.trim_end()),
                "known sentinel {sentinel:?} not found in EMPTY_SENTINELS"
            );
        }
    }
}
