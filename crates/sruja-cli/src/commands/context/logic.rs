use super::super::federation::{find_system_index, infer_repo_id, load_system_index};
use super::types::*;
use crate::commands::CliError;
use crate::utils::architecture_path::resolve_architecture_path;
use sruja_language::ElementKind;
use sruja_scan::graph::ComponentImportance;
use sruja_scan::{Graph, NodeKind};
use std::collections::HashMap;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;

pub fn build_architecture_context(
    graph: &Graph,
    repo: &str,
    file: Option<&str>,
    intent: Option<&str>,
    depth: usize,
    max_tokens: usize,
) -> Result<ArchitectureContext, CliError> {
    let modules = count_kind(graph, NodeKind::Module);
    let services = count_kind(graph, NodeKind::Service);
    let databases = count_kind(graph, NodeKind::Database);
    let external_apis = count_kind(graph, NodeKind::ExternalApi);

    let layers = infer_layers(graph);

    let max_boundary_rules = if max_tokens < 3000 { 5 } else { 30 };
    let mut boundaries = infer_boundaries(graph);
    boundaries.truncate(max_boundary_rules);

    let forbidden_patterns = vec![
        "Avoid direct database access from routes/handlers - use a service layer".to_string(),
        "Do not import internal modules from other services directly".to_string(),
        "UI components should not directly call database layers".to_string(),
    ];

    let centrality = sruja_scan::graph::compute_all_centrality(graph);

    let focus = file
        .map(|f| build_focus_context(graph, repo, f, intent, depth, max_tokens, &centrality))
        .transpose()?;

    let system_context = build_system_context(repo);

    let mut active_decisions = Vec::new();
    let adr_dir = Path::new(repo).join("docs").join("architecture").join("decisions");
    if adr_dir.exists() {
        let parser = sruja_intent::AdrParser::new();
        if let Ok(adrs) = parser.parse_dir(&adr_dir) {
            for adr in adrs {
                if adr.status == sruja_intent::AdrStatus::Accepted {
                    active_decisions.push(format!("{}: {}", adr.title, adr.decision));
                }
            }
        }
    }

    Ok(ArchitectureContext {
        repo: repo.to_string(),
        summary: ContextSummary {
            total_modules: modules,
            total_services: services,
            total_databases: databases,
            total_external_apis: external_apis,
        },
        layers,
        boundaries,
        forbidden_patterns,
        active_decisions,
        focus,
        system_context,
        max_tokens,
    })
}

pub fn count_kind(graph: &Graph, kind: NodeKind) -> usize {
    graph.nodes.iter().filter(|n| n.kind == kind).count()
}

pub fn build_system_context(repo_root: &str) -> Option<SystemContext> {
    let root = Path::new(repo_root);
    let current_repo_id = infer_repo_id(root);
    let index_path = find_system_index(root)?;
    let index = load_system_index(&index_path).ok()?;
    let mut cross_repo_elements = Vec::new();
    let mut cross_repo_edges = Vec::new();
    let mut conflicts = Vec::new();

    let current_repo_prefix = format!("{}::", current_repo_id);
    let mut seen_canonical_ids = HashSet::new();
    let mut relevant_canonical_ids = HashSet::new();

    for edge in &index.edges {
        let is_source_current = edge.source.starts_with(&current_repo_prefix);
        let is_target_current = edge.target.starts_with(&current_repo_prefix);

        if is_source_current || is_target_current {
            cross_repo_edges.push(CrossRepoEdge {
                source: edge.source.clone(),
                target: edge.target.clone(),
                label: edge.label.clone(),
                repo_id: edge.repo_id.clone(),
            });

            if !is_source_current {
                relevant_canonical_ids.insert(edge.source.clone());
            }
            if !is_target_current {
                relevant_canonical_ids.insert(edge.target.clone());
            }
        }
    }

    for node in &index.nodes {
        if relevant_canonical_ids.contains(&node.canonical_id)
            && seen_canonical_ids.insert(node.canonical_id.clone())
        {
            cross_repo_elements.push(CrossRepoElement {
                canonical_id: node.canonical_id.clone(),
                kind: node.kind.clone(),
                label: node.label.clone(),
                technology: node.technology.clone(),
                repo_id: node.repo_id.clone(),
                owner: node.owner.clone(),
                domain: node.domain.clone(),
                criticality: node.criticality,
                logical_id: node.logical_id.clone(),
                aliases: node.aliases.clone(),
            });
        }
    }

    for conflict in &index.conflicts {
        if conflict.repos.contains(&current_repo_id) {
            conflicts.push(format!("{}: {}", conflict.key, conflict.message));
        }
    }

    Some(SystemContext {
        index_path: index_path.to_string_lossy().to_string(),
        total_repos: index.repos.len(),
        total_nodes: index.nodes.len(),
        total_edges: index.edges.len(),
        total_conflicts: index.conflicts.len(),
        cross_repo_elements,
        cross_repo_edges,
        conflicts,
    })
}

