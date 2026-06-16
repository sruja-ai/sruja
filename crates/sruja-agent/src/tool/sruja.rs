//! Built-in sruja deterministic tools — the agent's "eyes".
//!
//! Each tool wraps a sruja CLI command, spawning it as a subprocess and
//! returning the JSON output. These are what make the agent sruja-native
//! rather than a generic LLM wrapper.
//!
//! The agent is *forced* to cite architecture element IDs from these tools
//! before making changes — no guessing allowed.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde_json::{json, Value};

use super::{Tool, ToolError};

/// Resolve the sruja binary, checking common locations.
pub fn find_sruja(repo_root: &Path) -> PathBuf {
    // 1. Explicit env var
    if let Ok(path) = std::env::var("SRUJA_PATH") {
        return PathBuf::from(path);
    }
    // 2. target/release/sruja in workspace
    let release = repo_root.join("target/release/sruja");
    if release.exists() {
        return release;
    }
    // 3. target/debug/sruja
    let debug = repo_root.join("target/debug/sruja");
    if debug.exists() {
        return debug;
    }
    // 4. Assume it's in PATH
    PathBuf::from("sruja")
}

/// Run a sruja command and return stdout.
async fn run_sruja(
    sruja_path: &Path,
    repo_root: &Path,
    args: &[&str],
) -> Result<String, ToolError> {
    let output = tokio::process::Command::new(sruja_path)
        .args(args)
        .current_dir(repo_root)
        .output()
        .await
        .map_err(|e| ToolError::Execution(format!("failed to run sruja: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ToolError::Execution(format!(
            "sruja exited with {}: {stderr}",
            output.status.code().unwrap_or(-1)
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ---------------------------------------------------------------------------
// Focus — blast radius, boundaries, decisions, memory hits for a target
// ---------------------------------------------------------------------------

/// Get a focus briefing for a file or element — the pre-edit context.
pub struct SrujaFocusTool {
    pub sruja_path: PathBuf,
    pub repo_root: PathBuf,
}

impl SrujaFocusTool {
    pub fn new(repo_root: impl Into<PathBuf>) -> Self {
        let root = repo_root.into();
        Self {
            sruja_path: find_sruja(&root),
            repo_root: root,
        }
    }
}

#[async_trait]
impl Tool for SrujaFocusTool {
    fn name(&self) -> &str {
        "sruja_focus"
    }

    fn description(&self) -> &str {
        "Get a focus briefing for a file or architecture element. Returns: blast radius, \
         boundaries, active drift violations, decisions, memory hits, anti-patterns, \
         and AI instructions. Use this BEFORE any code change."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "File path (e.g. 'src/main.rs') or element ID (e.g. 'Sruja.CLI')"
                }
            },
            "required": ["target"]
        })
    }

    async fn call(&self, params: Value) -> Result<String, ToolError> {
        let target = super::builtin::str_param(&params, "target")?;
        let output = run_sruja(
            &self.sruja_path,
            &self.repo_root,
            &["focus", "--file", &target, "--format", "json"],
        )
        .await?;
        Ok(truncate_json(&output, 12_000))
    }
}

// ---------------------------------------------------------------------------
// Explain — describe an architecture element and its relationships
// ---------------------------------------------------------------------------

/// Explain an architecture element: what it is, what depends on it, what it depends on.
pub struct SrujaExplainTool {
    pub sruja_path: PathBuf,
    pub repo_root: PathBuf,
}

impl SrujaExplainTool {
    pub fn new(repo_root: impl Into<PathBuf>) -> Self {
        let root = repo_root.into();
        Self {
            sruja_path: find_sruja(&root),
            repo_root: root,
        }
    }
}

#[async_trait]
impl Tool for SrujaExplainTool {
    fn name(&self) -> &str {
        "sruja_explain"
    }

    fn description(&self) -> &str {
        "Explain an architecture element: description, technology, relationships, \
         dependencies, and role in the system. Use to understand before modifying."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "element_id": {
                    "type": "string",
                    "description": "Architecture element ID (e.g. 'Sruja.CLI' or 'System.Container.Component')"
                }
            },
            "required": ["element_id"]
        })
    }

    async fn call(&self, params: Value) -> Result<String, ToolError> {
        let element_id = super::builtin::str_param(&params, "element_id")?;
        let output = run_sruja(
            &self.sruja_path,
            &self.repo_root,
            &["explain", &element_id, "--format", "json"],
        )
        .await?;
        Ok(truncate_json(&output, 8_000))
    }
}

