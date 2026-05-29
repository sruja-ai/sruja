//! Discovery commands: question bank for intelligent architecture capture.
//!
//! Use with the sruja-architecture skill so the AI asks users these questions
//! before or during discovery. See skills/sruja-architecture/REFERENCE.md.

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

use sruja_scan::graph::compute_all_centrality;
use sruja_scan::scan_scope::resolve_scan_scope;
use sruja_scan::{generate_repomap_from_graph, EdgeKind, Graph, NodeKind, RepoMapOptions};

use super::{scan_repo_cached, scan_repo_cached_with_opts, CliError};
use crate::context_detection::build_repo_context;
use crate::integrations::{
    resolve_enrichment_plan, resolve_openai_auth, run_cmd_enrichment, run_openai_markdown,
};

const QUESTION_BANK: &str = r#"# Sruja discovery question bank

Ask the user 2–5 of these (adapt to context). Use answers to set scope, subpath, names, and externals.

## Context / shape
- Is this a single service, a monolith with modules, or several microservices?
- Should we capture one area first or the whole repo?

## Large repo
- The repo is big. Should we focus on a specific area (e.g. services/auth, apps/web) or the whole codebase? I can capture by subpath and we can stitch later.
- Which directory or service should we start with?

## Scope
- Do you want a minimal sketch (entry points + main deps), standard (10–30 components), or a deeper model (internal layers, error paths)?

## Boundaries
- What are your main bounded contexts or team-owned areas?
- Any external systems (payments, auth, notifications) that must appear in the diagram?

## Entry points and flows
- What's the main user-facing entry (web app, public API, CLI)?
- Any key flows (e.g. checkout, auth) I should make explicit?

## Refinement (after first draft)
- Does this match how you think about the system? Any services or boundaries missing?
- Prefer different names for systems or containers?

---
Use with: npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
Then in Cursor: run the agent and ask it to discover architecture; it will use this question bank.
"#;

/// Print the discovery question bank for use with the sruja-architecture skill.
pub fn discover_questions() -> Result<(), CliError> {
    println!("{}", QUESTION_BANK);
    Ok(())
}

/// Machine-readable repo context for agents (JSON).
#[derive(Debug, serde::Serialize)]
pub struct DiscoverContextJson {
    pub repo: String,
    pub scan_scope: sruja_scan::scan_scope::ScanScope,
    pub components: usize,
    pub edges: usize,
    pub primary_language: String,
    pub framework: Option<String>,
    pub architecture_style: String,
    pub domain: Option<String>,
    pub suggested_areas: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiscoverEnrichment {
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

#[derive(Debug, serde::Serialize)]
struct DiscoverEdgeConfidenceBreakdown {
    extracted: usize,
    inferred: usize,
    ambiguous: usize,
}

#[derive(Debug, serde::Serialize)]
struct DiscoverCommunity {
    id: u32,
    suggested_label: String,
    member_count: usize,
    cohesion: f64,
    top_members: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct DiscoverExplanationJson {
    context: DiscoverContextJson,
    kind_counts: BTreeMap<String, usize>,
    reasoning: Vec<String>,
    top_directories: Vec<DiscoverDirectorySummary>,
    key_elements: Vec<DiscoverElementSummary>,
    god_nodes: Vec<DiscoverElementSummary>,
    key_relationships: Vec<DiscoverRelationshipSummary>,
    surprising_connections: Vec<DiscoverRelationshipSummary>,
    suggested_questions: Vec<String>,
    confidence: DiscoverConfidence,
    edge_confidence_breakdown: DiscoverEdgeConfidenceBreakdown,
    architecture_digest: String,
    communities: Vec<DiscoverCommunity>,
    next_steps: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrichment: Option<DiscoverEnrichment>,
}

#[derive(Debug, serde::Serialize)]
struct DiscoverDirectorySummary {
    area: String,
    nodes: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
struct DiscoverElementSummary {
    id: String,
    label: String,
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    incoming: usize,
    outgoing: usize,
    pagerank: f64,
    why_it_matters: String,
}

#[derive(Debug, serde::Serialize)]
struct DiscoverRelationshipSummary {
    source: String,
    target: String,
    kind: String,
    confidence: String,
    why_it_matters: String,
}

#[derive(Debug, serde::Serialize)]
struct DiscoverConfidence {
    level: String,
    signals: Vec<String>,
    blind_spots: Vec<String>,
}

/// Build repo context summary as a string (for prompts or discovery).
pub fn discover_context_string(repo: &str) -> Result<String, CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo),
        )));
    }

    let graph = scan_repo_cached(repo_path)?;
    discover_context_string_from_graph(repo, repo_path, &graph)
}

