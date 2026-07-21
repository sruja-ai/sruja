use super::super::super::federation::{find_system_index, infer_repo_id, load_system_index};
use super::super::types::*;
use super::focus::build_focus_context;
use crate::commands::CliError;
use sruja_scan::{Graph, NodeKind};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Try to load classification from .sruja/classification.json.
fn load_classification(
    repo: &str,
) -> Option<crate::commands::utility_domain::classify::Classification> {
    let path = Path::new(repo).join(".sruja").join("classification.json");
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn build_architecture_context(
    graph: &Graph,
    repo: &str,
    file: Option<&str>,
    intent: Option<&str>,
    depth: usize,
    max_tokens: usize,
) -> Result<ArchitectureContext, CliError> {
    let classification = load_classification(repo);

    let (total_crates, total_modules, layers, boundaries, forbidden_patterns) =
        if let Some(ref cls) = classification {
            let total_crates = cls.summary.crates;
            let total_modules = cls.summary.crates.unwrap_or(cls.summary.source_files);

            let layers: Vec<LayerInfo> = cls
                .layers
                .iter()
                .map(|l| LayerInfo {
                    name: l.name.clone(),
                    modules: l.members.len(),
                    can_depend_on: vec![],
                })
                .collect();

            let boundaries: Vec<BoundaryRule> = cls
                .boundaries
                .iter()
                .map(|b| BoundaryRule {
                    from: b.from.clone(),
                    to: b.to.clone(),
                    allowed: b.allowed,
                    reason: b.reason.clone(),
                })
                .collect();

            let forbidden_patterns = cls.forbidden_patterns.clone();

            (
                total_crates,
                total_modules,
                layers,
                boundaries,
                forbidden_patterns,
            )
        } else {
            let is_rust_workspace = count_crates(repo).is_some();

            let (total_crates, total_modules) = if let Some(crate_count) = count_crates(repo) {
                (Some(crate_count), crate_count)
            } else {
                (
                    None,
                    count_file_level_kind(graph, NodeKind::new(NodeKind::MODULE)),
                )
            };

            let layers = if is_rust_workspace {
                let tiers = classify_crate_tiers(repo);
                tiers
                    .into_iter()
                    .map(|(name, crates)| LayerInfo {
                        name,
                        modules: crates.len(),
                        can_depend_on: vec![],
                    })
                    .collect()
            } else {
                infer_layers(graph)
            };

            let boundaries = if is_rust_workspace {
                infer_boundaries_from_deps(repo)
            } else {
                infer_boundaries(graph)
            };

            let forbidden_patterns = if is_rust_workspace {
                vec![
                    "Lower-tier crates must not depend on higher-tier crates".to_string(),
                    "sruja-cli is the top-level aggregator — no other crate should depend on it"
                        .to_string(),
                    "WASM-only crates must not use native-only APIs (tree-sitter, fastembed)"
                        .to_string(),
                ]
            } else {
                vec![
                    "Avoid direct database access from routes/handlers - use a service layer"
                        .to_string(),
                    "Do not import internal modules from other services directly".to_string(),
                    "UI components should not directly call database layers".to_string(),
                ]
            };

            (
                total_crates,
                total_modules,
                layers,
                boundaries,
                forbidden_patterns,
            )
        };

    let services = count_file_level_kind(graph, NodeKind::new(NodeKind::SERVICE));
    let databases = count_file_level_kind(graph, NodeKind::new(NodeKind::DATABASE));
    let external_apis = count_file_level_kind(graph, NodeKind::new(NodeKind::EXTERNAL_API));

    let max_boundary_rules = if max_tokens < 3000 { 5 } else { 30 };
    let mut boundaries = boundaries;
    boundaries.truncate(max_boundary_rules);

    let centrality = crate::commands::compute_all_centrality_cached(Path::new(repo), graph, false)?;

    let focus = file
        .map(|f| build_focus_context(graph, repo, f, intent, depth, max_tokens, &centrality))
        .transpose()?;

    let system_context = build_system_context(repo);

    let mut active_decisions = Vec::new();
    let adr_dir = Path::new(repo)
        .join("docs")
        .join("architecture")
        .join("decisions");
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
            total_crates,
            total_modules,
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

pub fn count_file_level_kind(graph: &Graph, kind: NodeKind) -> usize {
    graph
        .nodes
        .iter()
        .filter(|n| n.kind == kind && !n.id.contains('#'))
        .count()
}

pub fn count_crates(repo_root: &str) -> Option<usize> {
    let crates_dir = Path::new(repo_root).join("crates");
    if !crates_dir.is_dir() {
        return None;
    }
    let count = std::fs::read_dir(&crates_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.is_dir() && path.join("Cargo.toml").exists()
        })
        .count();
    Some(count)
}

pub fn classify_crate_tiers(repo_root: &str) -> Vec<(String, Vec<String>)> {
    let crates_dir = Path::new(repo_root).join("crates");
    if !crates_dir.is_dir() {
        return Vec::new();
    }

    let mut crate_names: Vec<String> = std::fs::read_dir(&crates_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.is_dir() && path.join("Cargo.toml").exists()
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    crate_names.sort();

    let delivery: Vec<String> = crate_names
        .iter()
        .filter(|n| n.ends_with("-cli") || n.ends_with("-wasm"))
        .cloned()
        .collect();
    let secondary: Vec<String> = crate_names
        .iter()
        .filter(|n| {
            n.ends_with("-agent")
                || n.ends_with("-memory")
                || n.ends_with("-diff")
                || n.ends_with("-intent")
        })
        .cloned()
        .collect();
    let extraction: Vec<String> = crate_names
        .iter()
        .filter(|n| n.ends_with("-extract") || n.ends_with("-export"))
        .cloned()
        .collect();
    let core: Vec<String> = crate_names
        .iter()
        .filter(|n| !delivery.contains(n) && !secondary.contains(n) && !extraction.contains(n))
        .cloned()
        .collect();

    let mut tiers = Vec::new();
    if !core.is_empty() {
        tiers.push(("Core Engine".to_string(), core));
    }
    if !extraction.is_empty() {
        tiers.push(("Extraction".to_string(), extraction));
    }
    if !delivery.is_empty() {
        tiers.push(("Delivery".to_string(), delivery));
    }
    if !secondary.is_empty() {
        tiers.push(("Secondary".to_string(), secondary));
    }
    tiers
}

pub fn infer_boundaries_from_deps(repo_root: &str) -> Vec<BoundaryRule> {
    let crates_dir = Path::new(repo_root).join("crates");
    if !crates_dir.is_dir() {
        return Vec::new();
    }

    let mut boundaries = Vec::new();

    let crate_entries: Vec<(String, Vec<String>)> = std::fs::read_dir(&crates_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.is_dir() && path.join("Cargo.toml").exists()
        })
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let cargo_toml = std::fs::read_to_string(e.path().join("Cargo.toml")).ok()?;
            let deps: Vec<String> = cargo_toml
                .lines()
                .filter_map(|line| {
                    let trimmed = line.trim();
                    if trimmed.starts_with("sruja-") && trimmed.contains("path") {
                        trimmed.split('=').next().map(|s| s.trim().to_string())
                    } else {
                        None
                    }
                })
                .collect();
            Some((name, deps))
        })
        .collect();

    for (name, deps) in &crate_entries {
        if deps.contains(&"sruja-cli".to_string()) && name != "sruja-cli" {
            boundaries.push(BoundaryRule {
                from: name.clone(),
                to: "sruja-cli".to_string(),
                allowed: false,
                reason: format!(
                    "{} should not depend on sruja-cli (CLI is the top-level aggregator)",
                    name
                ),
            });
        }
    }

    boundaries
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

pub fn infer_layers(graph: &Graph) -> Vec<LayerInfo> {
    let mut layer_counts: HashMap<String, usize> = HashMap::new();

    for node in &graph.nodes {
        if node.kind == NodeKind::MODULE && !node.id.contains('#') {
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
        .filter(|n| n.kind == NodeKind::SERVICE && !n.id.contains('#'))
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
