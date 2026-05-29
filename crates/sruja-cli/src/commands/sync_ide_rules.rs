//! Sync IDE integration files (.cursorrules, CLAUDE.md, copilot instructions).
//!
//! CI uses this command in `--check` mode to ensure generated editor context files
//! are up to date with the current repository architecture.

use std::path::{Path, PathBuf};

use crate::commands::context::{
    build_architecture_context, format_copilot_instructions, format_cursor_rules,
    format_llms_architecture,
};
use crate::commands::CliError;

#[derive(Debug, Clone)]
pub struct SyncIdeRulesOptions<'a> {
    pub repo: &'a str,
    pub max_tokens: usize,
    pub check: bool,
}

fn read_file_opt(path: &Path) -> Result<Option<String>, CliError> {
    match std::fs::read_to_string(path) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(CliError::Io(e)),
    }
}

fn normalize_for_compare(s: &str) -> String {
    // Trim trailing newlines so re-normalizing a file we just wrote stays stable.
    let s = s.trim_end_matches('\n').replace("\r\n", "\n");
    let mut out = String::with_capacity(s.len() + 1);
    for line in s.split('\n') {
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

fn write_atomic(path: &Path, contents: &str) -> Result<(), CliError> {
    super::super::scan_domain::sync_cmd::atomic_write_file(path, contents.as_bytes())
}

fn repo_rel(repo_root: &Path, rel: &str) -> PathBuf {
    repo_root.join(rel)
}

fn sync_or_check_file(path: &Path, generated: &str, check: bool) -> Result<(), CliError> {
    let generated_norm = normalize_for_compare(generated);
    if check {
        let Some(existing) = read_file_opt(path)?.map(|s| normalize_for_compare(&s)) else {
            return Err(CliError::validation(format!(
                "Missing IDE context file: {} (run `sruja sync-ide-rules -r .` to generate)",
                path.display()
            )));
        };
        if existing != generated_norm {
            // Emit a small, deterministic hint for CI logs (avoid dumping whole file).
            let mut existing_lines = existing.lines();
            let mut generated_lines = generated_norm.lines();
            let mut first_diff: Option<(usize, String, String)> = None;
            for i in 1.. {
                let e = existing_lines.next();
                let g = generated_lines.next();
                match (e, g) {
                    (Some(el), Some(gl)) if el == gl => continue,
                    (Some(el), Some(gl)) => {
                        first_diff = Some((i, el.to_string(), gl.to_string()));
                        break;
                    }
                    (None, Some(gl)) => {
                        first_diff = Some((i, "<EOF>".to_string(), gl.to_string()));
                        break;
                    }
                    (Some(el), None) => {
                        first_diff = Some((i, el.to_string(), "<EOF>".to_string()));
                        break;
                    }
                    (None, None) => break,
                }
            }
            if let Some((line_no, e, g)) = first_diff {
                eprintln!(
                    "sync-ide-rules mismatch {} at line {} (existing_bytes={}, generated_bytes={})",
                    path.display(),
                    line_no,
                    existing.len(),
                    generated_norm.len()
                );
                eprintln!("  existing: {}", e);
                eprintln!("  generated: {}", g);
            }
            return Err(CliError::validation(format!(
                "IDE context file out of date: {} (run `sruja sync-ide-rules -r .` to update)",
                path.display()
            )));
        }
    } else {
        write_atomic(path, &generated_norm)?;
    }
    Ok(())
}

pub async fn sync_ide_rules(options: SyncIdeRulesOptions<'_>) -> Result<(), CliError> {
    let repo_root = Path::new(options.repo);
    if !repo_root.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", options.repo),
        )));
    }

    // Scan fresh rather than using `.sruja/cache/scan.json`.
    //
    // Rationale: CI runs tests/build before `sync-ide-rules --check`. Some commands/tests may
    // (legitimately) write a cached scan graph with different settings than the IDE rule
    // generator expects. Using the cache here can create false negatives where `--check`
    // fails even though the checked-in files were generated from a clean scan.
    let graph = sruja_scan::scan_repo(repo_root).map_err(|e| CliError::scan(e.to_string()))?;
    let arch_ctx =
        build_architecture_context(&graph, options.repo, None, None, 2, options.max_tokens)?;

    let cursor_rules = format_cursor_rules(&arch_ctx);
    let copilot = format_copilot_instructions(&arch_ctx);
    let llms_arch = format_llms_architecture(&arch_ctx);

    // Only generate architecture data files. Hand-written files like CLAUDE.md
    // and AGENTS.md are managed by the user, not by sruja.
    let targets: [(&str, &str); 1] = [(".cursorrules", &cursor_rules)];

    for (rel, generated) in targets {
        sync_or_check_file(&repo_rel(repo_root, rel), generated, options.check)?;
    }

    let copilot_path = repo_rel(repo_root, ".github/copilot-instructions.md");
    if options.check {
        if read_file_opt(&copilot_path)?.is_some() {
            sync_or_check_file(&copilot_path, &copilot, true)?;
        }
    } else {
        sync_or_check_file(&copilot_path, &copilot, false)?;
    }

    sync_or_check_file(
        &repo_rel(repo_root, "llms-architecture.txt"),
        &llms_arch,
        options.check,
    )?;

    if !options.check {
        eprintln!(
            "Synced IDE rules: .cursorrules, .github/copilot-instructions.md, llms-architecture.txt"
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    #[test]
    fn normalize_for_compare_is_stable_and_trims_line_trailing_whitespace() {
        let input = "a  \r\nb\t\r\n";
        let normalized = normalize_for_compare(input);
        assert_eq!(normalized, "a\nb\n");

        // Re-normalizing should be stable (important because we write the normalized version).
        assert_eq!(normalize_for_compare(&normalized), normalized);
    }

    #[test]
    fn read_file_opt_returns_none_for_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.txt");

        let got = read_file_opt(&missing).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn sync_or_check_file_check_mode_errors_on_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.txt");

        let err = sync_or_check_file(&path, "x\n", true).unwrap_err();
        match err {
            CliError::Validation { message, .. } => {
                assert!(message.contains("Missing IDE context file"));
                assert!(message.contains("sync-ide-rules"));
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn sync_or_check_file_check_mode_errors_on_mismatch_after_normalization() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file.txt");

        // Note the trailing spaces + missing final newline; normalization should still mismatch.
        fs::write(&path, "a \n").unwrap();
        let err = sync_or_check_file(&path, "a\nb\n", true).unwrap_err();
        match err {
            CliError::Validation { message, .. } => {
                assert!(message.contains("out of date"));
                assert!(message.contains(path.to_string_lossy().as_ref()));
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn sync_or_check_file_check_mode_ok_when_equal_after_normalization() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file.txt");

        // Existing file has CRLF and trailing whitespace; generated is canonical.
        fs::write(&path, "a  \r\nb\t\r\n").unwrap();
        sync_or_check_file(&path, "a\nb\n", true).unwrap();
    }

    #[test]
    fn sync_or_check_file_write_mode_writes_normalized_contents() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file.txt");

        sync_or_check_file(&path, "a  \r\nb\t\r\n", false).unwrap();
        let got = fs::read_to_string(&path).unwrap();
        assert_eq!(got, "a\nb\n");
    }
}
