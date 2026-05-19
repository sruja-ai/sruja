//! Summary-tier `repo.sruja.draft` generation (deterministic, no LLM).
//!
//! Prefers **workspace structure** (Cargo/npm manifests) over tree-sitter call graphs.
//! Call/import graphs are implementation detail, not architecture — drafts surface
//! package boundaries and declared workspace dependencies only.

use std::collections::{HashMap, HashSet};

use sruja_scan::{Edge, Graph, Node, NodeKind};

use super::output::{
    element_kind_for_node, path_production_relevant, qualified_ident_from_id, sanitize_identifier,
};

pub const DRAFT_BASELINE_FILE: &str = "repo.sruja.draft";
pub const MAX_SUMMARY_CONTAINERS: usize = 12;
pub const MAX_SUMMARY_SPECIALS: usize = 8;
pub const MAX_SUMMARY_EDGES: usize = 30;

const MANIFEST_EDGE_RULES: &[&str] = &["cargo_metadata_dep", "package_json_dep"];

#[derive(Debug, Clone)]
struct SummaryBucket {
    key: String,
    display_title: String,
    technology: Option<String>,
    module_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EdgeAggKey {
    from: String,
    to: String,
}

pub fn draft_baseline_path(repo_root: &std::path::Path) -> std::path::PathBuf {
    repo_root.join(DRAFT_BASELINE_FILE)
}

/// Build a compact architecture program suitable for human review (not a scan dump).
pub fn build_summary_draft_program(graph: &Graph, filename: &str) -> sruja_language::Program {
    let workspace_units: Vec<&Node> = graph
        .nodes
        .iter()
        .filter(|n| is_workspace_unit(n))
        .collect();

    if !workspace_units.is_empty() {
        build_workspace_draft_program(graph, filename, &workspace_units)
    } else {
        build_path_cluster_draft_program(graph, filename)
    }
}

/// Draft from `crate:*` / `npm:*` nodes and manifest-declared workspace edges only.
fn build_workspace_draft_program(
    graph: &Graph,
    filename: &str,
    workspace_units: &[&Node],
) -> sruja_language::Program {
    let repo_name = repo_display_name(filename);
    let system_name = sanitize_identifier(&repo_name);

    let mut ranked: Vec<(&Node, String)> = workspace_units
        .iter()
        .filter_map(|n| workspace_unit_key(n).map(|k| (*n, k)))
        .collect();
    ranked.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.label.cmp(&b.0.label)));

    let mut node_id_to_container: HashMap<String, String> = HashMap::new();
    let mut system_items: Vec<sruja_language::ElementDefBodyItem> = Vec::new();

    for (node, unit_key) in ranked.iter().take(MAX_SUMMARY_CONTAINERS) {
        let container_name = sanitize_identifier(&title_case(&unit_key.replace('-', "_")));
        node_id_to_container.insert(node.id.clone(), container_name.clone());

        let tech = node
            .technology
            .clone()
            .or_else(|| infer_technology(node))
            .unwrap_or_else(|| "Unknown".to_string());

        system_items.push(sruja_language::ElementDefBodyItem::ElementDef(Box::new(
            make_element_def(
                filename,
                container_name,
                sruja_language::ElementKind::Container,
                None,
                if node.label.is_empty() {
                    humanize_identifier(unit_key)
                } else {
                    node.label.clone()
                },
                Some(tech),
                workspace_container_description(node, unit_key),
            ),
        )));
    }

    if ranked.len() > MAX_SUMMARY_CONTAINERS {
        let omitted = ranked.len() - MAX_SUMMARY_CONTAINERS;
        system_items.push(sruja_language::ElementDefBodyItem::ElementDef(Box::new(
            make_element_def(
                filename,
                sanitize_identifier("OtherPackages"),
                sruja_language::ElementKind::Container,
                None,
                "Other packages".to_string(),
                None,
                format!(
                    "{omitted} additional workspace package(s) omitted (cap {MAX_SUMMARY_CONTAINERS}). \
                     Split or merge boundaries manually after review."
                ),
            ),
        )));
    }

    for node in graph.nodes.iter().filter(|n| is_special_kind(&n.kind)) {
        if specials_in_draft(system_items.len()) {
            let name = sanitize_identifier(&unique_special_name(node));
            let (kind, sub_kind) = element_kind_for_node(node.kind.clone());
            system_items.push(sruja_language::ElementDefBodyItem::ElementDef(Box::new(
                make_element_def(
                    filename,
                    name,
                    kind,
                    sub_kind,
                    node.label.clone(),
                    node.technology.clone().or_else(|| infer_technology(node)),
                    format!(
                        "Inferred {} from scan manifests or heuristics — verify before promoting.",
                        node.kind.as_str()
                    ),
                ),
            )));
        }
    }

    let mut edge_seen: HashSet<EdgeAggKey> = HashSet::new();
    let mut manifest_edges: Vec<EdgeAggKey> = Vec::new();

    for edge in &graph.edges {
        if !is_manifest_edge(edge) {
            continue;
        }
        let Some(from) = node_id_to_container.get(&edge.source) else {
            continue;
        };
        let Some(to) = node_id_to_container.get(&edge.target) else {
            continue;
        };
        if from == to {
            continue;
        }
        let key = EdgeAggKey {
            from: from.clone(),
            to: to.clone(),
        };
        if edge_seen.insert(key.clone()) {
            manifest_edges.push(key);
        }
    }

    manifest_edges.sort_by(|a, b| a.from.cmp(&b.from).then_with(|| a.to.cmp(&b.to)));
    manifest_edges.truncate(MAX_SUMMARY_EDGES);

    for edge_key in manifest_edges {
        let rel = sruja_language::Relation {
            location: source_loc(filename),
            from: qualified_ident_from_id(&format!("{}.{}", system_name, edge_key.from)),
            to: qualified_ident_from_id(&format!("{}.{}", system_name, edge_key.to)),
            label: Some("depends on (workspace)".to_string()),
            description: Some(
                "Declared package dependency from Cargo.toml or package.json — not a runtime/data-flow edge."
                    .to_string(),
            ),
            technology: None,
            tags: Vec::new(),
        };
        system_items.push(sruja_language::ElementDefBodyItem::Relation(rel));
    }

    finish_system_program(
        filename,
        repo_name,
        system_name,
        system_items,
        workspace_system_description(),
    )
}

