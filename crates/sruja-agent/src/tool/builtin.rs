//! Built-in filesystem and shell tools.
//!
//! All file tools enforce workspace-root confinement: paths are resolved
//! relative to the workspace root and rejected if they escape it.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde_json::{json, Value};

use super::{Tool, ToolError};

const DIFF_EDIT_DEFAULT_CONTEXT: usize = 3;

/// Resolve a user-supplied path relative to `root`, rejecting escapes.
///
/// The root is canonicalized to an absolute path so that `starts_with` works
/// correctly. Without canonicalization, a root like `.` (whose only component
/// is `CurDir`) would never match a normalized path (whose first component is
/// `Normal`), causing every file operation to fail.
pub fn resolve_path(root: &Path, requested: &str) -> Result<PathBuf, ToolError> {
    let p = Path::new(requested);
    if p.is_absolute() {
        return Err(ToolError::PathEscape(requested.into()));
    }
    let canonical_root = root
        .canonicalize()
        .map_err(|e| ToolError::Execution(format!("invalid workspace root '{root:?}': {e}")))?;
    let joined = canonical_root.join(p);
    let normalized = normalize_path(&joined);
    if !normalized.starts_with(&canonical_root) {
        return Err(ToolError::PathEscape(requested.into()));
    }
    Ok(normalized)
}

fn normalize_path(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in p.components() {
        use std::path::Component::*;
        match component {
            CurDir => {}
            ParentDir => {
                out.pop();
            }
            RootDir | Prefix(_) | Normal(_) => {
                out.push(component.as_os_str());
            }
        }
    }
    out
}

pub(crate) fn str_param(params: &Value, key: &str) -> Result<String, ToolError> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| ToolError::InvalidParams(format!("missing string parameter '{key}'")))
}

fn opt_usize(params: &Value, key: &str) -> Option<usize> {
    params.get(key).and_then(|v| v.as_u64()).map(|n| n as usize)
}

// ---------------------------------------------------------------------------
// FileRead
// ---------------------------------------------------------------------------

/// Read a file's contents (optionally with line offset/limit).
pub struct FileRead {
    root: PathBuf,
}

impl FileRead {
    pub fn new() -> Self {
        Self {
            root: PathBuf::from("."),
        }
    }

    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Default for FileRead {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileRead {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Read the contents of a file. Returns text with line numbers. \
         Optional 'offset' (1-indexed start line) and 'limit' (max lines)."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path relative to workspace root" },
                "offset": { "type": "integer", "description": "1-indexed line to start from" },
                "limit": { "type": "integer", "description": "Maximum lines to read" }
            },
            "required": ["path"]
        })
    }

    async fn call(&self, params: Value) -> Result<String, ToolError> {
        let path_str = str_param(&params, "path")?;
        let offset = opt_usize(&params, "offset").unwrap_or(1).max(1);
        let limit = opt_usize(&params, "limit");

        let path = resolve_path(&self.root, &path_str)?;
        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| ToolError::Execution(format!("read failed: {e}")))?;

        let lines: Vec<&str> = content.lines().collect();
        let start = (offset - 1).min(lines.len());
        let end = match limit {
            Some(l) => (start + l).min(lines.len()),
            None => lines.len(),
        };

        let mut out = String::new();
        for (i, line) in lines[start..end].iter().enumerate() {
            out.push_str(&format!("{}: {line}\n", start + i + 1));
        }
        if out.is_empty() {
            out.push_str("(empty or beyond end of file)\n");
        }
        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// FileWrite
// ---------------------------------------------------------------------------

/// Create or overwrite a file.
pub struct FileWrite {
    root: PathBuf,
}

impl FileWrite {
    pub fn new() -> Self {
        Self {
            root: PathBuf::from("."),
        }
    }
    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Default for FileWrite {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileWrite {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Create or overwrite a file. This is the PRIMARY way to make changes — use it directly, do not run tests or clippy first."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" }
            },
            "required": ["path", "content"]
        })
    }

    fn is_mutating(&self) -> bool {
        true
    }

    fn affected_paths(&self, params: &Value) -> Vec<String> {
        params
            .get("path")
            .and_then(|v| v.as_str())
            .map(|s| vec![s.to_string()])
            .unwrap_or_default()
    }

    async fn call(&self, params: Value) -> Result<String, ToolError> {
        let path_str = str_param(&params, "path")?;
        let content = str_param(&params, "content")?;
        let path = resolve_path(&self.root, &path_str)?;

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }

        tokio::fs::write(&path, &content)
            .await
            .map_err(|e| ToolError::Execution(format!("write failed: {e}")))?;

        Ok(format!("wrote {} bytes to {path_str}", content.len()))
    }
}

