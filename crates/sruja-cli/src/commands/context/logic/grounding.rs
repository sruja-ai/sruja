use super::super::types::*;
use super::focus::build_focus_context;
use super::task::TaskSelectors;
use crate::commands::CliError;
use crate::utils::architecture_path::resolve_architecture_path;
use sruja_language::ElementKind;
use sruja_scan::{Graph, NodeKind};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct GroundingTraceInputs<'a> {
    pub selectors: TaskSelectors<'a>,
    pub max_tokens: usize,
    pub selection_reason: &'a SelectionReason,
    pub focus_ids: &'a [String],
    pub semantic_candidates: &'a [TaskSemanticCandidate],
    pub neighbors: &'a [TaskNeighbor],
    pub impacted: &'a ImpactBuckets,
    pub source_bindings: &'a [TaskSourceBinding],
    pub hydrated_files: &'a [TaskHydratedFile],
}

pub fn build_grounding_trace(input: GroundingTraceInputs<'_>) -> Vec<GroundingStep> {
    let selector_kind = if input.selectors.element_id.is_some() {
        "element_id"
    } else if input.selectors.file.is_some() {
        "file"
    } else if input.selectors.base_ref.is_some() && input.selectors.head_ref.is_some() {
        "git_diff"
    } else if input.selectors.query.is_some() {
        "query"
    } else {
        "architecture_overview"
    };

    let depth = input.selectors.depth.unwrap_or(1);
    let mut out = Vec::new();

    out.push(GroundingStep {
        phase: GroundingPhase::Input,
        summary: format!("Selected focus using selector `{}`.", selector_kind),
        details: Some(serde_json::json!({
            "element_id": input.selectors.element_id,
            "file": input.selectors.file,
            "query": input.selectors.query,
            "base_ref": input.selectors.base_ref,
            "head_ref": input.selectors.head_ref,
            "depth": depth,
            "max_tokens": input.max_tokens,
        })),
        refs: Vec::new(),
    });

    out.push(GroundingStep {
        phase: GroundingPhase::FocusResolution,
        summary: format!(
            "Resolved focus via `{}` with path {:?}.",
            input.selection_reason.primary, input.selection_reason.resolution_path
        ),
        details: input.selection_reason.details.clone(),
        refs: input.focus_ids.to_vec(),
    });

    if input.selectors.query.is_some() {
        let candidates: Vec<serde_json::Value> = input
            .semantic_candidates
            .iter()
            .take(10)
            .map(|c| {
                serde_json::json!({
                    "element_id": c.element_id,
                    "score": c.score,
                    "label": c.label,
                    "description": c.description,
                    "features": c.features,
                })
            })
            .collect();

        out.push(GroundingStep {
            phase: if input.focus_ids.is_empty() {
                GroundingPhase::SemanticFallback
            } else {
                GroundingPhase::FocusResolution
            },
            summary: format!(
                "Evaluated {} semantic candidate(s) for query matching.",
                input.semantic_candidates.len()
            ),
            details: Some(serde_json::json!({
                "candidates": candidates,
            })),
            refs: input
                .semantic_candidates
                .iter()
                .take(10)
                .map(|c| c.element_id.clone())
                .collect(),
        });
    }

    out.push(GroundingStep {
        phase: GroundingPhase::NeighborExpansion,
        summary: format!(
            "Expanded neighbors to depth {} ({} neighbor edge(s)).",
            depth,
            input.neighbors.len()
        ),
        details: Some(serde_json::json!({
            "impacted_systems": input.impacted.systems.len(),
            "impacted_containers": input.impacted.containers.len(),
            "impacted_components": input.impacted.components.len(),
        })),
        refs: input
            .impacted
            .systems
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<String>>(),
    });

    out.push(GroundingStep {
        phase: GroundingPhase::SourceBinding,
        summary: format!(
            "Bound {} source reference(s) and hydrated {} file(s).",
            input.source_bindings.len(),
            input.hydrated_files.len()
        ),
        details: Some(serde_json::json!({
            "source_bindings": input.source_bindings.iter().take(20).map(|b| serde_json::json!({
                "element_id": b.element_id,
                "source_type": b.source_type,
                "path": b.path,
                "description": b.description,
            })).collect::<Vec<_>>(),
            "source_bindings_truncated": input.source_bindings.len() > 20,
            "hydrated_files": input.hydrated_files.iter().take(10).map(|f| serde_json::json!({
                "element_id": f.element_id,
                "path": f.path,
                "truncated": f.truncated,
            })).collect::<Vec<_>>(),
            "hydrated_files_truncated": input.hydrated_files.len() > 10,
        })),
        refs: input
            .source_bindings
            .iter()
            .take(20)
            .map(|b| b.path.clone())
            .collect(),
    });

    out
}