pub fn build_focus_context(
    graph: &Graph,
    repo_root: &str,
    file: &str,
    intent: Option<&str>,
    depth: usize,
    _max_tokens: usize,
    centrality: &HashMap<String, ComponentImportance>,
) -> Result<FocusContext, CliError> {
    let repo_path = Path::new(repo_root);
    let repo_canon = repo_path
        .canonicalize()
        .unwrap_or_else(|_| repo_path.to_path_buf());

    let requested_path = Path::new(file);
    let absolute = if requested_path.is_absolute() {
        requested_path.to_path_buf()
    } else {
        repo_path.join(requested_path)
    };

    let absolute_canon = absolute.canonicalize().unwrap_or(absolute.clone());
    let rel = absolute_canon
        .strip_prefix(&repo_canon)
        .ok()
        .map(|p| p.to_string_lossy().to_string());

    let absolute_str = absolute.to_string_lossy().to_string();
    let absolute_canon_str = absolute_canon.to_string_lossy().to_string();
    let mut candidates: Vec<String> = vec![absolute_str, absolute_canon_str];
    if let Some(ref r) = rel {
        candidates.push(r.clone());
    }
    for c in &mut candidates {
        *c = normalize_path(c);
    }
    candidates.sort();
    candidates.dedup();

    let mut matched: Vec<&sruja_scan::Node> = graph
        .nodes
        .iter()
        .filter(|n| {
            n.path
                .as_ref()
                .is_some_and(|p| path_matches_any(p, &candidates))
        })
        .collect();

    matched.sort_by(|a, b| {
        let a_score = score_path_match(a.path.as_deref(), &candidates);
        let b_score = score_path_match(b.path.as_deref(), &candidates);

        let a_centrality = centrality.get(&a.id).map(|s| s.pagerank).unwrap_or(0.0);
        let b_centrality = centrality.get(&b.id).map(|s| s.pagerank).unwrap_or(0.0);

        a_score
            .cmp(&b_score)
            .reverse()
            .then_with(|| {
                b_centrality
                    .partial_cmp(&a_centrality)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.id.cmp(&b.id))
    });

    let matched_nodes: Vec<FocusNode> = matched
        .iter()
        .take(10)
        .map(|n| FocusNode {
            id: n.id.clone(),
            kind: n.kind.clone(),
            label: n.label.clone(),
            path: n.path.clone(),
            owner: n.owner.clone(),
            domain: n.domain.clone(),
            criticality: n.criticality,
            gotchas: n.gotchas.clone(),
            operational_constraints: n.operational_constraints.clone(),
            runbooks: n.runbooks.clone(),
        })
        .collect();

    let blast_target = matched
        .iter()
        .find(|n| !n.id.contains('#'))
        .or_else(|| matched.first())
        .map(|n| n.id.as_str());

    let mut blast_radius = blast_target
        .filter(|_| depth > 0)
        .map(|id| graph.blast_radius(id, depth));

    if let Some(ref mut br) = blast_radius {
        let sort_blast = |nodes: &mut Vec<sruja_scan::BlastRadiusNode>| {
            nodes.sort_by(|a, b| {
                let a_c = centrality.get(&a.id).map(|s| s.pagerank).unwrap_or(0.0);
                let b_c = centrality.get(&b.id).map(|s| s.pagerank).unwrap_or(0.0);
                a.depth
                    .cmp(&b.depth)
                    .then_with(|| b_c.partial_cmp(&a_c).unwrap_or(std::cmp::Ordering::Equal))
                    .then_with(|| a.id.cmp(&b.id))
            });
            nodes.truncate(20);
        };
        sort_blast(&mut br.upstream);
        sort_blast(&mut br.downstream);
    }

    let suggested_checks = suggested_checks(intent);

    Ok(FocusContext {
        file: file.to_string(),
        intent: intent.map(|s| s.to_string()),
        depth,
        matched_nodes,
        blast_radius,
        suggested_checks,
    })
}

pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

pub fn path_matches_any(node_path: &str, candidates: &[String]) -> bool {
    let node_norm = normalize_path(node_path);
    if candidates.contains(&node_norm) {
        return true;
    }
    candidates.iter().any(|c| node_norm.ends_with(c))
}

pub fn score_path_match(node_path: Option<&str>, candidates: &[String]) -> usize {
    let Some(p) = node_path else {
        return 0;
    };
    let p_norm = normalize_path(p);
    if candidates.contains(&p_norm) {
        return 3;
    }
    if candidates.iter().any(|c| p_norm.ends_with(c)) {
        return 2;
    }
    1
}

pub fn suggested_checks(intent: Option<&str>) -> Vec<String> {
    let mut checks: Vec<String> = vec![
        "cargo fmt --all".to_string(),
        "cargo clippy -- -D warnings".to_string(),
        "cargo test --workspace".to_string(),
        "sruja drift -r .".to_string(),
    ];

    match intent {
        Some("add-test") => {
            checks.insert(0, "cargo test -p <crate> <test_name>".to_string());
        }
        Some("fix-bug") | Some("refactor") | Some("add-feature") => {
            checks.insert(0, "cargo test --workspace".to_string());
        }
        _ => {}
    }

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    checks
        .into_iter()
        .filter(|c| seen.insert(c.clone()))
        .collect()
}

pub fn infer_layers(graph: &Graph) -> Vec<LayerInfo> {
    let mut layer_counts: HashMap<String, usize> = HashMap::new();

    for node in &graph.nodes {
        if node.kind == NodeKind::Module {
            if let Some(path) = &node.path {
                let layer = infer_layer_from_path(path);
                *layer_counts.entry(layer).or_default() += 1;
            }
        }
    }

    let mut layers: Vec<LayerInfo> = layer_counts
        .into_iter()
        .map(|(name, count): (String, usize)| {
            let can_depend_on = match name.as_str() {
                "api" | "routes" | "handlers" => vec!["services".to_string()],
                "services" => vec!["data".to_string(), "models".to_string()],
                "data" | "db" | "repository" => vec!["models".to_string()],
                "models" | "entities" => vec![],
                "utils" | "lib" | "common" => vec![],
                _ => vec![],
            };
            LayerInfo {
                name,
                modules: count,
                can_depend_on,
            }
        })
        .collect();
    layers.sort_by(|a, b| a.name.cmp(&b.name));
    layers
}

#[derive(Debug, Clone, Copy)]
pub struct TaskSelectors<'a> {
    pub file: Option<&'a str>,
    pub element_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub base_ref: Option<&'a str>,
    pub head_ref: Option<&'a str>,
    pub depth: Option<usize>,
}