/// Build repo context summary from a pre-scanned graph (includes actual structure).
pub fn discover_context_string_from_graph(
    repo: &str,
    repo_path: &Path,
    graph: &Graph,
) -> Result<String, CliError> {
    let context = build_repo_context(repo_path, graph);

    let repo_prefix = repo_path
        .canonicalize()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.replace('\\', "/")));
    let repo_arg_norm = repo.replace('\\', "/").trim_end_matches('/').to_string();

    let mut areas: HashSet<String> = HashSet::new();
    for node in &graph.nodes {
        if let Some(ref path) = node.path {
            let normalized = path.replace('\\', "/");
            let rel = if let Some(ref prefix) = repo_prefix {
                normalized
                    .strip_prefix(prefix)
                    .or_else(|| normalized.strip_prefix(&format!("{}/", prefix)))
                    .unwrap_or(normalized.as_str())
                    .trim_start_matches('/')
            } else if !repo_arg_norm.is_empty() {
                normalized
                    .strip_prefix(&format!("{}/", repo_arg_norm))
                    .or_else(|| normalized.strip_prefix(&repo_arg_norm))
                    .unwrap_or(normalized.as_str())
                    .trim_start_matches('/')
            } else {
                normalized.as_str()
            };

            if rel.is_empty() {
                continue;
            }

            let first = rel.split('/').next().unwrap_or("");
            if !first.is_empty()
                && !first.starts_with('.')
                && first != "test-repos"
                && first != "evaluation"
            {
                areas.insert(first.to_string());
            }
        }
    }
    let mut areas: Vec<String> = areas.into_iter().collect();
    areas.sort();

    let arch_style = if context.is_microservices {
        "microservices"
    } else if context.is_monolith {
        "monolith"
    } else {
        "mixed/unclear"
    };

    let mut out = String::new();
    out.push_str("# Repo context (for contextual discovery questions)\n\n");
    out.push_str(&format!("**Repo:** {}\n", repo));
    out.push_str(&format!("**Components (scan):** {}\n", graph.nodes.len()));
    out.push_str(&format!("**Edges:** {}\n", graph.edges.len()));
    out.push_str(&format!(
        "**Primary language:** {}\n",
        context.primary_language
    ));
    if let Some(ref fw) = context.framework {
        out.push_str(&format!("**Framework:** {}\n", fw));
    }
    out.push_str(&format!("**Architecture style:** {}\n", arch_style));
    if let Some(ref domain) = context.domain {
        out.push_str(&format!("**Domain (inferred):** {}\n", domain));
    }
    let areas_str = if areas.is_empty() {
        "(none — single directory or flat structure)".to_string()
    } else {
        areas.join(", ")
    };
    out.push_str(&format!(
        "**Suggested areas (from paths):** {}\n",
        areas_str
    ));

    out.push_str("\n## Key Components\n\n");

    let mut file_nodes: Vec<_> = graph.nodes.iter().filter(|n| n.path.is_some()).collect();

    let mut incoming_count: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::new();
    for edge in &graph.edges {
        *incoming_count.entry(&edge.target).or_default() += 1;
    }

    file_nodes.sort_by(|a, b| {
        let a_count = incoming_count.get(a.id.as_str()).copied().unwrap_or(0);
        let b_count = incoming_count.get(b.id.as_str()).copied().unwrap_or(0);
        b_count.cmp(&a_count)
    });

    let top_files = file_nodes.iter().take(20);
    for node in top_files {
        if let Some(ref path) = node.path {
            let rel_path = if let Some(ref prefix) = repo_prefix {
                path.replace('\\', "/")
                    .strip_prefix(prefix)
                    .or_else(|| path.strip_prefix(&format!("{}/", prefix)))
                    .unwrap_or(path.as_str())
                    .trim_start_matches('/')
                    .to_string()
            } else {
                path.clone()
            };
            let import_count = incoming_count.get(node.id.as_str()).copied().unwrap_or(0);
            out.push_str(&format!(
                "- `{}` ({}, {} imports)\n",
                rel_path,
                node.kind.as_str(),
                import_count
            ));
        }
    }

    out.push_str("\n## Exported Interfaces\n\n");

    let export_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| n.id.contains('#'))
        .take(30)
        .collect();

    for node in export_nodes {
        out.push_str(&format!("- `{}` ({})\n", node.label, node.kind.as_str()));
    }

    out.push_str("\n## Classification Signals\n\n");
    let mut ambiguous = graph
        .nodes
        .iter()
        .filter(|n| n.confidence.unwrap_or(100) < 70)
        .collect::<Vec<_>>();
    ambiguous.sort_by_key(|n| n.confidence.unwrap_or(100));

    if ambiguous.is_empty() {
        out.push_str("- All nodes classified with high confidence (>70%).\n");
    } else {
        for node in ambiguous.iter().take(10) {
            let signals = node
                .metadata
                .get("classification.signals")
                .cloned()
                .unwrap_or_else(|| "none".to_string());
            out.push_str(&format!(
                "- `{}` (kind={}, confidence={}%, signals=[{}])\n",
                node.id,
                node.kind.as_str(),
                node.confidence.unwrap_or(0),
                signals
            ));
        }
    }

    out.push_str("\nUse this context to derive 2–5 questions tailored to this repo (see skill: contextual discovery).\n");
    Ok(out)
}

/// Build repo context as JSON for machine-readable consumption by agents.
pub fn discover_context_json(repo: &str) -> Result<DiscoverContextJson, CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo),
        )));
    }
    let graph = scan_repo_cached(repo_path)?;
    discover_context_json_from_graph(repo, repo_path, &graph)
}

fn relative_graph_path(repo: &str, repo_path: &Path, path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let repo_prefix = repo_path
        .canonicalize()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.replace('\\', "/")));
    let repo_arg_norm = repo.replace('\\', "/").trim_end_matches('/').to_string();

    if let Some(prefix) = repo_prefix {
        return normalized
            .strip_prefix(&format!("{}/", prefix))
            .or_else(|| normalized.strip_prefix(&prefix))
            .unwrap_or(normalized.as_str())
            .trim_start_matches('/')
            .trim_start_matches("./")
            .to_string();
    }

    normalized
        .strip_prefix(&format!("{}/", repo_arg_norm))
        .or_else(|| normalized.strip_prefix(&repo_arg_norm))
        .unwrap_or(normalized.as_str())
        .trim_start_matches('/')
        .trim_start_matches("./")
        .to_string()
}

