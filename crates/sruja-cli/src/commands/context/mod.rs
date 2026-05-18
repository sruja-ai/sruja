pub mod format;
pub mod logic;
pub mod types;

use std::fs;
use std::path::Path;

use self::format::*;
use self::logic::*;
use self::types::*;
use crate::commands::{scan_repo_cached, CliError};
use crate::utils::run_id::generate_run_id;
use crate::utils::run_snapshots::{blake3_hex, write_json_snapshot};

#[derive(Debug, Clone, Copy)]
pub struct ContextRequest<'a> {
    pub run_id: Option<&'a str>,
    pub file: Option<&'a str>,
    pub element_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub base_ref: Option<&'a str>,
    pub head_ref: Option<&'a str>,
    pub intent: Option<&'a str>,
    pub depth: usize,
    pub max_tokens: usize,
    /// Split `for-ai` JSON into invariant/tools/volatile blocks for prompt-cache-friendly payloads.
    pub cache_friendly: bool,
}

pub async fn context_string(
    repo_root: &str,
    format: &str,
    req: ContextRequest<'_>,
) -> Result<String, CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = scan_repo_cached(repo_path)?;

    if format == "json" || format == "for-ai" {
        let run_id = req
            .run_id
            .map(|s| s.to_string())
            .unwrap_or_else(generate_run_id);
        let selectors = TaskSelectors {
            file: req.file,
            element_id: req.element_id,
            query: req.query,
            base_ref: req.base_ref,
            head_ref: req.head_ref,
            depth: Some(req.depth),
        };
        let mut ctx = build_task_context(&graph, repo_root, selectors, req.max_tokens)?;
        ctx.run_id = Some(run_id.clone());
        // Persist a bounded snapshot for replay/resume.
        let snapshot = serde_json::json!({
            "schema_version": "context_snapshot/v1",
            "run_id": ctx.run_id,
            "repo": repo_root,
            "selectors": {
                "file": req.file,
                "element_id": req.element_id,
                "query": req.query,
                "base_ref": req.base_ref,
                "head_ref": req.head_ref,
                "depth": req.depth,
                "max_tokens": req.max_tokens,
            },
            "focus_ids": ctx.focus_elements.iter().map(|e| e.element_id.clone()).collect::<Vec<_>>(),
            "truth_status": ctx.truth_status,
            "confidence": ctx.confidence,
            "risk": ctx.risk,
            "grounding_trace": ctx.grounding_trace,
            "hydrated_files": ctx.hydrated_files.iter().map(|f| serde_json::json!({
                "element_id": f.element_id,
                "path": f.path,
                "truncated": f.truncated,
                "blake3": blake3_hex(&f.content),
            })).collect::<Vec<_>>(),
            "semantic_candidates": ctx.semantic_candidates,
        });
        if let Some(run_id) = ctx.run_id.as_deref() {
            let _ = write_json_snapshot(repo_path, run_id, "task_context.json", &snapshot);
        }
        if format == "for-ai" && req.cache_friendly {
            let arch = build_architecture_context(
                &graph,
                repo_root,
                None,
                None,
                req.depth,
                req.max_tokens,
            )?;
            let export = build_cache_friendly_task_export(repo_root, &arch, ctx);
            return Ok(serde_json::to_string_pretty(&export)?);
        }
        return Ok(serde_json::to_string_pretty(&ctx)?);
    }

    let context = build_architecture_context(
        &graph,
        repo_root,
        req.file,
        req.intent,
        req.depth,
        req.max_tokens,
    )?;

    match format {
        "cursor-rules" => Ok(format_cursor_rules(&context)),
        "copilot-instructions" => Ok(format_copilot_instructions(&context)),
        "markdown" => Ok(format_markdown(&context)),
        "repomap" => Ok(format_repomap(&context, &graph)),
        "legacy-json" => Ok(serde_json::to_string_pretty(&context)?),
        _ => Err(CliError::validation(format!(
            "Unknown format: {}. Use: cursor-rules, copilot-instructions, markdown, repomap, json, for-ai, legacy-json",
            format
        ))),
    }
}

