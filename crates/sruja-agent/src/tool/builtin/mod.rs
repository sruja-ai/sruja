//! Built-in filesystem and shell tools.
//!
//! All file tools enforce workspace-root confinement: paths are resolved
//! relative to the workspace root and rejected if they escape it.

mod file_tools;
mod search_tools;
mod shell_tools;

use std::path::{Path, PathBuf};

use serde_json::Value;

use super::{Tool, ToolError};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

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

// Re-export everything sub-modules publish.
pub use file_tools::{DiffEdit, FileEdit, FileRead, FileWrite};
pub(crate) use search_tools::glob_match;
pub use search_tools::{Glob, Grep};
pub use shell_tools::Shell;

/// Re-exports for ergonomic registration.
pub mod tools {
    pub use super::{DiffEdit, FileEdit, FileRead, FileWrite, Glob, Grep, Shell};
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
        let root = Path::new(".");
        assert!(resolve_path(root, "Cargo.toml").is_ok());
        assert!(resolve_path(root, "crates/sruja-cli/Cargo.toml").is_ok());
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
                    "search": "    let x = 99;",
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
                    "header": "test.rs:10-5",
                    "search": "foo",
                    "replace": "bar"
                }]
            }))
            .await
            .unwrap_err();

        assert!(matches!(err, ToolError::InvalidParams(_)));
    }
}