pub fn build_task_context(
    graph: &Graph,
    repo_root: &str,
    selectors: TaskSelectors<'_>,
    max_tokens: usize,
) -> Result<TaskContext, CliError> {
    let repo_path = Path::new(repo_root);
    let baseline = load_baseline_elements(repo_path);

    let (focus_ids, selection_reason, truth_status, confidence) =
        resolve_focus(graph, repo_root, selectors, &baseline)?;

    let focus_elements = build_focus_elements(graph, &focus_ids, &baseline, &selection_reason);
    let (neighbors, impacted) =
        expand_neighbors_and_impact(graph, &focus_ids, selectors.depth.unwrap_or(1));
    let (source_bindings, hydrated_files) = assemble_sources_and_hydration(
        graph, repo_root, &focus_ids, &neighbors, &baseline, max_tokens,
    )?;

    let semantic_candidates = if selectors.query.is_some() && focus_ids.is_empty() {
        semantic_candidates_from_scan(graph, selectors.query.unwrap_or_default(), 10)
    } else {
        Vec::new()
    };

    let risk = estimate_risk(graph, &focus_ids, &neighbors, &baseline);

    Ok(TaskContext {
        schema_version: "task_context/v1".to_string(),
        selection_reason,
        focus_elements,
        impacted_systems: impacted.systems,
        impacted_containers: impacted.containers,
        impacted_components: impacted.components,
        neighbors,
        source_bindings,
        hydrated_files,
        risk,
        truth_status,
        confidence,
        semantic_candidates,
    })
}

