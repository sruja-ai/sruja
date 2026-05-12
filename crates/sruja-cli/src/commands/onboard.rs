//! Onboarding command: produce a single high-signal architecture brief.
//!
//! Design goals:
//! - One command for humans + AI agents: `sruja onboard -r .`
//! - Deterministic output (stable ordering)
//! - Clear trust signals: truth status, drift counts, context score
//! - CI-friendly format: GitHub Actions annotations

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::commands::CliError;
use crate::commands::LlmConfig;
use crate::context_detection::build_repo_context;
use crate::integrations::{
    resolve_enrichment_plan, resolve_openai_auth, run_cmd_enrichment, run_openai_markdown,
    EnrichmentLimits,
};
use crate::utils::{architecture_path, context as context_utils};

use sruja_scan::{graph::compute_all_centrality, EdgeKind, Graph, NodeKind};

#[derive(Debug, Clone, serde::Serialize)]
pub struct OnboardOutput {
    pub repo: String,
    pub truth_status: String,
    pub drift: OnboardDrift,
    pub context_score: u8,
    pub context: OnboardRepoContext,
    pub entrypoints: Vec<OnboardEntrypoint>,
    pub data_stores: Vec<OnboardDataStore>,
    pub key_elements: Vec<OnboardKeyElement>,
    pub key_relationships: Vec<OnboardKeyRelationship>,
    pub suggested_next_files: Vec<String>,
    pub suggested_commands: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrichment: Option<OnboardEnrichment>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OnboardEnrichment {
    /// "ok" | "skipped" | "error"
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub narrative_markdown: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OnboardRepoContext {
    pub primary_language: String,
    pub framework: Option<String>,
    pub architecture_style: String,
    pub domain: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OnboardDrift {
    pub health_score: u8,
    pub violations_total: usize,
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OnboardEntrypoint {
    pub id: String,
    pub kind: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OnboardDataStore {
    pub id: String,
    pub kind: String,
    pub technology: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OnboardKeyElement {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub path: Option<String>,
    pub pagerank: f64,
    pub why_it_matters: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OnboardKeyRelationship {
    pub source: String,
    pub target: String,
    pub kind: String,
    pub why_it_matters: String,
}

// Clap flattens many flags at the CLI boundary; options are passed through to enrichment helpers.
#[allow(clippy::too_many_arguments)]
pub async fn onboard(
    repo_root: &str,
    format: &str,
    max_items: usize,
    enrich: bool,
    enrich_provider: Option<&str>,
    enrich_cmd: Option<&str>,
    enrich_model: Option<&str>,
    enrich_base_url: Option<&str>,
    enrich_timeout_ms: u64,
    enrich_max_bytes: usize,
    llm: LlmConfig<'_>,
    output: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = super::scan_repo_cached(repo_path)?;

    // Truth status: if baseline exists, compare scan vs DSL; else unknown.
    let baseline_path = architecture_path::resolve_architecture_path(repo_path);
    let truth_status = if let Some(ref p) = baseline_path {
        super::scan::drift::truth_status_from_baseline_compare(&graph, p)
            .ok()
            .map(|s| match s {
                sruja_diff::TruthStatus::Reviewed => "reviewed",
                sruja_diff::TruthStatus::Drifted => "drifted",
                sruja_diff::TruthStatus::Unknown => "unknown",
            })
            .unwrap_or("unknown")
            .to_string()
    } else {
        "unknown".to_string()
    };

    let drift_report = sruja_diff::detect_architectural_drift(&graph);
    let (errors, warnings, info) = drift_counts(&drift_report.violations);

    // Context score (AI readiness / trust signal)
    let kg = crate::graph_store::load_or_build_graph(repo_path)?;
    let age_hours = context_utils::context_age_hours(repo_path);
    let context_score =
        sruja_graph::compute_context_score(&kg, graph.nodes.len(), repo_path, age_hours).score;

    // Repo context
    let ctx = build_repo_context(repo_path, &graph);
    let architecture_style = if ctx.is_microservices {
        "microservices"
    } else if ctx.is_monolith {
        "monolith"
    } else {
        "mixed/unclear"
    }
    .to_string();

    let max_items = max_items.clamp(3, 30);
    let entrypoints = discover_entrypoints(repo_path, &graph, max_items);
    let data_stores = discover_data_stores(repo_path, &graph, max_items);
    let (key_elements, suggested_next_files) =
        discover_key_elements_and_files(repo_path, &graph, max_items);
    let key_relationships = discover_key_relationships(&graph, max_items);
    let suggested_commands = suggested_commands(repo_path);

    let mut out = OnboardOutput {
        repo: repo_root.to_string(),
        truth_status,
        drift: OnboardDrift {
            health_score: drift_report.health_score,
            violations_total: drift_report.violations.len(),
            errors,
            warnings,
            info,
        },
        context_score,
        context: OnboardRepoContext {
            primary_language: ctx.primary_language.clone(),
            framework: ctx.framework.clone(),
            architecture_style,
            domain: ctx.domain.clone(),
        },
        entrypoints,
        data_stores,
        key_elements,
        key_relationships,
        suggested_next_files,
        suggested_commands,
        enrichment: None,
    };

    if enrich || enrich_cmd.is_some() {
        let plan = resolve_enrichment_plan(
            repo_path,
            enrich_cmd,
            enrich_model,
            enrich_base_url,
            Some(enrich_timeout_ms),
            Some(enrich_max_bytes),
        );
        let provider = enrich_provider.unwrap_or(plan.provider.as_str());
        out.enrichment = Some(enrich_onboard(
            &out,
            provider,
            plan.cmd.as_deref(),
            plan.model.as_deref(),
            plan.base_url.as_deref(),
            plan.limits,
            llm,
        ));
    }

    let rendered = match format {
        "json" => {
            serde_json::to_string_pretty(&out).map_err(|e| CliError::validation(e.to_string()))?
        }
        "github" | "github-actions" => format_onboard_github_actions(&out),
        _ => format_onboard_markdown(&out),
    };

    if let Some(p) = output {
        std::fs::write(p, rendered)?;
        return Ok(());
    }

    println!("{}", rendered);
    Ok(())
}

fn enrich_onboard(
    out: &OnboardOutput,
    provider: &str,
    enrich_cmd: Option<&str>,
    enrich_model: Option<&str>,
    enrich_base_url: Option<&str>,
    limits: EnrichmentLimits,
    llm: LlmConfig<'_>,
) -> OnboardEnrichment {
    if provider == "cmd" {
        let Some(cmd) = enrich_cmd else {
            return OnboardEnrichment {
                status: "skipped".to_string(),
                provider: Some("external_cmd".to_string()),
                model: None,
                error: Some("No command configured. Pass --enrich-cmd or set SRUJA_ENRICH_CMD (or .sruja/config.toml [integrations].cmd).".to_string()),
                narrative_markdown: None,
            };
        };
        return enrich_onboard_via_command(out, cmd, limits);
    }

    if provider != "openai" {
        return OnboardEnrichment {
            status: "skipped".to_string(),
            provider: Some(provider.to_string()),
            model: None,
            error: Some(
                "Unsupported provider. Use provider=cmd (recommended) or provider=openai."
                    .to_string(),
            ),
            narrative_markdown: None,
        };
    }

    let provider = llm
        .provider
        .map(|s| s.to_string())
        .or_else(|| std::env::var("SRUJA_ENRICH_PROVIDER").ok())
        .or_else(|| std::env::var("SRUJA_LLM_PROVIDER").ok()) // back-compat
        .unwrap_or_else(|| "openai".to_string());
    let model = llm
        .model
        .map(|s| s.to_string())
        .or_else(|| enrich_model.map(|s| s.to_string()))
        .or_else(|| std::env::var("SRUJA_ENRICH_MODEL").ok())
        .or_else(|| std::env::var("SRUJA_LLM_MODEL").ok()) // back-compat
        .unwrap_or_else(|| "gpt-4o-mini".to_string());
    let base_url = llm
        .base_url
        .map(|s| s.to_string())
        .or_else(|| enrich_base_url.map(|s| s.to_string()))
        .or_else(|| std::env::var("SRUJA_ENRICH_BASE_URL").ok())
        .or_else(|| std::env::var("SRUJA_LLM_BASE_URL").ok()) // back-compat
        .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

    // Default posture: do not fail the command if enrichment cannot run.
    let api_key = resolve_openai_auth();

    let Some(key) = api_key else {
        return OnboardEnrichment {
            status: "skipped".to_string(),
            provider: Some(provider),
            model: Some(model),
            error: Some("Missing API key (set OPENAI_API_KEY or SRUJA_ENRICH_API_KEY; SRUJA_LLM_API_KEY is deprecated).".to_string()),
            narrative_markdown: None,
        };
    };

    // provider must be openai here

    let payload = match serde_json::to_value(out) {
        Ok(v) => v,
        Err(e) => {
            return OnboardEnrichment {
                status: "error".to_string(),
                provider: Some(provider),
                model: Some(model),
                error: Some(format!("Failed to serialize onboard JSON: {e}")),
                narrative_markdown: None,
            }
        }
    };

    let prompt = format!(
        r#"You are assisting developers onboarding to a repo.

You MUST only use the JSON facts provided below. Do not invent endpoints, technologies, or file paths. If something is unknown, say "unknown".

Produce markdown with these sections:
- "What this repo likely is"
- "Key runtime boundaries (based on relationships)"
- "Risks / unknowns to verify" (bullets)
- "Suggested onboarding path" (ordered steps referencing specific files/IDs if present)
- "Questions to ask maintainers" (bullets)

JSON facts:
{}"#,
        payload
    );

    match run_openai_markdown(
        "You are a careful architecture analyst. Never fabricate.",
        &prompt,
        &model,
        &base_url,
        &key,
    ) {
        Ok(md) => OnboardEnrichment {
            status: "ok".to_string(),
            provider: Some(provider),
            model: Some(model),
            error: None,
            narrative_markdown: Some(md),
        },
        Err(e) => OnboardEnrichment {
            status: "error".to_string(),
            provider: Some(provider),
            model: Some(model),
            error: Some(e),
            narrative_markdown: None,
        },
    }
}

fn enrich_onboard_via_command(
    out: &OnboardOutput,
    cmd: &str,
    limits: EnrichmentLimits,
) -> OnboardEnrichment {
    // Security posture: this is opt-in and runs a local command.
    // We pass only the grounded onboard JSON.
    let payload = match serde_json::to_vec(out) {
        Ok(v) => v,
        Err(e) => {
            return OnboardEnrichment {
                status: "error".to_string(),
                provider: Some("external_cmd".to_string()),
                model: None,
                error: Some(format!("Failed to serialize onboard JSON: {e}")),
                narrative_markdown: None,
            }
        }
    };

    match run_cmd_enrichment(cmd, &payload, limits) {
        Ok(narrative) => OnboardEnrichment {
            status: "ok".to_string(),
            provider: Some("external_cmd".to_string()),
            model: None,
            error: None,
            narrative_markdown: Some(narrative),
        },
        Err(e) => OnboardEnrichment {
            status: "error".to_string(),
            provider: Some("external_cmd".to_string()),
            model: None,
            error: Some(e),
            narrative_markdown: None,
        },
    }
}

fn drift_counts(violations: &[sruja_diff::Violation]) -> (usize, usize, usize) {
    let mut errors = 0usize;
    let mut warnings = 0usize;
    let mut info = 0usize;
    for v in violations {
        match v.severity {
            sruja_diff::Severity::Error => errors += 1,
            sruja_diff::Severity::Warning => warnings += 1,
            sruja_diff::Severity::Info => info += 1,
        }
    }
    (errors, warnings, info)
}

fn relativize_path(repo_root: &Path, path: &str) -> Option<String> {
    let normalized = path.replace('\\', "/");
    let prefix = repo_root
        .canonicalize()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.replace('\\', "/")))?;
    Some(
        normalized
            .strip_prefix(&format!("{}/", prefix))
            .or_else(|| normalized.strip_prefix(&prefix))
            .unwrap_or(normalized.as_str())
            .trim_start_matches('/')
            .trim_start_matches("./")
            .to_string(),
    )
}

fn discover_entrypoints(
    repo_root: &Path,
    graph: &Graph,
    max_items: usize,
) -> Vec<OnboardEntrypoint> {
    let mut has_incoming: HashMap<&str, usize> = HashMap::new();
    for edge in &graph.edges {
        *has_incoming.entry(edge.target.as_str()).or_default() += 1;
    }

    let mut entries: Vec<&sruja_scan::Node> = graph
        .nodes
        .iter()
        .filter(|n| !n.id.contains('#'))
        .filter(|node| {
            let is_high_level = matches!(
                node.kind,
                NodeKind::Service | NodeKind::ExternalApi | NodeKind::System | NodeKind::Frontend
            );
            let no_incoming = has_incoming.get(node.id.as_str()).copied().unwrap_or(0) == 0;
            is_high_level || no_incoming
        })
        .collect();

    entries.sort_by(|a, b| a.id.cmp(&b.id));
    entries.truncate(max_items);

    entries
        .into_iter()
        .map(|n| OnboardEntrypoint {
            id: n.id.clone(),
            kind: n.kind.as_str().to_string(),
            path: n
                .path
                .as_deref()
                .and_then(|p| relativize_path(repo_root, p)),
        })
        .collect()
}

fn discover_data_stores(
    repo_root: &Path,
    graph: &Graph,
    max_items: usize,
) -> Vec<OnboardDataStore> {
    let mut stores: Vec<&sruja_scan::Node> = graph
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Database | NodeKind::Queue))
        .collect();
    stores.sort_by(|a, b| a.id.cmp(&b.id));
    stores.truncate(max_items);
    stores
        .into_iter()
        .map(|n| OnboardDataStore {
            id: n.id.clone(),
            kind: n.kind.as_str().to_string(),
            technology: n.technology.clone(),
            path: n
                .path
                .as_deref()
                .and_then(|p| relativize_path(repo_root, p)),
        })
        .collect()
}

fn discover_key_elements_and_files(
    repo_root: &Path,
    graph: &Graph,
    max_items: usize,
) -> (Vec<OnboardKeyElement>, Vec<String>) {
    let centrality = compute_all_centrality(graph);

    let mut incoming: HashMap<&str, usize> = HashMap::new();
    let mut outgoing: HashMap<&str, usize> = HashMap::new();
    for edge in &graph.edges {
        *incoming.entry(edge.target.as_str()).or_default() += 1;
        *outgoing.entry(edge.source.as_str()).or_default() += 1;
    }

    let mut nodes: Vec<&sruja_scan::Node> =
        graph.nodes.iter().filter(|n| !n.id.contains('#')).collect();
    nodes.sort_by(|a, b| {
        let ap = centrality.get(&a.id).map(|s| s.pagerank).unwrap_or(0.0);
        let bp = centrality.get(&b.id).map(|s| s.pagerank).unwrap_or(0.0);
        bp.total_cmp(&ap).then_with(|| a.id.cmp(&b.id))
    });

    let mut key = Vec::new();
    let mut files = Vec::new();
    let mut seen_files = HashSet::new();

    for node in nodes.into_iter().take(max_items) {
        let pr = centrality.get(&node.id).map(|s| s.pagerank).unwrap_or(0.0);
        let inc = incoming.get(node.id.as_str()).copied().unwrap_or(0);
        let out = outgoing.get(node.id.as_str()).copied().unwrap_or(0);
        let why = if matches!(node.kind, NodeKind::Database) {
            format!("Data boundary referenced by {} upstream component(s).", inc)
        } else if inc > 0 && out > 0 {
            format!(
                "Central coordinator: {} incoming, {} outgoing dependency(ies).",
                inc, out
            )
        } else if inc > 0 {
            format!(
                "Shared dependency referenced by {} upstream component(s).",
                inc
            )
        } else if out > 0 {
            format!(
                "Caller/entry candidate with {} outgoing dependency(ies).",
                out
            )
        } else {
            "Standalone node with low connectivity; review if it’s a clean boundary or missing edges."
                .to_string()
        };

        let rel_path = node
            .path
            .as_deref()
            .and_then(|p| relativize_path(repo_root, p));
        key.push(OnboardKeyElement {
            id: node.id.clone(),
            label: node.label.clone(),
            kind: node.kind.as_str().to_string(),
            path: rel_path.clone(),
            pagerank: pr,
            why_it_matters: why,
        });

        if let Some(rel) = rel_path {
            if seen_files.insert(rel.clone()) {
                files.push(rel);
            }
        }
    }

    files.truncate(max_items);
    (key, files)
}

fn discover_key_relationships(graph: &Graph, max_items: usize) -> Vec<OnboardKeyRelationship> {
    let centrality = compute_all_centrality(graph);
    let kind_by_id: HashMap<&str, NodeKind> = graph
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n.kind.clone()))
        .collect();

    let mut edges: Vec<&sruja_scan::Edge> = graph
        .edges
        .iter()
        .filter(|e| !matches!(e.kind, EdgeKind::Contains | EdgeKind::Owns))
        .filter(|e| !e.source.contains('#') && !e.target.contains('#'))
        .collect();

    edges.sort_by(|a, b| {
        let a_score = centrality.get(&a.source).map(|s| s.pagerank).unwrap_or(0.0)
            + centrality.get(&a.target).map(|s| s.pagerank).unwrap_or(0.0);
        let b_score = centrality.get(&b.source).map(|s| s.pagerank).unwrap_or(0.0)
            + centrality.get(&b.target).map(|s| s.pagerank).unwrap_or(0.0);
        b_score
            .total_cmp(&a_score)
            .then_with(|| a.source.cmp(&b.source))
            .then_with(|| a.target.cmp(&b.target))
            .then_with(|| a.kind.as_str().cmp(b.kind.as_str()))
    });

    edges.truncate(max_items);
    edges
        .into_iter()
        .map(|e| {
            let target_kind = kind_by_id.get(e.target.as_str()).cloned();
            let why = match target_kind {
                Some(NodeKind::Database) => {
                    "Service/data dependency: verify it stays intentional.".to_string()
                }
                Some(NodeKind::ExternalApi) => {
                    "External boundary: changes here often require coordination.".to_string()
                }
                Some(NodeKind::Service) | Some(NodeKind::Frontend) => {
                    "High-signal runtime/user-facing relationship.".to_string()
                }
                _ if matches!(
                    e.kind,
                    EdgeKind::Calls | EdgeKind::DependsOn | EdgeKind::Uses
                ) =>
                {
                    "Meaningful internal dependency worth checking as a boundary.".to_string()
                }
                _ => "Structurally important relationship in the scanned graph.".to_string(),
            };
            OnboardKeyRelationship {
                source: e.source.clone(),
                target: e.target.clone(),
                kind: e.kind.as_str().to_string(),
                why_it_matters: why,
            }
        })
        .collect()
}

fn suggested_commands(repo_path: &Path) -> Vec<String> {
    let mut commands = Vec::new();
    if repo_path.join("justfile").exists() {
        commands.push("just check".to_string());
    } else if repo_path.join("Makefile").exists() {
        commands.push("make check".to_string());
    }
    commands.push("sruja quickstart -r .".to_string());
    commands.push("sruja discover explain -r .".to_string());
    commands.push("sruja context-score -r .".to_string());
    commands.push("sruja drift -r .".to_string());
    commands.push("sruja check -r .".to_string());
    commands.sort();
    commands.dedup();
    commands
}

fn format_onboard_markdown(out: &OnboardOutput) -> String {
    let mut md = String::new();
    md.push_str("# Sruja Onboarding Brief\n\n");
    md.push_str(&format!("- Repo: `{}`\n", out.repo));
    md.push_str(&format!("- Truth: `{}`\n", out.truth_status));
    md.push_str(&format!(
        "- Drift: `{}/100` (violations: {}, errors: {}, warnings: {}, info: {})\n",
        out.drift.health_score,
        out.drift.violations_total,
        out.drift.errors,
        out.drift.warnings,
        out.drift.info
    ));
    md.push_str(&format!("- Context score: `{}/100`\n\n", out.context_score));

    md.push_str("## Repo Context\n\n");
    md.push_str(&format!(
        "- Primary language: `{}`\n",
        out.context.primary_language
    ));
    if let Some(ref fw) = out.context.framework {
        md.push_str(&format!("- Framework: `{}`\n", fw));
    }
    md.push_str(&format!(
        "- Architecture style: `{}`\n",
        out.context.architecture_style
    ));
    if let Some(ref domain) = out.context.domain {
        md.push_str(&format!("- Domain hint: `{}`\n", domain));
    }
    md.push('\n');

    md.push_str("## Entrypoints\n\n");
    if out.entrypoints.is_empty() {
        md.push_str("- None detected.\n\n");
    } else {
        for e in &out.entrypoints {
            let path = e
                .path
                .as_deref()
                .map(|p| format!(" ({})", p))
                .unwrap_or_default();
            md.push_str(&format!("- `{}` [{}]{}\n", e.id, e.kind, path));
        }
        md.push('\n');
    }

    md.push_str("## Data Stores\n\n");
    if out.data_stores.is_empty() {
        md.push_str("- None detected.\n\n");
    } else {
        for s in &out.data_stores {
            let tech = s
                .technology
                .as_deref()
                .map(|t| format!(" [{}]", t))
                .unwrap_or_default();
            let path = s
                .path
                .as_deref()
                .map(|p| format!(" ({})", p))
                .unwrap_or_default();
            md.push_str(&format!("- `{}` [{}]{}{}\n", s.id, s.kind, tech, path));
        }
        md.push('\n');
    }

    md.push_str("## High-Signal Elements\n\n");
    for el in &out.key_elements {
        let path = el
            .path
            .as_deref()
            .map(|p| format!(" ({})", p))
            .unwrap_or_default();
        md.push_str(&format!(
            "- `{}` [{}]{path} (pr={:.3})\n  - {}\n",
            el.id, el.kind, el.pagerank, el.why_it_matters
        ));
    }
    md.push('\n');

    md.push_str("## Key Relationships\n\n");
    for rel in &out.key_relationships {
        md.push_str(&format!(
            "- `{}` -> `{}` [{}]\n  - {}\n",
            rel.source, rel.target, rel.kind, rel.why_it_matters
        ));
    }
    md.push('\n');

    md.push_str("## Suggested Next Files\n\n");
    if out.suggested_next_files.is_empty() {
        md.push_str("- None\n\n");
    } else {
        for f in &out.suggested_next_files {
            md.push_str(&format!("- `{}`\n", f));
        }
        md.push('\n');
    }

    md.push_str("## Suggested Commands\n\n");
    for c in &out.suggested_commands {
        md.push_str(&format!("- `{}`\n", c));
    }

    if let Some(ref e) = out.enrichment {
        md.push_str("\n## LLM Enrichment (opt-in)\n\n");
        md.push_str(
            "- This section is **LLM-generated** and must be treated as **interpretation**, not truth.\n",
        );
        md.push_str(
            "- It is grounded in the JSON facts above; if it contradicts them, prefer the grounded scan output.\n\n",
        );
        md.push_str(&format!("- Status: `{}`\n", e.status));
        if let Some(ref p) = e.provider {
            md.push_str(&format!("- Provider: `{}`\n", p));
        }
        if let Some(ref m) = e.model {
            md.push_str(&format!("- Model: `{}`\n", m));
        }
        if let Some(ref err) = e.error {
            md.push_str(&format!("- Error: `{}`\n", err));
        }
        md.push('\n');
        if let Some(ref narrative) = e.narrative_markdown {
            md.push_str(narrative);
            md.push('\n');
        }
    }
    md
}

fn escape_github_actions_message(input: &str) -> String {
    input
        .replace('%', "%25")
        .replace('\n', "%0A")
        .replace('\r', "%0D")
}

fn format_onboard_github_actions(out: &OnboardOutput) -> String {
    // Emit a single summary notice + a few informational notices.
    // This keeps CI output readable while still being “clickable”.
    let file = "repo.sruja";
    let title = "Sruja Onboard";
    let msg = format!(
        "Truth: {}. Drift: {}/100 ({} violations). Context: {}/100.",
        out.truth_status, out.drift.health_score, out.drift.violations_total, out.context_score
    );
    let mut lines = Vec::new();
    lines.push(format!(
        "::notice file={}::title={}::{}",
        file,
        escape_github_actions_message(title),
        escape_github_actions_message(&msg)
    ));

    for e in out.entrypoints.iter().take(5) {
        let detail = format!("Entrypoint: {} ({})", e.id, e.kind);
        lines.push(format!(
            "::notice file={}::title={}::{}",
            file,
            escape_github_actions_message("Sruja Entrypoint"),
            escape_github_actions_message(&detail)
        ));
    }

    for s in out.data_stores.iter().take(5) {
        let tech = s.technology.as_deref().unwrap_or("unknown");
        let detail = format!("Data store: {} ({}, tech={})", s.id, s.kind, tech);
        lines.push(format!(
            "::notice file={}::title={}::{}",
            file,
            escape_github_actions_message("Sruja Data Store"),
            escape_github_actions_message(&detail)
        ));
    }

    lines.join("\n")
}