#[derive(Debug, Default, Clone)]
pub struct BaselineElements {
    pub kinds_by_id: BTreeMap<String, NodeKind>,
    pub labels_by_id: BTreeMap<String, String>,
    pub sources_by_id: BTreeMap<String, Vec<sruja_language::SourceBinding>>,
    pub gotchas_by_id: BTreeMap<String, Vec<String>>,
    pub constraints_by_id: BTreeMap<String, Vec<String>>,
    pub runbooks_by_id: BTreeMap<String, Vec<String>>,
}

pub fn load_baseline_elements(repo_root: &Path) -> BaselineElements {
    let Some(path) = resolve_architecture_path(repo_root) else {
        return BaselineElements::default();
    };
    let Ok(content) = fs::read_to_string(&path) else {
        return BaselineElements::default();
    };
    let parser = sruja_language::Parser::new(path.to_string_lossy().to_string());
    let Ok(program) = parser.parse(&content) else {
        return BaselineElements::default();
    };
    let (elements, _) = sruja_language::collect_elements(&program);

    let mut out = BaselineElements::default();
    for (fqn, elem) in elements {
        let kind = element_kind_to_node_kind(&elem.assignment.kind);
        let label = elem
            .assignment
            .title
            .clone()
            .unwrap_or_else(|| elem.assignment.name.clone());
        let sources = elem
            .assignment
            .body
            .as_ref()
            .map(|b| b.sources.clone())
            .unwrap_or_default();

        out.kinds_by_id.insert(fqn.clone(), kind);
        out.labels_by_id.insert(fqn.clone(), label);
        if !sources.is_empty() {
            out.sources_by_id.insert(fqn.clone(), sources);
        }

        if let Some(body) = elem.assignment.body.as_ref() {
            if !body.gotchas.is_empty() {
                out.gotchas_by_id.insert(fqn.clone(), body.gotchas.clone());
            }
            if !body.operational_constraints.is_empty() {
                out.constraints_by_id
                    .insert(fqn.clone(), body.operational_constraints.clone());
            }
            if !body.runbooks.is_empty() {
                out.runbooks_by_id
                    .insert(fqn.clone(), body.runbooks.clone());
            }
        }
    }
    out
}

fn element_kind_to_node_kind(kind: &ElementKind) -> NodeKind {
    match kind {
        ElementKind::System => NodeKind::new(NodeKind::SYSTEM),
        ElementKind::Container => NodeKind::new(NodeKind::CONTAINER),
        ElementKind::Component => NodeKind::new(NodeKind::COMPONENT),
        ElementKind::Queue => NodeKind::new(NodeKind::QUEUE),
        ElementKind::Database | ElementKind::DataStore => NodeKind::new(NodeKind::DATABASE),
        ElementKind::ExternalSystem => NodeKind::new(NodeKind::EXTERNAL_API),
        _ => NodeKind::new(NodeKind::MODULE),
    }
}

