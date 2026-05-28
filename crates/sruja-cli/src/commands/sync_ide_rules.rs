//! Sync IDE integration files (.cursorrules, CLAUDE.md, copilot instructions).
//!
//! CI uses this command in `--check` mode to ensure generated editor context files
//! are up to date with the current repository architecture.

use std::path::{Path, PathBuf};

use crate::commands::context::{
    build_architecture_context, format_copilot_instructions, format_cursor_rules,
    format_llms_architecture,
};
use crate::commands::{scan_repo_cached, CliError};

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

    // Single scan for all outputs so `--check` after `sync` is stable (no cache refresh mid-command).
    let graph = scan_repo_cached(repo_root)?;
    let arch_ctx =
        build_architecture_context(&graph, options.repo, None, None, 2, options.max_tokens)?;

    let cursor_rules = format_cursor_rules(&arch_ctx);
    let copilot = format_copilot_instructions(&arch_ctx);
    let llms_arch = format_llms_architecture(&arch_ctx);

    let targets: [(&str, &str); 3] = [
        (".cursorrules", &cursor_rules),
        ("CLAUDE.md", &cursor_rules),
        (".gemini/AGENTS.md", &cursor_rules),
    ];

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
            "Synced IDE rules: .cursorrules, .github/copilot-instructions.md, CLAUDE.md, .gemini/AGENTS.md, llms-architecture.txt"
        );
    }

    Ok(())
}