fn workspace_system_description() -> String {
    "Structural map from workspace manifests (packages + declared deps). \
     This is not domain architecture: add actors, data stores, and runtime flows in repo.sruja after review."
        .to_string()
}

fn workspace_container_description(node: &Node, unit_key: &str) -> String {
    let mut hints = Vec::new();
    if node.metadata.get("hint:db_client") == Some(&"true".to_string()) {
        hints.push("manifest lists DB client libraries");
    }
    if node.metadata.get("hint:http_client") == Some(&"true".to_string()) {
        hints.push("manifest lists HTTP client libraries");
    }
    if node.metadata.get("hint:workspace_dep") == Some(&"true".to_string()) {
        hints.push("part of JS/TS workspace");
    }

    let hint_text = if hints.is_empty() {
        String::new()
    } else {
        format!(" Hints: {}.", hints.join("; "))
    };

    format!(
        "Workspace package `{unit_key}` from Cargo/npm metadata.{hint_text} \
         Rename to match how your team talks about the system; add relationships that reflect runtime behavior."
    )
}

fn is_workspace_unit(node: &Node) -> bool {
    node.id.starts_with("crate:") || node.id.starts_with("npm:")
}

fn workspace_unit_key(node: &Node) -> Option<String> {
    if let Some(name) = node.id.strip_prefix("crate:") {
        return Some(name.to_string());
    }
    node.id.strip_prefix("npm:").map(str::to_string)
}

fn is_manifest_edge(edge: &Edge) -> bool {
    edge.evidence
        .iter()
        .any(|e| MANIFEST_EDGE_RULES.contains(&e.rule.as_str()))
}

fn specials_in_draft(current_item_count: usize) -> bool {
    current_item_count < MAX_SUMMARY_CONTAINERS + MAX_SUMMARY_SPECIALS
}