pub fn resolve_focus(
    graph: &Graph,
    repo_root: &str,
    selectors: TaskSelectors<'_>,
    baseline: &BaselineElements,
) -> Result<
    (
        Vec<String>,
        SelectionReason,
        TaskTruthStatus,
        TaskConfidence,
    ),
    CliError,
> {
    if let Some(id) = selectors.element_id {
        let resolved = resolve_element_id(id, baseline, graph);
        let mut reason = SelectionReason {
            primary: "element_id".to_string(),
            resolution_path: vec!["exact_id".to_string()],
            details: Some(serde_json::json!({ "element_id": id, "resolved": resolved })),
        };
        let (truth_status, confidence) = if baseline.kinds_by_id.contains_key(&resolved) {
            (TaskTruthStatus::ArchitecturalTruth, TaskConfidence::High)
        } else if graph.nodes.iter().any(|n| n.id == resolved) {
            (TaskTruthStatus::InferredFromCode, TaskConfidence::High)
        } else {
            reason.primary = "element_id_not_found".to_string();
            reason.resolution_path = vec!["exact_id".to_string(), "not_found".to_string()];
            (TaskTruthStatus::Unknown, TaskConfidence::Low)
        };
        return Ok((vec![resolved], reason, truth_status, confidence));
    }

    if let (Some(base), Some(head)) = (selectors.base_ref, selectors.head_ref) {
        let repo_path = Path::new(repo_root);
        let diffs = sruja_diff::map_git_diff(repo_path, base, head, graph)
            .map_err(|e| CliError::validation(format!("Failed to map git diff: {}", e)))?;

        let mut focus: Vec<String> = diffs.into_iter().map(|d| d.component_id).collect();
        focus.sort();
        focus.dedup();

        let reason = SelectionReason {
            primary: "git_diff".to_string(),
            resolution_path: vec!["diff_binding".to_string(), "scan_graph".to_string()],
            details: Some(serde_json::json!({ "base_ref": base, "head_ref": head })),
        };
        return Ok((
            focus,
            reason,
            TaskTruthStatus::InferredFromScan,
            TaskConfidence::Medium,
        ));
    }

    if let Some(path) = selectors.file {
        let centrality =
            crate::commands::compute_all_centrality_cached(Path::new(repo_root), graph, false)?;
        let focus_ctx = build_focus_context(graph, repo_root, path, None, 0, 0, &centrality)?;
        let mut focus: Vec<String> = focus_ctx.matched_nodes.into_iter().map(|n| n.id).collect();
        focus.truncate(1);
        let reason = SelectionReason {
            primary: "file".to_string(),
            resolution_path: vec!["file_binding".to_string(), "scan_graph".to_string()],
            details: Some(serde_json::json!({ "file": path })),
        };
        let confidence = if focus.is_empty() {
            TaskConfidence::Low
        } else {
            TaskConfidence::Medium
        };
        return Ok((focus, reason, TaskTruthStatus::InferredFromCode, confidence));
    }

    if let Some(q) = selectors.query {
        let candidates = semantic_candidates_from_scan(graph, q, 5);
        let focus = candidates
            .first()
            .map(|c| vec![c.element_id.clone()])
            .unwrap_or_default();

        let confidence = TaskConfidence::Low;
        let reason = SelectionReason {
            primary: "query".to_string(),
            resolution_path: vec!["semantic_fallback".to_string(), "scan_graph".to_string()],
            details: Some(serde_json::json!({ "query": q })),
        };
        return Ok((focus, reason, TaskTruthStatus::Unknown, confidence));
    }

    let mut overview_ids: Vec<String> = Vec::new();
    for (id, kind) in &baseline.kinds_by_id {
        match kind.as_str() {
            NodeKind::SYSTEM | NodeKind::EXTERNAL_API => {
                overview_ids.push(id.clone());
            }
            NodeKind::CONTAINER | NodeKind::DATABASE | NodeKind::QUEUE => {
                let dot_count = id.matches('.').count();
                if dot_count <= 1 {
                    overview_ids.push(id.clone());
                }
            }
            _ => {}
        }
    }
    if overview_ids.is_empty() {
        for node in &graph.nodes {
            if matches!(
                node.kind.as_str(),
                NodeKind::SYSTEM | NodeKind::SERVICE | NodeKind::EXTERNAL_API
            ) {
                overview_ids.push(node.id.clone());
            }
        }
        overview_ids.truncate(30);
    }
    let confidence = if overview_ids.is_empty() {
        TaskConfidence::Low
    } else {
        TaskConfidence::Medium
    };
    Ok((
        overview_ids,
        SelectionReason {
            primary: "architecture_overview".to_string(),
            resolution_path: vec!["no_selector".to_string()],
            details: Some(serde_json::json!({
                "total_scan_nodes": graph.nodes.len(),
            })),
        },
        TaskTruthStatus::Unknown,
        confidence,
    ))
}

