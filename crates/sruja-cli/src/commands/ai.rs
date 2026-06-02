//! AI coding brief command.
//!
//! Produces a paste-ready briefing for AI coding assistants by combining the
//! current worktree, architecture signals, task context, and verification hints.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use super::context::{context_string, ContextRequest};
use super::{scan_repo_cached, CliError};
use crate::integrations::EnrichmentResult;

#[derive(Debug, Clone, Copy)]
pub struct AiBriefOptions<'a> {
    pub repo: &'a str,
    pub task: Option<&'a str>,
    pub file: Option<&'a str>,
    pub element_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub base_ref: Option<&'a str>,
    pub head_ref: Option<&'a str>,
    pub staged: bool,
    pub max_tokens: usize,
    pub output: Option<&'a str>,
    pub enrich: &'a crate::enrichment::EnrichmentRef<'a>,
}

pub async fn ai_brief(options: AiBriefOptions<'_>) -> Result<(), CliError> {
    let repo_path = Path::new(options.repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", options.repo),
        )));
    }

    let branch = git_first_line(repo_path, &["status", "--short", "--branch"])
        .unwrap_or_else(|| "not a git repository or git unavailable".to_string());
    let changed_files = changed_files(repo_path, options.staged);
    let selected_file = options
        .file
        .map(str::to_string)
        .or_else(|| select_focus_file(&changed_files).map(str::to_string));

    let graph = scan_repo_cached(repo_path)?;
    let drift = sruja_diff::detect_architectural_drift(&graph);
    let mut kg = sruja_graph::KnowledgeGraph::new();
    sruja_graph::merge_scan_into_graph(&mut kg, &graph, &repo_path.display().to_string());
    let context_age_hours = crate::utils::context::context_age_hours(repo_path);
    let context_score =
        sruja_graph::compute_context_score(&kg, graph.nodes.len(), repo_path, context_age_hours);

    let task_context = context_string(
        options.repo,
        "for-ai",
        ContextRequest {
            run_id: None,
            file: selected_file.as_deref(),
            element_id: options.element_id,
            query: options.query,
            base_ref: options.base_ref,
            head_ref: options.head_ref,
            intent: None,
            depth: 2,
            max_tokens: options.max_tokens,
            cache_friendly: false,
        },
    )
    .await
    .unwrap_or_else(|err| {
        format!(
            "{{\n  \"warning\": \"Could not build task context: {}\"\n}}",
            escape_json_string(&err.to_string())
        )
    });

    let brief = format_brief(AiBriefRender {
        options,
        branch: &branch,
        changed_files: &changed_files,
        selected_file: selected_file.as_deref(),
        drift: &drift,
        context_score: context_score.score,
        task_context: &task_context,
        enrichment: build_ai_enrichment(
            repo_path,
            options,
            &branch,
            &changed_files,
            selected_file.as_deref(),
            &task_context,
        ),
        repo_path,
    });

    if let Some(output) = options.output {
        fs::write(output, brief)?;
        eprintln!("Written AI coding brief to {}", output);
    } else {
        println!("{}", brief);
    }

    Ok(())
}

struct AiBriefRender<'a> {
    options: AiBriefOptions<'a>,
    branch: &'a str,
    changed_files: &'a [String],
    selected_file: Option<&'a str>,
    drift: &'a sruja_diff::DriftReport,
    context_score: u8,
    task_context: &'a str,
    enrichment: Option<EnrichmentResult>,
    repo_path: &'a Path,
}

