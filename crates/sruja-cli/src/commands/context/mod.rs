pub mod format;
pub mod logic;
pub mod types;

use std::fs;
use std::path::Path;

use self::format::*;
use self::logic::*;
use self::types::*;
use crate::commands::{scan_repo_cached, CliError};

#[derive(Debug, Clone, Copy)]
pub struct ContextRequest<'a> {
    pub file: Option<&'a str>,
    pub element_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub base_ref: Option<&'a str>,
    pub head_ref: Option<&'a str>,
    pub intent: Option<&'a str>,
    pub depth: usize,
    pub max_tokens: usize,
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
        let selectors = TaskSelectors {
            file: req.file,
            element_id: req.element_id,
            query: req.query,
            base_ref: req.base_ref,
            head_ref: req.head_ref,
            depth: Some(req.depth),
        };
        let ctx = build_task_context(&graph, repo_root, selectors, req.max_tokens)?;
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
        _ => Err(CliError::Validation(format!(
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

    let multi = MultiRepoArchitectureContext {
        repos: contexts,
        combined_summary,
        cross_repo_rules,
    };

    match format {
        "cursor-rules" => Ok(format_cursor_rules_multi(&multi)),
        "copilot-instructions" => Ok(format_copilot_instructions_multi(&multi)),
        "markdown" => Ok(format_markdown_multi(&multi)),
        "json" | "for-ai" | "legacy-json" => Ok(serde_json::to_string_pretty(&multi)?),
        _ => Err(CliError::Validation(format!(
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