fn resolve_element_id(id: &str, baseline: &BaselineElements, graph: &Graph) -> String {
    if baseline.kinds_by_id.contains_key(id) || graph.nodes.iter().any(|n| n.id == id) {
        return id.to_string();
    }
    if let Some((fqn, _)) = baseline
        .kinds_by_id
        .iter()
        .find(|(fqn, _)| fqn.ends_with(&format!(".{}", id)))
    {
        return fqn.clone();
    }
    if let Some(node) = graph
        .nodes
        .iter()
        .find(|n| n.id == id || n.id.ends_with(&format!(".{}", id)))
    {
        return node.id.clone();
    }
    id.to_string()
}

pub fn build_focus_elements(
    graph: &Graph,
    focus_ids: &[String],
    baseline: &BaselineElements,
    selection_reason: &SelectionReason,
) -> Vec<TaskFocusElement> {
    let mut out = Vec::new();
    for id in focus_ids {
        let (kind, label) = if let Some(kind) = baseline.kinds_by_id.get(id) {
            (
                kind.clone(),
                baseline
                    .labels_by_id
                    .get(id)
                    .cloned()
                    .or_else(|| Some(id.clone())),
            )
        } else if let Some(node) = graph.nodes.iter().find(|n| n.id == *id) {
            (node.kind.clone(), Some(node.label.clone()))
        } else {
            (NodeKind::new(NodeKind::MODULE), Some(id.clone()))
        };

        let lineage = baseline
            .kinds_by_id
            .contains_key(id)
            .then(|| compute_lineage(id, baseline));

        let evidence = match selection_reason.primary.as_str() {
            "element_id" => vec![TaskEvidence {
                kind: TaskEvidenceKind::ExactId,
                summary: format!("Resolved by element_id: {}", id),
                locator: None,
            }],
            "git_diff" => vec![TaskEvidence {
                kind: TaskEvidenceKind::DiffFile,
                summary: "Mapped from git diff to impacted components".to_string(),
                locator: None,
            }],
            "file" => vec![TaskEvidence {
                kind: TaskEvidenceKind::FileMatch,
                summary: format!("Matched from file selector: {}", id),
                locator: selection_reason
                    .details
                    .as_ref()
                    .and_then(|d| d.get("file"))
                    .and_then(|v| v.as_str())
                    .map(|p| TaskLocator {
                        path: p.to_string(),
                        line_start: None,
                        line_end: None,
                    }),
            }],
            _ => vec![TaskEvidence {
                kind: TaskEvidenceKind::ScanInferred,
                summary: "Selected from scan graph".to_string(),
                locator: None,
            }],
        };

        let (gotchas, constraints, runbooks) =
            if let Some(node) = graph.nodes.iter().find(|n| n.id == *id) {
                (
                    node.gotchas.clone(),
                    node.operational_constraints.clone(),
                    node.runbooks.clone(),
                )
            } else {
                (
                    baseline.gotchas_by_id.get(id).cloned().unwrap_or_default(),
                    baseline
                        .constraints_by_id
                        .get(id)
                        .cloned()
                        .unwrap_or_default(),
                    baseline.runbooks_by_id.get(id).cloned().unwrap_or_default(),
                )
            };

        out.push(TaskFocusElement {
            element_id: id.clone(),
            kind,
            label,
            lineage,
            evidence,
            gotchas,
            operational_constraints: constraints,
            runbooks,
        });
    }
    out
}