#[derive(Debug, Default, Clone)]
struct BaselineElements {
    kinds_by_id: BTreeMap<String, NodeKind>,
    labels_by_id: BTreeMap<String, String>,
    sources_by_id: BTreeMap<String, Vec<sruja_language::SourceBinding>>,
    gotchas_by_id: BTreeMap<String, Vec<String>>,
    constraints_by_id: BTreeMap<String, Vec<String>>,
    runbooks_by_id: BTreeMap<String, Vec<String>>,
}

fn load_baseline_elements(repo_root: &Path) -> BaselineElements {
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
                out.runbooks_by_id.insert(fqn.clone(), body.runbooks.clone());
            }
        }
    }
    out
}

fn element_kind_to_node_kind(kind: &ElementKind) -> NodeKind {
    match kind {
        ElementKind::System => NodeKind::System,
        ElementKind::Container => NodeKind::Container,
        ElementKind::Component => NodeKind::Component,
        ElementKind::Queue => NodeKind::Queue,
        ElementKind::Database | ElementKind::DataStore => NodeKind::Database,
        ElementKind::ExternalSystem => NodeKind::ExternalApi,
        _ => NodeKind::Module,
    }
}

fn resolve_focus(
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
        let centrality = sruja_scan::graph::compute_all_centrality(graph);
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
        match kind {
            sruja_scan::NodeKind::System | sruja_scan::NodeKind::ExternalApi => {
                overview_ids.push(id.clone());
            }
            sruja_scan::NodeKind::Container | sruja_scan::NodeKind::Database | sruja_scan::NodeKind::Queue => {
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
                node.kind,
                sruja_scan::NodeKind::System
                    | sruja_scan::NodeKind::Service
                    | sruja_scan::NodeKind::ExternalApi
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

fn build_focus_elements(
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
            (NodeKind::Module, Some(id.clone()))
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

        let (gotchas, constraints, runbooks) = if let Some(node) = graph.nodes.iter().find(|n| n.id == *id) {
            (
                node.gotchas.clone(),
                node.operational_constraints.clone(),
                node.runbooks.clone(),
            )
        } else {
            (
                baseline.gotchas_by_id.get(id).cloned().unwrap_or_default(),
                baseline.constraints_by_id.get(id).cloned().unwrap_or_default(),
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
            .is_some_and(|k| *k == NodeKind::System)
        {
            system = Some(sys);
        }
    }
    for i in (1..parts.len()).rev() {
        let prefix = parts[..=i].join(".");
        if let Some(kind) = baseline.kinds_by_id.get(&prefix) {
            if *kind == NodeKind::Container && container.is_none() {
                container = Some(prefix.clone());
            }
            if *kind == NodeKind::Component && component.is_none() {
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
struct ImpactBuckets {
    systems: Vec<String>,
    containers: Vec<String>,
    components: Vec<String>,
}

fn expand_neighbors_and_impact(
    graph: &Graph,
    focus_ids: &[String],
    depth: usize,
) -> (Vec<TaskNeighbor>, ImpactBuckets) {
    let mut neighbors = Vec::new();
    let mut impacted = ImpactBuckets::default();
    let mut seen_neighbors: HashSet<(String, String)> = HashSet::new();
    let mut seen_impacted: HashSet<String> = HashSet::new();

    // Use current depth for resolution
    let expansion_depth = depth.clamp(1, 4);

    let kind_for_id = |id: &str| -> NodeKind {
        graph
            .nodes
            .iter()
            .find(|n| n.id == id)
            .map(|n| n.kind.clone())
            .unwrap_or(NodeKind::Module)
    };

    for id in focus_ids {
        if let Some(node) = graph.nodes.iter().find(|n| n.id == *id) {
            record_impact(&mut impacted, &mut seen_impacted, &node.id, node.kind.clone());
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
    match kind {
        NodeKind::System => impacted.systems.push(id.to_string()),
        NodeKind::Container | NodeKind::Service => impacted.containers.push(id.to_string()),
        NodeKind::Component | NodeKind::Module | NodeKind::Frontend => {
            impacted.components.push(id.to_string())
        }
        _ => impacted.components.push(id.to_string()),
    }
}

fn assemble_sources_and_hydration(
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

fn semantic_candidates_from_scan(
    graph: &Graph,
    query: &str,
    top_k: usize,
) -> Vec<TaskSemanticCandidate> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    let terms: Vec<&str> = q.split_whitespace().collect();
    let mut scored: Vec<(String, f32)> = Vec::new();
    for n in &graph.nodes {
        let hay = format!("{} {}", n.id.to_lowercase(), n.label.to_lowercase());
        let mut score = 0f32;
        for t in &terms {
            if hay.contains(t) {
                score += 1.0;
            }
        }
        if score > 0.0 {
            scored.push((n.id.clone(), score));
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
        .map(|(id, score)| {
            let node = graph.nodes.iter().find(|n| n.id == id);
            TaskSemanticCandidate {
                element_id: id.clone(),
                score,
                label: node.map(|n| n.label.clone()),
                description: node.and_then(|n| n.metadata.get("description").cloned()),
            }
        })
        .collect()
}

fn estimate_risk(
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
            if matches!(node.kind, NodeKind::Database | NodeKind::Queue) {
                touches_data = true;
            }
        }
        if let Some(kind) = baseline.kinds_by_id.get(id) {
            if matches!(kind, NodeKind::Database | NodeKind::Queue) {
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

pub fn infer_layer_from_path(path: &str) -> String {
    let path_lower = path.to_lowercase();
    let parts: Vec<&str> = path_lower.split('/').collect();

    for part in &parts {
        match *part {
            "api" | "routes" | "handlers" | "controllers" | "endpoints" => {
                return "api".to_string()
            }
            "services" | "service" => return "services".to_string(),
            "data" | "db" | "database" | "repository" | "repos" | "dal" => {
                return "data".to_string()
            }
            "models" | "model" | "entities" | "entity" | "domain" => return "models".to_string(),
            "utils" | "lib" | "common" | "shared" | "helpers" => return "utils".to_string(),
            "components" | "ui" | "views" | "pages" => return "ui".to_string(),
            _ => {}
        }
    }

    "other".to_string()
}

pub fn infer_boundaries(graph: &Graph) -> Vec<BoundaryRule> {
    let mut boundaries = Vec::new();

    let mut services: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Service)
        .collect();
    services.sort_by(|a, b| a.id.cmp(&b.id));

    if services.len() > 1 {
        for s1 in &services {
            for s2 in &services {
                if s1.id != s2.id {
                    boundaries.push(BoundaryRule {
                        from: s1.id.clone(),
                        to: s2.id.clone(),
                        allowed: false,
                        reason: "Services should communicate via APIs/events, not direct imports"
                            .to_string(),
                    });
                }
            }
        }
    }

    boundaries.push(BoundaryRule {
        from: "ui".to_string(),
        to: "data".to_string(),
        allowed: false,
        reason: "UI should not directly access data layer - use services".to_string(),
    });

    boundaries
}
