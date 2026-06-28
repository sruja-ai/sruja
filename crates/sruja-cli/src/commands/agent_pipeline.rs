use std::path::PathBuf;
use std::sync::Arc;

use sruja_agent::llm::{OpenAiClient, TieredClient};
use sruja_agent::pipeline::PipelineOrchestrator;
use sruja_agent::tool::ToolRegistry;

use super::error::CliError;
use crate::config;

/// Handle `sruja agent pipeline` command.
///
/// Resolves multi-provider config from `.sruja/config.toml`, builds a
/// `TieredClient` that routes each stage's model to its provider, then
/// runs the pipeline.
pub async fn handle_pipeline(
    repo: &str,
    goal: &str,
    dry_run: bool,
    judge_only: bool,
    focus: Option<String>,
    max_cycles: Option<usize>,
    format: &str,
) -> Result<(), CliError> {
    let repo_path = PathBuf::from(repo);

    // ── Resolve multi-provider configuration ──────────────────────────────
    let multi_config = config::resolve_multi_provider_config(&repo_path)?;

    // ── Build TieredClient (same pattern as agent_loop) ──────────────────
    let mid_config = &multi_config.mid;
    let default_client = Arc::new(
        OpenAiClient::new(&mid_config.api_key, &mid_config.base_url, &mid_config.model)
            .map_err(|e| CliError::validation(format!("LLM client error: {e}")))?,
    );

    let mut tiered = TieredClient::new(default_client.clone());
    for tier_cfg in [
        &multi_config.cheap,
        &multi_config.mid,
        &multi_config.premium,
        &multi_config.review,
    ] {
        let needs_own_client =
            tier_cfg.api_key != mid_config.api_key || tier_cfg.base_url != mid_config.base_url;

        if needs_own_client {
            let client = Arc::new(
                OpenAiClient::new(&tier_cfg.api_key, &tier_cfg.base_url, &tier_cfg.model)
                    .map_err(|e| CliError::validation(format!("LLM client error: {e}")))?,
            );
            tiered = tiered.with_route(&tier_cfg.model, client.clone());
            let family = model_family(&tier_cfg.model);
            if !family.is_empty() {
                tiered = tiered.with_provider_name_contains(family, client);
            }
        }
    }

    // ── Build architecture context for grounding ─────────────────────────
    let context = build_architecture_context(&repo_path)?;

    // ── Build orchestrator with tiered routing + context + tools ────────
    let tools = Arc::new(
        ToolRegistry::with_builtin(repo_path.to_path_buf(), vec![
            "find".into(), "grep".into(), "head".into(), "tail".into(),
            "wc".into(), "cat".into(), "ls".into(),
        ])
        .dry_run()  // Pipeline tools are read-only
    );

    let orchestrator = PipelineOrchestrator::new(
        &repo_path,
        goal.to_string(),
        dry_run,
        focus,
        max_cycles,
    )
    .with_tiered(tiered)
    .with_context(context)
    .with_tools(tools);

    // ── Judge-only short circuit ─────────────────────────────────────────
    if judge_only {
        return handle_judge_only(orchestrator).await;
    }

    println!("Pipeline goal: {goal}");
    println!(
        "{}",
        if dry_run {
            "Dry-run mode: analyzer + judge only, no code changes"
        } else {
            "Full pipeline: analyzer → prober → fixer → judge"
        }
    );

    let mut orchestrator = orchestrator;
    let result = orchestrator.run().await;

    // ── Print summary ────────────────────────────────────────────────────
    println!("\n{}", "=".repeat(60));
    println!("Pipeline {}", if result.converged { "CONVERGED ✅" } else { "STOPPED" });
    println!("Reason: {}", result.reason);
    println!("Cycles: {} | Stages: {}", result.cycles, result.stages.len());
    println!("Lessons recorded: {}", result.lessons_recorded);

    if let Some(ref score) = result.scorecard {
        println!("\n{}", format_scorecard(score));
    }

    if format == "json" {
        let summary = serde_json::json!({
            "converged": result.converged,
            "reason": result.reason,
            "cycles": result.cycles,
            "lesson_count": result.lessons_recorded,
            "scorecard": result.scorecard.as_ref().map(|s| {
                serde_json::json!({
                    "total": s.total,
                    "functional_correctness": s.functional_correctness,
                    "code_quality": s.code_quality,
                    "test_coverage": s.test_coverage,
                    "ux_quality": s.ux_quality,
                    "cost_efficiency": s.cost_efficiency,
                    "summary": s.summary,
                })
            }),
        });
        println!("\n{}", serde_json::to_string_pretty(&summary).unwrap_or_default());
    }

    println!("\nReport: .sruja/pipeline/LIVE.md");

    if !result.converged {
        eprintln!("\nPipeline stopped: {}", result.reason);
    }

    Ok(())
}