/// Fallback when no manifest units exist: cluster by top-level folder, ignore call graph.
fn build_path_cluster_draft_program(graph: &Graph, filename: &str) -> sruja_language::Program {
    let repo_name = repo_display_name(filename);
    let system_name = sanitize_identifier(&repo_name);

    let mut node_to_cluster: HashMap<String, String> = HashMap::new();
    let mut cluster_counts: HashMap<String, usize> = HashMap::new();
    let mut cluster_meta: HashMap<String, (String, Option<String>)> = HashMap::new();
    let mut specials: Vec<&Node> = Vec::new();

    for node in &graph.nodes {
        if is_noise_node(node) {
            continue;
        }
        if let Some(path) = node.path.as_deref() {
            if !path_production_relevant(path) {
                continue;
            }
        }

        if is_special_kind(&node.kind) {
            if specials.len() < MAX_SUMMARY_SPECIALS {
                specials.push(node);
            }
            continue;
        }

        if !is_summary_eligible_kind(&node.kind) {
            continue;
        }

        let Some(cluster_key) = bucket_key_for_node(node) else {
            continue;
        };

        node_to_cluster.insert(node.id.clone(), cluster_key.clone());
        *cluster_counts.entry(cluster_key.clone()).or_insert(0) += 1;
        cluster_meta
            .entry(cluster_key.clone())
            .or_insert_with(|| (humanize_identifier(&cluster_key), infer_technology(node)));
    }

    let mut ranked_clusters: Vec<(String, usize)> = cluster_counts.into_iter().collect();
    ranked_clusters.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let other_cluster = "OtherModules".to_string();
    let kept: Vec<String> = ranked_clusters
        .iter()
        .take(MAX_SUMMARY_CONTAINERS)
        .map(|(k, _)| k.clone())
        .collect();
    let collapsed_count: usize = ranked_clusters
        .iter()
        .skip(MAX_SUMMARY_CONTAINERS)
        .map(|(_, n)| n)
        .sum();

    let mut buckets: Vec<SummaryBucket> = kept
        .iter()
        .filter_map(|key| {
            cluster_meta
                .get(key)
                .map(|(display_title, technology)| SummaryBucket {
                    key: key.clone(),
                    display_title: display_title.clone(),
                    technology: technology.clone(),
                    module_count: ranked_clusters
                        .iter()
                        .find(|(k, _)| k == key)
                        .map(|(_, n)| *n)
                        .unwrap_or(0),
                })
        })
        .collect();

    if collapsed_count > 0 {
        buckets.push(SummaryBucket {
            key: other_cluster.clone(),
            display_title: "Other modules".to_string(),
            technology: None,
            module_count: collapsed_count,
        });
    }

    let bucket_id: HashMap<String, String> = buckets
        .iter()
        .map(|b| {
            (
                b.key.clone(),
                sanitize_identifier(&title_case(&b.display_title)),
            )
        })
        .collect();

    let mut system_items: Vec<sruja_language::ElementDefBodyItem> = Vec::new();

    for node in &specials {
        let name = sanitize_identifier(&unique_special_name(node));
        let (kind, sub_kind) = element_kind_for_node(node.kind.clone());
        system_items.push(sruja_language::ElementDefBodyItem::ElementDef(Box::new(
            make_element_def(
                filename,
                name,
                kind,
                sub_kind,
                node.label.clone(),
                node.technology.clone().or_else(|| infer_technology(node)),
                format!(
                    "Inferred {} from scan — verify before promoting.",
                    node.kind.as_str()
                ),
            ),
        )));
    }

    for bucket in &buckets {
        let container_name = bucket_id
            .get(&bucket.key)
            .cloned()
            .unwrap_or_else(|| sanitize_identifier(&bucket.key));
        let tech = bucket
            .technology
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());
        let desc = if bucket.key == "OtherModules" {
            format!(
                "Collapsed {0} scan paths (cap {MAX_SUMMARY_CONTAINERS}). \
                 No workspace manifest found — run from a Cargo/npm monorepo root for better boundaries.",
                bucket.module_count
            )
        } else {
            format!(
                "Folder cluster of {} scan symbol(s). Structural hint only — not reviewed architecture.",
                bucket.module_count
            )
        };

        system_items.push(sruja_language::ElementDefBodyItem::ElementDef(Box::new(
            make_element_def(
                filename,
                container_name,
                sruja_language::ElementKind::Container,
                None,
                bucket.display_title.clone(),
                Some(tech),
                desc,
            ),
        )));
    }

    let folder_desc = format!(
        "Structural folder map for {repo_name} (no Cargo/npm workspace detected). \
         Tree-sitter call graphs are intentionally omitted. Add repo.sruja with domain boundaries and runtime flows."
    );
    finish_system_program(filename, repo_name, system_name, system_items, folder_desc)
}

fn finish_system_program(
    filename: &str,
    repo_name: String,
    system_name: String,
    system_items: Vec<sruja_language::ElementDefBodyItem>,
    description: String,
) -> sruja_language::Program {
    let system_body = sruja_language::ElementDefBody {
        description: Some(description),
        items: system_items,
        ..Default::default()
    };

    let system_def = sruja_language::ElementDef {
        location: source_loc(filename),
        assignment: sruja_language::ElementAssignment {
            location: source_loc(filename),
            name: system_name,
            kind: sruja_language::ElementKind::System,
            sub_kind: None,
            title: Some(repo_name),
            tag_refs: Vec::new(),
            body: Some(system_body),
        },
    };
    let items = vec![sruja_language::TopLevelItem::ElementDef(Box::new(
        system_def,
    ))];
    sruja_language::Program::new().with_items(items)
}