fn compute_lineage(element_id: &str, baseline: &BaselineElements) -> TaskLineage {
    let mut system = None;
    let mut container = None;
    let mut component = None;

    let parts: Vec<&str> = element_id.split('.').collect();
    if !parts.is_empty() {
        let sys = parts[0].to_string();
        if baseline
            .kinds_by_id
            .get(&sys)
            .is_some_and(|k| *k == NodeKind::SYSTEM)
        {
            system = Some(sys);
        }
    }
    for i in (1..parts.len()).rev() {
        let prefix = parts[..=i].join(".");
        if let Some(kind) = baseline.kinds_by_id.get(&prefix) {
            if *kind == NodeKind::CONTAINER && container.is_none() {
                container = Some(prefix.clone());
            }
            if *kind == NodeKind::COMPONENT && component.is_none() {
                component = Some(prefix.clone());
            }
        }
    }

    TaskLineage {
        system,
        container,
        component,
    }
}

#[derive(Debug, Default)]
pub struct ImpactBuckets {
    pub systems: Vec<String>,
    pub containers: Vec<String>,
    pub components: Vec<String>,
}

pub fn expand_neighbors_and_impact(
    graph: &Graph,
    focus_ids: &[String],
    depth: usize,
) -> (Vec<TaskNeighbor>, ImpactBuckets) {
    let mut neighbors = Vec::new();
    let mut impacted = ImpactBuckets::default();
    let mut seen_neighbors: HashSet<(String, String)> = HashSet::new();
    let mut seen_impacted: HashSet<String> = HashSet::new();

    let expansion_depth = depth.clamp(1, 4);

    let kind_for_id = |id: &str| -> NodeKind {
        graph
            .nodes
            .iter()
            .find(|n| n.id == id)
            .map(|n| n.kind.clone())
            .unwrap_or(NodeKind::new(NodeKind::MODULE))
    };

    for id in focus_ids {
        if let Some(node) = graph.nodes.iter().find(|n| n.id == *id) {
            record_impact(
                &mut impacted,
                &mut seen_impacted,
                &node.id,
                node.kind.clone(),
            );
        }

        if graph.nodes.iter().any(|n| n.id == *id) {
            let blast = graph.blast_radius(id, expansion_depth);
            for n in blast.upstream {
                if seen_neighbors.insert((n.id.clone(), "upstream".to_string())) {
                    let kind = kind_for_id(&n.id);
                    neighbors.push(TaskNeighbor {
                        element_id: n.id.clone(),
                        kind: kind.clone(),
                        direction: "upstream".to_string(),
                    });
                    record_impact(&mut impacted, &mut seen_impacted, &n.id, kind);
                }
            }
            for n in blast.downstream {
                if seen_neighbors.insert((n.id.clone(), "downstream".to_string())) {
                    let kind = kind_for_id(&n.id);
                    neighbors.push(TaskNeighbor {
                        element_id: n.id.clone(),
                        kind: kind.clone(),
                        direction: "downstream".to_string(),
                    });
                    record_impact(&mut impacted, &mut seen_impacted, &n.id, kind);
                }
            }
        }
    }

    neighbors.sort_by(|a, b| {
        a.element_id
            .cmp(&b.element_id)
            .then_with(|| a.direction.cmp(&b.direction))
    });
    impacted.systems.sort();
    impacted.systems.dedup();
    impacted.containers.sort();
    impacted.containers.dedup();
    impacted.components.sort();
    impacted.components.dedup();

    (neighbors, impacted)
}

