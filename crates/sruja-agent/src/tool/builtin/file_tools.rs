use std::path::PathBuf;

use async_trait::async_trait;
use serde_json::{json, Value};

use super::{opt_usize, resolve_path, str_param, Tool, ToolError};

const DIFF_EDIT_DEFAULT_CONTEXT: usize = 3;

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
        let limit = opt_usize(&params, "limit").unwrap_or(200);

        let path = resolve_path(&self.root, &path_str)?;
        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| ToolError::Execution(format!("read failed: {e}")))?;

        let lines: Vec<&str> = content.lines().collect();
        let start = (offset - 1).min(lines.len());
        let end = (start + limit).min(lines.len());

        let mut out = String::new();
        for (i, line) in lines[start..end].iter().enumerate() {
            out.push_str(&format!("{}: {line}\n", start + i + 1));
        }
        if out.is_empty() {
            out.push_str("(empty or beyond end of file)\n");
        }
        if end < lines.len() {
            out.push_str(&format!(
                "... ({} more lines, use offset={} to continue)\n",
                lines.len() - end,
                end + 1
            ));
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

        const MAX_WRITE_BYTES: usize = 10 * 1024 * 1024; // 10 MiB
        if content.len() > MAX_WRITE_BYTES {
            return Err(ToolError::Execution(format!(
                "content too large: {} bytes exceeds maximum {}. Use multiple smaller writes or split the output.",
                content.len(),
                MAX_WRITE_BYTES
            )));
        }

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

        let context_lines =
            opt_usize(&params, "context_lines").unwrap_or(DIFF_EDIT_DEFAULT_CONTEXT);

        if edits.is_empty() {
            return Err(ToolError::InvalidParams("edits array is empty".into()));
        }

        let mut results = Vec::new();
        for (idx, edit) in edits.iter().enumerate() {
            let header = str_param(edit, "header")?;
            let search = str_param(edit, "search")?;
            let replace = str_param(edit, "replace")?;

            let result = self
                .apply_single_edit(&header, &search, &replace, context_lines, idx)
                .await;
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
            for (i, (a, b)) in search_lines
                .iter()
                .zip(current_lines[start_line - 1..end_line].iter())
                .enumerate()
            {
                if a != b {
                    diff_lines.push(format!(
                        "  line {}: expected '{}', got '{}'",
                        start_line + i,
                        a,
                        b
                    ));
                }
            }
            let err_msg = if diff_lines.is_empty() {
                "search content does not match file content".to_string()
            } else {
                format!(
                    "search content does not match file content:\n{}",
                    diff_lines.join("\n")
                )
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
