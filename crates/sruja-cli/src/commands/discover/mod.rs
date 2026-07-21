//! Discovery commands: question bank, context, explanation, and repomap
//! for intelligent architecture capture.
//!
//! Use with the sruja-architecture skill so the AI asks users these questions
//! before or during discovery. See skills/sruja-architecture/REFERENCE.md.

pub mod questions;
pub mod models;
pub mod context;
pub mod analysis;
pub mod explanation;

pub use questions::discover_questions;
pub use context::{
    discover_context_string, discover_context_json_from_graph,
};
pub use explanation::{discover_explanation_markdown, discover_explanation_value_from_graph};

use std::path::Path;

use sruja_agent::DEFAULT_MODEL;
use sruja_scan::{generate_repomap_from_graph, RepoMapOptions};

use super::{scan_repo_cached, scan_repo_cached_with_opts, CliError};
use crate::integrations::{
    resolve_enrichment_plan, resolve_openai_auth, run_cmd_enrichment, run_openai_markdown,
};

use self::explanation::{build_discover_explanation, format_discovery_explanation};
use self::models::DiscoverExplanationJson;

/// Print repo context summary for the agent to derive contextual questions.
pub async fn discover_context(repo: &str, format: &str) -> Result<(), CliError> {
    if format == "json" {
        let json = context::discover_context_json(repo)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&json).map_err(|e| CliError::validation(e.to_string()))?
        );
        return Ok(());
    }
    let s = context::discover_context_string(repo)?;
    println!("{}", s);
    Ok(())
}

