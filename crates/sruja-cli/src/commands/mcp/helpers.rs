use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::super::CliError;
use crate::integrations::{
    resolve_enrichment_plan, resolve_openai_auth, run_cmd_enrichment, run_openai_markdown,
};
use sruja_agent::DEFAULT_MODEL;

#[allow(clippy::too_many_arguments)]
pub(crate) fn enrich_wrapper_json(
    repo_path: &Path,
    enrich_provider: Option<&str>,
    enrich_cmd: Option<&str>,
    enrich_model: Option<&str>,
    enrich_base_url: Option<&str>,
    enrich_timeout_ms: u64,
    enrich_max_bytes: usize,
    kind: &str,
    grounded: Value,
) -> Value {
    let plan = resolve_enrichment_plan(
        repo_path,
        enrich_cmd,
        enrich_model,
        enrich_base_url,
        Some(enrich_timeout_ms),
        Some(enrich_max_bytes),
    );
    let provider = enrich_provider.unwrap_or(plan.provider.as_str());

    let input = json!({
        "schema_version": "mcp_enrichment_input/v1",
        "kind": kind,
        "grounded": grounded,
    });
    let stdin_payload = serde_json::to_vec(&input).unwrap_or_default();

    let enrichment = if provider == "cmd" {
        match plan.cmd.as_deref() {
            Some(cmd) => match run_cmd_enrichment(cmd, &stdin_payload, plan.limits) {
                Ok(md) => json!({
                    "status": "ok",
                    "provider": "external_cmd",
                    "model": Value::Null,
                    "error": Value::Null,
                    "narrative_markdown": md
                }),
                Err(e) => json!({
                    "status": "error",
                    "provider": "external_cmd",
                    "model": Value::Null,
                    "error": e,
                    "narrative_markdown": Value::Null
                }),
            },
            None => json!({
                "status": "skipped",
                "provider": "cmd",
                "model": Value::Null,
                "error": "No command configured. Provide enrich_cmd or set SRUJA_ENRICH_CMD / .sruja/config.toml [integrations].cmd.",
                "narrative_markdown": Value::Null
            }),
        }
    } else if provider == "openai" {
        let model = plan.model.as_deref().unwrap_or(DEFAULT_MODEL);
        let base_url = plan
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");
        match resolve_openai_auth() {
            Some(key) => {
                let user_prompt = format!(
                    "You are assisting an AI coding agent.\n\n\
                     {}\n\n\
                     Produce markdown with these sections:\n\
                     - \"Summary\"\n\
                     - \"Risks / unknowns to verify\" (bullets)\n\
                     - \"Suggested verification steps\" (bullets)\n\n\
                     JSON facts:\n{}",
                    crate::integrations::ENRICHMENT_FACTS_PREAMBLE,
                    input
                );
                match run_openai_markdown(
                    "You are a careful repo assistant. Never fabricate.",
                    &user_prompt,
                    model,
                    base_url,
                    &key,
                ) {
                    Ok(md) => json!({
                        "status": "ok",
                        "provider": "openai",
                        "model": model,
                        "error": Value::Null,
                        "narrative_markdown": md
                    }),
                    Err(e) => json!({
                        "status": "error",
                        "provider": "openai",
                        "model": model,
                        "error": e,
                        "narrative_markdown": Value::Null
                    }),
                }
            }
            None => json!({
                "status": "skipped",
                "provider": "openai",
                "model": model,
                "error": "Missing API key (set OPENAI_API_KEY or SRUJA_ENRICH_API_KEY; SRUJA_LLM_API_KEY is deprecated).",
                "narrative_markdown": Value::Null
            }),
        }
    } else {
        json!({
            "status": "skipped",
            "provider": provider,
            "model": Value::Null,
            "error": "Unsupported provider. Use cmd (recommended) or openai.",
            "narrative_markdown": Value::Null
        })
    };

    json!({
        "schema_version": "mcp_enriched_output/v1",
        "grounded": input.get("grounded").cloned().unwrap_or(Value::Null),
        "enrichment": enrichment
    })
}

pub(crate) async fn get_or_scan_graph(
    cache: &Arc<Mutex<HashMap<String, sruja_scan::Graph>>>,
    repo_path: &str,
) -> Result<sruja_scan::Graph, CliError> {
    let mut cache = cache.lock().await;
    if let Some(g) = cache.get(repo_path) {
        return Ok(g.clone());
    }

    let g = crate::commands::scan_repo_cached(std::path::Path::new(repo_path))?;
    cache.insert(repo_path.to_string(), g.clone());
    Ok(g)
}

pub(crate) async fn add_element(
    repo: &str,
    id: &str,
    kind: &str,
    title: &str,
    description: Option<&str>,
    technology: Option<&str>,
) -> Result<(), CliError> {
    validate_ident(id, "id")?;
    validate_ident(kind, "kind")?;
    let target_file = find_best_sruja_file(repo)?;
    let mut content = tokio::fs::read_to_string(&target_file)
        .await
        .unwrap_or_default();

    if !content.ends_with('\n') && !content.is_empty() {
        content.push('\n');
    }

    content.push('\n');
    content.push_str(&format!(
        "{} = {} \"{}\"",
        id,
        kind,
        escape_dsl_string(title)?
    ));

    if description.is_some() || technology.is_some() {
        content.push_str(" {\n");
        if let Some(tech) = technology {
            content.push_str(&format!("  technology \"{}\"\n", escape_dsl_string(tech)?));
        }
        if let Some(desc) = description {
            content.push_str(&format!("  description \"{}\"\n", escape_dsl_string(desc)?));
        }
        content.push_str("}\n");
    } else {
        content.push('\n');
    }

    tokio::fs::write(&target_file, content).await?;
    Ok(())
}