/// Skip tree-sitter directory modules and file-level symbols — they produce call-graph noise.
fn is_noise_node(node: &Node) -> bool {
    node.id.starts_with("module:")
        || node
            .path
            .as_deref()
            .is_some_and(|p| p.replace('\\', "/").contains("/src/") && !p.ends_with("Cargo.toml"))
}

fn source_loc(filename: &str) -> sruja_diagnostics::SourceLocation {
    sruja_diagnostics::SourceLocation::new(filename.to_string(), 1, 1)
}

fn make_element_def(
    filename: &str,
    name: String,
    kind: sruja_language::ElementKind,
    sub_kind: Option<String>,
    title: String,
    technology: Option<String>,
    description: String,
) -> sruja_language::ElementDef {
    sruja_language::ElementDef {
        location: source_loc(filename),
        assignment: sruja_language::ElementAssignment {
            location: source_loc(filename),
            name,
            kind,
            sub_kind,
            title: Some(title),
            tag_refs: Vec::new(),
            body: Some(sruja_language::ElementDefBody {
                description: Some(description),
                technology,
                ..Default::default()
            }),
        },
    }
}

fn repo_display_name(filename: &str) -> String {
    let path = std::path::Path::new(filename);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && parent != std::path::Path::new(".") {
            return parent
                .file_name()
                .and_then(|n| n.to_str())
                .map(humanize_identifier)
                .unwrap_or_else(|| "MySystem".to_string());
        }
    }
    std::env::current_dir()
        .ok()
        .and_then(|d| {
            d.file_name()
                .and_then(|n| n.to_str().map(|s| s.to_string()))
        })
        .map(|s| humanize_identifier(&s))
        .unwrap_or_else(|| "MySystem".to_string())
}

fn is_special_kind(kind: &NodeKind) -> bool {
    matches!(
        kind.as_str(),
        NodeKind::DATABASE | NodeKind::EXTERNAL_API | NodeKind::QUEUE
    )
}

fn is_summary_eligible_kind(kind: &NodeKind) -> bool {
    matches!(
        kind.as_str(),
        NodeKind::MODULE
            | NodeKind::SERVICE
            | NodeKind::CONTAINER
            | NodeKind::COMPONENT
            | NodeKind::FRONTEND
    )
}

fn bucket_key_for_node(node: &Node) -> Option<String> {
    if matches!(
        node.kind.as_str(),
        NodeKind::SERVICE | NodeKind::CONTAINER | NodeKind::FRONTEND | NodeKind::COMPONENT
    ) {
        return Some(format!("svc:{}", sanitize_identifier(&node.id)));
    }

    let path = node.path.as_deref()?;
    cluster_key_from_path(path)
}

fn cluster_key_from_path(path: &str) -> Option<String> {
    let normalized = path.replace('\\', "/");
    if !path_production_relevant(&normalized) {
        return None;
    }

    let parts: Vec<&str> = normalized
        .split('/')
        .filter(|p| !p.is_empty() && *p != ".")
        .collect();

    if parts.is_empty() {
        return None;
    }

    const SKIP_SEGMENTS: &[&str] = &[
        "test",
        "tests",
        "testing",
        "fixtures",
        "fixture",
        "examples",
        "example",
        "book",
        "e2e",
        "target",
        "node_modules",
        "dist",
        "build",
        "vendor",
        ".git",
    ];
    if parts
        .iter()
        .any(|p| SKIP_SEGMENTS.contains(&p.to_ascii_lowercase().as_str()))
        && parts.first() != Some(&"crates")
        && parts.first() != Some(&"packages")
    {
        return None;
    }

    if parts[0] == "crates" && parts.len() >= 2 {
        return Some(parts[1].to_string());
    }
    if matches!(
        parts[0],
        "packages" | "apps" | "services" | "cmd" | "internal"
    ) && parts.len() >= 2
    {
        return Some(parts[1].to_string());
    }
    if parts[0] == "src" {
        return Some("application".to_string());
    }

    Some(parts[0].to_string())
}

