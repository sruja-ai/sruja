//! Architecture graph diffing and comparison.
//!
//! This crate provides functionality to compare architecture graphs,
//! identifying differences, new components, missing elements, and potential violations.

use serde::{Deserialize, Serialize};
use sruja_language::traversal::collect_elements;
use sruja_language::{ElementKind, Program};
use sruja_scan::{Edge, EdgeEvidence, EdgeKind, Graph, Node, NodeKind};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

fn edge_evidence_to_source_refs(evidence: &[EdgeEvidence]) -> Vec<SourceRef> {
    let mut refs = Vec::new();
    for ev in evidence {
        if ev.file.is_some() || ev.detail.is_some() {
            refs.push(SourceRef {
                file: ev.file.clone(),
                line: ev.line,
                detail: ev.detail.clone(),
            });
        }
    }
    refs
}

/// Collect unique source refs for edges that form a cycle (consecutive pairs in cycle).
fn collect_cycle_sources(graph: &Graph, cycle: &[String]) -> Vec<SourceRef> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for w in cycle.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        for edge in &graph.edges {
            if edge.source == *a && edge.target == *b {
                for ev in &edge.evidence {
                    let key = (ev.file.clone(), ev.line);
                    if seen.insert(key) {
                        out.push(SourceRef {
                            file: ev.file.clone(),
                            line: ev.line,
                            detail: ev.detail.clone(),
                        });
                    }
                }
                break;
            }
        }
    }
    if cycle.len() > 1 {
        let (last, first) = (cycle.last().unwrap(), cycle.first().unwrap());
        for edge in &graph.edges {
            if edge.source == *last && edge.target == *first {
                for ev in &edge.evidence {
                    let key = (ev.file.clone(), ev.line);
                    if seen.insert(key) {
                        out.push(SourceRef {
                            file: ev.file.clone(),
                            line: ev.line,
                            detail: ev.detail.clone(),
                        });
                    }
                }
                break;
            }
        }
    }
    out
}

fn collect_edge_sources(graph: &Graph, source: &str, target: &str) -> Vec<SourceRef> {
    for edge in &graph.edges {
        if edge.source == source && edge.target == target {
            return edge_evidence_to_source_refs(&edge.evidence);
        }
    }
    Vec::new()
}

fn collect_node_path_source(graph: &Graph, node_id: &str) -> Vec<SourceRef> {
    for node in &graph.nodes {
        if node.id == node_id {
            if let Some(ref path) = node.path {
                return vec![SourceRef {
                    file: Some(path.clone()),
                    line: None,
                    detail: None,
                }];
            }
            return Vec::new();
        }
    }
    Vec::new()
}

/// Convert a DSL Program to sruja_scan::Graph for comparison with scanned architecture.
pub fn program_to_graph(program: &Program) -> Graph {
    let (elements, relations) = collect_elements(program);
    let mut nodes = Vec::with_capacity(elements.len());
    let mut edges = Vec::with_capacity(relations.len());

    for (fqn, elem) in &elements {
        let a = &elem.assignment;
        let kind = element_kind_to_node_kind(&a.kind);
        let label = a
            .title
            .as_deref()
            .unwrap_or(&a.name)
            .to_string();
        let technology = a
            .body
            .as_ref()
            .and_then(|b| b.technology.as_ref())
            .cloned();

        nodes.push(Node {
            id: fqn.clone(),
            kind,
            label,
            technology,
            path: None,
            metadata: HashMap::new(),
        });
    }

    for rel in &relations {
        let from_id = rel.from.as_string();
        let to_id = rel.to.as_string();
        if elements.contains_key(&from_id) && elements.contains_key(&to_id) {
            let kind = relation_label_to_edge_kind(rel.label.as_deref().unwrap_or("calls"));
            edges.push(Edge {
                source: from_id,
                target: to_id,
                kind,
                evidence: vec![EdgeEvidence {
                    rule: "dsl".to_string(),
                    file: None,
                    line: None,
                    detail: rel.label.clone(),
                }],
            });
        }
    }

    Graph {
        metadata: HashMap::new(),
        nodes,
        edges,
    }
}

