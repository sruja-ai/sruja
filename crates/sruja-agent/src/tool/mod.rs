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

pub use builtin::tools;
pub use policy::{FileGuard, Phase, TestPathClassifier};

use std::collections::HashMap;

use crate::llm::FunctionSchema;

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
        self.tools.values().map(|t| tool_schema(t.as_ref())).collect()
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
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Echo;
    #[async_trait::async_trait]
    impl Tool for Echo {
        fn name(&self) -> &str { "echo" }
        fn description(&self) -> &str { "Echoes input" }
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
        fn name(&self) -> &str { "write" }
        fn description(&self) -> &str { "Writes" }
        fn parameters(&self) -> serde_json::Value { serde_json::json!({}) }
        async fn call(&self, _p: serde_json::Value) -> Result<String, ToolError> { Ok("ok".into()) }
        fn is_mutating(&self) -> bool { true }
    }

    #[tokio::test]
    async fn dispatch_works() {
        let reg = ToolRegistry::new().with(Box::new(Echo));
        assert_eq!(reg.dispatch("echo", serde_json::json!({})).await.unwrap(), "echo");
    }

    #[tokio::test]
    async fn dry_run_blocks_mutating() {
        let reg = ToolRegistry::new().dry_run().with(Box::new(Write));
        let err = reg.dispatch("write", serde_json::json!({})).await.unwrap_err();
        assert!(matches!(err, ToolError::BlockedByDryRun(_)));
    }
}