fn infer_technology(node: &Node) -> Option<String> {
    if let Some(tech) = &node.technology {
        if !tech.is_empty() && tech != "Unknown" {
            return Some(tech.clone());
        }
    }
    let path = node.path.as_deref()?;
    let normalized = path.replace('\\', "/").to_ascii_lowercase();
    if normalized.contains("cargo.toml") || normalized.ends_with(".rs") {
        return Some("Rust".to_string());
    }
    if normalized.contains("package.json")
        || normalized.ends_with(".ts")
        || normalized.ends_with(".tsx")
    {
        return Some("TypeScript".to_string());
    }
    if normalized.ends_with(".go") {
        return Some("Go".to_string());
    }
    if normalized.ends_with(".py") {
        return Some("Python".to_string());
    }
    None
}

fn humanize_identifier(raw: &str) -> String {
    let cleaned = raw.replace(['-', '.'], "_");
    title_case(&cleaned)
}

fn title_case(raw: &str) -> String {
    raw.split(['_', '-', '.'])
        .filter(|s| !s.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn unique_special_name(node: &Node) -> String {
    sanitize_identifier(&node.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_scan::graph::EdgeConfidence;
    use sruja_scan::EdgeEvidence;

    fn test_node(id: &str, path: &str, kind: NodeKind) -> Node {
        Node {
            id: id.to_string(),
            kind,
            label: id.to_string(),
            path: Some(path.to_string()),
            ..Default::default()
        }
    }

    fn crate_node(name: &str) -> Node {
        Node {
            id: format!("crate:{name}"),
            kind: NodeKind::new(NodeKind::MODULE),
            label: name.to_string(),
            technology: Some("Rust".to_string()),
            path: Some(format!("crates/{name}/Cargo.toml")),
            ..Default::default()
        }
    }

    #[test]
    fn workspace_draft_ignores_call_graph() {
        let mut g = Graph::default();
        g.nodes.push(crate_node("foo"));
        g.nodes.push(crate_node("bar"));
        g.nodes.push(test_node(
            "module:crates_foo_src",
            "crates/foo/src/lib.rs",
            NodeKind::new(NodeKind::MODULE),
        ));
        g.edges.push(Edge {
            source: "module:crates_foo_src".into(),
            target: "crate:bar".into(),
            kind: sruja_scan::EdgeKind::new(sruja_scan::EdgeKind::CALLS),
            evidence: vec![EdgeEvidence {
                rule: "imports".into(),
                file: None,
                line: None,
                detail: None,
            }],
            confidence: EdgeConfidence::default(),
        });
        g.edges.push(Edge {
            source: "crate:foo".into(),
            target: "crate:bar".into(),
            kind: sruja_scan::EdgeKind::new(sruja_scan::EdgeKind::CALLS),
            evidence: vec![EdgeEvidence {
                rule: "cargo_metadata_dep".into(),
                file: None,
                line: None,
                detail: None,
            }],
            confidence: EdgeConfidence::default(),
        });

        let printed = sruja_export::DslPrinter::new()
            .print(&build_summary_draft_program(&g, "repo.sruja.draft"));
        assert!(printed.contains("depends on (workspace)"));
        assert!(!printed.contains("imports"));
        assert!(!printed.contains("module:crates"));
    }

    #[test]
    fn workspace_draft_prefers_crates_over_directories() {
        let mut g = Graph::default();
        g.nodes.push(crate_node("sruja-cli"));
        g.nodes.push(crate_node("sruja-engine"));
        g.nodes.push(test_node(
            "module:crates_sruja_cli_src_commands",
            "crates/sruja-cli/src/commands/mod.rs",
            NodeKind::new(NodeKind::MODULE),
        ));

        let printed = sruja_export::DslPrinter::new()
            .print(&build_summary_draft_program(&g, "repo.sruja.draft"));
        assert!(printed.contains("sruja-cli") || printed.contains("SrujaCli"));
        assert!(printed.contains("sruja-engine") || printed.contains("SrujaEngine"));
        assert!(!printed.contains("commands"));
    }

    #[test]
    fn path_fallback_caps_buckets_without_workspace() {
        let mut g = Graph::default();
        for i in 0..25 {
            g.nodes.push(test_node(
                &format!("file{i}"),
                &format!("crates/crate{i}/src/lib.rs"),
                NodeKind::new(NodeKind::MODULE),
            ));
        }
        let printed = sruja_export::DslPrinter::new()
            .print(&build_summary_draft_program(&g, "repo.sruja.draft"));
        let container_lines = printed
            .lines()
            .filter(|l| l.contains("= container"))
            .count();
        assert!(container_lines <= MAX_SUMMARY_CONTAINERS + 1);
    }
}