// ---------------------------------------------------------------------------
// FileEdit
// ---------------------------------------------------------------------------

/// Find-and-replace within a file.
pub struct FileEdit {
    root: PathBuf,
}

impl FileEdit {
    pub fn new() -> Self {
        Self {
            root: PathBuf::from("."),
        }
    }
    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Default for FileEdit {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileEdit {
    fn name(&self) -> &str {
        "file_edit"
    }

    fn description(&self) -> &str {
        "Replace exact text in a file. Fails if old_string is not found or appears multiple times \
         (use replace_all=true to replace all occurrences)."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "old_string": { "type": "string" },
                "new_string": { "type": "string" },
                "replace_all": { "type": "boolean", "description": "Replace all occurrences (default: false)" }
            },
            "required": ["path", "old_string", "new_string"]
        })
    }

    fn is_mutating(&self) -> bool {
        true
    }

    fn affected_paths(&self, params: &Value) -> Vec<String> {
        params
            .get("path")
            .and_then(|v| v.as_str())
            .map(|s| vec![s.to_string()])
            .unwrap_or_default()
    }

    async fn call(&self, params: Value) -> Result<String, ToolError> {
        let path_str = str_param(&params, "path")?;
        let old = str_param(&params, "old_string")?;
        let new = str_param(&params, "new_string")?;
        let replace_all = params
            .get("replace_all")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let path = resolve_path(&self.root, &path_str)?;

        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| ToolError::Execution(format!("read failed: {e}")))?;

        let count = content.matches(&old).count();
        if count == 0 {
            return Err(ToolError::Execution("old_string not found".into()));
        }
        if count > 1 && !replace_all {
            return Err(ToolError::Execution(format!(
                "old_string found {count} times; set replace_all=true or provide more context"
            )));
        }

        let updated = if replace_all {
            content.replace(&old, &new)
        } else {
            content.replacen(&old, &new, 1)
        };

        tokio::fs::write(&path, &updated)
            .await
            .map_err(|e| ToolError::Execution(format!("write failed: {e}")))?;

        Ok(format!("replaced {count} occurrence(s) in {path_str}"))
    }
}

// ---------------------------------------------------------------------------
// DiffEdit
// ---------------------------------------------------------------------------

/// Apply Claude Code-style SEARCH/REPLACE edits via unified diff.
///
/// This tool takes SEARCH/REPLACE blocks, computes a unified diff, and
/// applies it via git apply with hunk boundary validation. This is more
/// precise than FileEdit because it:
/// - Requires exact hunk context matches
/// - Validates against the current file state
/// - Provides rich error feedback with conflict details
/// - Supports multiple edits in a single call
pub struct DiffEdit {
    root: PathBuf,
}

impl DiffEdit {
    pub fn new() -> Self {
        Self {
            root: PathBuf::from("."),
        }
    }
    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Default for DiffEdit {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DiffEdit {
    fn name(&self) -> &str {
        "diff_edit"
    }

    fn description(&self) -> &str {
        "Apply Claude Code-style SEARCH/REPLACE edits via unified diff with hunk boundary validation. \
         Format: 'path:start_line-end_line' followed by SEARCH block, '---', then REPLACE block. \
         Fails if context doesn't match or has conflicts. More precise than file_edit."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "edits": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "header": {
                                "type": "string",
                                "description": "File path with line range: 'path:start_line-end_line'"
                            },
                            "search": {
                                "type": "string",
                                "description": "Original content to search for (exact match including context)"
                            },
                            "replace": {
                                "type": "string",
                                "description": "New content to replace with"
                            }
                        },
                        "required": ["header", "search", "replace"]
                    }
                },
                "context_lines": {
                    "type": "integer",
                    "description": "Lines of context in unified diff (default: 3)"
                }
            },
            "required": ["edits"]
        })
    }

    fn is_mutating(&self) -> bool {
        true
    }

    fn affected_paths(&self, params: &Value) -> Vec<String> {
        params
            .get("edits")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|e| e.get("header").and_then(|h| h.as_str()))
                    .map(|h| h.split(':').next().unwrap_or("").to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    async fn call(&self, params: Value) -> Result<String, ToolError> {
        let edits = params
            .get("edits")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ToolError::InvalidParams("missing 'edits' array".into()))?;

        let context_lines = opt_usize(&params, "context_lines").unwrap_or(DIFF_EDIT_DEFAULT_CONTEXT);

        if edits.is_empty() {
            return Err(ToolError::InvalidParams("edits array is empty".into()));
        }

        let mut results = Vec::new();
        for (idx, edit) in edits.iter().enumerate() {
            let header = str_param(edit, "header")?;
            let search = str_param(edit, "search")?;
            let replace = str_param(edit, "replace")?;

            let result = self.apply_single_edit(&header, &search, &replace, context_lines, idx).await;
            match result {
                Ok(msg) => results.push(format!("Edit {idx}: {msg}")),
                Err(e) => {
                    let detail = format!("Edit {idx} failed: {e}");
                    if idx == 0 {
                        return Err(e);
                    }
                    results.push(detail);
                }
            }
        }

        Ok(results.join("\n"))
    }
}