fn format_brief(render: AiBriefRender<'_>) -> String {
    let mut out = String::new();
    out.push_str("# AI Coding Brief\n\n");
    out.push_str(
        "Use this brief to make a focused, reviewable code change in this repository.\n\n",
    );

    out.push_str("## Mission\n\n");
    if let Some(task) = render.options.task {
        out.push_str(task.trim());
        out.push('\n');
    } else if render.changed_files.is_empty() {
        out.push_str("Inspect the repository and propose the smallest useful improvement.\n");
    } else {
        out.push_str("Continue the current worktree changes and make them safer, clearer, and easier to verify.\n");
    }

    out.push_str("\n## Repo Snapshot\n\n");
    out.push_str(&format!("- Repo: `{}`\n", render.repo_path.display()));
    out.push_str(&format!("- Git: `{}`\n", render.branch));
    out.push_str(&format!(
        "- Scanned modules: `{}`\n",
        render.drift.total_modules
    ));
    out.push_str(&format!(
        "- Architecture health: `{}/100`\n",
        render.drift.health_score
    ));
    out.push_str(&format!(
        "- AI context score: `{}/100`\n",
        render.context_score
    ));
    out.push_str(&format!(
        "- Truth status: `{}`\n",
        truth_status_label(&render.drift.truth_status)
    ));
    out.push_str(&format!(
        "- Active drift notices: `{}`\n",
        render.drift.violations.len()
    ));
    if let Some(file) = render.selected_file {
        out.push_str(&format!("- Primary focus file: `{}`\n", file));
    }

    out.push_str("\n## Changed Files\n\n");
    if render.changed_files.is_empty() {
        out.push_str("- No git changes detected.\n");
    } else {
        for file in render.changed_files.iter().take(30) {
            out.push_str(&format!("- `{}`\n", file));
        }
        if render.changed_files.len() > 30 {
            out.push_str(&format!(
                "- ... {} more file(s) omitted\n",
                render.changed_files.len() - 30
            ));
        }
    }

    out.push_str("\n## AI Guardrails\n\n");
    out.push_str("- First explain the intended change in one or two sentences.\n");
    out.push_str("- Prefer the existing project patterns over new abstractions.\n");
    out.push_str("- Keep edits scoped to the task and touched architectural area.\n");
    out.push_str("- When modifying architecture files, run `sruja lint <file>` after each edit.\n");
    out.push_str(
        "- After code edits, verify behavior against the stated intent, not only compilation.\n",
    );
    out.push_str("- Preserve unrelated worktree changes.\n");

    out.push_str("\n## Suggested Verification\n\n");
    for command in suggested_commands(render.repo_path, render.changed_files) {
        out.push_str(&format!("- `{}`\n", command));
    }

    out.push_str("\n## Task Context\n\n");
    out.push_str("```json\n");
    out.push_str(render.task_context.trim());
    out.push_str("\n```\n");

    if let Some(ref e) = render.enrichment {
        out.push_str("\n## Enrichment (opt-in)\n\n");
        out.push_str("- This section is **LLM/tool-generated** and must be treated as **interpretation**, not truth.\n");
        out.push_str("- It is grounded in the Task Context JSON above; if it contradicts it, prefer the grounded output.\n\n");
        out.push_str(&format!("- Status: `{}`\n", e.status));
        out.push_str(&format!("- Provider: `{}`\n", e.provider));
        if let Some(ref m) = e.model {
            out.push_str(&format!("- Model: `{}`\n", m));
        }
        if let Some(ref err) = e.error {
            out.push_str(&format!("- Error: `{}`\n", err));
        }
        out.push('\n');
        if let Some(ref md) = e.narrative_markdown {
            out.push_str(md.trim());
            out.push('\n');
        }
    }

    out.push_str("\n## Useful Follow-Up Commands\n\n");
    out.push_str("- `sruja doctor -r .`\n");
    out.push_str("- `sruja ai-context -r . -f for-ai --max-tokens ");
    out.push_str(&render.options.max_tokens.to_string());
    if let Some(file) = render.selected_file {
        out.push_str(" --file ");
        out.push_str(file);
    }
    if let Some(element_id) = render.options.element_id {
        out.push_str(" --element-id ");
        out.push_str(element_id);
    }
    out.push_str("`\n");
    out.push_str("- `sruja agent record -r . --context \"<what changed>\" --hypothesis \"<what you expected>\" --outcome success --guardrail \"<future advice>\"`\n");

    out
}

fn build_ai_enrichment(
    repo_path: &Path,
    options: AiBriefOptions<'_>,
    branch: &str,
    changed_files: &[String],
    selected_file: Option<&str>,
    task_context: &str,
) -> Option<EnrichmentResult> {
    let parsed_task_context: serde_json::Value = serde_json::from_str(task_context)
        .unwrap_or_else(|_| serde_json::Value::String(task_context.to_string()));

    let payload = serde_json::json!({
        "schema_version": "ai_brief_enrichment_input/v1",
        "repo": repo_path.display().to_string(),
        "git": branch,
        "selected_file": selected_file,
        "changed_files": changed_files,
        "task": options.task,
        "task_context": parsed_task_context
    });
    crate::integrations::build_enrichment(
        repo_path,
        &payload,
        options.enrich,
        "You are a careful repo assistant. Never fabricate.",
        crate::integrations::DEFAULT_ENRICHMENT_PROMPT_TEMPLATE,
    )
}

fn changed_files(repo_path: &Path, staged: bool) -> Vec<String> {
    let mut files = BTreeSet::new();
    let diff_args: &[&str] = if staged {
        &["diff", "--cached", "--name-only"]
    } else {
        &["diff", "HEAD", "--name-only"]
    };

    if let Some(output) = git_stdout(repo_path, diff_args) {
        for line in output
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
        {
            files.insert(line.to_string());
        }
    }

    if !staged {
        if let Some(output) = git_stdout(repo_path, &["ls-files", "--others", "--exclude-standard"])
        {
            for line in output
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
            {
                files.insert(line.to_string());
            }
        }
    }

    files.into_iter().collect()
}