fn record_impact(
    impacted: &mut ImpactBuckets,
    seen: &mut HashSet<String>,
    id: &str,
    kind: NodeKind,
) {
    if !seen.insert(format!("{}::{:?}", id, kind)) {
        return;
    }
    match kind.as_str() {
        NodeKind::SYSTEM => impacted.systems.push(id.to_string()),
        NodeKind::CONTAINER | NodeKind::SERVICE => impacted.containers.push(id.to_string()),
        NodeKind::COMPONENT | NodeKind::MODULE | NodeKind::FRONTEND => {
            impacted.components.push(id.to_string())
        }
        _ => impacted.components.push(id.to_string()),
    }
}

pub fn assemble_sources_and_hydration(
    graph: &Graph,
    repo_root: &str,
    focus_ids: &[String],
    neighbors: &[TaskNeighbor],
    baseline: &BaselineElements,
    max_tokens: usize,
) -> Result<(Vec<TaskSourceBinding>, Vec<TaskHydratedFile>), CliError> {
    let repo_path = Path::new(repo_root);
    let mut bindings = Vec::new();

    for id in focus_ids {
        if let Some(sources) = baseline.sources_by_id.get(id) {
            for s in sources {
                bindings.push(TaskSourceBinding {
                    element_id: id.clone(),
                    source_type: s.kind.as_str().to_string(),
                    path: s.path.clone(),
                    description: s.description.clone(),
                });
            }
        }
        if let Some(node) = graph.nodes.iter().find(|n| n.id == *id) {
            for s in &node.sources {
                bindings.push(TaskSourceBinding {
                    element_id: id.clone(),
                    source_type: s.kind.as_str().to_string(),
                    path: s.path.clone(),
                    description: s.description.clone(),
                });
            }
        }
    }

    let mut file_candidates: Vec<(String, String)> = Vec::new();
    let mut add_node_files = |node_id: &str| {
        if let Some(node) = graph.nodes.iter().find(|n| n.id == node_id) {
            for s in &node.sources {
                file_candidates.push((node_id.to_string(), s.path.clone()));
            }
            if node.sources.is_empty() {
                if let Some(p) = &node.path {
                    file_candidates.push((node_id.to_string(), p.clone()));
                }
            }
        }
    };

    for id in focus_ids {
        add_node_files(id);
    }
    for n in neighbors {
        add_node_files(&n.element_id);
    }

    file_candidates.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
    file_candidates.dedup_by(|a, b| a.1 == b.1);

    let mut hydrated = Vec::new();
    let mut budget = TokenBudget::new(max_tokens);
    for (node_id, rel_path) in file_candidates {
        let full_path = repo_path.join(&rel_path);
        let Ok(content) = fs::read_to_string(&full_path) else {
            continue;
        };
        let remaining_tokens = budget.max_tokens.saturating_sub(budget.used_tokens);
        if remaining_tokens == 0 {
            break;
        }
        let mut truncated = false;
        let max_chars = remaining_tokens.saturating_mul(4);
        let out_content = if TokenBudget::estimate_tokens(&content) > remaining_tokens {
            truncated = true;
            content.chars().take(max_chars).collect::<String>()
        } else {
            content
        };
        let used = TokenBudget::estimate_tokens(&out_content);
        if used > remaining_tokens {
            break;
        }
        budget.used_tokens = budget.used_tokens.saturating_add(used);
        hydrated.push(TaskHydratedFile {
            element_id: node_id,
            path: rel_path,
            content: out_content,
            truncated,
        });
    }

    bindings.sort_by(|a, b| {
        a.element_id
            .cmp(&b.element_id)
            .then_with(|| a.path.cmp(&b.path))
    });
    bindings.dedup_by(|a, b| {
        a.element_id == b.element_id && a.path == b.path && a.source_type == b.source_type
    });

    Ok((bindings, hydrated))
}