pub async fn context_string_multi(
    repo_roots: &[String],
    format: &str,
    req: ContextRequest<'_>,
) -> Result<String, CliError> {
    let repos: Vec<String> = if repo_roots.is_empty() {
        vec![".".to_string()]
    } else {
        repo_roots.to_vec()
    };

    if repos.len() == 1 {
        return context_string(&repos[0], format, req).await;
    }

    if req.cache_friendly && (format == "for-ai" || format == "json") {
        return Err(CliError::validation(
            "--cache-friendly is only supported for a single repository (-r .); use one -r per export",
        ));
    }

    let mut contexts: Vec<ArchitectureContext> = Vec::with_capacity(repos.len());
    let mut repomaps: Vec<String> = Vec::new();
    for repo_root in &repos {
        let repo_path = Path::new(repo_root);
        if !repo_path.exists() {
            return Err(CliError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Repository not found: {}", repo_root),
            )));
        }

        let graph = scan_repo_cached(repo_path)?;

        let context = build_architecture_context(
            &graph,
            repo_root,
            req.file,
            req.intent,
            req.depth,
            req.max_tokens,
        )?;
        if format == "repomap" {
            repomaps.push(format_repomap(&context, &graph));
        }
        contexts.push(context);
    }

    if format == "repomap" {
        return Ok(repomaps.join("\n\n---\n\n"));
    }

    let combined_summary = ContextSummary {
        total_modules: contexts.iter().map(|c| c.summary.total_modules).sum(),
        total_services: contexts.iter().map(|c| c.summary.total_services).sum(),
        total_databases: contexts.iter().map(|c| c.summary.total_databases).sum(),
        total_external_apis: contexts.iter().map(|c| c.summary.total_external_apis).sum(),
    };

    let cross_repo_rules =
        vec!(
        "Treat each repo as a hard boundary: avoid cross-repo imports; integrate via API/events"
            .to_string(),
        "If a change spans repos, update contracts (OpenAPI/SDK) and bump versions together"
            .to_string(),
        "Run checks in every touched repo (format, lint/typecheck, unit tests, integration tests)"
            .to_string(),
    );

    let run_id = if format == "json" || format == "for-ai" {
        Some(
            req.run_id
                .map(|s| s.to_string())
                .unwrap_or_else(generate_run_id),
        )
    } else {
        req.run_id.map(|s| s.to_string())
    };

    let multi = MultiRepoArchitectureContext {
        run_id,
        repos: contexts,
        combined_summary,
        cross_repo_rules,
    };

    match format {
        "cursor-rules" => Ok(format_cursor_rules_multi(&multi)),
        "copilot-instructions" => Ok(format_copilot_instructions_multi(&multi)),
        "markdown" => Ok(format_markdown_multi(&multi)),
        "json" | "for-ai" | "legacy-json" => Ok(serde_json::to_string_pretty(&multi)?),
        _ => Err(CliError::validation(format!(
            "Unknown format: {}. Use: cursor-rules, copilot-instructions, markdown, repomap, json, for-ai, legacy-json",
            format
        ))),
    }
}

pub async fn context_export(
    repo_roots: &[String],
    format: &str,
    output: Option<&str>,
    req: ContextRequest<'_>,
) -> Result<(), CliError> {
    let content = context_string_multi(repo_roots, format, req).await?;

    if let Some(path) = output {
        fs::write(path, &content)?;
        eprintln!("Written context to {}", path);
    } else {
        println!("{}", content);
    }

    Ok(())
}

const IDE_RULES_FOOTER: &str = "\n# Global AI Agent Guidelines\nYou MUST read and strictly adhere to the instructions located in `AGENTS.md` before proceeding with any task.\n";

/// Write editor rule files from validated architecture (cursor-rules, copilot, Claude, Gemini, llms-architecture.txt).
pub async fn sync_ide_rules(repo_roots: &[String], max_tokens: usize) -> Result<(), CliError> {
    let repos: Vec<String> = if repo_roots.is_empty() {
        vec![".".to_string()]
    } else {
        repo_roots.to_vec()
    };
    if repos.len() != 1 {
        return Err(CliError::validation(
            "sync-ide-rules supports a single repository (-r .) only",
        ));
    }
    let repo = &repos[0];
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {repo}"),
        )));
    }

    let graph = scan_repo_cached(repo_path)?;
    let arch = build_architecture_context(&graph, repo, None, None, 2, max_tokens)?;

    let mut cursor = format_cursor_rules(&arch);
    cursor.push_str(IDE_RULES_FOOTER);
    fs::write(repo_path.join(".cursorrules"), &cursor)?;

    let mut copilot = format_copilot_instructions(&arch);
    copilot.push_str(IDE_RULES_FOOTER);
    let copilot_path = repo_path.join(".github/copilot-instructions.md");
    if let Some(parent) = copilot_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&copilot_path, &copilot)?;

    let mut claude = format_cursor_rules(&arch);
    claude.insert_str(
        0,
        "<!-- Generated by sruja sync-ide-rules; same rules as .cursorrules -->\n\n",
    );
    claude.push_str(IDE_RULES_FOOTER);
    fs::write(repo_path.join("CLAUDE.md"), &claude)?;

    let gemini_dir = repo_path.join(".gemini");
    fs::create_dir_all(&gemini_dir)?;
    let mut gemini = format_cursor_rules(&arch);
    gemini.push_str(IDE_RULES_FOOTER);
    fs::write(gemini_dir.join("AGENTS.md"), &gemini)?;

    let llms = format_llms_architecture(&arch);
    fs::write(repo_path.join("llms-architecture.txt"), &llms)?;

    eprintln!("Synced IDE rules: .cursorrules, .github/copilot-instructions.md, CLAUDE.md, .gemini/AGENTS.md, llms-architecture.txt");
    Ok(())
}

pub use format::{format_invariant_markdown, format_llms_architecture};
pub use logic::build_architecture_context;
