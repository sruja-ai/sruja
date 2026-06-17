//! Built-in filesystem and shell tools.
//!
//! All file tools enforce workspace-root confinement: paths are resolved
//! relative to the workspace root and rejected if they escape it.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde_json::{json, Value};

use super::{Tool, ToolError};

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
        "Create or overwrite a file with the given content."
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
         Optional 'pattern' in file path to scope the search (e.g. '*.rs')."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Substring to search for" },
                "file_pattern": { "type": "string", "description": "Glob to scope files (e.g. '*.rs')" },
                "case_insensitive": { "type": "boolean" }
            },
            "required": ["query"]
        })
    }

    async fn call(&self, params: Value) -> Result<String, ToolError> {
        let query = str_param(&params, "query")?;
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
        "Run an allowlisted command (e.g. cargo, npm, git, just). \
         Returns stdout, stderr, and exit code."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Executable name" },
                "args": { "type": "array", "items": { "type": "string" } },
                "timeout_ms": { "type": "integer", "description": "Max runtime (default: 60000)" }
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

        let timeout_ms = opt_usize(&params, "timeout_ms").unwrap_or(60_000);
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
    pub use super::{FileEdit, FileRead, FileWrite, Glob, Grep, Shell};
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
}
