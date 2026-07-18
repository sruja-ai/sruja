//! Built-in sruja deterministic tools — the agent's "eyes".
//!
//! Each tool wraps a sruja CLI command, spawning it as a subprocess and
//! returning the JSON output. These are what make the agent sruja-native
//! rather than a generic LLM wrapper.
//!
//! The agent is *forced* to cite architecture element IDs from these tools
//! before making changes — no guessing allowed.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde_json::{json, Value};

use super::{Tool, ToolError};

/// Simple TTL cache for sruja CLI results to avoid redundant subprocess spawns.
struct SrujaResultCache {
    entries: Mutex<std::collections::HashMap<String, (String, Instant)>>,
    ttl: Duration,
}

impl SrujaResultCache {
    fn new(ttl: Duration) -> Self {
        Self {
            entries: Mutex::new(std::collections::HashMap::new()),
            ttl,
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        let mut cache = self.entries.lock().unwrap();
        if let Some((value, inserted_at)) = cache.get(key) {
            if inserted_at.elapsed() < self.ttl {
                return Some(value.clone());
            }
            cache.remove(key);
        }
        None
    }

    fn insert(&self, key: String, value: String) {
        let mut cache = self.entries.lock().unwrap();
        // Evict expired entries on insert to prevent unbounded growth
        cache.retain(|_, (_, t)| t.elapsed() < self.ttl);
        cache.insert(key, (value, Instant::now()));
    }
}

/// Global cache for sruja CLI results — 60-second TTL.
static SRUJA_CACHE: std::sync::OnceLock<SrujaResultCache> = std::sync::OnceLock::new();

fn sruja_cache() -> &'static SrujaResultCache {
    SRUJA_CACHE.get_or_init(|| SrujaResultCache::new(Duration::from_secs(60)))
}

/// Returns `true` if `name` belongs to a sruja deterministic tool.
pub fn is_sruja_tool(name: &str) -> bool {
    name.starts_with("sruja_")
}

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
    // 4. Search PATH — validate before returning
    if let Ok(path) = std::process::Command::new("which")
        .arg("sruja")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    {
        if !path.is_empty() {
            return PathBuf::from(path);
        }
    }
    // 5. Bare fallback — will fail at spawn time with a clear error
    tracing::error!(
        "Cannot locate sruja binary. Set SRUJA_PATH, build with 'cargo build --release', \
         or ensure 'sruja' is on PATH."
    );
    PathBuf::from("sruja")
}

/// Run a sruja command and return stdout. Uses a 60-second TTL cache to avoid
/// redundant subprocess spawns for repeated calls with the same arguments.
async fn run_sruja(
    sruja_path: &Path,
    repo_root: &Path,
    args: &[&str],
) -> Result<String, ToolError> {
    let cache_key = format!(
        "{}:{}:{}",
        repo_root.display(),
        sruja_path.display(),
        args.join(" ")
    );

    // Check cache first
    if let Some(cached) = sruja_cache().get(&cache_key) {
        tracing::debug!("sruja cache hit for args: {:?}", args);
        return Ok(cached);
    }

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

    let result = String::from_utf8_lossy(&output.stdout).to_string();
    sruja_cache().insert(cache_key, result.clone());
    Ok(result)
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
            &["explain", &element_id, "--json"],
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
// Lookup — compact concept card for a single architecture element
// ---------------------------------------------------------------------------

/// Fetch a compact concept card for one architecture element — the cheap,
/// single-element alternative to `sruja_focus`. Use when you need just one
/// element's shape (kind, purpose, tech, edges) instead of a full briefing.
pub struct SrujaLookupTool {
    pub sruja_path: PathBuf,
    pub repo_root: PathBuf,
}

impl SrujaLookupTool {
    pub fn new(repo_root: impl Into<PathBuf>) -> Self {
        let root = repo_root.into();
        Self {
            sruja_path: find_sruja(&root),
            repo_root: root,
        }
    }
}

#[async_trait]
impl Tool for SrujaLookupTool {
    fn name(&self) -> &str {
        "sruja_lookup"
    }

    fn description(&self) -> &str {
        "Fetch a compact concept card for one architecture element. Returns kind, purpose, \
         technology, path, and incoming/outgoing edges as a small JSON record. Use INSTEAD of \
         sruja_focus when you need only one element's shape — far fewer tokens."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Element name or ID (e.g. 'Sruja.CLI' or 'Auth.Handler'); exact match, then best-effort suffix match"
                }
            },
            "required": ["name"]
        })
    }

    async fn call(&self, params: Value) -> Result<String, ToolError> {
        let name = super::builtin::str_param(&params, "name")?;
        let output = run_sruja(
            &self.sruja_path,
            &self.repo_root,
            &["lookup", &name, "-r", ".", "--format", "json"],
        )
        .await?;
        Ok(truncate_json(&output, 4_000))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn truncate_json(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars {
        s.to_string()
    } else {
        // Use char_indices to avoid splitting multi-byte UTF-8 characters
        let truncated: String = s.chars().take(max_chars).collect();
        let total_chars = s.chars().count();
        format!(
            "{}\n... (truncated, {} total chars)",
            truncated, total_chars
        )
    }
}

/// Re-exports for ergonomic registration.
pub mod tools {
    pub use super::{
        SrujaComplianceTool, SrujaDriftTool, SrujaExplainTool, SrujaFocusTool, SrujaLookupTool,
        SrujaQueryTool,
    };
}
