//! Focus command: file-scoped or element-scoped context briefing for AI agents.
//!
//! Answers: "I'm about to edit X. What does my AI agent need to know?"
//! Combines impact analysis, decisions, boundaries, hotspot status, and
//! external context into a single, actionable briefing.

use colored::Colorize;
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use crate::commands::context::types::TokenBudget;
use crate::commands::CliError;
use crate::graph_store;
use crate::integrations::{
    resolve_enrichment_plan, resolve_openai_auth, run_cmd_enrichment, run_openai_markdown,
};
use crate::utils::colors;
use crate::utils::run_id::generate_run_id;
use crate::utils::run_snapshots::write_json_snapshot;
use sruja_agent::{AgenticMemory, ExperimentOutcome, MemoryError};
use sruja_graph::{compute_context_score, KnowledgeGraph, ReasonedWhyStep};

const FOCUS_FOR_AI_SCHEMA_VERSION: &str = "focus_for_ai/v1";

#[derive(Debug, Clone, Serialize)]
pub struct MemoryHit {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hitl_kind: Option<String>,
    pub outcome: String,
    pub match_reason: String,
    pub timestamp: String,
    pub hypothesis: String,
    pub guardrail_advice: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TemporalContextBrief {
    pub base_ref: String,
    pub head_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture_relative_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_fingerprint_head: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_fingerprint_base: Option<String>,
    pub diff_mapped_component_ids: Vec<String>,
    pub touches_focus_target: bool,
}

#[derive(Debug, Serialize)]
pub struct FocusBriefing {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    pub target: FocusTarget,
    pub blast_radius: BlastRadius,
    pub reasoned_traces: Vec<ReasonedTrace>,
    pub decisions: Vec<LinkedDecision>,
    pub boundaries: Vec<BoundaryInfo>,
    pub external_context: Vec<ExternalContextRef>,
    pub hotspot_status: HotspotStatus,
    pub ai_instructions: Vec<String>,
    pub anti_patterns: Vec<String>,
    pub pointer_traces: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub memory_hits: Vec<MemoryHit>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub memory_truncated: bool,
    pub context_score: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporal: Option<TemporalContextBrief>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrichment: Option<FocusEnrichment>,
    /// Recent decision/workflow lineage from `.sruja/context_events.jsonl` (v2 kinds) for this element.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decision_trace_events: Vec<crate::commands::context_events::ContextEventRecord>,
    /// On-disk Decision Records (`.sruja/decisions/`) whose `elements` include this focus target.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decision_records: Vec<crate::commands::decision::DecisionListItem>,
    /// Learnings actually injected into this briefing (subset of `find_relevant`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub surfaced_learning_ids: Vec<String>,
}

/// Learnings surfaced for a focus target (token-budget capped), with optional retrieval accounting.
#[derive(Debug, Clone)]
pub struct SurfacedLearnings {
    pub hits: Vec<MemoryHit>,
    pub ids: Vec<String>,
    pub truncated: bool,
    pub anti_patterns: Vec<String>,
    pub pointer_traces: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ReasonedTrace {
    pub node_id: String,
    pub node_label: String,
    pub direction: String,
    pub reasoning: String,
    pub decision_ref: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FocusForAiOutput {
    /// `deterministic_brief` — grounded scan/graph facts; optional enrichment is separate.
    pub artifact_kind: String,
    pub schema_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    pub repo: String,
    pub target: FocusForAiTarget,
    pub briefing: FocusBriefing,
    pub suggested_next_steps: Vec<SuggestedCommand>,
}

#[derive(Debug, Serialize)]
pub struct FocusForAiTarget {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,
    pub resolved_element_id: String,
}

#[derive(Debug, Serialize)]
pub struct SuggestedCommand {
    pub purpose: String,
    pub argv: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FocusEnrichment {
    pub status: String,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub narrative_markdown: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FocusTarget {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub technology: Option<String>,
    pub system: Option<String>,
    pub gotchas: Vec<String>,
    pub operational_constraints: Vec<String>,
    pub runbooks: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BlastRadius {
    pub total_affected: usize,
    pub upstream: Vec<AffectedNode>,
    pub downstream: Vec<AffectedNode>,
}

#[derive(Debug, Serialize)]
pub struct AffectedNode {
    pub id: String,
    pub label: String,
    pub depth: usize,
    pub relationship: String,
}

#[derive(Debug, Serialize)]
pub struct LinkedDecision {
    pub id: String,
    pub title: String,
    pub status: String,
    pub summary: String,
}

#[derive(Debug, Serialize)]
pub struct BoundaryInfo {
    pub from: String,
    pub to: String,
    pub allowed: bool,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct ExternalContextRef {
    pub file: String,
    pub category: String,
    pub excerpt: String,
}

#[derive(Debug, Serialize)]
pub struct HotspotStatus {
    pub is_hotspot: bool,
    pub role: String,
    pub in_degree: usize,
    pub out_degree: usize,
}

/// Resolve a target ID from a file path or element ID.
pub fn resolve_target(
    graph: &KnowledgeGraph,
    repo_path: &Path,
    file: Option<&str>,
    element_id: Option<&str>,
) -> Result<String, CliError> {
    // Direct element ID match
    if let Some(eid) = element_id {
        if graph.nodes.contains_key(eid) {
            return Ok(eid.to_string());
        }
        // Match by suffix (aligns with `context` element_id resolution).
        let suffix = format!(".{}", eid);
        let mut matches: Vec<&str> = graph
            .nodes
            .keys()
            .filter(|k| *k == eid || k.ends_with(&suffix))
            .map(|k| k.as_str())
            .collect();
        matches.sort_unstable();
        matches.dedup();
        match matches.len() {
            0 => Err(CliError::validation(format!(
                "No architecture element matches '{}'. Run 'sruja list repo.sruja' to see available elements.",
                eid
            ))),
            1 => Ok(matches[0].to_string()),
            _ => {
                let preview: Vec<&str> = matches.iter().take(5).copied().collect();
                Err(CliError::validation(format!(
                    "Ambiguous element '{}'. Matches: {}",
                    eid,
                    preview.join(", ")
                )))
            }
        }?;
    }

    // File path match — find nodes whose metadata, label, or source ref mention the file
    if let Some(file_path) = file {
        // Delegate to the scan-based focus matcher (aligns with `context --file` ordering).
        let scan = sruja_scan::scan_repo(repo_path).map_err(|e| {
            CliError::validation(format!(
                "Failed to scan repo for file focus resolution: {e}"
            ))
        })?;
        let centrality = sruja_scan::graph::compute_all_centrality(&scan);
        let focus_ctx = crate::commands::context::logic::build_focus_context(
            &scan,
            repo_path.to_string_lossy().as_ref(),
            file_path,
            None,
            0,
            0,
            &centrality,
        )?;
        if let Some(first) = focus_ctx.matched_nodes.first() {
            return Ok(first.id.clone());
        }

        return Err(CliError::validation(format!(
            "Could not resolve file '{}' to an architecture element. Try --element-id instead, or ensure your .sruja maps this file.",
            file_path
        )));
    }

    Err(CliError::validation(
        "Provide --file or --element-id to focus on a specific part of the architecture."
            .to_string(),
    ))
}

fn git_arch_blob_blake3(repo: &Path, git_ref: &str, path_in_repo: &str) -> Option<String> {
    let spec = format!("{git_ref}:{path_in_repo}");
    let out = Command::new("git")
        .args(["show", spec.as_str()])
        .current_dir(repo)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(blake3::hash(&out.stdout).to_hex().to_string())
}

/// Git-range snapshot: diff-mapped components and optional declared-architecture fingerprints at base vs working tree.
pub fn load_temporal_context(
    repo_path: &Path,
    base_ref: &str,
    head_ref: &str,
    target_id: &str,
) -> Result<TemporalContextBrief, CliError> {
    let arch_path = crate::utils::architecture_path::resolve_architecture_path(repo_path);
    let rel = arch_path.as_ref().and_then(|p| {
        p.strip_prefix(repo_path).ok().map(|r| {
            r.components()
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join("/")
        })
    });
    let head_fp = crate::commands::context_events::policy_fingerprint(repo_path);
    let base_fp = rel
        .as_deref()
        .and_then(|r| git_arch_blob_blake3(repo_path, base_ref, r));
    let scan = sruja_scan::scan_repo(repo_path).map_err(|e| CliError::validation(e.to_string()))?;
    let diffs = sruja_diff::map_git_diff(repo_path, base_ref, head_ref, &scan).map_err(|e| {
        CliError::validation(format!(
            "Git diff mapping failed (are '{base_ref}' and '{head_ref}' valid refs?): {e}"
        ))
    })?;
    let mut ids: Vec<String> = diffs.iter().map(|d| d.component_id.clone()).collect();
    ids.sort();
    ids.dedup();
    let touches = ids.iter().any(|id| {
        id == target_id
            || target_id.starts_with(&format!("{id}."))
            || id.starts_with(&format!("{target_id}."))
    });
    Ok(TemporalContextBrief {
        base_ref: base_ref.to_string(),
        head_ref: head_ref.to_string(),
        architecture_relative_path: rel,
        policy_fingerprint_head: head_fp,
        policy_fingerprint_base: base_fp,
        diff_mapped_component_ids: ids,
        touches_focus_target: touches,
    })
}

/// Surface agentic learnings for a target (same cap as focus briefing). When
/// `record_retrievals` is true, increments counters and persists memory (standalone focus / MCP).
pub fn surface_agent_learnings(
    repo_path: &Path,
    target_id: &str,
    record_retrievals: bool,
) -> Result<SurfacedLearnings, MemoryError> {
    let mut memory = AgenticMemory::load(repo_path)?;
    let mut relevant = memory.find_relevant(target_id);
    relevant.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| a.id.cmp(&b.id)));

    let ce_cfg = crate::integrations::load_repo_config(repo_path)
        .map(|c| c.context_engineering)
        .unwrap_or_default();
    let max_items = ce_cfg.bm25_max_results_focus.unwrap_or(10).max(1);

    let mut budget = TokenBudget::new(800);
    let mut hits = Vec::new();
    let mut ids = Vec::new();
    let mut anti_patterns = Vec::new();
    let mut pointer_traces = Vec::new();
    let mut truncated = false;

    for entry in relevant.into_iter().take(max_items) {
        let match_reason = if entry.affected_elements.iter().any(|e| {
            e == target_id
                || target_id.starts_with(&format!("{}.", e))
                || e.starts_with(&format!("{}.", target_id))
        }) {
            "affected_elements"
        } else if entry
            .context
            .to_lowercase()
            .contains(&target_id.to_lowercase())
        {
            "context_keyword"
        } else {
            "unknown"
        };

        let hit_str = format!(
            "{} {} {} {}",
            entry.id,
            entry.hypothesis,
            entry.guardrail_advice,
            entry.reason.clone().unwrap_or_default()
        );
        if budget
            .used_tokens
            .saturating_add(TokenBudget::estimate_tokens(&hit_str))
            > budget.max_tokens
        {
            truncated = true;
            break;
        }
        budget.used_tokens = budget
            .used_tokens
            .saturating_add(TokenBudget::estimate_tokens(&hit_str));

        let outcome = match entry.outcome {
            ExperimentOutcome::Success => "success",
            ExperimentOutcome::Failed => "failed",
        }
        .to_string();
        let kind = entry.kind.map(|k| format!("{k:?}").to_lowercase());

        ids.push(entry.id.clone());
        hits.push(MemoryHit {
            id: entry.id.clone(),
            kind,
            hitl_kind: entry.hitl_kind.clone(),
            outcome,
            match_reason: match_reason.to_string(),
            timestamp: entry.timestamp.to_rfc3339(),
            hypothesis: entry.hypothesis.clone(),
            guardrail_advice: entry.guardrail_advice.clone(),
        });

        anti_patterns.push(entry.guardrail_advice.clone());
        if let Some(reason) = &entry.reason {
            pointer_traces.push(format!(
                "Failed hypothesis: {} ({})",
                entry.hypothesis, reason
            ));
        } else {
            pointer_traces.push(format!("Prior learning: {}", entry.hypothesis));
        }
    }

    if record_retrievals && !ids.is_empty() {
        let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        memory.record_retrievals(&refs);
        memory.save(repo_path)?;
    }

    Ok(SurfacedLearnings {
        hits,
        ids,
        truncated,
        anti_patterns,
        pointer_traces,
    })
}

/// Build the focus briefing.
pub fn build_focus_briefing(
    graph: &KnowledgeGraph,
    target_id: &str,
    repo_path: &Path,
    scan_node_count: usize,
    temporal: Option<TemporalContextBrief>,
    record_retrievals: bool,
) -> FocusBriefing {
    let node = graph.nodes.get(target_id);

    // -- Target Info --
    let target = FocusTarget {
        id: target_id.to_string(),
        kind: node
            .map(|n| format!("{:?}", n.kind))
            .unwrap_or_else(|| "unknown".to_string()),
        label: node
            .map(|n| n.label.clone())
            .unwrap_or_else(|| target_id.to_string()),
        technology: node.and_then(|n| n.technology().map(|s| s.to_string())),
        system: infer_system(target_id),
        gotchas: node.map(|n| n.gotchas()).unwrap_or_default(),
        operational_constraints: node
            .map(|n| n.operational_constraints())
            .unwrap_or_default(),
        runbooks: node.map(|n| n.runbooks()).unwrap_or_default(),
    };

    // -- Blast Radius --
    let upstream = collect_dependents(graph, target_id, 3);
    let downstream = collect_dependencies(graph, target_id, 3);
    let blast_radius = BlastRadius {
        total_affected: upstream.len() + downstream.len(),
        upstream,
        downstream,
    };

    // -- Decisions --
    let decisions: Vec<LinkedDecision> = graph
        .decisions
        .values()
        .filter(|d| {
            d.affects.iter().any(|a| a == target_id)
                || d.affects.iter().any(|a| target_id.starts_with(a.as_str()))
        })
        .map(|d| LinkedDecision {
            id: d.id.clone(),
            title: d.title.clone(),
            status: format!("{:?}", d.status),
            summary: truncate(&d.decision, 120),
        })
        .collect();

    // -- Boundaries --
    let boundaries = infer_boundaries(graph, target_id);

    // -- External Context --
    let external_context = find_relevant_external_context(repo_path, target_id);

    // -- Hotspot Status --
    let in_degree = graph.edges.iter().filter(|e| e.target == target_id).count();
    let out_degree = graph.edges.iter().filter(|e| e.source == target_id).count();
    let total_degree = in_degree + out_degree;
    let avg_degree = if graph.nodes.is_empty() {
        0.0
    } else {
        (graph.edges.len() as f64 * 2.0) / graph.nodes.len() as f64
    };
    let is_hotspot = total_degree as f64 > avg_degree * 1.5;
    let role = if in_degree > out_degree * 2 {
        "Hub (many dependents)"
    } else if out_degree > in_degree * 2 {
        "Orchestrator (many dependencies)"
    } else if is_hotspot {
        "Bridge (connects many components)"
    } else {
        "Regular"
    };

    let hotspot_status = HotspotStatus {
        is_hotspot,
        role: role.to_string(),
        in_degree,
        out_degree,
    };

    // -- AI Instructions --
    let mut ai_instructions: Vec<String> = Vec::new();

    if is_hotspot {
        ai_instructions.push(format!(
            "⚠️  This is a {} node — changes affect {} components. Proceed carefully.",
            role.to_lowercase(),
            blast_radius.total_affected
        ));
    }

    for d in &decisions {
        ai_instructions.push(format!("Must respect {}: {}", d.id, truncate(&d.title, 60)));
    }

    for b in &boundaries {
        if !b.allowed {
            ai_instructions.push(format!(
                "⛔ Must NOT introduce coupling: {} → {} ({})",
                b.from, b.to, b.reason
            ));
        }
    }

    // -- Tribal Knowledge AI Instructions --
    for g in &target.gotchas {
        ai_instructions.push(format!("💡 Gotcha: {}", g));
    }
    for c in &target.operational_constraints {
        ai_instructions.push(format!("⚠️  Constraint: {}", c));
    }

    if ai_instructions.is_empty() {
        ai_instructions
            .push("No special constraints found. Standard coding practices apply.".to_string());
    }

    // -- Architectural Guardrails (From Agentic Memory) --
    let (
        memory_hits,
        surfaced_learning_ids,
        memory_truncated,
        mut anti_patterns,
        mut pointer_traces,
    ) = match surface_agent_learnings(repo_path, target_id, record_retrievals) {
        Ok(s) => (
            s.hits,
            s.ids,
            s.truncated,
            s.anti_patterns,
            s.pointer_traces,
        ),
        Err(_) => (Vec::new(), Vec::new(), false, Vec::new(), Vec::new()),
    };

    let decision_trace_events = crate::commands::context_events::read_context_events_query(
        repo_path,
        crate::commands::context_events::ContextEventQuery {
            limit: 12,
            kind_filter: None,
            details_substring: None,
            decision_id: None,
            trace_id: None,
            run_id: None,
            element_id: Some(target_id),
            decision_lineage_only: true,
        },
    )
    .unwrap_or_default();

    let decision_records: Vec<crate::commands::decision::DecisionListItem> =
        crate::commands::list_decisions(repo_path)
            .unwrap_or_default()
            .into_iter()
            .filter(|it| {
                it.elements.iter().any(|e| {
                    e == target_id
                        || target_id.starts_with(&format!("{e}."))
                        || e.starts_with(&format!("{target_id}."))
                })
            })
            .take(10)
            .collect();

    if !decision_trace_events.is_empty() {
        ai_instructions.push(
            "Recent decision/workflow lineage events exist for this element — see briefing.decision_trace_events."
                .to_string(),
        );
    }
    if !decision_records.is_empty() {
        ai_instructions.push(
            "On-disk Decision Records reference this element — see briefing.decision_records (treat learned_facts as hypotheses until reviewed)."
                .to_string(),
        );
    }

    // -- Fallback to Legacy Anti Patterns (From AI Scratchpad) --
    let scratchpad_path = repo_path.join(".sruja").join("ai-scratchpad.md");
    if let Ok(content) = std::fs::read_to_string(&scratchpad_path) {
        let lines: Vec<&str> = content.lines().collect();
        let mut in_section = false;
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("##") {
                in_section = trimmed.to_lowercase().contains("what not to try")
                    || trimmed.to_lowercase().contains("failed hypothesis");
                continue;
            }
            if in_section && !trimmed.is_empty() && !trimmed.starts_with('#') {
                let advice = truncate(
                    trimmed.trim_start_matches("- ").trim_start_matches("* "),
                    120,
                );
                if !anti_patterns.contains(&advice) {
                    anti_patterns.push(advice);
                }
            }
        }
    }

    if !anti_patterns.is_empty() && pointer_traces.is_empty() {
        pointer_traces.push(
            "Review .sruja/ai-scratchpad.md for recent failed hypotheses before proceeding."
                .to_string(),
        );
    }

    // Inject anti-patterns into AI instructions for high visibility
    for ap in &anti_patterns {
        ai_instructions.insert(0, format!("🛑 ARCHITECTURAL GUARDRAIL: {}", ap));
    }

    // -- Context Score --
    let score = compute_context_score(graph, scan_node_count, repo_path, 0);

    if let Some(t) = &temporal {
        ai_instructions.push(format!(
            "Git range {}..{} maps the diff to {} scan-graph component(s).",
            t.base_ref,
            t.head_ref,
            t.diff_mapped_component_ids.len()
        ));
        if t.touches_focus_target {
            ai_instructions.push(
                "This focus target is in (or under) the diff-mapped component set for that range."
                    .to_string(),
            );
        }
        if t.policy_fingerprint_base.is_some()
            && t.policy_fingerprint_head.is_some()
            && t.policy_fingerprint_base != t.policy_fingerprint_head
        {
            ai_instructions.push(
                "Declared architecture file content (fingerprint) differs between base ref and working tree."
                    .to_string(),
            );
        }
    }

    FocusBriefing {
        run_id: None,
        target,
        blast_radius,
        reasoned_traces: collect_reasoned_traces(graph, target_id),
        decisions,
        boundaries,
        external_context,
        hotspot_status,
        ai_instructions,
        anti_patterns,
        pointer_traces,
        memory_hits,
        memory_truncated,
        context_score: score.score,
        temporal,
        enrichment: None,
        decision_trace_events,
        decision_records,
        surfaced_learning_ids,
    }
}

pub fn build_focus_for_ai_output(
    repo_path: &Path,
    file: Option<&str>,
    element_id: Option<&str>,
    run_id: Option<&str>,
    briefing: FocusBriefing,
) -> FocusForAiOutput {
    let resolved = briefing.target.id.clone();
    FocusForAiOutput {
        artifact_kind: "deterministic_brief".to_string(),
        schema_version: FOCUS_FOR_AI_SCHEMA_VERSION.to_string(),
        run_id: run_id.map(|s| s.to_string()),
        repo: repo_path.display().to_string(),
        target: FocusForAiTarget {
            file: file.map(|s| s.to_string()),
            element_id: element_id.map(|s| s.to_string()),
            resolved_element_id: resolved.clone(),
        },
        suggested_next_steps: suggested_next_steps(&resolved),
        briefing,
    }
}

fn suggested_next_steps(resolved_element_id: &str) -> Vec<SuggestedCommand> {
    vec![
        SuggestedCommand {
            purpose: "Explain this element using the reviewed architecture (if available)"
                .to_string(),
            argv: vec![
                "sruja".to_string(),
                "explain".to_string(),
                resolved_element_id.to_string(),
                "--json".to_string(),
            ],
        },
        SuggestedCommand {
            purpose: "Blast radius analysis from the scan graph".to_string(),
            argv: vec![
                "sruja".to_string(),
                "impact".to_string(),
                resolved_element_id.to_string(),
                "-r".to_string(),
                ".".to_string(),
                "--depth".to_string(),
                "3".to_string(),
                "-f".to_string(),
                "json".to_string(),
            ],
        },
        SuggestedCommand {
            purpose: "Get larger task context JSON (for other agents / tools)".to_string(),
            argv: vec![
                "sruja".to_string(),
                "context".to_string(),
                "-r".to_string(),
                ".".to_string(),
                "-f".to_string(),
                "for-ai".to_string(),
                "--element-id".to_string(),
                resolved_element_id.to_string(),
                "--max-tokens".to_string(),
                "2000".to_string(),
            ],
        },
        SuggestedCommand {
            purpose: "Show prior learnings / guardrails recorded for this element".to_string(),
            argv: vec![
                "sruja".to_string(),
                "agent".to_string(),
                "history".to_string(),
                "-r".to_string(),
                ".".to_string(),
                "-e".to_string(),
                resolved_element_id.to_string(),
                "-f".to_string(),
                "json".to_string(),
            ],
        },
        SuggestedCommand {
            purpose: "Check for architectural drift and policy violations".to_string(),
            argv: vec![
                "sruja".to_string(),
                "drift".to_string(),
                "-r".to_string(),
                ".".to_string(),
                "-f".to_string(),
                "json".to_string(),
            ],
        },
    ]
}

#[allow(clippy::too_many_arguments)]
fn build_focus_enrichment(
    repo_path: &Path,
    briefing: &FocusBriefing,
    enrich: bool,
    enrich_provider: Option<&str>,
    enrich_cmd: Option<&str>,
    enrich_model: Option<&str>,
    enrich_base_url: Option<&str>,
    enrich_timeout_ms: u64,
    enrich_max_bytes: usize,
) -> Option<FocusEnrichment> {
    if !enrich && enrich_cmd.is_none() {
        return None;
    }

    let plan = resolve_enrichment_plan(
        repo_path,
        enrich_cmd,
        enrich_model,
        enrich_base_url,
        Some(enrich_timeout_ms),
        Some(enrich_max_bytes),
    );
    let provider = enrich_provider.unwrap_or(plan.provider.as_str());
    let limits = plan.limits;

    let payload = serde_json::json!({
        "schema_version": "focus_enrichment_input/v1",
        "repo": repo_path.display().to_string(),
        "briefing": briefing,
    });
    let stdin_payload = serde_json::to_vec(&payload).unwrap_or_default();

    if provider == "cmd" {
        let Some(cmd) = plan.cmd.as_deref() else {
            return Some(FocusEnrichment {
                status: "skipped".to_string(),
                provider: "cmd".to_string(),
                model: None,
                error: Some("No command configured. Pass --enrich-cmd or set SRUJA_ENRICH_CMD (or .sruja/config.toml [integrations].cmd).".to_string()),
                narrative_markdown: None,
            });
        };
        return Some(match run_cmd_enrichment(cmd, &stdin_payload, limits) {
            Ok(md) => FocusEnrichment {
                status: "ok".to_string(),
                provider: "cmd".to_string(),
                model: None,
                error: None,
                narrative_markdown: Some(md),
            },
            Err(e) => FocusEnrichment {
                status: "error".to_string(),
                provider: "cmd".to_string(),
                model: None,
                error: Some(e),
                narrative_markdown: None,
            },
        });
    }

    if provider != "openai" {
        return Some(FocusEnrichment {
            status: "skipped".to_string(),
            provider: provider.to_string(),
            model: None,
            error: Some(
                "Unsupported provider. Use provider=cmd (recommended) or provider=openai."
                    .to_string(),
            ),
            narrative_markdown: None,
        });
    }

    let model = plan.model.as_deref().unwrap_or("gpt-4o-mini");
    let base_url = plan
        .base_url
        .as_deref()
        .unwrap_or("https://api.openai.com/v1");
    let Some(api_key) = resolve_openai_auth() else {
        return Some(FocusEnrichment {
            status: "skipped".to_string(),
            provider: "openai".to_string(),
            model: Some(model.to_string()),
            error: Some("Missing API key (set OPENAI_API_KEY or SRUJA_ENRICH_API_KEY; SRUJA_LLM_API_KEY is deprecated).".to_string()),
            narrative_markdown: None,
        });
    };

    let user_prompt = format!(
        r#"You are assisting an AI coding agent.

You MUST only use the JSON facts provided below. Do not invent modules, APIs, or file paths. If something is unknown, say "unknown".

Produce markdown with these sections:
- "One-paragraph plan"
- "Risks / unknowns to verify" (bullets)
- "Suggested test/verification steps" (bullets)
- "Clarifying questions" (bullets)

JSON facts:
{}"#,
        payload
    );

    match run_openai_markdown(
        "You are a careful repo assistant. Never fabricate.",
        &user_prompt,
        model,
        base_url,
        &api_key,
    ) {
        Ok(md) => Some(FocusEnrichment {
            status: "ok".to_string(),
            provider: "openai".to_string(),
            model: Some(model.to_string()),
            error: None,
            narrative_markdown: Some(md),
        }),
        Err(e) => Some(FocusEnrichment {
            status: "error".to_string(),
            provider: "openai".to_string(),
            model: Some(model.to_string()),
            error: Some(e),
            narrative_markdown: None,
        }),
    }
}

/// Collect upstream dependents (who depends on this node).
fn collect_dependents(
    graph: &KnowledgeGraph,
    target_id: &str,
    max_depth: usize,
) -> Vec<AffectedNode> {
    let mut result = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(target_id.to_string());
    let mut frontier: Vec<(String, usize)> = vec![(target_id.to_string(), 0)];

    while let Some((current, depth)) = frontier.pop() {
        if depth >= max_depth {
            continue;
        }
        for edge in &graph.edges {
            if edge.target == current && !visited.contains(&edge.source) {
                visited.insert(edge.source.clone());
                let label = graph
                    .nodes
                    .get(&edge.source)
                    .map(|n| n.label.clone())
                    .unwrap_or_else(|| edge.source.clone());
                let relationship = edge
                    .label
                    .as_deref()
                    .unwrap_or(&format!("{}", edge.kind))
                    .to_string();
                result.push(AffectedNode {
                    id: edge.source.clone(),
                    label,
                    depth: depth + 1,
                    relationship,
                });
                frontier.push((edge.source.clone(), depth + 1));
            }
        }
    }

    result.sort_by(|a, b| a.depth.cmp(&b.depth).then_with(|| a.id.cmp(&b.id)));
    result
}

/// Collect downstream dependencies (what this node depends on).
fn collect_dependencies(
    graph: &KnowledgeGraph,
    target_id: &str,
    max_depth: usize,
) -> Vec<AffectedNode> {
    let mut result = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(target_id.to_string());
    let mut frontier: Vec<(String, usize)> = vec![(target_id.to_string(), 0)];

    while let Some((current, depth)) = frontier.pop() {
        if depth >= max_depth {
            continue;
        }
        for edge in &graph.edges {
            if edge.source == current && !visited.contains(&edge.target) {
                visited.insert(edge.target.clone());
                let label = graph
                    .nodes
                    .get(&edge.target)
                    .map(|n| n.label.clone())
                    .unwrap_or_else(|| edge.target.clone());
                let relationship = edge
                    .label
                    .as_deref()
                    .unwrap_or(&format!("{}", edge.kind))
                    .to_string();
                result.push(AffectedNode {
                    id: edge.target.clone(),
                    label,
                    depth: depth + 1,
                    relationship,
                });
                frontier.push((edge.target.clone(), depth + 1));
            }
        }
    }

    result.sort_by(|a, b| a.depth.cmp(&b.depth).then_with(|| a.id.cmp(&b.id)));
    result
}

/// Infer system name from dotted ID (e.g., "Auth.Handler" → "Auth").
fn infer_system(id: &str) -> Option<String> {
    let parts: Vec<&str> = id.split('.').collect();
    if parts.len() > 1 {
        Some(parts[0].to_string())
    } else {
        None
    }
}

/// Collect reasoned traces from the target node using tree-search why.
fn collect_reasoned_traces(graph: &KnowledgeGraph, target_id: &str) -> Vec<ReasonedTrace> {
    let result = match graph.query_why_reasoned(target_id, 3) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    result
        .steps
        .into_iter()
        .take(6)
        .map(|s: ReasonedWhyStep| ReasonedTrace {
            node_id: s.node_id,
            node_label: s.node_label,
            direction: s.direction,
            reasoning: s.reasoning,
            decision_ref: s.decision_ref,
        })
        .collect()
}

/// Infer boundary rules from the graph structure.
fn infer_boundaries(graph: &KnowledgeGraph, target_id: &str) -> Vec<BoundaryInfo> {
    let mut boundaries = Vec::new();

    // Check existing edges as "allowed" boundaries
    for edge in &graph.edges {
        if edge.source == target_id {
            boundaries.push(BoundaryInfo {
                from: target_id.to_string(),
                to: edge.target.clone(),
                allowed: true,
                reason: edge
                    .label
                    .as_deref()
                    .unwrap_or("defined relationship")
                    .to_string(),
            });
        }
    }

    // Check policy violations for "not allowed" boundaries
    let violations = graph.find_policy_violations();
    for v in &violations {
        if v.source == target_id || v.target == target_id {
            boundaries.push(BoundaryInfo {
                from: v.source.clone(),
                to: v.target.clone(),
                allowed: false,
                reason: v.message.clone(),
            });
        }
    }

    boundaries
}

/// Find external context files relevant to a target element using BM25-ranked retrieval.
///
/// Builds a sparse inverted index over `.sruja/context/` and queries it with the
/// target element ID plus its dot-separated parts for broader recall. Falls back to
/// an empty result if no context directory exists.
fn find_relevant_external_context(repo_path: &Path, target_id: &str) -> Vec<ExternalContextRef> {
    let context_dir = repo_path.join(".sruja").join("context");
    if !context_dir.exists() {
        return Vec::new();
    }

    let index = sruja_graph::SparseIndex::build(repo_path);
    if index.doc_count() == 0 {
        return Vec::new();
    }

    let target_parts: Vec<&str> = target_id.split('.').collect();
    let query = if target_parts.len() > 1 {
        format!("{} {}", target_id, target_parts.join(" "))
    } else {
        target_id.to_string()
    };

    let max_results = crate::integrations::load_repo_config(repo_path)
        .and_then(|c| c.context_engineering.bm25_max_results_focus)
        .unwrap_or(10);
    let hits = index.search(&query, max_results);

    hits.into_iter()
        .map(|hit| {
            let name = std::path::Path::new(&hit.path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            ExternalContextRef {
                file: name,
                category: hit.category,
                excerpt: hit.excerpt,
            }
        })
        .collect()
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        if max_len <= 3 {
            ".".repeat(max_len)
        } else {
            let keep = max_len.saturating_sub(3);
            let prefix: String = s.chars().take(keep).collect();
            format!("{}...", prefix)
        }
    }
}

// ──────────────────────────────────────────────────────────────
// CLI entry point
// ──────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub async fn focus(
    repo: &str,
    file: Option<&str>,
    element_id: Option<&str>,
    format: &str,
    run_id: Option<&str>,
    enrich: bool,
    enrich_provider: Option<&str>,
    enrich_cmd: Option<&str>,
    enrich_model: Option<&str>,
    enrich_base_url: Option<&str>,
    enrich_timeout_ms: u64,
    enrich_max_bytes: usize,
    base_ref: Option<&str>,
    head_ref: Option<&str>,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo),
        )));
    }

    // Load or build the knowledge graph
    let kg = graph_store::load_or_build_graph(repo_path)?;

    // Also get scan node count for context score
    let scan_node_count = match sruja_scan::scan_repo(repo_path) {
        Ok(g) => g.nodes.len(),
        Err(_) => kg.nodes.len(),
    };

    // Resolve the target
    let target_id = resolve_target(&kg, repo_path, file, element_id)?;

    let temporal = match (base_ref, head_ref) {
        (Some(b), Some(h)) => Some(load_temporal_context(repo_path, b, h, &target_id)?),
        (Some(b), None) => Some(load_temporal_context(repo_path, b, "HEAD", &target_id)?),
        (None, Some(_)) => {
            return Err(CliError::validation(
                "--head-ref requires --base-ref (use both for git-range temporal context)"
                    .to_string(),
            ));
        }
        (None, None) => None,
    };

    let mut briefing =
        build_focus_briefing(&kg, &target_id, repo_path, scan_node_count, temporal, true);
    let run_id = run_id
        .map(|s| s.to_string())
        .unwrap_or_else(generate_run_id);
    briefing.run_id = Some(run_id.clone());

    // Persist a bounded snapshot for replay/resume.
    let snapshot = serde_json::json!({
        "schema_version": "focus_snapshot/v1",
        "run_id": run_id,
        "repo": repo,
        "selectors": { "file": file, "element_id": element_id, "base_ref": base_ref, "head_ref": head_ref },
        "resolved_target_id": target_id,
        "external_context": briefing.external_context.iter().map(|e| serde_json::json!({
            "file": e.file,
            "category": e.category,
        })).collect::<Vec<_>>(),
        "memory_hits": briefing.memory_hits.iter().map(|h| serde_json::json!({
            "id": h.id,
            "kind": h.kind,
            "outcome": h.outcome,
            "match_reason": h.match_reason,
        })).collect::<Vec<_>>(),
        "memory_truncated": briefing.memory_truncated,
    });
    let _ = write_json_snapshot(repo_path, &run_id, "focus.json", &snapshot);
    briefing.enrichment = build_focus_enrichment(
        repo_path,
        &briefing,
        enrich,
        enrich_provider,
        enrich_cmd,
        enrich_model,
        enrich_base_url,
        enrich_timeout_ms,
        enrich_max_bytes,
    );

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&briefing)?);
        }
        "for-ai" => {
            let out =
                build_focus_for_ai_output(repo_path, file, element_id, Some(&run_id), briefing);
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        _ => {
            print_focus_briefing(&briefing);
            if let Some(enrichment) = &briefing.enrichment {
                if let Some(md) = enrichment.narrative_markdown.as_deref() {
                    println!();
                    println!("{}", md);
                }
            }
        }
    }

    Ok(())
}