fn discover_top_directories(
    repo: &str,
    repo_path: &Path,
    graph: &Graph,
) -> Vec<DiscoverDirectorySummary> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for node in &graph.nodes {
        let Some(path) = node.path.as_deref() else {
            continue;
        };
        let relative = relative_graph_path(repo, repo_path, path);
        if !relative.contains('/') {
            continue;
        }
        let first = relative.split('/').next().unwrap_or("").trim();
        if !first.is_empty()
            && !first.starts_with('.')
            && first != "test-repos"
            && first != "evaluation"
        {
            *counts.entry(first.to_string()).or_default() += 1;
        }
    }

    let mut dirs: Vec<_> = counts
        .into_iter()
        .map(|(area, nodes)| DiscoverDirectorySummary { area, nodes })
        .collect();
    dirs.sort_by(|a, b| b.nodes.cmp(&a.nodes).then_with(|| a.area.cmp(&b.area)));
    dirs.truncate(5);
    dirs
}

fn discover_kind_counts(graph: &Graph) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for node in &graph.nodes {
        *counts.entry(node.kind.as_str().to_string()).or_insert(0) += 1;
    }
    counts
}

fn kind_priority(kind: NodeKind) -> u8 {
    match kind.as_str() {
        NodeKind::SERVICE => 6,
        NodeKind::FRONTEND => 5,
        NodeKind::DATABASE => 4,
        NodeKind::EXTERNAL_API => 3,
        NodeKind::CONTAINER => 2,
        NodeKind::COMPONENT => 2,
        NodeKind::QUEUE => 2,
        NodeKind::SYSTEM => 1,
        NodeKind::MODULE => 0,
        _ => 0,
    }
}

fn explain_node_relevance(node: &sruja_scan::Node, incoming: usize, outgoing: usize) -> String {
    match node.kind.as_str() {
        NodeKind::DATABASE => format!(
            "Acts as a data boundary with {} upstream dependency(ies).",
            incoming
        ),
        NodeKind::EXTERNAL_API => format!(
            "Represents an external integration touched by {} internal component(s).",
            incoming
        ),
        NodeKind::SERVICE | NodeKind::FRONTEND | NodeKind::CONTAINER | NodeKind::COMPONENT => {
            if incoming > 0 && outgoing > 0 {
                format!(
                    "Sits in the middle of the graph with {} incoming and {} outgoing dependency(ies).",
                    incoming, outgoing
                )
            } else if outgoing > 0 {
                format!(
                    "Looks like a coordinator with {} downstream dependency(ies).",
                    outgoing
                )
            } else if incoming > 0 {
                format!(
                    "Looks like a shared dependency referenced by {} upstream component(s).",
                    incoming
                )
            } else {
                "Was ranked highly despite being lightly connected, which usually means it is a clean entry point.".to_string()
            }
        }
        _ => {
            if incoming > outgoing {
                format!(
                    "Mostly acts as a shared dependency with {} incoming reference(s).",
                    incoming
                )
            } else if outgoing > 0 {
                format!(
                    "Mostly acts as a caller with {} outgoing dependency(ies).",
                    outgoing
                )
            } else {
                "Provides a standalone boundary candidate in the scanned graph.".to_string()
            }
        }
    }
}

fn discover_key_elements(
    repo: &str,
    repo_path: &Path,
    graph: &Graph,
) -> Vec<DiscoverElementSummary> {
    let centrality = compute_all_centrality(graph);
    let mut incoming_counts: HashMap<&str, usize> = HashMap::new();
    let mut outgoing_counts: HashMap<&str, usize> = HashMap::new();

    for edge in &graph.edges {
        *incoming_counts.entry(edge.target.as_str()).or_default() += 1;
        *outgoing_counts.entry(edge.source.as_str()).or_default() += 1;
    }

    let mut nodes: Vec<_> = graph.nodes.iter().collect();
    let preferred_nodes: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| !node.id.contains('#'))
        .collect();
    if preferred_nodes.len() >= 3 {
        nodes = preferred_nodes;
    }
    nodes.sort_by(|a, b| {
        let a_rank = centrality
            .get(&a.id)
            .map(|s| s.pagerank)
            .unwrap_or_default();
        let b_rank = centrality
            .get(&b.id)
            .map(|s| s.pagerank)
            .unwrap_or_default();
        b_rank
            .total_cmp(&a_rank)
            .then_with(|| kind_priority(b.kind.clone()).cmp(&kind_priority(a.kind.clone())))
            .then_with(|| a.id.cmp(&b.id))
    });

    nodes.truncate(5);
    nodes
        .into_iter()
        .map(|node| {
            let incoming = incoming_counts.get(node.id.as_str()).copied().unwrap_or(0);
            let outgoing = outgoing_counts.get(node.id.as_str()).copied().unwrap_or(0);
            let pagerank = centrality
                .get(&node.id)
                .map(|s| s.pagerank)
                .unwrap_or_default();
            DiscoverElementSummary {
                id: node.id.clone(),
                label: node.label.clone(),
                kind: node.kind.as_str().to_string(),
                path: node
                    .path
                    .as_deref()
                    .map(|path| relative_graph_path(repo, repo_path, path)),
                incoming,
                outgoing,
                pagerank,
                why_it_matters: explain_node_relevance(node, incoming, outgoing),
            }
        })
        .collect()
}