/// Explain what Sruja discovered, why it inferred that shape, and what to review next.
#[allow(clippy::too_many_arguments)]
pub async fn discover_explain(
    repo: &str,
    format: &str,
    export_report: Option<&str>,
    enrich: &crate::enrichment::EnrichmentRef<'_>,
    incremental: bool,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo),
        )));
    }

    let graph = scan_repo_cached_with_opts(repo_path, incremental)?;
    let mut explanation = build_discover_explanation(repo, repo_path, &graph)?;

    if enrich.enrich || enrich.cmd.is_some() {
        explanation.enrichment = enrich_discover_explain(&explanation, repo_path, enrich);
    }

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&explanation)
                .map_err(|e| CliError::validation(e.to_string()))?;
            if let Some(path) = export_report {
                std::fs::write(path, &json)?;
            } else {
                println!("{}", json);
            }
        }
        "text" => {
            let text = format_discovery_explanation(&explanation);
            if let Some(path) = export_report {
                std::fs::write(path, &text)?;
            } else {
                println!("{}", text);
            }
        }
        _ => {
            return Err(CliError::validation(format!(
                "Unknown format: {}. Use: text or json",
                format
            )));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn enrich_discover_explain(
    explanation: &DiscoverExplanationJson,
    repo_path: &Path,
    enrich: &crate::enrichment::EnrichmentRef<'_>,
) -> Option<models::DiscoverEnrichment> {
    if !enrich.enrich && enrich.cmd.is_none() {
        return None;
    }

    let plan = resolve_enrichment_plan(
        repo_path,
        enrich.cmd,
        enrich.model,
        enrich.base_url,
        Some(enrich.timeout_ms),
        Some(enrich.max_bytes),
    );
    let provider = enrich.provider.unwrap_or(plan.provider.as_str());
    let limits = plan.limits;

    let payload = match serde_json::to_value(explanation) {
        Ok(v) => v,
        Err(e) => {
            return Some(models::DiscoverEnrichment {
                status: "error".to_string(),
                provider: Some(provider.to_string()),
                model: None,
                error: Some(format!("Failed to serialize explanation JSON: {e}")),
                narrative_markdown: None,
            });
        }
    };

    if provider == "cmd" {
        let Some(cmd) = plan.cmd.as_deref() else {
            return Some(models::DiscoverEnrichment {
                status: "skipped".to_string(),
                provider: Some("external_cmd".to_string()),
                model: None,
                error: Some("No command configured. Pass --enrich-cmd or set SRUJA_ENRICH_CMD (or .sruja/config.toml [integrations].cmd).".to_string()),
                narrative_markdown: None,
            });
        };
        let stdin_payload = serde_json::to_vec(&payload).unwrap_or_default();
        return Some(match run_cmd_enrichment(cmd, &stdin_payload, limits) {
            Ok(md) => models::DiscoverEnrichment {
                status: "ok".to_string(),
                provider: Some("external_cmd".to_string()),
                model: None,
                error: None,
                narrative_markdown: Some(md),
            },
            Err(e) => models::DiscoverEnrichment {
                status: "error".to_string(),
                provider: Some("external_cmd".to_string()),
                model: None,
                error: Some(e),
                narrative_markdown: None,
            },
        });
    }

    if provider != "openai" {
        return Some(models::DiscoverEnrichment {
            status: "skipped".to_string(),
            provider: Some(provider.to_string()),
            model: None,
            error: Some(
                "Unsupported provider. Use provider=cmd (recommended) or provider=openai."
                    .to_string(),
            ),
            narrative_markdown: None,
        });
    }

    let model = plan.model.as_deref().unwrap_or(DEFAULT_MODEL);
    let base_url = plan
        .base_url
        .as_deref()
        .unwrap_or("https://api.openai.com/v1");
    let Some(key) = resolve_openai_auth() else {
        return Some(models::DiscoverEnrichment {
            status: "skipped".to_string(),
            provider: Some("openai".to_string()),
            model: Some(model.to_string()),
            error: Some(
                "Missing API key (set OPENAI_API_KEY or SRUJA_ENRICH_API_KEY).".to_string(),
            ),
            narrative_markdown: None,
        });
    };

    let user_prompt = format!(
        "You are analyzing an architecture graph explanation.\n\n\
         {}\n\n\
         Produce markdown with these sections:\n\
         - \"Architectural Narrative\" (narrative summary of the architecture and role)\n\
         - \"Why the God Nodes matter\" (bullets explaining significance based on facts)\n\
         - \"Surprising Connections insights\" (analysis of Surprising Connections and suggestions about coupling)\n\
         - \"Architectural Risks & Questions\" (suggested investigative paths)\n\n\
         JSON facts:\n{}",
        crate::integrations::ENRICHMENT_FACTS_PREAMBLE,
        payload
    );

    match run_openai_markdown(
        "You are a careful architecture analyst. Never fabricate.",
        &user_prompt,
        model,
        base_url,
        &key,
    ) {
        Ok(md) => Some(models::DiscoverEnrichment {
            status: "ok".to_string(),
            provider: Some("openai".to_string()),
            model: Some(model.to_string()),
            error: None,
            narrative_markdown: Some(md),
        }),
        Err(e) => Some(models::DiscoverEnrichment {
            status: "error".to_string(),
            provider: Some("openai".to_string()),
            model: Some(model.to_string()),
            error: Some(e),
            narrative_markdown: None,
        }),
    }
}

/// Generate a repository map with tree-sitter signatures for top files.
pub fn discover_repomap(
    repo: &str,
    max_files: usize,
    max_tokens: usize,
) -> Result<String, CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo),
        )));
    }

    let graph = scan_repo_cached(repo_path)?;

    let options = RepoMapOptions {
        max_files,
        max_tokens,
        include_signatures: true,
    };

    generate_repomap_from_graph(repo_path, &graph, &options)
        .map_err(|e| CliError::scan(e.to_string()))
}

/// Print repository map for LLM context.
pub async fn discover_repomap_cmd(
    repo: &str,
    max_files: usize,
    max_tokens: usize,
) -> Result<(), CliError> {
    let repomap = discover_repomap(repo, max_files, max_tokens)?;
    println!("{}", repomap);
    Ok(())
}