fn print_focus_briefing(b: &FocusBriefing) {
    let width = 56;
    let border = "─".repeat(width);

    println!();
    println!(
        "╭─ {} {} ─{}╮",
        "Context Focus:".bold(),
        colors::info(&b.target.label),
        "─".repeat(width.saturating_sub(18 + b.target.label.len()))
    );
    println!("│{:width$}│", "", width = width);

    // Target info
    println!(
        "│  📍 Component: {:width$}│",
        colors::style(&b.target.id).bold(),
        width = width - 17
    );
    if let Some(ref sys) = b.target.system {
        println!("│  🏗  System:    {:width$}│", sys, width = width - 17);
    }
    if let Some(ref tech) = b.target.technology {
        println!("│  🔧 Technology: {:width$}│", tech, width = width - 18);
    }

    if !b.target.gotchas.is_empty() {
        println!(
            "│  💡 Gotchas:    {} recorded{:width$}│",
            b.target.gotchas.len(),
            "",
            width = width - 30
        );
    }
    if !b.target.operational_constraints.is_empty() {
        println!(
            "│  ⚠️  Constraints: {} recorded{:width$}│",
            b.target.operational_constraints.len(),
            "",
            width = width - 30
        );
    }

    println!("│{:width$}│", "", width = width);

    // Blast radius
    let risk = if b.blast_radius.total_affected > 10 {
        colors::error("HIGH").to_string()
    } else if b.blast_radius.total_affected > 5 {
        colors::warning("MEDIUM").to_string()
    } else {
        colors::success("LOW").to_string()
    };
    println!(
        "│  Blast Radius: {} components affected{:width$}│",
        b.blast_radius.total_affected,
        "",
        width = width - 42
    );
    println!(
        "│  Risk Level:   {}{:width$}│",
        risk,
        "",
        width = width - 20
    );

    if b.hotspot_status.is_hotspot {
        println!(
            "│  🔥 Hotspot:   {}{:width$}│",
            b.hotspot_status.role,
            "",
            width = width.saturating_sub(18 + b.hotspot_status.role.len())
        );
    }

    println!("│{:width$}│", "", width = width);

    // Upstream
    if !b.blast_radius.upstream.is_empty() {
        println!(
            "│  ── Upstream (depends on this) ──{:width$}│",
            "",
            width = width - 36
        );
        for node in b.blast_radius.upstream.iter().take(5) {
            println!(
                "│  • {} (depth {}) — {}{:width$}│",
                node.id,
                node.depth,
                truncate(&node.relationship, 20),
                "",
                width =
                    width.saturating_sub(10 + node.id.len() + 10 + node.relationship.len().min(20))
            );
        }
        println!("│{:width$}│", "", width = width);
    }

    // Downstream
    if !b.blast_radius.downstream.is_empty() {
        println!(
            "│  ── Downstream (this depends on) ──{:width$}│",
            "",
            width = width - 38
        );
        for node in b.blast_radius.downstream.iter().take(5) {
            println!(
                "│  • {} (depth {}) — {}{:width$}│",
                node.id,
                node.depth,
                truncate(&node.relationship, 20),
                "",
                width =
                    width.saturating_sub(10 + node.id.len() + 10 + node.relationship.len().min(20))
            );
        }
        println!("│{:width$}│", "", width = width);
    }

    // Decisions
    if !b.decisions.is_empty() {
        println!(
            "│  ── Active Decisions ──{:width$}│",
            "",
            width = width - 26
        );
        for d in &b.decisions {
            println!(
                "│  {}: {}{:width$}│",
                d.id,
                truncate(&d.title, 40),
                "",
                width = width.saturating_sub(6 + d.id.len() + d.title.len().min(40))
            );
        }
        println!("│{:width$}│", "", width = width);
    }

    // Boundaries
    let not_allowed: Vec<&BoundaryInfo> = b.boundaries.iter().filter(|b| !b.allowed).collect();
    if !not_allowed.is_empty() {
        println!("│  ── Boundaries ──{:width$}│", "", width = width - 20);
        for bi in &not_allowed {
            println!(
                "│  ⛔ {} → {}: NOT allowed{:width$}│",
                bi.from,
                bi.to,
                "",
                width = width.saturating_sub(10 + bi.from.len() + bi.to.len() + 15)
            );
        }
        println!("│{:width$}│", "", width = width);
    }

    // Git temporal (optional)
    if let Some(ref t) = b.temporal {
        println!(
            "│  ── Git temporal ({}..{}) ──{:width$}│",
            t.base_ref,
            t.head_ref,
            "",
            width = width.saturating_sub(21 + t.base_ref.len() + t.head_ref.len())
        );
        println!(
            "│  Diff-mapped components: {}{:width$}│",
            t.diff_mapped_component_ids.len(),
            "",
            width =
                width.saturating_sub(29 + format!("{}", t.diff_mapped_component_ids.len()).len())
        );
        if t.touches_focus_target {
            println!(
                "│  Target overlaps diff map: yes{:width$}│",
                "",
                width = width - 30
            );
        }
        println!("│{:width$}│", "", width = width);
    }

    // External Context
    if !b.external_context.is_empty() {
        println!(
            "│  ── External Context ──{:width$}│",
            "",
            width = width - 26
        );
        for ec in &b.external_context {
            println!(
                "│  📄 {} [{}]{:width$}│",
                ec.file,
                ec.category,
                "",
                width = width.saturating_sub(8 + ec.file.len() + ec.category.len())
            );
        }
        println!("│{:width$}│", "", width = width);
    }

    // AI Instructions
    println!(
        "│  ── AI Agent Instructions ──{:width$}│",
        "",
        width = width - 31
    );
    for (i, instr) in b.ai_instructions.iter().enumerate() {
        let display = truncate(instr, width - 8);
        println!(
            "│  {}. {}{:width$}│",
            i + 1,
            display,
            "",
            width = width.saturating_sub(6 + display.len())
        );
    }

    // Evo-style Anti Patterns
    if !b.anti_patterns.is_empty() {
        println!("│{:width$}│", "", width = width);
        println!(
            "│  ── What NOT to try (Scratchpad) ──{:width$}│",
            "",
            width = width - 37
        );
        for ap in b.anti_patterns.iter().take(5) {
            let display = truncate(ap, width - 8);
            println!(
                "│  ⛔ {}{:width$}│",
                display,
                "",
                width = width.saturating_sub(6 + display.len())
            );
        }
    }

    // Context Score
    println!("│{:width$}│", "", width = width);
    println!(
        "│  Context Score: {}{:width$}│",
        colors::health_bar(b.context_score, 15),
        "",
        width = width.saturating_sub(45)
    );

    println!("╰{}╯", border);
    println!();
}
