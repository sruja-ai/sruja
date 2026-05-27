//! Sync IDE integration files (.cursorrules, CLAUDE.md, copilot instructions).
//!
//! CI uses this command in `--check` mode to ensure generated editor context files
//! are up to date with the current repository architecture.

use std::path::{Path, PathBuf};

use crate::commands::context::{
    build_architecture_context, context_string, format_llms_architecture, ContextRequest,
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
    let s = s.replace("\r\n", "\n");
    let mut out = String::with_capacity(s.len() + 1);
    for line in s.split('\n') {
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

fn write_atomic(path: &Path, contents: &str) -> Result<(), CliError> {
    super::sync_cmd::atomic_write_file(path, contents.as_bytes())
}

fn repo_rel(repo_root: &Path, rel: &str) -> PathBuf {
    repo_root.join(rel)
}

pub async fn sync_ide_rules(options: SyncIdeRulesOptions<'_>) -> Result<(), CliError> {
    let repo_root = Path::new(options.repo);
    if !repo_root.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", options.repo),
        )));
    }

    // Use the same generator as `ai-context -f cursor-rules`.
    let cursor_rules = context_string(
        options.repo,
        "cursor-rules",
        ContextRequest {
            run_id: None,
            file: None,
            element_id: None,
            query: None,
            base_ref: None,
            head_ref: None,
            intent: None,
            depth: 2,
            max_tokens: options.max_tokens,
            cache_friendly: false,
        },
    )
    .await?;

    // Sruja projects typically mirror cursor rules into both of these entry points.
    let targets: Vec<(&str, String)> = vec![
        (".cursorrules", cursor_rules.clone()),
        ("CLAUDE.md", cursor_rules.clone()),
        (".gemini/AGENTS.md", cursor_rules),
    ];

    // Optional Copilot instructions file.
    let copilot = context_string(
        options.repo,
        "copilot-instructions",
        ContextRequest {
            run_id: None,
            file: None,
            element_id: None,
            query: None,
            base_ref: None,
            head_ref: None,
            intent: None,
            depth: 2,
            max_tokens: options.max_tokens,
            cache_friendly: false,
        },
    )
    .await?;
    let copilot_path = repo_rel(repo_root, ".github/copilot-instructions.md");

    // llms-architecture.txt is a small, stable file meant for quick LLM overviews.
    let graph = crate::commands::scan_repo_cached(repo_root)?;
    let arch_ctx =
        build_architecture_context(&graph, options.repo, None, None, 2, options.max_tokens)?;
    let llms_arch = format_llms_architecture(&arch_ctx);
    let llms_arch_path = repo_rel(repo_root, "llms-architecture.txt");

    for (rel, generated) in targets {
        let path = repo_rel(repo_root, rel);
        let generated_norm = normalize_for_compare(&generated);
        let existing_norm = read_file_opt(&path)?.map(|s| normalize_for_compare(&s));

        if options.check {
            let Some(existing) = existing_norm else {
                return Err(CliError::validation(format!(
                    "Missing IDE context file: {} (run `sruja sync-ide-rules -r .` to generate)",
                    path.display()
                )));
            };
            if existing != generated_norm {
                return Err(CliError::validation(format!(
                    "IDE context file out of date: {} (run `sruja sync-ide-rules -r .` to update)",
                    path.display()
                )));
            }
        } else {
            write_atomic(&path, &generated_norm)?;
        }
    }

    if options.check {
        if let Some(existing) = read_file_opt(&copilot_path)? {
            if normalize_for_compare(&existing) != normalize_for_compare(&copilot) {
                return Err(CliError::validation(format!(
                    "IDE context file out of date: {} (run `sruja sync-ide-rules -r .` to update)",
                    copilot_path.display()
                )));
            }
        }
    } else {
        write_atomic(&copilot_path, &normalize_for_compare(&copilot))?;
    }

    let llms_existing = read_file_opt(&llms_arch_path)?.map(|s| normalize_for_compare(&s));
    let llms_generated = normalize_for_compare(&llms_arch);
    if options.check {
        let Some(existing) = llms_existing else {
            return Err(CliError::validation(format!(
                "Missing IDE context file: {} (run `sruja sync-ide-rules -r .` to generate)",
                llms_arch_path.display()
            )));
        };
        if existing != llms_generated {
            return Err(CliError::validation(format!(
                "IDE context file out of date: {} (run `sruja sync-ide-rules -r .` to update)",
                llms_arch_path.display()
            )));
        }
    } else {
        write_atomic(&llms_arch_path, &llms_generated)?;
    }

    Ok(())
}