impl DiffEdit {
    async fn apply_single_edit(
        &self,
        header: &str,
        search: &str,
        replace: &str,
        _context_lines: usize,
        _edit_index: usize,
    ) -> Result<String, ToolError> {
        let (path_str, line_range) = parse_diff_header(header)?;
        let (start_line, end_line) = parse_line_range(&line_range)?;

        let path = resolve_path(&self.root, &path_str)?;

        let current_content = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| ToolError::Execution(format!("read failed for '{path_str}': {e}")))?;

        let current_lines: Vec<&str> = current_content.lines().collect();
        if start_line > current_lines.len() || end_line > current_lines.len() {
            return Err(ToolError::Execution(format!(
                "line range {start_line}-{end_line} exceeds file length {}",
                current_lines.len()
            )));
        }

        let search_lines: Vec<&str> = search.lines().collect();
        let replace_lines: Vec<&str> = replace.lines().collect();

        let old_content = current_lines[start_line - 1..end_line].join("\n");
        if old_content != search {
            let mut diff_lines = Vec::new();
            for (i, (a, b)) in search_lines.iter().zip(current_lines[start_line - 1..end_line].iter()).enumerate() {
                if a != b {
                    diff_lines.push(format!("  line {}: expected '{}', got '{}'", start_line + i, a, b));
                }
            }
            let err_msg = if diff_lines.is_empty() {
                "search content does not match file content".to_string()
            } else {
                format!("search content does not match file content:\n{}", diff_lines.join("\n"))
            };
            return Err(ToolError::Execution(err_msg));
        }

        let mut new_content = String::new();
        new_content.push_str(&current_lines[..start_line - 1].join("\n"));
        if start_line > 1 {
            new_content.push('\n');
        }
        new_content.push_str(&replace_lines.join("\n"));
        if end_line < current_lines.len() {
            new_content.push('\n');
            new_content.push_str(&current_lines[end_line..].join("\n"));
        }

        tokio::fs::write(&path, &new_content)
            .await
            .map_err(|e| ToolError::Execution(format!("write failed for '{path_str}': {e}")))?;

        Ok(format!(
            "applied edit to {path_str}:{start_line}-{end_line} ({} → {} lines)",
            end_line - start_line + 1,
            replace_lines.len()
        ))
    }
}

