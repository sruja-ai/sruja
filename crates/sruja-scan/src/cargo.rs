use crate::graph::{Edge, EdgeEvidence, EdgeKind, Graph, Node, NodeKind};
use crate::ScanError;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
    resolve: Option<CargoResolve>,
    workspace_members: Vec<String>,
    workspace_root: String,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    id: String,
    name: String,
    manifest_path: String,
    dependencies: Vec<CargoDependency>,
}

#[derive(Debug, Deserialize)]
struct CargoDependency {
    name: String,
}

#[derive(Debug, Deserialize)]
struct CargoResolve {
    nodes: Vec<CargoResolveNode>,
}

#[derive(Debug, Deserialize)]
struct CargoResolveNode {
    id: String,
    deps: Vec<CargoResolveDep>,
}

#[derive(Debug, Deserialize)]
struct CargoResolveDep {
    pkg: String,
}

pub(crate) fn scan_cargo_repo(repo_root: &Path) -> Result<Graph, ScanError> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .current_dir(repo_root)
        .output()?;

    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(ScanError::CargoMetadata { message });
    }

    let metadata: CargoMetadata = serde_json::from_slice(&output.stdout)?;

    let workspace_member_ids: HashSet<String> = metadata.workspace_members.into_iter().collect();
    let workspace_root = PathBuf::from(metadata.workspace_root);

    let mut packages_by_id: HashMap<String, CargoPackage> = HashMap::new();
    for p in metadata.packages {
        packages_by_id.insert(p.id.clone(), p);
    }

    let mut graph = Graph::new();
    graph.metadata.insert(
        "scanner".to_string(),
        "cargo_metadata(format_version=1)".to_string(),
    );

    let mut node_id_by_pkg_id: HashMap<String, String> = HashMap::new();

    for pkg_id in workspace_member_ids.iter() {
        let Some(pkg) = packages_by_id.get(pkg_id) else {
            continue;
        };

        let node_id = format!("crate:{}", pkg.name);
        node_id_by_pkg_id.insert(pkg_id.clone(), node_id.clone());

        let manifest_path = PathBuf::from(&pkg.manifest_path);
        let rel_manifest = manifest_path
            .strip_prefix(&workspace_root)
            .unwrap_or(&manifest_path);

        let mut node = Node {
            id: node_id,
            kind: NodeKind::Module,
            label: pkg.name.clone(),
            technology: Some("Rust".to_string()),
            path: Some(rel_manifest.to_string_lossy().to_string()),
            metadata: HashMap::new(),
        };

        // Preserve a hint for later heuristics without encoding policy here.
        if pkg
            .dependencies
            .iter()
            .any(|d| d.name.eq_ignore_ascii_case("sqlx") || d.name.eq_ignore_ascii_case("diesel"))
        {
            node.metadata
                .insert("hint:db_client".to_string(), "true".to_string());
        }
        if pkg.dependencies.iter().any(|d| {
            d.name.eq_ignore_ascii_case("reqwest")
                || d.name.eq_ignore_ascii_case("hyper")
                || d.name.eq_ignore_ascii_case("ureq")
        }) {
            node.metadata
                .insert("hint:http_client".to_string(), "true".to_string());
        }

        graph.nodes.push(node);
    }

    let Some(resolve) = metadata.resolve else {
        return Ok(graph);
    };

    for node in resolve.nodes {
        if !workspace_member_ids.contains(&node.id) {
            continue;
        }
        let Some(source_node_id) = node_id_by_pkg_id.get(&node.id) else {
            continue;
        };

        for dep in node.deps {
            if !workspace_member_ids.contains(&dep.pkg) {
                continue;
            }
            let Some(target_node_id) = node_id_by_pkg_id.get(&dep.pkg) else {
                continue;
            };

            graph.edges.push(Edge {
                source: source_node_id.clone(),
                target: target_node_id.clone(),
                kind: EdgeKind::Calls,
                evidence: vec![EdgeEvidence {
                    rule: "cargo_metadata_dep".to_string(),
                    file: None,
                    line: None,
                    detail: None,
                }],
            });
        }
    }

    Ok(graph)
}
