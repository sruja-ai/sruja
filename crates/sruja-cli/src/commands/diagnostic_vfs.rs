//! Head/tail truncation for large diagnostic payloads and local VFS storage under `.sruja/vfs/`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::CliError;

pub const DEFAULT_HEAD_CHARS: usize = 500;
pub const DEFAULT_TAIL_CHARS: usize = 1000;
/// Default token budget for JSON lint output before head/tail + VFS storage.
pub const LINT_JSON_DIAGNOSTIC_TOKEN_BUDGET: usize = 1200;
/// Token budget for architecture index validation log attachment.
pub const INDEX_VALIDATION_LOG_TOKEN_BUDGET: usize = 800;

const LINT_JSON_KEEP_HEAD: usize = 5;
const LINT_JSON_KEEP_TAIL: usize = 3;

/// URI prefix for MCP/CLI references to stored full diagnostics.
pub const VFS_URI_PREFIX: &str = "sruja-vfs://diagnostics/";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruncatedDiagnosticPayload {
    pub schema_version: String,
    pub truncated: bool,
    pub head: String,
    pub tail: String,
    pub full_uri: Option<String>,
    pub line_count: usize,
    pub estimated_tokens: usize,
}

pub fn estimate_tokens(text: &str) -> usize {
    super::context::types::TokenBudget::estimate_tokens(text)
}

