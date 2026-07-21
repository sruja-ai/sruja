use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde_json::{json, Value};

use super::{str_param, Tool, ToolError};

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
        let root = self.root.clone();

        let mut results = tokio::task::spawn_blocking(move || {
            let mut results = Vec::new();
            walk_dir(&root, &root, &pattern, &mut results);
            results
        })
        .await
        .map_err(|e| ToolError::Execution(format!("glob walk panicked: {e}")))?;

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

        let root = self.root.clone();
        let file_pattern_owned = file_pattern.map(String::from);

        let results = tokio::task::spawn_blocking(move || {
            let mut files = Vec::new();
            walk_dir(&root, &root, "**/*", &mut files);

            let mut results: Vec<String> = Vec::new();
            for rel in &files {
                if let Some(ref fp) = file_pattern_owned {
                    if !glob_match(fp, &rel.to_string_lossy()) {
                        continue;
                    }
                }
                let full = root.join(rel);
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
            results
        })
        .await
        .map_err(|e| ToolError::Execution(format!("grep walk panicked: {e}")))?;

        if results.is_empty() {
            Ok("(no matches)\n".into())
        } else {
            Ok(format!("{}\n", results.join("\n")))
        }
    }
}