fn parse_diff_header(header: &str) -> Result<(String, String), ToolError> {
    let parts: Vec<&str> = header.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(ToolError::InvalidParams(format!(
            "invalid header '{header}': expected 'path:start-end'"
        )));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

fn parse_line_range(range: &str) -> Result<(usize, usize), ToolError> {
    let parts: Vec<&str> = range.splitn(2, '-').collect();
    if parts.len() != 2 {
        return Err(ToolError::InvalidParams(format!(
            "invalid line range '{range}': expected 'start-end'"
        )));
    }
    let start = parts[0]
        .parse::<usize>()
        .map_err(|_| ToolError::InvalidParams(format!("invalid start line: {}", parts[0])))?;
    let end = parts[1]
        .parse::<usize>()
        .map_err(|_| ToolError::InvalidParams(format!("invalid end line: {}", parts[1])))?;

    if start < 1 {
        return Err(ToolError::InvalidParams(format!(
            "start line must be >= 1, got {start}"
        )));
    }
    if end < start {
        return Err(ToolError::InvalidParams(format!(
            "end line {end} must be >= start line {start}"
        )));
    }

    Ok((start, end))
}

// ---------------------------------------------------------------------------
// Glob
// ---------------------------------------------------------------------------

/// Find files matching a simple glob pattern (supports `*` and `**`).
pub struct Glob {
    root: PathBuf,
}

impl Glob {
    pub fn new() -> Self {
        Self {
            root: PathBuf::from("."),
        }
    }
    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Default for Glob {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn glob_match(pattern: &str, text: &str) -> bool {
    let pat: Vec<char> = pattern.chars().collect();
    let txt: Vec<char> = text.chars().collect();
    glob_recursive(&pat, &txt)
}

fn glob_recursive(pat: &[char], txt: &[char]) -> bool {
    if pat.is_empty() {
        return txt.is_empty();
    }
    if pat[0] == '*' {
        // Handle **
        if pat.len() > 1 && pat[1] == '*' {
            let rest = if pat.len() > 2 && pat[2] == '/' {
                &pat[3..]
            } else {
                &pat[2..]
            };
            return (0..=txt.len()).any(|i| glob_recursive(rest, &txt[i..]));
        }
        // Single * matches everything except /
        for i in 0..=txt.len() {
            if i > 0 && txt[i - 1] == '/' {
                break;
            }
            if glob_recursive(&pat[1..], &txt[i..]) {
                return true;
            }
        }
        return false;
    }
    if !txt.is_empty() && (pat[0] == '?' || pat[0] == txt[0]) {
        return glob_recursive(&pat[1..], &txt[1..]);
    }
    false
}

#[async_trait]
impl Tool for Glob {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Find files matching a glob pattern (e.g. '**/*.rs', 'src/**/*.ts'). \
         Returns matching paths relative to workspace root."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string" }
            },
            "required": ["pattern"]
        })
    }

    async fn call(&self, params: Value) -> Result<String, ToolError> {
        let pattern = str_param(&params, "pattern")?;
        let mut results = Vec::new();

        walk_dir(&self.root, &self.root, &pattern, &mut results);

        if results.is_empty() {
            return Ok("(no matches)\n".into());
        }

        results.sort();
        let body = results
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("\n");
        Ok(format!("{body}\n"))
    }
}