fn explain_edge_relevance(
    edge: &sruja_scan::Edge,
    target_kind: Option<NodeKind>,
    evidence_count: usize,
) -> String {
    let evidence_suffix = if evidence_count > 0 {
        format!(" Backed by {} evidence item(s).", evidence_count)
    } else {
        String::new()
    };

    let base = match target_kind.as_ref().map(|k| k.as_str()) {
        Some(NodeKind::DATABASE) => {
            "Highlights a core service-to-data dependency that should usually stay intentional."
        }
        Some(NodeKind::EXTERNAL_API) => {
            "Highlights an external integration boundary that is important for change planning."
        }
        Some(NodeKind::SERVICE) | Some(NodeKind::FRONTEND) => {
            "Connects high-signal runtime or user-facing components."
        }
        _ if matches!(
            edge.kind.as_str(),
            EdgeKind::CALLS | EdgeKind::DEPENDS_ON | EdgeKind::USES
        ) =>
        {
            "Represents a meaningful internal dependency worth validating as a boundary."
        }
        _ => "Captures a structurally important relationship in the scanned graph.",
    };

    format!("{base}{evidence_suffix}")
}

fn discover_key_relationships(graph: &Graph) -> Vec<DiscoverRelationshipSummary> {
    let centrality = compute_all_centrality(graph);
    let node_kind_by_id: HashMap<&str, NodeKind> = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node.kind.clone()))
        .collect();

    let mut edges: Vec<_> = graph
        .edges
        .iter()
        .filter(|edge| !matches!(edge.kind.as_str(), EdgeKind::CONTAINS | EdgeKind::OWNS))
        .collect();
    let preferred_edges: Vec<_> = edges
        .iter()
        .copied()
        .filter(|edge| !edge.source.contains('#') && !edge.target.contains('#'))
        .collect();
    if preferred_edges.len() >= 3 {
        edges = preferred_edges;
    }

    edges.sort_by(|a, b| {
        let a_score = centrality
            .get(&a.source)
            .map(|s| s.pagerank)
            .unwrap_or_default()
            + centrality
                .get(&a.target)
                .map(|s| s.pagerank)
                .unwrap_or_default();
        let b_score = centrality
            .get(&b.source)
            .map(|s| s.pagerank)
            .unwrap_or_default()
            + centrality
                .get(&b.target)
                .map(|s| s.pagerank)
                .unwrap_or_default();
        b_score
            .total_cmp(&a_score)
            .then_with(|| b.evidence.len().cmp(&a.evidence.len()))
            .then_with(|| a.source.cmp(&b.source))
            .then_with(|| a.target.cmp(&b.target))
            .then_with(|| match (a.kind.clone(), b.kind.clone()) {
                (ak, bk) if ak == bk => Ordering::Equal,
                (ak, bk) => ak.as_str().cmp(bk.as_str()),
            })
    });

    edges.truncate(5);
    edges
        .into_iter()
        .map(|edge| DiscoverRelationshipSummary {
            source: edge.source.clone(),
            target: edge.target.clone(),
            kind: edge.kind.as_str().to_string(),
            confidence: format!("{:?}", edge.confidence).to_lowercase(),
            why_it_matters: explain_edge_relevance(
                edge,
                node_kind_by_id.get(edge.target.as_str()).cloned(),
                edge.evidence.len(),
            ),
        })
        .collect()
}

fn discover_reasoning(
    context: &DiscoverContextJson,
    graph: &Graph,
    top_directories: &[DiscoverDirectorySummary],
) -> Vec<String> {
    let service_count = graph
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::SERVICE)
        .count();
    let exported_interfaces = graph
        .nodes
        .iter()
        .filter(|node| node.id.contains('#'))
        .count();

    let mut reasoning = Vec::new();
    let architecture_reason = if service_count == 0 {
        "No service nodes were detected, so Sruja is currently reading this repo mostly as a module-level graph inside one codebase."
            .to_string()
    } else {
        match context.architecture_style.as_str() {
            "microservices" => format!(
                "Detected {service_count} service node(s), so this reads as a multi-service codebase rather than a single deployable unit."
            ),
            "monolith" => format!(
                "Detected {service_count} service node(s), which points to a single deployable unit or a monolith with internal modules."
            ),
            _ => format!(
                "Detected {service_count} service node(s), so Sruja is keeping the architecture style flexible instead of forcing a monolith or microservices label."
            ),
        }
    };
    reasoning.push(architecture_reason);

    if let Some(framework) = &context.framework {
        reasoning.push(format!(
            "Framework markers suggest {framework}, which helps Sruja recognize likely entry points and boundaries."
        ));
    } else {
        reasoning.push(
            "No strong framework markers were detected, so Sruja is leaning more on file structure and dependency edges."
                .to_string(),
        );
    }

    if !top_directories.is_empty() {
        let areas = top_directories
            .iter()
            .take(3)
            .map(|dir| format!("`{}`", dir.area))
            .collect::<Vec<_>>()
            .join(", ");
        reasoning.push(format!(
            "Most discovered elements cluster under {areas}, which is where the scanner sees the clearest architectural seams."
        ));
    }

    if exported_interfaces > 0 {
        reasoning.push(format!(
            "Found {exported_interfaces} exported interface node(s), which gives the scan stable API surfaces to anchor on."
        ));
    } else {
        reasoning.push(
            "The scan found few exported interfaces, so the graph is driven mostly by file-level structure and imports."
                .to_string(),
        );
    }

    reasoning
}