/// Render diagnostics as newline-separated human-readable lines (for VFS storage).
pub fn diagnostics_to_text(diagnostics: &[sruja_diagnostics::Diagnostic]) -> String {
    diagnostics
        .iter()
        .map(sruja_diagnostics::format_diagnostic)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Shrink a lint JSON payload when the full diagnostic list is too large for agents.
pub fn apply_lint_json_truncation(
    repo: &Path,
    storage_name: &str,
    diagnostics: &[sruja_diagnostics::Diagnostic],
    max_tokens: usize,
) -> Result<
    (
        Vec<sruja_diagnostics::Diagnostic>,
        Option<TruncatedDiagnosticPayload>,
    ),
    CliError,
> {
    let text = diagnostics_to_text(diagnostics);
    if diagnostics.len() <= LINT_JSON_KEEP_HEAD + LINT_JSON_KEEP_TAIL
        && estimate_tokens(&text) <= max_tokens
    {
        return Ok((diagnostics.to_vec(), None));
    }

    let truncation = truncate_and_store_if_needed(repo, storage_name, &text, max_tokens)?;
    let mut kept = Vec::new();
    kept.extend_from_slice(&diagnostics[..diagnostics.len().min(LINT_JSON_KEEP_HEAD)]);
    if diagnostics.len() > LINT_JSON_KEEP_HEAD + LINT_JSON_KEEP_TAIL {
        let tail_start = diagnostics.len().saturating_sub(LINT_JSON_KEEP_TAIL);
        kept.extend_from_slice(&diagnostics[tail_start..]);
    }
    Ok((kept, Some(truncation)))
}

/// Snap truncation to line boundaries where possible.
pub fn truncate_text_head_tail(
    text: &str,
    head_chars: usize,
    tail_chars: usize,
) -> (String, String, bool) {
    if text.len() <= head_chars.saturating_add(tail_chars) {
        return (text.to_string(), String::new(), false);
    }

    let head_end = text
        .char_indices()
        .nth(head_chars)
        .map(|(i, _)| i)
        .unwrap_or(text.len());
    let mut head = text[..head_end].to_string();
    if let Some(pos) = head.rfind('\n') {
        head.truncate(pos + 1);
    }

    let tail_start = text
        .char_indices()
        .rev()
        .nth(tail_chars.saturating_sub(1))
        .map(|(i, _)| i)
        .unwrap_or(0);
    let mut tail = text[tail_start.min(text.len())..].to_string();
    if let Some(pos) = tail.find('\n') {
        tail = tail[pos + 1..].to_string();
    }

    (head, tail, true)
}

pub fn vfs_diagnostics_dir(repo: &Path) -> PathBuf {
    repo.join(".sruja").join("vfs").join("diagnostics")
}

/// Write full diagnostic text; returns `sruja-vfs://diagnostics/<filename>`.
pub fn write_vfs_diagnostic(
    repo: &Path,
    filename: &str,
    content: &str,
) -> Result<String, CliError> {
    let dir = vfs_diagnostics_dir(repo);
    std::fs::create_dir_all(&dir).map_err(CliError::Io)?;
    let path = dir.join(filename);
    std::fs::write(&path, content).map_err(CliError::Io)?;
    Ok(format!("{VFS_URI_PREFIX}{filename}"))
}

pub fn read_vfs_diagnostic(repo: &Path, uri_or_filename: &str) -> Result<String, CliError> {
    let filename = uri_or_filename
        .strip_prefix(VFS_URI_PREFIX)
        .or_else(|| uri_or_filename.strip_prefix("context://vfs/diagnostics/"))
        .unwrap_or(uri_or_filename);
    let path = vfs_diagnostics_dir(repo).join(filename);
    if !path.exists() {
        return Err(CliError::validation(format!(
            "Diagnostic not found: {filename} (expected under .sruja/vfs/diagnostics/)"
        )));
    }
    std::fs::read_to_string(&path).map_err(CliError::Io)
}

/// If `text` exceeds `max_tokens`, store the full payload in VFS and return head/tail summary.
pub fn truncate_and_store_if_needed(
    repo: &Path,
    storage_name: &str,
    text: &str,
    max_tokens: usize,
) -> Result<TruncatedDiagnosticPayload, CliError> {
    let estimated = estimate_tokens(text);
    let line_count = text.lines().count();
    if estimated <= max_tokens {
        return Ok(TruncatedDiagnosticPayload {
            schema_version: "diagnostic_truncation/v1".to_string(),
            truncated: false,
            head: text.to_string(),
            tail: String::new(),
            full_uri: None,
            line_count,
            estimated_tokens: estimated,
        });
    }

    let full_uri = write_vfs_diagnostic(repo, storage_name, text)?;
    let (head, tail, _) = truncate_text_head_tail(text, DEFAULT_HEAD_CHARS, DEFAULT_TAIL_CHARS);
    let summary_tokens = estimate_tokens(&head) + estimate_tokens(&tail);
    Ok(TruncatedDiagnosticPayload {
        schema_version: "diagnostic_truncation/v1".to_string(),
        truncated: true,
        head,
        tail,
        full_uri: Some(full_uri),
        line_count,
        estimated_tokens: summary_tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_head_tail_on_long_text() {
        let text = (0..200).map(|i| format!("line {i}\n")).collect::<String>();
        let (head, tail, truncated) =
            truncate_text_head_tail(&text, DEFAULT_HEAD_CHARS, DEFAULT_TAIL_CHARS);
        assert!(truncated);
        assert!(!head.is_empty());
        assert!(!tail.is_empty());
        assert!(head.len() + tail.len() < text.len());
    }

    #[test]
    fn vfs_round_trip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let uri = write_vfs_diagnostic(dir.path(), "run-1.txt", "full log\n").expect("write");
        assert!(uri.starts_with(VFS_URI_PREFIX));
        let back = read_vfs_diagnostic(dir.path(), &uri).expect("read");
        assert_eq!(back, "full log\n");
    }

    fn sample_diagnostic(i: usize) -> sruja_diagnostics::Diagnostic {
        sruja_diagnostics::Diagnostic::new(
            format!("E{i}"),
            sruja_diagnostics::Severity::Error,
            format!("message {i}"),
            sruja_diagnostics::SourceLocation::new("repo.sruja".to_string(), i as u32 + 1, 1),
        )
    }

    #[test]
    fn lint_json_truncation_keeps_head_and_tail() {
        let dir = tempfile::tempdir().expect("tempdir");
        let diags: Vec<sruja_diagnostics::Diagnostic> = (0..20).map(sample_diagnostic).collect();
        let (kept, trunc) =
            apply_lint_json_truncation(dir.path(), "lint-repo.txt", &diags, 50).expect("truncate");
        assert!(trunc.is_some());
        assert!(kept.len() < diags.len());
    }

    #[test]
    fn truncate_stores_when_over_token_budget() {
        let dir = tempfile::tempdir().expect("tempdir");
        let big = (0..400)
            .map(|i| format!("diagnostic line {i}\n"))
            .collect::<String>();
        let out = truncate_and_store_if_needed(dir.path(), "diag.txt", &big, 50).expect("truncate");
        assert!(out.truncated);
        assert!(out.full_uri.is_some());
        assert!(!out.head.is_empty());
    }
}