async fn handle_judge_only(orchestrator: PipelineOrchestrator) -> Result<(), CliError> {
    println!("Judge-only mode: re-scoring current state...");
    let scorecard = orchestrator.judge_only().await
        .map_err(|e| CliError::validation(format!("Judge failed: {e}")))?;
    println!("{}", format_scorecard(&scorecard));
    Ok(())
}

/// Build a rich architecture context string from the repo scan + file reads.
///
/// This is injected into every pipeline stage's task prompt so the LLM
/// has grounded structural facts instead of flying blind.
fn build_architecture_context(repo_path: &PathBuf) -> Result<String, CliError> {
    let graph = super::scan_repo_cached(repo_path)?;

    // ── File inventory: source files grouped by package ───────────────
    let mut files_by_package: std::collections::BTreeMap<String, Vec<(String, u64)>> =
        std::collections::BTreeMap::new();

    let mut god_modules = Vec::new();

    for node in &graph.nodes {
        if let Some(ref path) = node.path {
            if path.contains("node_modules") || path.contains(".git")
                || path.contains("__pycache__") || path.contains(".venv")
            {
                continue;
            }
            let pkg = path.split('/').take(3).collect::<Vec<_>>().join("/");
            let complexity = node.metadata.get("complexity")
                .and_then(|c| c.parse::<u64>().ok())
                .unwrap_or(0);
            files_by_package.entry(pkg).or_default().push((path.clone(), complexity));
        }

        if let Some(deps) = node.metadata.get("outgoing_count")
            .and_then(|c| c.parse::<usize>().ok())
        {
            if deps > 10 {
                god_modules.push((node.label.clone(), deps));
            }
        }
    }

    god_modules.sort_by(|a, b| b.1.cmp(&a.1));

    // ── Build context ────────────────────────────────────────────────
    let mut ctx = String::new();

    ctx.push_str(&format!(
        "Repository: {} | {} nodes | {} edges\n\n",
        repo_path.display(),
        graph.nodes.len(),
        graph.edges.len()
    ));

    // Package overview
    ctx.push_str("## Packages\n");
    let mut pkgs: Vec<_> = files_by_package.iter()
        .map(|(k, v)| (k.as_str(), v.len()))
        .collect();
    pkgs.sort_by(|a, b| b.1.cmp(&a.1));
    for (pkg, count) in pkgs.iter().take(10) {
        ctx.push_str(&format!("  {pkg}: {count} files\n"));
    }

    // God modules
    if !god_modules.is_empty() {
        ctx.push_str("\n## God Modules (high fan-out, high regression risk)\n");
        for (name, deps) in god_modules.iter().take(10) {
            ctx.push_str(&format!("  - {name}: {deps} dependencies\n"));
        }
    }

    // Key source files
    ctx.push_str("\n## Key Source Files (by complexity)\n");
    let mut all_files: Vec<_> = files_by_package.values()
        .flat_map(|v| v.iter().cloned())
        .collect();
    all_files.sort_by(|a, b| b.1.cmp(&a.1));
    for (path, complexity) in all_files.iter().take(30) {
        ctx.push_str(&format!("  - {path} (complexity: {complexity})\n"));
    }

    // ── Actual file reads + grep (what a real PM agent would do) ─────
    ctx.push_str("\n\n## Discovered Facts (from file reads)\n");

    // Read AGENTS.md (project rules)
    if let Ok(content) = std::fs::read_to_string(repo_path.join("AGENTS.md")) {
        let truncated = if content.len() > 3000 {
            let end = content.floor_char_boundary(3000);
            format!("{}...", &content[..end])
        } else {
            content
        };
        ctx.push_str(&format!("\n### AGENTS.md (first 3000 chars)\n```\n{truncated}\n```\n"));
    }

    // Read CI config
    if let Ok(content) = std::fs::read_to_string(repo_path.join(".gitlab-ci.yml")) {
        let truncated = if content.len() > 3000 {
            let end = content.floor_char_boundary(3000);
            format!("{}...", &content[..end])
        } else {
            content
        };
        ctx.push_str(&format!("\n### .gitlab-ci.yml (first 3000 chars)\n```\n{truncated}\n```\n"));
    }

    // Read pyproject.toml testpaths
    if let Ok(content) = std::fs::read_to_string(repo_path.join("pyproject.toml")) {
        let lines: Vec<&str> = content.lines().collect();
        let mut in_tool = false;
        let mut testpaths = Vec::new();
        for line in &lines {
            if line.contains("[tool.pytest") { in_tool = true; }
            if in_tool && line.contains("testpaths") {
                testpaths.push(line.to_string());
            }
            if in_tool && line.starts_with('[') && !line.contains("[tool.pytest") {
                in_tool = false;
            }
        }
        if !testpaths.is_empty() {
            ctx.push_str(&format!("\n### pyproject.toml testpaths\n{}\n", testpaths.join("\n")));
        }
    }

    // Grep for exception patterns (production quality indicator)
    if let Ok(output) = std::process::Command::new("grep")
        .args(["-rn", "except\\b", "packages/", "--include=*.py", "-l"])
        .current_dir(repo_path)
        .output()
    {
        let files = String::from_utf8_lossy(&output.stdout);
        let file_count = files.lines().filter(|l| !l.is_empty() && !l.contains("node_modules")).count();
        ctx.push_str(&format!("\n### Exception patterns: {file_count} files with try/except\n"));
    }

    // Count tests
    if let Ok(output) = std::process::Command::new("find")
        .args(["tests/", "-name", "*.py", "-path", "*/unit/*"])
        .current_dir(repo_path)
        .output()
    {
        let unit_count = String::from_utf8_lossy(&output.stdout).lines().filter(|l| !l.is_empty()).count();
        ctx.push_str(&format!("  tests/unit/: {unit_count} files\n"));
    }
    if let Ok(output) = std::process::Command::new("find")
        .args(["tests/", "-name", "*.py", "-path", "*/integration/*"])
        .current_dir(repo_path)
        .output()
    {
        let int_count = String::from_utf8_lossy(&output.stdout).lines().filter(|l| !l.is_empty()).count();
        ctx.push_str(&format!("  tests/integration/: {int_count} files\n"));
    }

    // Check CI test paths vs actual test locations
    ctx.push_str("\n### CI vs Actual Test Coverage\n");
    if let Ok(output) = std::process::Command::new("grep")
        .args(["pytest", ".gitlab-ci.yml"])
        .current_dir(repo_path)
        .output()
    {
        let ci_lines = String::from_utf8_lossy(&output.stdout);
        ctx.push_str("  CI runs:\n");
        for line in ci_lines.lines().filter(|l| !l.is_empty()) {
            ctx.push_str(&format!("    {line}\n"));
        }
    }

    Ok(ctx)
}

/// Extract the alphabetic prefix of a model name for name-substring routing.
fn model_family(model: &str) -> String {
    let base = model.rsplit('/').next().unwrap_or(model);
    base.chars()
        .take_while(|c| c.is_alphabetic())
        .collect::<String>()
        .to_lowercase()
}

fn format_scorecard(score: &sruja_agent::pipeline::Scorecard) -> String {
    format!(
        "\
Scorecard
---------
functional_correctness: {fc}/5
code_quality:           {cq}/5
test_coverage:          {tc}/5
ux_quality:             {ux}/5
cost_efficiency:        {ce}/5
total:                  {t:.1}/5
summary:                {summary}
improved_from_previous: {imp}",
        fc = score.functional_correctness,
        cq = score.code_quality,
        tc = score.test_coverage,
        ux = score.ux_quality,
        ce = score.cost_efficiency,
        t = score.total,
        summary = score.summary,
        imp = score.improved_from_previous,
    )
}