fn discover_confidence(
    context: &DiscoverContextJson,
    graph: &Graph,
    top_directories: &[DiscoverDirectorySummary],
) -> DiscoverConfidence {
    let nodes_with_paths = graph
        .nodes
        .iter()
        .filter(|node| node.path.is_some())
        .count();
    let level = if graph.nodes.is_empty() {
        "AMBIGUOUS"
    } else if graph.edges.is_empty() || context.framework.is_none() || top_directories.is_empty() {
        "INFERRED"
    } else {
        "EXTRACTED"
    };

    let mut signals = vec![format!(
        "Static analysis produced {} node(s) and {} relationship(s).",
        graph.nodes.len(),
        graph.edges.len()
    )];
    signals.push(format!(
        "{} of those node(s) map back to concrete file paths.",
        nodes_with_paths
    ));
    if let Some(framework) = &context.framework {
        signals.push(format!("Framework detection matched {framework}."));
    }
    if !top_directories.is_empty() {
        signals.push(format!(
            "The scan found clear top-level hotspots in {}.",
            top_directories
                .iter()
                .take(3)
                .map(|dir| format!("`{}`", dir.area))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let mut blind_spots = vec![
        "This is static analysis, so runtime-only calls, reflection, and generated code can still be missing.".to_string(),
        "Ownership, domain labels, and external system names are strongest after a reviewed repo.sruja baseline exists.".to_string(),
    ];
    if context.framework.is_none() {
        blind_spots.push(
            "Because the framework is unclear, boundary naming may need extra human review before you commit a baseline."
                .to_string(),
        );
    }

    DiscoverConfidence {
        level: level.to_string(),
        signals,
        blind_spots,
    }
}

fn discover_next_steps(graph: &Graph) -> Vec<String> {
    if graph.nodes.is_empty() {
        return vec![
            "Verify you are at the repo root and that the repo uses a supported language before relying on this scan.".to_string(),
            "Run `sruja discover context -r . --format json` to inspect what Sruja can detect from manifests and paths.".to_string(),
            "If the repo should have been discovered, capture a minimal repro and open an issue so scanner coverage can improve.".to_string(),
        ];
    }

    vec![
        "Review the highlighted elements and rename or regroup them in `repo.sruja` if they do not match your team language.".to_string(),
        "Run `sruja quickstart -r . --generate-baseline` for a structural draft (repo.sruja.draft), then author reviewed intent in repo.sruja with the sruja-architecture skill.".to_string(),
        "After repo.sruja exists, run `sruja drift -r . -a repo.sruja` in CI to keep declared architecture aligned with code.".to_string(),
    ]
}

fn discover_surprising_connections(graph: &Graph) -> Vec<DiscoverRelationshipSummary> {
    let mut surprising = Vec::new();
    for edge in &graph.edges {
        let source_parts: Vec<&str> = edge.source.split('_').collect();
        let target_parts: Vec<&str> = edge.target.split('_').collect();

        if source_parts.get(1) != target_parts.get(1)
            && !edge.source.contains("module:")
            && !edge.target.contains("module:")
        {
            surprising.push(DiscoverRelationshipSummary {
                source: edge.source.clone(),
                target: edge.target.clone(),
                kind: edge.kind.as_str().to_string(),
                confidence: format!("{:?}", edge.confidence).to_lowercase(),
                why_it_matters: "Crosses module boundaries and has low evidence, potentially indicating a hidden architectural coupling.".to_string(),
            });
        }
    }
    surprising.truncate(3);
    surprising
}

fn discover_suggested_questions(
    god_nodes: &[DiscoverElementSummary],
    surprising: &[DiscoverRelationshipSummary],
) -> Vec<String> {
    let mut questions = Vec::new();
    for node in god_nodes.iter().take(2) {
        questions.push(format!(
            "Why is `{}` a central hub (incoming: {}, outgoing: {})? Should its responsibilities be split?",
            node.id, node.incoming, node.outgoing
        ));
    }
    for edge in surprising.iter().take(2) {
        questions.push(format!(
            "What is the nature of the direct dependency from `{}` to `{}`? Can this coupling be decoupled or abstracted?",
            edge.source, edge.target
        ));
    }
    questions.push("What boundaries should be introduced to isolate changing core domains from external adapters?".to_string());
    questions.push(
        "How does the data access pattern scale with the observed coupling to data stores?"
            .to_string(),
    );
    questions
}

fn build_discover_explanation(
    repo: &str,
    repo_path: &Path,
    graph: &Graph,
) -> Result<DiscoverExplanationJson, CliError> {
    let context = discover_context_json_from_graph(repo, repo_path, graph)?;
    let top_directories = discover_top_directories(repo, repo_path, graph);
    let reasoning = discover_reasoning(&context, graph, &top_directories);
    let confidence = discover_confidence(&context, graph, &top_directories);
    let god_nodes = discover_key_elements(repo, repo_path, graph);
    let key_relationships = discover_key_relationships(graph);
    let surprising_connections = discover_surprising_connections(graph);
    let suggested_questions = discover_suggested_questions(&god_nodes, &surprising_connections);

    let mut extracted = 0;
    let mut inferred = 0;
    let mut ambiguous = 0;
    for edge in &graph.edges {
        match edge.confidence {
            sruja_scan::graph::EdgeConfidence::Extracted => extracted += 1,
            sruja_scan::graph::EdgeConfidence::Inferred => inferred += 1,
            sruja_scan::graph::EdgeConfidence::Ambiguous => ambiguous += 1,
        }
    }
    let edge_confidence_breakdown = DiscoverEdgeConfidenceBreakdown {
        extracted,
        inferred,
        ambiguous,
    };

    let mut digest_parts = Vec::new();
    digest_parts.push(format!(
        "The scanned repository '{}' is identified as a {} system primarily written in {}.",
        context.repo, context.architecture_style, context.primary_language
    ));
    if let Some(ref framework) = context.framework {
        digest_parts.push(format!(
            "It leverages the {} framework to organize its components.",
            framework
        ));
    }
    digest_parts.push(format!(
        "The static analysis discovered {} logical component(s) connected by {} relationship(s).",
        graph.nodes.len(),
        graph.edges.len()
    ));
    if !graph.nodes.is_empty() {
        let db_count = graph
            .nodes
            .iter()
            .filter(|n| n.kind == sruja_scan::NodeKind::DATABASE)
            .count();
        if db_count > 0 {
            digest_parts.push(format!(
                "The architecture integrates {} database datastore(s) for persistent storage.",
                db_count
            ));
        } else {
            digest_parts.push("The architecture appears to be modular without directly scanned database dependencies at this level.".to_string());
        }
    }
    let architecture_digest = digest_parts.join(" ");

    let raw_communities = sruja_scan::detect_communities(graph);
    let community_infos = sruja_scan::summarize_communities(graph, &raw_communities);
    let communities: Vec<DiscoverCommunity> = community_infos
        .into_iter()
        .map(|c| DiscoverCommunity {
            id: c.id,
            suggested_label: c.suggested_label,
            member_count: c.member_count,
            cohesion: c.cohesion,
            top_members: c.members.into_iter().take(5).collect(),
        })
        .collect();

    Ok(DiscoverExplanationJson {
        kind_counts: discover_kind_counts(graph),
        key_elements: god_nodes.clone(),
        god_nodes,
        key_relationships,
        surprising_connections,
        suggested_questions,
        next_steps: discover_next_steps(graph),
        reasoning,
        confidence,
        edge_confidence_breakdown,
        architecture_digest,
        communities,
        top_directories,
        context,
        enrichment: None,
    })
}

pub fn discover_explanation_string(repo: &str) -> Result<String, CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo),
        )));
    }

    let graph = scan_repo_cached(repo_path)?;
    discover_explanation_string_from_graph(repo, repo_path, &graph)
}