fn element_kind_to_node_kind(kind: &ElementKind) -> NodeKind {
    match kind {
        ElementKind::Database | ElementKind::DataStore => NodeKind::Database,
        ElementKind::ExternalSystem => NodeKind::ExternalApi,
        ElementKind::Person
        | ElementKind::Role
        | ElementKind::System
        | ElementKind::Container
        | ElementKind::Component
        | ElementKind::Queue
        | ElementKind::Policy
        | ElementKind::Requirement
        | ElementKind::Adr
        | ElementKind::Flow
        | ElementKind::Scenario
        | ElementKind::Story
        | ElementKind::Custom(_) => NodeKind::Module,
    }
}

fn relation_label_to_edge_kind(label: &str) -> EdgeKind {
    let lower = label.to_lowercase();
    if lower.contains("read") || lower == "reads" {
        EdgeKind::ReadsFrom
    } else if lower.contains("write") || lower == "writes" {
        EdgeKind::WritesTo
    } else {
        EdgeKind::Calls
    }
}

#[derive(Debug, Error)]
pub enum DiffError {
    #[error("Graph comparison error: {0}")]
    Comparison(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeMatch {
    pub proposal_id: String,
    pub actual_id: String,
    pub similarity: f32,
    pub kind_match: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeDiff {
    pub added: Vec<DiffNode>,
    pub removed: Vec<DiffNode>,
    pub matched: Vec<NodeMatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeDiff {
    pub added: Vec<DiffEdge>,
    pub removed: Vec<DiffEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffNode {
    pub id: String,
    pub kind: NodeKind,
    pub label: String,
    pub technology: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffEdge {
    pub source: String,
    pub target: String,
    pub kind: EdgeKind,
    pub label: Option<String>,
}

/// Reference to a source location (file, line) for evidence in reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SourceRef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl SourceRef {
    /// Format as a short reference string, e.g. `path/to/file.ts:42` or `path/to/file.ts`.
    #[must_use]
    pub fn display_string(&self) -> String {
        match (&self.file, self.line) {
            (Some(f), Some(l)) => format!("{}:{}", f, l),
            (Some(f), None) => f.clone(),
            _ => self.detail.clone().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Violation {
    pub kind: ViolationKind,
    pub severity: Severity,
    pub message: String,
    pub location: Option<String>,
    pub suggestion: Option<String>,
    /// Source references (file, line) so findings can be traced back to code or docs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<SourceRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationKind {
    LayerViolation,
    MissingDependency,
    OrphanComponent,
    CircularDependency,
    UndocumentedComponent,
    PatternMismatch,
    GodModule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffResult {
    pub proposal_title: String,
    pub node_diff: NodeDiff,
    pub edge_diff: EdgeDiff,
    pub violations: Vec<Violation>,
    pub suggestions: Vec<String>,
    pub summary: DiffSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffSummary {
    pub proposed_components: usize,
    pub existing_components: usize,
    pub new_components: usize,
    pub missing_components: usize,
    pub new_dependencies: usize,
    pub removed_dependencies: usize,
    pub health_score: u8,
}

impl DiffResult {
    pub fn is_empty(&self) -> bool {
        self.node_diff.added.is_empty()
            && self.node_diff.removed.is_empty()
            && self.edge_diff.added.is_empty()
            && self.edge_diff.removed.is_empty()
    }

    pub fn has_issues(&self) -> bool {
        !self.violations.is_empty() || !self.node_diff.removed.is_empty()
    }
}

pub fn compare_graphs(actual: &Graph, proposed: &Graph) -> DiffResult {
    let node_diff = compare_nodes(&actual.nodes, &proposed.nodes);
    let edge_diff = compare_edges(&actual.edges, &proposed.edges);
    let violations = detect_violations(actual, proposed, &node_diff, &edge_diff);
    let suggestions = generate_suggestions(&node_diff, &edge_diff, &violations);

    let summary = DiffSummary {
        proposed_components: proposed.nodes.len(),
        existing_components: actual.nodes.len(),
        new_components: node_diff.added.len(),
        missing_components: node_diff.removed.len(),
        new_dependencies: edge_diff.added.len(),
        removed_dependencies: edge_diff.removed.len(),
        health_score: calculate_health_score(&node_diff, &edge_diff, &violations),
    };

    DiffResult {
        proposal_title: "Architecture Comparison".to_string(),
        node_diff,
        edge_diff,
        violations,
        suggestions,
        summary,
    }
}

fn compare_nodes(actual: &[Node], proposed: &[Node]) -> NodeDiff {
    let actual_ids: HashSet<&str> = actual.iter().map(|n| n.id.as_str()).collect();
    let proposed_ids: HashSet<&str> = proposed.iter().map(|n| n.id.as_str()).collect();

    let added: Vec<DiffNode> = proposed
        .iter()
        .filter(|n| !actual_ids.contains(n.id.as_str()))
        .map(|n| DiffNode {
            id: n.id.clone(),
            kind: n.kind,
            label: n.label.clone(),
            technology: n.technology.clone(),
            description: None,
        })
        .collect();

    let removed: Vec<DiffNode> = actual
        .iter()
        .filter(|n| !proposed_ids.contains(n.id.as_str()))
        .map(|n| DiffNode {
            id: n.id.clone(),
            kind: n.kind,
            label: n.label.clone(),
            technology: n.technology.clone(),
            description: None,
        })
        .collect();

    let matched: Vec<NodeMatch> = proposed
        .iter()
        .filter(|n| actual_ids.contains(n.id.as_str()))
        .map(|pn| {
            let actual_node = actual.iter().find(|an| an.id == pn.id).unwrap();
            NodeMatch {
                proposal_id: pn.id.clone(),
                actual_id: actual_node.id.clone(),
                similarity: calculate_similarity(&pn.label, &actual_node.label),
                kind_match: pn.kind == actual_node.kind,
            }
        })
        .collect();

    NodeDiff {
        added,
        removed,
        matched,
    }
}

fn compare_edges(actual: &[Edge], proposed: &[Edge]) -> EdgeDiff {
    let actual_set: HashSet<(String, String, EdgeKind)> = actual
        .iter()
        .map(|e| (e.source.clone(), e.target.clone(), e.kind))
        .collect();

    let proposed_set: HashSet<(String, String, EdgeKind)> = proposed
        .iter()
        .map(|e| (e.source.clone(), e.target.clone(), e.kind))
        .collect();

    let added: Vec<DiffEdge> = proposed
        .iter()
        .filter(|e| !actual_set.contains(&(e.source.clone(), e.target.clone(), e.kind)))
        .map(|e| DiffEdge {
            source: e.source.clone(),
            target: e.target.clone(),
            kind: e.kind,
            label: None,
        })
        .collect();

    let removed: Vec<DiffEdge> = actual
        .iter()
        .filter(|e| !proposed_set.contains(&(e.source.clone(), e.target.clone(), e.kind)))
        .map(|e| DiffEdge {
            source: e.source.clone(),
            target: e.target.clone(),
            kind: e.kind,
            label: None,
        })
        .collect();

    EdgeDiff { added, removed }
}

fn detect_violations(
    actual: &Graph,
    proposed: &Graph,
    node_diff: &NodeDiff,
    edge_diff: &EdgeDiff,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    for edge in &edge_diff.added {
        let source = proposed.nodes.iter().find(|n| n.id == edge.source);
        let target = proposed.nodes.iter().find(|n| n.id == edge.target);

        if let (Some(src), Some(tgt)) = (source, target) {
            if src.kind == NodeKind::Module && tgt.kind == NodeKind::Database {
                let sources = collect_edge_sources(actual, &edge.source, &edge.target);
                violations.push(Violation {
                    kind: ViolationKind::LayerViolation,
                    severity: Severity::Warning,
                    message: format!(
                        "Direct database access from '{}' - consider adding a service layer",
                        src.label
                    ),
                    location: Some(format!("{} -> {}", edge.source, edge.target)),
                    suggestion: Some(format!(
                        "Add a data access service between {} and {}",
                        src.label, tgt.label
                    )),
                    sources,
                });
            }
        }
    }

    for node in &node_diff.added {
        let has_incoming = proposed.edges.iter().any(|e| e.target == node.id);
        let has_outgoing = proposed.edges.iter().any(|e| e.source == node.id);

        if !has_incoming && !has_outgoing {
            let sources = collect_node_path_source(actual, &node.id);
            violations.push(Violation {
                kind: ViolationKind::OrphanComponent,
                severity: Severity::Warning,
                message: format!("Component '{}' has no connections", node.label),
                location: Some(node.id.clone()),
                suggestion: Some(format!(
                    "Define how '{}' interacts with other components",
                    node.label
                )),
                sources,
            });
        }
    }

    for node in &node_diff.added {
        if node.kind == NodeKind::Service && node.technology.is_none() {
            let sources = collect_node_path_source(actual, &node.id);
            violations.push(Violation {
                kind: ViolationKind::UndocumentedComponent,
                severity: Severity::Info,
                message: format!("Service '{}' has no technology specified", node.label),
                location: Some(node.id.clone()),
                suggestion: Some(format!(
                    "Specify the technology for '{}' (e.g., Node.js, Go, Python)",
                    node.label
                )),
                sources,
            });
        }
    }

    violations
}

fn generate_suggestions(
    node_diff: &NodeDiff,
    edge_diff: &EdgeDiff,
    violations: &[Violation],
) -> Vec<String> {
    let mut suggestions = Vec::new();

    if !node_diff.added.is_empty() {
        let db_added: Vec<_> = node_diff
            .added
            .iter()
            .filter(|n| n.kind == NodeKind::Database)
            .collect();
        if !db_added.is_empty() {
            suggestions.push(format!(
                "Consider data migration strategy for new database(s): {}",
                db_added
                    .iter()
                    .map(|n| n.label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    for violation in violations {
        if let Some(ref sugg) = violation.suggestion {
            suggestions.push(sugg.clone());
        }
    }

    if !edge_diff.added.is_empty() {
        let external_edges: Vec<_> = edge_diff
            .added
            .iter()
            .filter(|e| e.kind == EdgeKind::Calls)
            .collect();
        if !external_edges.is_empty() {
            suggestions.push(
                "Add error handling and retry logic for new synchronous dependencies".to_string(),
            );
        }
    }

    suggestions.sort();
    suggestions.dedup();
    suggestions
}

fn calculate_similarity(a: &str, b: &str) -> f32 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    if a_lower == b_lower {
        return 1.0;
    }

    let a_words: HashSet<&str> = a_lower.split_whitespace().collect();
    let b_words: HashSet<&str> = b_lower.split_whitespace().collect();

    if a_words.is_empty() || b_words.is_empty() {
        return 0.0;
    }

    let intersection = a_words.intersection(&b_words).count();
    let union = a_words.union(&b_words).count();

    intersection as f32 / union as f32
}

/// Penalties applied per violation severity when computing health score.
#[derive(Debug, Clone, Copy)]
pub struct HealthScorePenalties {
    pub error: u8,
    pub warning: u8,
    pub info: u8,
}

impl Default for HealthScorePenalties {
    fn default() -> Self {
        Self {
            error: 15,
            warning: 5,
            info: 2,
        }
    }
}

/// Compute health score from violations using unified penalty scheme.
pub fn calculate_health_score_from_violations(
    violations: &[Violation],
    penalties: HealthScorePenalties,
) -> u8 {
    let mut score: u8 = 100;
    for v in violations {
        match v.severity {
            Severity::Error => score = score.saturating_sub(penalties.error),
            Severity::Warning => score = score.saturating_sub(penalties.warning),
            Severity::Info => score = score.saturating_sub(penalties.info),
        }
    }
    score
}

fn calculate_health_score(
    node_diff: &NodeDiff,
    _edge_diff: &EdgeDiff,
    violations: &[Violation],
) -> u8 {
    let mut score =
        calculate_health_score_from_violations(violations, HealthScorePenalties::default());

    let orphan_penalty = node_diff
        .added
        .iter()
        .filter(|n| {
            violations.iter().any(|v| {
                v.kind == ViolationKind::OrphanComponent && v.location.as_deref() == Some(&n.id)
            })
        })
        .count() as u8;
    score = score.saturating_sub(orphan_penalty * 3);

    score
}

/// Configuration for architectural drift detection.
#[derive(Debug, Clone)]
pub struct DriftConfig {
    /// Minimum number of dependencies before a module is flagged as a god module.
    pub god_module_threshold: usize,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            god_module_threshold: 10,
        }
    }
}

/// Detect architectural drift in a codebase by analyzing the scanned graph
/// for common architectural issues like circular dependencies, god modules,
/// layer violations, and orphan components.
pub fn detect_architectural_drift(graph: &Graph) -> DriftReport {
    detect_architectural_drift_with_config(graph, &DriftConfig::default())
}

/// Detect architectural drift with custom configuration.
pub fn detect_architectural_drift_with_config(
    graph: &Graph,
    config: &DriftConfig,
) -> DriftReport {
    let mut violations = Vec::new();
    let mut suggestions = Vec::new();

    // Detect circular dependencies
    let circular = find_circular_dependencies(graph);
    for cycle in &circular {
        let sources = collect_cycle_sources(graph, cycle);
        violations.push(Violation {
            kind: ViolationKind::CircularDependency,
            severity: Severity::Error,
            message: format!("Circular dependency detected: {}", cycle.join(" -> ")),
            location: Some(cycle.first().cloned().unwrap_or_default()),
            suggestion: Some(
                "Consider introducing an interface or event-based communication to break the cycle"
                    .to_string(),
            ),
            sources,
        });
    }

    // Detect orphan modules
    let orphans = find_orphan_modules(graph);
    for orphan in &orphans {
        let sources = collect_node_path_source(graph, orphan);
        violations.push(Violation {
            kind: ViolationKind::OrphanComponent,
            severity: Severity::Warning,
            message: format!("Module '{}' has no incoming or outgoing dependencies", orphan),
            location: Some(orphan.clone()),
            suggestion: Some(
                "Consider if this module is still needed or if it should be connected to the rest of the system".to_string(),
            ),
            sources,
        });
    }

    // Detect layer violations
    let layer_violations = find_layer_violations_advanced(graph);
    for violation in &layer_violations {
        let sources = collect_edge_sources(graph, &violation.source, &violation.target);
        violations.push(Violation {
            kind: ViolationKind::LayerViolation,
            severity: Severity::Warning,
            message: format!(
                "Layer violation: '{}' directly accesses '{}'",
                violation.source, violation.target
            ),
            location: Some(format!("{} -> {}", violation.source, violation.target)),
            suggestion: Some(
                "Consider adding a service layer to abstract this dependency".to_string(),
            ),
            sources,
        });
    }

    // Detect god modules
    let god_modules = find_god_modules(graph, config.god_module_threshold);
    for module in &god_modules {
        let sources = collect_node_path_source(graph, &module.name);
        violations.push(Violation {
            kind: ViolationKind::GodModule,
            severity: Severity::Info,
            message: format!(
                "Module '{}' has {} dependencies (threshold: {})",
                module.name,
                module.dependency_count,
                config.god_module_threshold
            ),
            location: Some(module.name.clone()),
            suggestion: Some(
                "Consider splitting this module into smaller, focused components".to_string(),
            ),
            sources,
        });
    }

    // Generate suggestions
    if !circular.is_empty() {
        suggestions.push("Fix circular dependencies to improve maintainability".to_string());
    }
    if !orphans.is_empty() {
        suggestions
            .push("Review orphan modules - they may be dead code or need integration".to_string());
    }
    if !layer_violations.is_empty() {
        suggestions.push("Introduce proper layering to reduce coupling".to_string());
    }
    if !god_modules.is_empty() {
        suggestions.push("Refactor god modules into smaller components".to_string());
    }

    let health_score = calculate_drift_health_score(&violations);

    DriftReport {
        total_modules: graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Module)
            .count(),
        total_services: graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Service)
            .count(),
        total_databases: graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Database)
            .count(),
        total_dependencies: graph.edges.len(),
        circular_dependencies: circular.len(),
        orphan_modules: orphans.len(),
        layer_violations: layer_violations.len(),
        violations,
        suggestions,
        health_score,
    }
}

/// Result of architectural drift detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub total_modules: usize,
    pub total_services: usize,
    pub total_databases: usize,
    pub total_dependencies: usize,
    pub circular_dependencies: usize,
    pub orphan_modules: usize,
    pub layer_violations: usize,
    pub violations: Vec<Violation>,
    pub suggestions: Vec<String>,
    pub health_score: u8,
}

/// Find circular dependencies in the graph using DFS.
/// Returns deduplicated cycles (canonicalized by lexicographically smallest rotation).
pub fn find_circular_dependencies(graph: &Graph) -> Vec<Vec<String>> {
    use std::collections::{HashMap, HashSet};

    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &graph.edges {
        adj.entry(&edge.source).or_default().push(&edge.target);
    }

    let mut raw_cycles: Vec<Vec<&str>> = Vec::new();
    let mut visited: HashSet<&str> = HashSet::new();
    let mut rec_stack: HashSet<&str> = HashSet::new();
    let mut path: Vec<&str> = Vec::new();

    for node in &graph.nodes {
        if !visited.contains(node.id.as_str()) {
            dfs_cycles(
                node.id.as_str(),
                &adj,
                &mut visited,
                &mut rec_stack,
                &mut path,
                &mut raw_cycles,
            );
        }
    }

    // Canonicalize and deduplicate: use lexicographically smallest rotation as canonical form
    let mut seen: HashSet<Vec<String>> = HashSet::new();
    let mut result = Vec::new();
    for cycle in raw_cycles {
        let canonical = canonicalize_cycle(&cycle);
        if seen.insert(canonical.clone()) {
            result.push(canonical);
        }
    }
    result
}

/// Return the lexicographically smallest rotation of the cycle.
fn canonicalize_cycle(cycle: &[&str]) -> Vec<String> {
    if cycle.is_empty() {
        return Vec::new();
    }
    let n = cycle.len();
    let mut best_start = 0;
    for start in 1..n {
        for i in 0..n {
            let a = cycle[(best_start + i) % n];
            let b = cycle[(start + i) % n];
            match a.cmp(b) {
                std::cmp::Ordering::Less => break,
                std::cmp::Ordering::Greater => {
                    best_start = start;
                    break;
                }
                std::cmp::Ordering::Equal => {}
            }
        }
    }
    (0..n)
        .map(|i| cycle[(best_start + i) % n].to_string())
        .collect()
}

fn dfs_cycles<'a>(
    node: &'a str,
    adj: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    rec_stack: &mut HashSet<&'a str>,
    path: &mut Vec<&'a str>,
    cycles: &mut Vec<Vec<&'a str>>,
) {
    visited.insert(node);
    rec_stack.insert(node);
    path.push(node);

    if let Some(neighbors) = adj.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                dfs_cycles(neighbor, adj, visited, rec_stack, path, cycles);
            } else if rec_stack.contains(neighbor) {
                if let Some(cycle_start) = path.iter().position(|n| *n == *neighbor) {
                    let cycle: Vec<&'a str> = path[cycle_start..].to_vec();
                    cycles.push(cycle);
                }
            }
        }
    }

    path.pop();
    rec_stack.remove(node);
}

/// Find modules with no incoming or outgoing dependency edges.
/// Excludes containment edges (module:->file) so we detect truly disconnected modules.
pub fn find_orphan_modules(graph: &Graph) -> Vec<String> {
    let mut has_incoming: HashSet<&str> = HashSet::new();
    let mut has_outgoing: HashSet<&str> = HashSet::new();

    for edge in &graph.edges {
        // Exclude containment edges (source is a directory module like "module:root")
        if !edge.source.starts_with("module:") {
            has_outgoing.insert(edge.source.as_str());
            has_incoming.insert(edge.target.as_str());
        }
    }

    graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Module)
        .filter(|n| !n.id.starts_with("module:") && !n.id.contains('#')) // Only file-level modules, not directory or export nodes
        .filter(|n| !has_incoming.contains(n.id.as_str()) && !has_outgoing.contains(n.id.as_str()))
        .map(|n| n.id.clone())
        .collect()
}

/// Layer violation detection helper
struct LayerViolationInfo {
    source: String,
    target: String,
}

/// Find layer violations (e.g., frontend directly accessing database)
fn find_layer_violations_advanced(graph: &Graph) -> Vec<LayerViolationInfo> {
    let mut violations = Vec::new();

    let db_nodes: HashSet<&str> = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Database)
        .map(|n| n.id.as_str())
        .collect();

    let frontend_nodes: HashSet<&str> = graph
        .nodes
        .iter()
        .filter(|n| {
            n.label.contains("frontend") || n.label.contains("ui") || n.label.contains("web")
        })
        .map(|n| n.id.as_str())
        .collect();

    for edge in &graph.edges {
        if frontend_nodes.contains(edge.source.as_str()) && db_nodes.contains(edge.target.as_str())
        {
            violations.push(LayerViolationInfo {
                source: edge.source.clone(),
                target: edge.target.clone(),
            });
        }
    }

    violations
}

/// God module detection helper
struct GodModuleInfo {
    name: String,
    dependency_count: usize,
}

/// Find modules with too many dependencies
fn find_god_modules(graph: &Graph, threshold: usize) -> Vec<GodModuleInfo> {
    use std::collections::HashMap;

    let mut dep_counts: HashMap<&str, usize> = HashMap::new();

    for edge in &graph.edges {
        *dep_counts.entry(&edge.source).or_default() += 1;
    }

    graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Module)
        .filter_map(|n| {
            let count = dep_counts.get(n.id.as_str()).copied().unwrap_or(0);
            if count >= threshold {
                Some(GodModuleInfo {
                    name: n.id.clone(),
                    dependency_count: count,
                })
            } else {
                None
            }
        })
        .collect()
}

fn calculate_drift_health_score(violations: &[Violation]) -> u8 {
    calculate_health_score_from_violations(violations, HealthScorePenalties::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_scan::Graph;
    use std::collections::HashMap;

    fn make_node(id: &str, kind: NodeKind, label: &str) -> Node {
        Node {
            id: id.to_string(),
            kind,
            label: label.to_string(),
            technology: None,
            path: None,
            metadata: HashMap::new(),
        }
    }

    fn make_edge(source: &str, target: &str, kind: EdgeKind) -> Edge {
        Edge {
            source: source.to_string(),
            target: target.to_string(),
            kind,
            evidence: Vec::new(),
        }
    }

    #[test]
    fn test_compare_empty_graphs() {
        let actual = Graph::new();
        let proposed = Graph::new();
        let result = compare_graphs(&actual, &proposed);
        assert!(result.is_empty());
    }

    #[test]
    fn test_detect_added_node() {
        let actual = Graph::new();
        let mut proposed = Graph::new();
        proposed
            .nodes
            .push(make_node("api", NodeKind::Service, "API"));

        let result = compare_graphs(&actual, &proposed);
        assert_eq!(result.node_diff.added.len(), 1);
        assert_eq!(result.node_diff.added[0].id, "api");
    }

    #[test]
    fn test_detect_removed_node() {
        let mut actual = Graph::new();
        actual
            .nodes
            .push(make_node("old", NodeKind::Service, "Old Service"));
        let proposed = Graph::new();

        let result = compare_graphs(&actual, &proposed);
        assert_eq!(result.node_diff.removed.len(), 1);
    }

    #[test]
    fn test_detect_layer_violation() {
        let actual = Graph::new();
        let mut proposed = Graph::new();

        proposed
            .nodes
            .push(make_node("frontend", NodeKind::Module, "Frontend"));
        proposed
            .nodes
            .push(make_node("db", NodeKind::Database, "Database"));
        proposed
            .edges
            .push(make_edge("frontend", "db", EdgeKind::ReadsFrom));

        let result = compare_graphs(&actual, &proposed);
        assert!(result
            .violations
            .iter()
            .any(|v| v.kind == ViolationKind::LayerViolation));
    }

    #[test]
    fn test_detect_architectural_drift_cycle_and_orphan() {
        let mut graph = Graph::new();
        graph.nodes.push(make_node("a", NodeKind::Module, "A"));
        graph.nodes.push(make_node("b", NodeKind::Module, "B"));
        graph.nodes.push(make_node("c", NodeKind::Module, "C"));
        graph.nodes.push(make_node("orphan", NodeKind::Module, "Orphan"));
        graph.edges.push(make_edge("a", "b", EdgeKind::Calls));
        graph.edges.push(make_edge("b", "c", EdgeKind::Calls));
        graph.edges.push(make_edge("c", "a", EdgeKind::Calls));

        let report = detect_architectural_drift(&graph);

        assert!(report.violations.iter().any(|v| {
            v.kind == ViolationKind::CircularDependency
        }));
        assert!(report.violations.iter().any(|v| {
            v.kind == ViolationKind::OrphanComponent
        }));
        assert!(report.health_score <= 100);
        assert_eq!(report.total_modules, 4);
        assert!(!report.suggestions.is_empty());
    }
}