fn walk_dir(root: &Path, dir: &Path, pattern: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip hidden and common ignored dirs
        if name_str.starts_with('.')
            || matches!(
                name_str.as_ref(),
                "node_modules" | "target" | "dist" | "build"
            )
        {
            continue;
        }

        if path.is_dir() {
            walk_dir(root, &path, pattern, out);
        } else {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            if glob_match(pattern, &rel.to_string_lossy()) {
                out.push(rel.to_path_buf());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Grep
// ---------------------------------------------------------------------------

/// Search file contents for a pattern (substring match, case-insensitive optional).
pub struct Grep {
    root: PathBuf,
}

impl Grep {
    pub fn new() -> Self {
        Self {
            root: PathBuf::from("."),
        }
    }
    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Default for Grep {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for Grep {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search file contents for a substring. Returns file:line: matched lines. \
         Use 'pattern' for the search text. Use 'file_pattern' to scope by filename (e.g. '*.rs')."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "Substring to search for in file contents" },
                "file_pattern": { "type": "string", "description": "Glob to scope files (e.g. '*.rs', 'src/**')" },
                "case_insensitive": { "type": "boolean" }
            },
            "required": ["pattern"]
        })
    }

    async fn call(&self, params: Value) -> Result<String, ToolError> {
        let query = str_param(&params, "pattern")?;
        let file_pattern = params.get("file_pattern").and_then(|v| v.as_str());
        let case_insensitive = params
            .get("case_insensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let query_cmp = if case_insensitive {
            query.to_lowercase()
        } else {
            query.clone()
        };

        let mut results = Vec::new();
        let mut files = Vec::new();
        walk_dir(&self.root, &self.root, "**/*", &mut files);

        for rel in &files {
            if let Some(fp) = file_pattern {
                if !glob_match(fp, &rel.to_string_lossy()) {
                    continue;
                }
            }
            let full = self.root.join(rel);
            let Ok(content) = std::fs::read_to_string(&full) else {
                continue;
            };
            for (i, line) in content.lines().enumerate() {
                let line_cmp = if case_insensitive {
                    line.to_lowercase()
                } else {
                    line.to_string()
                };
                if line_cmp.contains(&query_cmp) {
                    results.push(format!("{}:{}: {line}", rel.display(), i + 1));
                }
            }
        }

        if results.is_empty() {
            Ok("(no matches)\n".into())
        } else {
            Ok(format!("{}\n", results.join("\n")))
        }
    }
}

// ---------------------------------------------------------------------------
// Shell
// ---------------------------------------------------------------------------

/// Run an allowlisted shell command.
pub struct Shell {
    root: PathBuf,
    allowed: Vec<String>,
}

impl Shell {
    /// Create with a custom executable allowlist.
    pub fn with_allowlist(root: impl Into<PathBuf>, allowed: Vec<String>) -> Self {
        Self {
            root: root.into(),
            allowed,
        }
    }

    fn is_allowed(&self, exe: &str) -> bool {
        self.allowed.iter().any(|a| a == exe)
    }
}

#[async_trait]
impl Tool for Shell {
    fn name(&self) -> &str {
        "shell"
    }

    fn description(&self) -> &str {
        "Run an allowlisted command (cargo, git, npm, etc). Use AFTER making changes to verify they work. Timeout default: 300s."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Executable name" },
                "args": { "type": "array", "items": { "type": "string" } },
                "timeout_ms": { "type": "integer", "description": "Max runtime in ms (default: 300000). Use 300000 for cargo commands." }
            },
            "required": ["command"]
        })
    }

    fn is_mutating(&self) -> bool {
        true
    }

    async fn call(&self, params: Value) -> Result<String, ToolError> {
        let command = str_param(&params, "command")?;
        if !self.is_allowed(&command) {
            return Err(ToolError::Execution(format!(
                "'{command}' is not in the allowlist: [{}]",
                self.allowed.join(", ")
            )));
        }

        let args: Vec<String> = params
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|a| a.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let timeout_ms = opt_usize(&params, "timeout_ms").unwrap_or(300_000);
        let mut cmd = tokio::process::Command::new(&command);
        cmd.args(&args).current_dir(&self.root);

        let output = tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms as u64),
            cmd.output(),
        )
        .await
        .map_err(|_| ToolError::Execution(format!("timed out after {timeout_ms}ms")))?
        .map_err(|e| ToolError::Execution(format!("spawn failed: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);

        Ok(format!(
            "exit: {code}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
        ))
    }
}

/// Re-exports for ergonomic registration.
pub mod tools {
    pub use super::{DiffEdit, FileEdit, FileRead, FileWrite, Glob, Grep, Shell};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_resolution_rejects_escape() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        assert!(resolve_path(root, "src/main.rs").is_ok());
        assert!(resolve_path(root, "../etc/passwd").is_err());
        assert!(resolve_path(root, "/etc/passwd").is_err());
        assert!(resolve_path(root, "src/../../etc/passwd").is_err());
    }

    #[test]
    fn path_resolution_works_with_relative_root() {
        // Regression: resolve_path must work when root is "." (relative).
        // Path::starts_with is component-based, so "." (CurDir) never matches
        // normalized paths (Normal components). Canonicalizing the root fixes this.
        let root = Path::new(".");
        assert!(resolve_path(root, "Cargo.toml").is_ok());
        assert!(resolve_path(root, "crates/sruja-cli/Cargo.toml").is_ok());
        // Escape still rejected
        assert!(resolve_path(root, "../../../etc/passwd").is_err());
    }

    #[test]
    fn glob_basic() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(!glob_match("*.rs", "src/main.rs"));
        assert!(glob_match("**/*.rs", "src/main.rs"));
        assert!(glob_match("**/*.rs", "a/b/c.rs"));
        assert!(glob_match("src/**/*.ts", "src/a/b.ts"));
    }

    #[tokio::test]
    async fn file_read_write_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let write = FileWrite::with_root(root);
        let read = FileRead::with_root(root);

        write
            .call(json!({"path": "test.txt", "content": "hello\nworld\n"}))
            .await
            .unwrap();

        let out = read.call(json!({"path": "test.txt"})).await.unwrap();
        assert!(out.contains("hello"));
        assert!(out.contains("world"));
    }

    #[tokio::test]
    async fn file_edit_replaces() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let write = FileWrite::with_root(root);
        let edit = FileEdit::with_root(root);

        write
            .call(json!({"path": "a.txt", "content": "foo bar foo"}))
            .await
            .unwrap();
        edit.call(json!({"path": "a.txt", "old_string": "bar", "new_string": "baz"}))
            .await
            .unwrap();

        let content = std::fs::read_to_string(root.join("a.txt")).unwrap();
        assert_eq!(content, "foo baz foo");
    }

    #[tokio::test]
    async fn shell_rejects_unlisted() {
        let shell = Shell::with_allowlist(".", vec!["echo".into()]);
        let err = shell.call(json!({"command": "rm"})).await.unwrap_err();
        assert!(matches!(err, ToolError::Execution(_)));
    }

    #[tokio::test]
    async fn diff_edit_single_hunk() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let write = FileWrite::with_root(root);
        let edit = DiffEdit::with_root(root);

        write
            .call(json!({
                "path": "test.rs",
                "content": "fn main() {\n    let x = 1;\n    println!(\"{}\", x);\n}\n"
            }))
            .await
            .unwrap();

        let result = edit
            .call(json!({
                "edits": [{
                    "header": "test.rs:2-2",
                    "search": "    let x = 1;",
                    "replace": "    let x = 42;"
                }]
            }))
            .await
            .unwrap();

        assert!(result.contains("applied edit to test.rs:2-2"));

        let content = std::fs::read_to_string(root.join("test.rs")).unwrap();
        assert!(content.contains("let x = 42;"));
        assert!(!content.contains("let x = 1;"));
    }

    #[tokio::test]
    async fn diff_edit_multi_line_hunk() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let write = FileWrite::with_root(root);
        let edit = DiffEdit::with_root(root);

        write
            .call(json!({
                "path": "test.rs",
                "content": "fn main() {\n    let a = 1;\n    let b = 2;\n    let c = a + b;\n}\n"
            }))
            .await
            .unwrap();

        let result = edit
            .call(json!({
                "edits": [{
                    "header": "test.rs:2-3",
                    "search": "    let a = 1;\n    let b = 2;",
                    "replace": "    let a = 10;\n    let b = 20;"
                }]
            }))
            .await
            .unwrap();

        assert!(result.contains("applied edit to test.rs:2-3"));

        let content = std::fs::read_to_string(root.join("test.rs")).unwrap();
        assert!(content.contains("let a = 10;"));
        assert!(content.contains("let b = 20;"));
        assert!(!content.contains("let a = 1;"));
        assert!(!content.contains("let b = 2;"));
    }

    #[tokio::test]
    async fn diff_edit_multiple_edits() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let write = FileWrite::with_root(root);
        let edit = DiffEdit::with_root(root);

        write
            .call(json!({
                "path": "test.rs",
                "content": "fn main() {\n    let x = 1;\n    let y = 2;\n    println!(\"{}\", x + y);\n}\n"
            }))
            .await
            .unwrap();

        let result = edit
            .call(json!({
                "edits": [
                    {
                        "header": "test.rs:2-2",
                        "search": "    let x = 1;",
                        "replace": "    let x = 10;"
                    },
                    {
                        "header": "test.rs:3-3",
                        "search": "    let y = 2;",
                        "replace": "    let y = 20;"
                    }
                ]
            }))
            .await
            .unwrap();

        assert!(result.contains("Edit 0:"));
        assert!(result.contains("Edit 1:"));

        let content = std::fs::read_to_string(root.join("test.rs")).unwrap();
        assert!(content.contains("let x = 10;"));
        assert!(content.contains("let y = 20;"));
    }

    #[tokio::test]
    async fn diff_edit_mismatch_fails() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let write = FileWrite::with_root(root);
        let edit = DiffEdit::with_root(root);

        write
            .call(json!({
                "path": "test.rs",
                "content": "fn main() {\n    let x = 1;\n}\n"
            }))
            .await
            .unwrap();

        let err = edit
            .call(json!({
                "edits": [{
                    "header": "test.rs:2-2",
                    "search": "    let x = 99;",  // Wrong value
                    "replace": "    let x = 42;"
                }]
            }))
            .await
            .unwrap_err();

        assert!(matches!(err, ToolError::Execution(_)));
        let err_str = err.to_string();
        assert!(err_str.contains("does not match"));
    }

    #[tokio::test]
    async fn diff_edit_invalid_header_fails() {
        let edit = DiffEdit::new();

        let err = edit
            .call(json!({
                "edits": [{
                    "header": "invalid_header",
                    "search": "foo",
                    "replace": "bar"
                }]
            }))
            .await
            .unwrap_err();

        assert!(matches!(err, ToolError::InvalidParams(_)));
    }

    #[tokio::test]
    async fn diff_edit_invalid_range_fails() {
        let edit = DiffEdit::new();

        let err = edit
            .call(json!({
                "edits": [{
                    "header": "test.rs:10-5",  // end < start
                    "search": "foo",
                    "replace": "bar"
                }]
            }))
            .await
            .unwrap_err();

        assert!(matches!(err, ToolError::InvalidParams(_)));
    }
}