pub fn semantic_candidates_from_scan(
    graph: &Graph,
    query: &str,
    top_k: usize,
) -> Vec<TaskSemanticCandidate> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    let terms: Vec<&str> = q.split_whitespace().collect();
    let centrality = sruja_scan::graph::compute_all_centrality(graph);
    let mut scored: Vec<(String, f32, serde_json::Value)> = Vec::new();
    for n in &graph.nodes {
        let id_l = n.id.to_lowercase();
        let label_l = n.label.to_lowercase();
        let hay = format!("{} {}", id_l, label_l);
        let mut exact_hits = 0u32;
        let mut substr_hits = 0u32;
        for t in &terms {
            if t.is_empty() {
                continue;
            }
            let exact = id_l
                .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
                .any(|tok| tok == *t)
                || label_l
                    .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
                    .any(|tok| tok == *t);
            if exact {
                exact_hits += 1;
            } else if hay.contains(t) {
                substr_hits += 1;
            }
        }
        let kind_boost = match n.kind.as_str() {
            NodeKind::SYSTEM => 1.2,
            NodeKind::CONTAINER | NodeKind::SERVICE | NodeKind::DATABASE | NodeKind::QUEUE => 1.0,
            NodeKind::COMPONENT => 0.8,
            _ => 0.2,
        };
        let pr = centrality.get(&n.id).map(|c| c.pagerank).unwrap_or(0.0) as f32;
        let score = (exact_hits as f32) * 2.0 + (substr_hits as f32) * 1.0 + kind_boost + pr * 0.25;

        if exact_hits > 0 || substr_hits > 0 {
            scored.push((
                n.id.clone(),
                score,
                serde_json::json!({
                    "exact_hits": exact_hits,
                    "substring_hits": substr_hits,
                    "kind": format!("{:?}", n.kind),
                    "kind_boost": kind_boost,
                    "pagerank": pr,
                }),
            ));
        }
    }
    scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    scored.truncate(top_k);
    scored
        .into_iter()
        .map(|(id, score, features)| {
            let node = graph.nodes.iter().find(|n| n.id == id);
            TaskSemanticCandidate {
                element_id: id.clone(),
                score,
                label: node.map(|n| n.label.clone()),
                description: node.and_then(|n| n.metadata.get("description").cloned()),
                features: Some(features),
            }
        })
        .collect()
}

pub fn estimate_risk(
    graph: &Graph,
    focus_ids: &[String],
    neighbors: &[TaskNeighbor],
    baseline: &BaselineElements,
) -> TaskRisk {
    let mut critical = false;
    let mut touches_data = false;

    let criticality_rank = |c: sruja_language::ast::Criticality| -> u8 {
        match c {
            sruja_language::ast::Criticality::Low => 0,
            sruja_language::ast::Criticality::Medium => 1,
            sruja_language::ast::Criticality::High => 2,
            sruja_language::ast::Criticality::Critical => 3,
        }
    };

    let mut check_id = |id: &str| {
        if let Some(node) = graph.nodes.iter().find(|n| n.id == id) {
            if node.criticality.is_some_and(|c| {
                criticality_rank(c) >= criticality_rank(sruja_language::ast::Criticality::High)
            }) {
                critical = true;
            }
            if matches!(node.kind.as_str(), NodeKind::DATABASE | NodeKind::QUEUE) {
                touches_data = true;
            }
        }
        if let Some(kind) = baseline.kinds_by_id.get(id) {
            if matches!(kind.as_str(), NodeKind::DATABASE | NodeKind::QUEUE) {
                touches_data = true;
            }
        }
    };

    for id in focus_ids {
        check_id(id);
    }
    for n in neighbors {
        check_id(&n.element_id);
    }

    if critical && touches_data {
        TaskRisk::High
    } else if critical || touches_data {
        TaskRisk::Medium
    } else {
        TaskRisk::Low
    }
}