pub fn discover_explanation_string_from_graph(
    repo: &str,
    repo_path: &Path,
    graph: &Graph,
) -> Result<String, CliError> {
    let explanation = build_discover_explanation(repo, repo_path, graph)?;
    Ok(format_discovery_explanation(&explanation))
}

/// Build discovery explanation as JSON from a pre-scanned graph (avoids rescanning).
#[expect(dead_code)]
pub fn discover_explanation_json_from_graph(
    repo: &str,
    repo_path: &Path,
    graph: &Graph,
) -> Result<String, CliError> {
    let explanation = build_discover_explanation(repo, repo_path, graph)?;
    serde_json::to_string_pretty(&explanation).map_err(|e| CliError::validation(e.to_string()))
}

/// Build discovery explanation as a JSON value from a pre-scanned graph (avoids rescanning).
pub fn discover_explanation_value_from_graph(
    repo: &str,
    repo_path: &Path,
    graph: &Graph,
) -> Result<serde_json::Value, CliError> {
    let explanation = build_discover_explanation(repo, repo_path, graph)?;
    serde_json::to_value(&explanation).map_err(|e| CliError::validation(e.to_string()))
}

pub fn discover_explanation_json(repo: &str) -> Result<String, CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo),
        )));
    }

    let graph = scan_repo_cached(repo_path)?;
    let explanation = build_discover_explanation(repo, repo_path, &graph)?;
    serde_json::to_string_pretty(&explanation).map_err(|e| CliError::validation(e.to_string()))
}