pub(crate) async fn add_relationship(
    repo: &str,
    source: &str,
    target: &str,
    label: Option<&str>,
    technology: Option<&str>,
) -> Result<(), CliError> {
    validate_ident(source, "source")?;
    validate_ident(target, "target")?;
    let target_file = find_best_sruja_file(repo)?;
    let mut content = tokio::fs::read_to_string(&target_file)
        .await
        .unwrap_or_default();

    if !content.ends_with('\n') && !content.is_empty() {
        content.push('\n');
    }

    content.push('\n');
    let mut rel = format!("{} -> {}", source, target);
    if let Some(l) = label {
        rel.push_str(&format!(" \"{}\"", escape_dsl_string(l)?));
    }
    if let Some(t) = technology {
        rel.push_str(&format!(" [technology=\"{}\"]", escape_dsl_string(t)?));
    }
    rel.push('\n');
    content.push_str(&rel);

    tokio::fs::write(&target_file, content).await?;
    Ok(())
}

pub(crate) fn validate_ident(value: &str, field: &str) -> Result<(), CliError> {
    if value.is_empty() {
        return Err(CliError::validation(format!("Missing {}", field)));
    }
    if value.trim() != value {
        return Err(CliError::validation(format!(
            "Invalid {}: leading/trailing whitespace",
            field
        )));
    }
    if value
        .chars()
        .any(|c| c.is_whitespace() || c == '"' || c == '{' || c == '}' || c == '\\')
    {
        return Err(CliError::validation(format!(
            "Invalid {}: contains forbidden characters",
            field
        )));
    }
    Ok(())
}

pub(crate) fn escape_dsl_string(value: &str) -> Result<String, CliError> {
    if value.chars().any(|c| c == '\n' || c == '\r') {
        return Err(CliError::validation("Invalid string: contains newline"));
    }
    Ok(value.replace('\\', "\\\\").replace('"', "\\\""))
}

pub(crate) fn find_best_sruja_file(repo: &str) -> Result<String, CliError> {
    let path = std::path::Path::new(repo);
    let repo_sruja = path.join("repo.sruja");
    if repo_sruja.exists() {
        return Ok(repo_sruja.to_string_lossy().to_string());
    }

    let files = crate::modules::file_operations::collect_sruja_files(path)?;
    if let Some(first) = files.first() {
        return Ok(first.clone());
    }

    Ok(repo_sruja.to_string_lossy().to_string())
}

pub(crate) fn load_architecture_program_best_effort(
    repo_path: &Path,
) -> (
    Option<(String, sruja_language::ast::Program)>,
    Option<String>,
) {
    let Some(arch_path) = crate::utils::architecture_path::resolve_architecture_path(repo_path)
    else {
        return (None, None);
    };

    let file = arch_path.to_string_lossy().to_string();
    let Ok(content) = std::fs::read_to_string(&arch_path) else {
        return (None, Some(format!("Cannot read architecture file: {file}")));
    };

    let parser = sruja_language::Parser::new(file.clone());
    match parser.parse(&content) {
        Ok(program) => (Some((file, program)), None),
        Err(diags) => (
            None,
            Some(format!(
                "Failed to parse architecture file: {} error(s)",
                diags.len()
            )),
        ),
    }
}

pub(crate) fn estimate_tokens(text: &str) -> usize {
    crate::commands::context::types::TokenBudget::estimate_tokens(text)
}

pub(crate) fn kind_matches_filter(kind: &str, filter: &[String]) -> bool {
    let k = kind.trim().to_lowercase();
    filter.iter().any(|f| f.trim().to_lowercase() == k)
}

pub(crate) fn trim_text(value: Option<&str>, max_chars: usize) -> Option<String> {
    let v = value?;
    let s = v.trim();
    if s.is_empty() {
        return None;
    }
    if s.chars().count() <= max_chars {
        Some(s.to_string())
    } else {
        Some(s.chars().take(max_chars).collect::<String>())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedId {
    pub(crate) id: String,
    pub(crate) ambiguous_matches: Vec<String>,
}

pub(crate) fn resolve_id_best_effort(id: &str, all_ids_sorted: &[String]) -> ResolvedId {
    let needle = id.trim();
    if needle.is_empty() {
        return ResolvedId {
            id: needle.to_string(),
            ambiguous_matches: Vec::new(),
        };
    }
    if all_ids_sorted.iter().any(|x| x == needle) {
        return ResolvedId {
            id: needle.to_string(),
            ambiguous_matches: Vec::new(),
        };
    }
    let suffix = format!(".{needle}");
    let matches: Vec<String> = all_ids_sorted
        .iter()
        .filter(|x| x.ends_with(&suffix))
        .cloned()
        .collect();
    match matches.len() {
        0 => ResolvedId {
            id: needle.to_string(),
            ambiguous_matches: Vec::new(),
        },
        1 => ResolvedId {
            id: matches[0].clone(),
            ambiguous_matches: Vec::new(),
        },
        _ => {
            let chosen = matches
                .iter()
                .min()
                .cloned()
                .unwrap_or_else(|| needle.to_string());
            ResolvedId {
                id: chosen,
                ambiguous_matches: matches,
            }
        }
    }
}

pub(crate) fn push_resolution_warnings(
    warnings: &mut Vec<String>,
    requested: &str,
    resolved: &ResolvedId,
) {
    if resolved.ambiguous_matches.len() > 1 {
        warnings.push(format!(
            "Ambiguous element id {requested:?}: suffix matched {:?}; using {:?}",
            resolved.ambiguous_matches, resolved.id
        ));
    }
}
