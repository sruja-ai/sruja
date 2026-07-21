use std::collections::HashSet;
use std::path::Path;

use crate::commands::CliError;
use crate::enrichment::EnrichmentRef;
use crate::graph_store;
use crate::integrations::load_repo_config;
use crate::utils::colors;
use crate::utils::run_id::generate_run_id;
use crate::utils::run_snapshots::write_json_snapshot;
use sruja_graph::{compute_context_score, KnowledgeGraph, ReasonedWhyStep, SparseIndex};

use super::types::{
    truncate, AcceptanceCriteriaSummary, AffectedNode, BlastRadius, BoundaryInfo,
    ExternalContextRef, FocusBriefing, FocusForAiOutput, FocusForAiTarget, FocusTarget,
    HotspotStatus, LinkedDecision, LinkedRequirementSummary, ReasonedTrace,
    SuggestedCommand, TemporalContextBrief,
};

mod resolve;
mod enrich;

pub use resolve::resolve_target;
pub use enrich::{surface_agent_learnings, load_ask_thresholds, compute_ask_plan};

use resolve::git_arch_blob_blake3;
use enrich::build_focus_enrichment;

const FOCUS_FOR_AI_SCHEMA_VERSION: &str = "focus_for_ai/v1";

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

pub fn build_focus_briefing(
    graph: &KnowledgeGraph,
    target_id: &str,
    repo_path: &Path,
    scan_node_count: usize,
    temporal: Option<TemporalContextBrief>,
    record_retrievals: bool,
    compact: bool,
) -> FocusBriefing {
    let node = graph.nodes.get(target_id);

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

    let active_drift_violations = if compact {
        let scan = sruja_scan::scan_repo(repo_path).ok();
        let violations = if let Some(actual_graph) = &scan {
            let resolved = crate::utils::architecture_path::resolve_architecture_path(repo_path);
            if let Some(arch_path) = resolved {
                if let Ok(content) = std::fs::read_to_string(&arch_path) {
                    if let Ok(program) =
                        sruja_language::Parser::new(arch_path.to_string_lossy().as_ref())
                            .parse(&content)
                    {
                        let proposed_graph = sruja_diff::program_to_graph(&program);
                        let diff_result = sruja_diff::compare_graphs(actual_graph, &proposed_graph);
                        diff_result.violations
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            } else {
                let drift_result = sruja_diff::detect_architectural_drift(actual_graph);
                drift_result.violations
            }
        } else {
            Vec::new()
        };

        violations
            .into_iter()
            .filter(|v| {
                if let Some(loc) = &v.location {
                    if loc == target_id
                        || loc.starts_with(&format!("{}.", target_id))
                        || target_id.starts_with(&format!("{}.", loc))
                    {
                        return true;
                    }
                }
                v.sources.iter().any(|s| {
                    let s_str = sruja_diff::SourceRef::display_string(s);
                    s_str.contains(target_id)
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    let blast_radius = if compact {
        BlastRadius {
            total_affected: 0,
            upstream: Vec::new(),
            downstream: Vec::new(),
        }
    } else {
        let upstream = collect_dependents(graph, target_id, 3);
        let downstream = collect_dependencies(graph, target_id, 3);
        BlastRadius {
            total_affected: upstream.len() + downstream.len(),
            upstream,
            downstream,
        }
    };

    let decisions: Vec<LinkedDecision> = if compact {
        Vec::new()
    } else {
        let blast_radius_decisions = graph.get_decisions_for_blast_radius(target_id);
        blast_radius_decisions
            .into_iter()
            .map(|d| LinkedDecision {
                id: d.id.clone(),
                title: d.title.clone(),
                status: format!("{:?}", d.status),
                summary: truncate(&d.decision, 120),
            })
            .collect()
    };

    let mut boundaries = infer_boundaries(graph, target_id);
    if compact {
        boundaries.retain(|b| !b.allowed);
    }

    let external_context = if compact {
        Vec::new()
    } else {
        find_relevant_external_context(repo_path, target_id)
    };

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

    let mut ai_instructions: Vec<String> = Vec::new();

    if is_hotspot && !compact {
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
        Err(_) => (Vec::new(), Vec::<String>::new(), false, Vec::<String>::new(), Vec::<String>::new()),
    };

    if memory_truncated {
        ai_instructions.push(format!(
            "More agentic learnings exist but were truncated for token budget. Use `sruja agent history -r . -e {}` to view full history.",
            target_id
        ));
    }

    let decision_trace_events = if compact {
        Vec::new()
    } else {
        crate::commands::context_events::read_context_events_query(
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
        .unwrap_or_default()
    };

    let decision_records: Vec<crate::commands::decision::DecisionListItem> = if compact {
        Vec::new()
    } else {
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
            .collect()
    };

    let linked_requirements: Vec<LinkedRequirementSummary> = if compact {
        Vec::new()
    } else {
        collect_linked_requirements(repo_path, target_id)
    };

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
    if !linked_requirements.is_empty() {
        let high_priority_count = linked_requirements
            .iter()
            .filter(|r| {
                r.priority
                    .as_deref()
                    .is_some_and(|p| p == "must" || p == "should")
            })
            .count();
        if high_priority_count > 0 {
            ai_instructions.push(format!(
                "{} linked requirement(s) with must/should priority affect this element — see briefing.linked_requirements. Read acceptance criteria before changing behavior.",
                high_priority_count
            ));
        } else {
            ai_instructions.push(
                "Requirements reference this element — see briefing.linked_requirements."
                    .to_string(),
            );
        }
    }

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

    for ap in &anti_patterns {
        ai_instructions.insert(0, format!("🛑 ARCHITECTURAL GUARDRAIL: {}", ap));
    }

    let score = compute_context_score(graph, scan_node_count, repo_path, 0);

    let ask_plan = if compact {
        None
    } else {
        let thresholds = load_ask_thresholds(repo_path);
        Some(compute_ask_plan(
            &target.kind,
            &target.label,
            blast_radius.total_affected,
            None,
            &memory_hits,
            &thresholds,
        ))
    };

    let last_session = load_last_session_summary(repo_path);

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
        active_drift_violations,
        target,
        blast_radius,
        reasoned_traces: if compact {
            Vec::new()
        } else {
            collect_reasoned_traces(graph, target_id)
        },
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
        linked_requirements,
        surfaced_learning_ids,
        last_session: if compact { None } else { last_session },
        ask_plan,
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

fn infer_system(id: &str) -> Option<String> {
    let parts: Vec<&str> = id.split('.').collect();
    if parts.len() > 1 {
        Some(parts[0].to_string())
    } else {
        None
    }
}

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

fn collect_linked_requirements(repo_path: &Path, target_id: &str) -> Vec<LinkedRequirementSummary> {
    let resolved = crate::utils::architecture_path::resolve_architecture_path(repo_path);
    let arch_path = match resolved {
        Some(p) => p,
        None => return Vec::new(),
    };
    let content = match std::fs::read_to_string(&arch_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let program =
        match sruja_language::Parser::new(arch_path.to_string_lossy().as_ref()).parse(&content) {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

    program
        .items
        .iter()
        .filter_map(|item| {
            if let sruja_language::TopLevelItem::Requirement(req) = item {
                let matches = req.affects.iter().any(|a| {
                    a == target_id
                        || target_id.starts_with(&format!("{a}."))
                        || a.starts_with(&format!("{target_id}."))
                });
                if matches {
                    Some(LinkedRequirementSummary {
                        id: req.id.clone(),
                        title: req.title.clone(),
                        r#type: req.r#type.clone(),
                        priority: req.priority.clone(),
                        status: req.status.clone(),
                        acceptance_criteria: req
                            .acceptance_criteria
                            .iter()
                            .map(|ac| AcceptanceCriteriaSummary {
                                given: ac.given.clone(),
                                when: ac.when.clone(),
                                then: ac.then.clone(),
                            })
                            .collect(),
                        affects: req.affects.clone(),
                        adrs: req.adrs.clone(),
                        scenarios: req.scenarios.clone(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .take(20)
        .collect()
}

fn infer_boundaries(graph: &KnowledgeGraph, target_id: &str) -> Vec<BoundaryInfo> {
    let mut boundaries = Vec::new();

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

fn find_relevant_external_context(repo_path: &Path, target_id: &str) -> Vec<ExternalContextRef> {
    let context_dir = repo_path.join(".sruja").join("context");
    if !context_dir.exists() {
        return Vec::new();
    }

    let index = SparseIndex::build(repo_path);
    if index.doc_count() == 0 {
        return Vec::new();
    }

    let target_parts: Vec<&str> = target_id.split('.').collect();
    let query = if target_parts.len() > 1 {
        format!("{} {}", target_id, target_parts.join(" "))
    } else {
        target_id.to_string()
    };

    let max_results = load_repo_config(repo_path)
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

fn load_last_session_summary(repo_path: &Path) -> Option<serde_json::Value> {
    let path = repo_path.join(".sruja").join("last_session_summary.json");
    let content = std::fs::read_to_string(&path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    let timestamp = value.get("timestamp")?.as_str()?;
    let parsed = chrono::DateTime::parse_from_rfc3339(timestamp).ok()?;
    let age = chrono::Utc::now().signed_duration_since(parsed);
    if age.num_days() > 7 {
        return None;
    }
    Some(value)
}

#[allow(clippy::too_many_arguments)]
pub async fn focus(
    repo: &str,
    file: Option<&str>,
    element_id: Option<&str>,
    format: &str,
    run_id: Option<&str>,
    enrich: &EnrichmentRef<'_>,
    base_ref: Option<&str>,
    head_ref: Option<&str>,
    compact: bool,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo),
        )));
    }

    let kg = graph_store::load_or_build_graph(repo_path)?;

    let scan_node_count = match sruja_scan::scan_repo(repo_path) {
        Ok(g) => g.nodes.len(),
        Err(_) => kg.nodes.len(),
    };

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

    let mut briefing = build_focus_briefing(
        &kg,
        &target_id,
        repo_path,
        scan_node_count,
        temporal,
        true,
        compact,
    );
    let run_id = run_id
        .map(|s| s.to_string())
        .unwrap_or_else(generate_run_id);
    briefing.run_id = Some(run_id.clone());

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
    if compact {
        briefing.enrichment = None;
    } else {
        briefing.enrichment = build_focus_enrichment(repo_path, &briefing, enrich);
    }

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
            super::format::print_focus_briefing(&briefing);
            if let Some(enrichment) = &briefing.enrichment {
                if let Some(md) = enrichment.narrative_markdown.as_deref() {
                    println!();
                    println!("{}", md);
                }
            }
            if let Some(hint) = crate::commands::density::density_hint(repo_path) {
                println!();
                println!("  {}", colors::dim(&hint));
            }
        }
    }

    Ok(())
}