fn format_discovery_explanation(explanation: &DiscoverExplanationJson) -> String {
    let mut out = String::new();
    out.push_str("# Sruja Discovery Explanation\n\n");
    out.push_str(&format!("**Repo:** {}\n", explanation.context.repo));
    out.push_str(&format!(
        "**Scan summary:** {} node(s), {} relationship(s)\n",
        explanation.context.components, explanation.context.edges
    ));
    out.push_str(&format!(
        "**Primary language:** {}\n",
        explanation.context.primary_language
    ));
    if let Some(framework) = &explanation.context.framework {
        out.push_str(&format!("**Framework:** {}\n", framework));
    }
    out.push_str(&format!(
        "**Architecture style:** {}\n",
        explanation.context.architecture_style
    ));
    if let Some(domain) = &explanation.context.domain {
        out.push_str(&format!("**Domain hint:** {}\n", domain));
    }

    if !explanation.kind_counts.is_empty() {
        let counts = explanation
            .kind_counts
            .iter()
            .map(|(kind, count)| format!("{kind}: {count}"))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("**Element mix:** {}\n", counts));
    }

    out.push_str("\n## Architecture Digest\n\n");
    out.push_str(&explanation.architecture_digest);
    out.push_str("\n\n## Why Sruja Thinks That\n\n");
    for reason in &explanation.reasoning {
        out.push_str(&format!("- {}\n", reason));
    }

    if !explanation.top_directories.is_empty() {
        out.push_str("\n## Top Directories\n\n");
        for dir in &explanation.top_directories {
            out.push_str(&format!("- `{}`: {} node(s)\n", dir.area, dir.nodes));
        }
    }

    out.push_str("\n## Dependencies Confidence Breakdown\n\n");
    out.push_str("| Confidence Level | Count |\n");
    out.push_str("|---|---|\n");
    out.push_str(&format!(
        "| **Extracted** (AST, manifest) | {} |\n",
        explanation.edge_confidence_breakdown.extracted
    ));
    out.push_str(&format!(
        "| **Inferred** (naming, heuristics) | {} |\n",
        explanation.edge_confidence_breakdown.inferred
    ));
    out.push_str(&format!(
        "| **Ambiguous** (unresolved, weak) | {} |\n",
        explanation.edge_confidence_breakdown.ambiguous
    ));

    out.push_str("\n## Discovery Confidence\n\n");
    out.push_str(&format!("**Level:** {}\n", explanation.confidence.level));
    for signal in &explanation.confidence.signals {
        out.push_str(&format!("- [✓] {}\n", signal));
    }
    for spot in &explanation.confidence.blind_spots {
        out.push_str(&format!("- [?] {}\n", spot));
    }

    if !explanation.god_nodes.is_empty() {
        out.push_str("\n## God Nodes (High-Signal Elements)\n\n");
        out.push_str("| ID | Label | Kind | Incoming | Outgoing | PageRank | Why It Matters |\n");
        out.push_str("|---|---|---|---|---|---|---|\n");
        for element in &explanation.god_nodes {
            let path_suffix = element
                .path
                .as_deref()
                .map(|path| format!("<br>`{path}`"))
                .unwrap_or_default();
            out.push_str(&format!(
                "| `{}`{} | {} | `{}` | {} | {} | {:.4} | {} |\n",
                element.id,
                path_suffix,
                element.label,
                element.kind,
                element.incoming,
                element.outgoing,
                element.pagerank,
                element.why_it_matters
            ));
        }
    }

    if !explanation.key_relationships.is_empty() {
        out.push_str("\n## Key Relationships\n\n");
        out.push_str("| Source | Target | Kind | Confidence | Why It Matters |\n");
        out.push_str("|---|---|---|---|---|\n");
        for relationship in &explanation.key_relationships {
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | {} |\n",
                relationship.source,
                relationship.target,
                relationship.kind,
                relationship.confidence,
                relationship.why_it_matters
            ));
        }
    }

    if !explanation.surprising_connections.is_empty() {
        out.push_str("\n## Surprising Connections\n\n");
        out.push_str("| Source | Target | Kind | Confidence | Why It Matters |\n");
        out.push_str("|---|---|---|---|---|\n");
        for relationship in &explanation.surprising_connections {
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | {} |\n",
                relationship.source,
                relationship.target,
                relationship.kind,
                relationship.confidence,
                relationship.why_it_matters
            ));
        }
    }

    if !explanation.suggested_questions.is_empty() {
        out.push_str("\n## Suggested Questions\n\n");
        for question in &explanation.suggested_questions {
            out.push_str(&format!("- {}\n", question));
        }
    }

    if !explanation.communities.is_empty() {
        out.push_str("\n## Module Communities (LPA Clusters)\n\n");
        out.push_str(
            "| ID | Suggested Label | Members | Cohesion | Boundary Status | Top Members |\n",
        );
        out.push_str("|---|---|---|---|---|---|\n");
        for community in &explanation.communities {
            let boundary_status = if community.cohesion < 0.3 {
                "⚠️ **Weakly Bounded**"
            } else {
                "✅ Strongly Bounded"
            };
            let members_joined = community
                .top_members
                .iter()
                .map(|m| format!("`{m}`"))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!(
                "| {} | **{}** | {} | {:.2} | {} | {} |\n",
                community.id,
                community.suggested_label,
                community.member_count,
                community.cohesion,
                boundary_status,
                members_joined
            ));
        }
    }

    out.push_str("\n## Next Steps\n\n");
    for step in &explanation.next_steps {
        out.push_str(&format!("- {}\n", step));
    }

    if let Some(ref e) = explanation.enrichment {
        out.push_str("\n## LLM Enrichment (opt-in)\n\n");
        out.push_str(
            "- This section is **LLM-generated** and must be treated as **interpretation**, not truth.\n",
        );
        out.push_str(
            "- It is grounded in the JSON facts above; if it contradicts them, prefer the grounded scan output.\n\n",
        );
        out.push_str(&format!("- Status: `{}`\n", e.status));
        if let Some(ref p) = e.provider {
            out.push_str(&format!("- Provider: `{}`\n", p));
        }
        if let Some(ref m) = e.model {
            out.push_str(&format!("- Model: `{}`\n", m));
        }
        if let Some(ref err) = e.error {
            out.push_str(&format!("- Error: `{}`\n", err));
        }
        out.push('\n');
        if let Some(ref narrative) = e.narrative_markdown {
            out.push_str(narrative);
            out.push('\n');
        }
    }

    out
}