fn suggested_commands(repo_path: &Path, changed_files: &[String]) -> Vec<String> {
    let mut commands = Vec::new();
    let touches_rust = changed_files
        .iter()
        .any(|f| f.ends_with(".rs") || f == "Cargo.toml" || f == "Cargo.lock");
    let touches_sruja = changed_files.iter().any(|f| f.ends_with(".sruja"));
    let touches_extension = changed_files
        .iter()
        .any(|f| f.starts_with("extension/") || f.ends_with(".ts") || f.ends_with(".tsx"));

    // Prefer `just` when both exist (Makefile is a compatibility shim).
    if repo_path.join("justfile").exists() {
        commands.push("just check".to_string());
    } else if repo_path.join("Makefile").exists() {
        commands.push("make check".to_string());
    }

    if touches_rust {
        commands.push("cargo fmt --check".to_string());
        commands.push("cargo clippy --workspace -- -D warnings".to_string());
        commands.push("cargo test --workspace".to_string());
    }

    if touches_extension && repo_path.join("extension/package.json").exists() {
        commands.push("cd extension && npm run compile".to_string());
        commands.push("cd extension && npm test".to_string());
    }

    if touches_sruja {
        for file in changed_files
            .iter()
            .filter(|f| f.ends_with(".sruja"))
            .take(5)
        {
            commands.push(format!("sruja lint {}", file));
        }
    }

    commands.push("sruja check -r .".to_string());
    commands.sort();
    commands.dedup();
    commands
}

fn select_focus_file(changed_files: &[String]) -> Option<&str> {
    changed_files
        .iter()
        .find(|file| is_source_file(file) && !is_low_signal_file(file))
        .or_else(|| changed_files.iter().find(|file| !is_low_signal_file(file)))
        .or_else(|| changed_files.first())
        .map(String::as_str)
}

fn is_source_file(file: &str) -> bool {
    [
        ".rs", ".ts", ".tsx", ".js", ".jsx", ".py", ".go", ".java", ".cs", ".rb", ".php", ".kt",
        ".scala", ".c", ".cc", ".cpp", ".h", ".hpp",
    ]
    .iter()
    .any(|ext| file.ends_with(ext))
}

fn is_low_signal_file(file: &str) -> bool {
    file.ends_with(".lock")
        || file.ends_with("package-lock.json")
        || file.ends_with("pnpm-lock.yaml")
        || file.ends_with("yarn.lock")
        || file.ends_with(".md")
        || file.starts_with("docs/")
        || file.starts_with("book/")
        || file.contains("/generated/")
        || file.contains("/fixtures/")
}

fn truth_status_label(status: &sruja_diff::TruthStatus) -> &'static str {
    match status {
        sruja_diff::TruthStatus::Reviewed => "reviewed",
        sruja_diff::TruthStatus::Drifted => "drifted",
        sruja_diff::TruthStatus::Unknown => "unknown",
    }
}

fn git_first_line(repo_path: &Path, args: &[&str]) -> Option<String> {
    git_stdout(repo_path, args).and_then(|output| output.lines().next().map(str::to_string))
}

fn git_stdout(repo_path: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn escape_json_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn suggested_commands_include_just_check_and_rust_checks() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("justfile"), "check:\n\tcargo test\n").unwrap();
        let commands =
            suggested_commands(dir.path(), &["crates/sruja-cli/src/main.rs".to_string()]);

        assert!(commands.contains(&"just check".to_string()));
        assert!(commands.contains(&"cargo fmt --check".to_string()));
        assert!(commands.contains(&"cargo clippy --workspace -- -D warnings".to_string()));
        assert!(commands.contains(&"cargo test --workspace".to_string()));
    }

    #[test]
    fn suggested_commands_include_sruja_lint_for_architecture_files() {
        let dir = tempdir().unwrap();
        let commands = suggested_commands(dir.path(), &["repo.sruja".to_string()]);

        assert!(commands.contains(&"sruja lint repo.sruja".to_string()));
        assert!(commands.contains(&"sruja check -r .".to_string()));
    }

    #[test]
    fn select_focus_file_prefers_source_over_lockfiles_and_docs() {
        let files = vec![
            "Cargo.lock".to_string(),
            "README.md".to_string(),
            "crates/sruja-cli/src/commands/ai.rs".to_string(),
        ];

        assert_eq!(
            select_focus_file(&files),
            Some("crates/sruja-cli/src/commands/ai.rs")
        );
    }
}