// ---------------------------------------------------------------------------
// Drift — detect architectural drift between reality and DSL
// ---------------------------------------------------------------------------

/// Detect architectural drift: what's changed vs what's documented.
pub struct SrujaDriftTool {
    pub sruja_path: PathBuf,
    pub repo_root: PathBuf,
}

impl SrujaDriftTool {
    pub fn new(repo_root: impl Into<PathBuf>) -> Self {
        let root = repo_root.into();
        Self {
            sruja_path: find_sruja(&root),
            repo_root: root,
        }
    }
}

#[async_trait]
impl Tool for SrujaDriftTool {
    fn name(&self) -> &str {
        "sruja_drift"
    }

    fn description(&self) -> &str {
        "Detect architectural drift: compare the actual codebase against the documented \
         architecture (repo.sruja). Returns violations, unproposed changes, and gaps. \
         Run BEFORE and AFTER making changes."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "fix": {
                    "type": "boolean",
                    "description": "Attempt to auto-fix drift (default: false)"
                }
            }
        })
    }

    async fn call(&self, params: Value) -> Result<String, ToolError> {
        let mut args = vec!["drift", "--format", "json"];
        let fix = params.get("fix").and_then(|v| v.as_bool()).unwrap_or(false);
        if fix {
            args.push("--fix");
        }
        let output = run_sruja(&self.sruja_path, &self.repo_root, &args).await?;
        Ok(truncate_json(&output, 10_000))
    }
}

// ---------------------------------------------------------------------------
// Compliance — validate changes against architecture boundaries
// ---------------------------------------------------------------------------

/// Check compliance: do changes respect architecture boundaries and policies?
pub struct SrujaComplianceTool {
    pub sruja_path: PathBuf,
    pub repo_root: PathBuf,
}

impl SrujaComplianceTool {
    pub fn new(repo_root: impl Into<PathBuf>) -> Self {
        let root = repo_root.into();
        Self {
            sruja_path: find_sruja(&root),
            repo_root: root,
        }
    }
}

#[async_trait]
impl Tool for SrujaComplianceTool {
    fn name(&self) -> &str {
        "sruja_compliance"
    }

    fn description(&self) -> &str {
        "Check compliance: do current changes respect architecture boundaries, policies, \
         and constraints? Returns violations with severity. This is the final gate \
         before merging."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "arch_file": {
                    "type": "string",
                    "description": "Architecture file to check against (default: repo.sruja)"
                }
            }
        })
    }

    async fn call(&self, params: Value) -> Result<String, ToolError> {
        let mut args = vec!["check", "--format", "json"];
        if let Some(arch) = params.get("arch_file").and_then(|v| v.as_str()) {
            args.push("-a");
            args.push(arch);
        }
        let output = run_sruja(&self.sruja_path, &self.repo_root, &args).await?;
        Ok(truncate_json(&output, 8_000))
    }
}

// ---------------------------------------------------------------------------
// Query — search the architecture knowledge graph
// ---------------------------------------------------------------------------

/// Query the architecture knowledge graph for relationships and patterns.
pub struct SrujaQueryTool {
    pub sruja_path: PathBuf,
    pub repo_root: PathBuf,
}

impl SrujaQueryTool {
    pub fn new(repo_root: impl Into<PathBuf>) -> Self {
        let root = repo_root.into();
        Self {
            sruja_path: find_sruja(&root),
            repo_root: root,
        }
    }
}

#[async_trait]
impl Tool for SrujaQueryTool {
    fn name(&self) -> &str {
        "sruja_query"
    }

    fn description(&self) -> &str {
        "Query the architecture knowledge graph. Find paths between elements, \
         discover entrypoints, locate data stores, or search for components."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "What to find (e.g. 'all database connections', 'path from API to DB')"
                }
            },
            "required": ["query"]
        })
    }

    async fn call(&self, params: Value) -> Result<String, ToolError> {
        let query = super::builtin::str_param(&params, "query")?;
        let output = run_sruja(
            &self.sruja_path,
            &self.repo_root,
            &["discover", "context", &query, "--format", "json"],
        )
        .await?;
        Ok(truncate_json(&output, 10_000))
    }
}
// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn truncate_json(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars {
        s.to_string()
    } else {
        format!("{}\n... (truncated, {} total chars)", &s[..max_chars], s.len())
    }
}

/// Re-exports for ergonomic registration.
pub mod tools {
    pub use super::{
        SrujaComplianceTool, SrujaDriftTool, SrujaExplainTool, SrujaFocusTool, SrujaQueryTool,
    };
}