/// Build repo context as JSON from a pre-scanned graph (avoids rescanning).
pub fn discover_context_json_from_graph(
    repo: &str,
    repo_path: &Path,
    graph: &Graph,
) -> Result<DiscoverContextJson, CliError> {
    let context = build_repo_context(repo_path, graph);
    let architecture_style = if context.is_microservices {
        "microservices"
    } else if context.is_monolith {
        "monolith"
    } else {
        "mixed/unclear"
    }
    .to_string();

    let (_, scan_scope) = resolve_scan_scope(repo_path);
    let repo_prefix = repo_path
        .canonicalize()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.replace('\\', "/")));
    let repo_arg_norm = repo.replace('\\', "/").trim_end_matches('/').to_string();
    let mut areas: HashSet<String> = HashSet::new();
    for node in &graph.nodes {
        if let Some(ref path) = node.path {
            let normalized = path.replace('\\', "/");
            let rel = if let Some(ref prefix) = repo_prefix {
                normalized
                    .strip_prefix(prefix)
                    .or_else(|| normalized.strip_prefix(&format!("{}/", prefix)))
                    .unwrap_or(normalized.as_str())
                    .trim_start_matches('/')
            } else if !repo_arg_norm.is_empty() {
                normalized
                    .strip_prefix(&format!("{}/", repo_arg_norm))
                    .or_else(|| normalized.strip_prefix(&repo_arg_norm))
                    .unwrap_or(normalized.as_str())
                    .trim_start_matches('/')
            } else {
                normalized.as_str()
            };
            if rel.is_empty() {
                continue;
            }
            let first = rel.split('/').next().unwrap_or("");
            if !first.is_empty()
                && !first.starts_with('.')
                && first != "test-repos"
                && first != "evaluation"
            {
                areas.insert(first.to_string());
            }
        }
    }
    let mut suggested_areas: Vec<String> = areas.into_iter().collect();
    suggested_areas.sort();
    Ok(DiscoverContextJson {
        repo: repo.to_string(),
        scan_scope,
        components: graph.nodes.len(),
        edges: graph.edges.len(),
        primary_language: context.primary_language.clone(),
        framework: context.framework.clone(),
        architecture_style,
        domain: context.domain.clone(),
        suggested_areas,
    })
}

/// Print repo context summary for the agent to derive contextual questions.
pub async fn discover_context(repo: &str, format: &str) -> Result<(), CliError> {
    if format == "json" {
        let json = discover_context_json(repo)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&json).map_err(|e| CliError::validation(e.to_string()))?
        );
        return Ok(());
    }
    let s = discover_context_string(repo)?;
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
) -> Option<DiscoverEnrichment> {
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
            return Some(DiscoverEnrichment {
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
            return Some(DiscoverEnrichment {
                status: "skipped".to_string(),
                provider: Some("external_cmd".to_string()),
                model: None,
                error: Some("No command configured. Pass --enrich-cmd or set SRUJA_ENRICH_CMD (or .sruja/config.toml [integrations].cmd).".to_string()),
                narrative_markdown: None,
            });
        };
        let stdin_payload = serde_json::to_vec(&payload).unwrap_or_default();
        return Some(match run_cmd_enrichment(cmd, &stdin_payload, limits) {
            Ok(md) => DiscoverEnrichment {
                status: "ok".to_string(),
                provider: Some("external_cmd".to_string()),
                model: None,
                error: None,
                narrative_markdown: Some(md),
            },
            Err(e) => DiscoverEnrichment {
                status: "error".to_string(),
                provider: Some("external_cmd".to_string()),
                model: None,
                error: Some(e),
                narrative_markdown: None,
            },
        });
    }

    if provider != "openai" {
        return Some(DiscoverEnrichment {
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

    let model = plan.model.as_deref().unwrap_or("gpt-4o-mini");
    let base_url = plan
        .base_url
        .as_deref()
        .unwrap_or("https://api.openai.com/v1");
    let Some(key) = resolve_openai_auth() else {
        return Some(DiscoverEnrichment {
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
        r#"You are analyzing an architecture graph explanation.

You MUST only use the JSON facts provided below. Do not invent components, APIs, or file paths. If something is unknown, say "unknown".

Produce markdown with these sections:
- "Architectural Narrative" (narrative summary of the architecture and role)
- "Why the God Nodes matter" (bullets explaining significance based on facts)
- "Surprising Connections insights" (analysis of Surprising Connections and suggestions about coupling)
- "Architectural Risks & Questions" (suggested investigative paths)

JSON facts:
{}"#,
        payload
    );

    match run_openai_markdown(
        "You are a careful architecture analyst. Never fabricate.",
        &user_prompt,
        model,
        base_url,
        &key,
    ) {
        Ok(md) => Some(DiscoverEnrichment {
            status: "ok".to_string(),
            provider: Some("openai".to_string()),
            model: Some(model.to_string()),
            error: None,
            narrative_markdown: Some(md),
        }),
        Err(e) => Some(DiscoverEnrichment {
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
