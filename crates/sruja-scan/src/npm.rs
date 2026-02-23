//! Node/npm backend: infers graph from package.json and workspaces.

use crate::graph::{Edge, EdgeEvidence, EdgeKind, Graph, Node, NodeKind};
use crate::ScanError;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize)]
struct PackageJson {
    name: Option<String>,
    #[serde(rename = "workspaces")]
    workspaces: Option<Workspaces>,
    dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Workspaces {
    Array(Vec<String>),
}

/// Resolve workspace globs to paths that contain package.json.
fn resolve_workspace_globs(repo_root: &Path, workspaces: &[String]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for ws in workspaces {
        let path = repo_root.join(ws.trim_end_matches('*').trim_end_matches('/'));
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&path) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_dir() && p.join("package.json").exists() {
                        out.push(p);
                    }
                }
            }
        }
        let single = repo_root.join(ws.trim_end_matches('/'));
        if single.join("package.json").exists() {
            out.push(single);
        }
    }
    out.sort();
    out.dedup();
    out
}

pub(crate) fn scan_npm_repo(repo_root: &Path) -> Result<Graph, ScanError> {
    let root_manifest = repo_root.join("package.json");
    let content = std::fs::read_to_string(&root_manifest)?;
    let root: PackageJson = serde_json::from_str(&content)?;

    let mut graph = Graph::new();
    graph
        .metadata
        .insert("scanner".to_string(), "npm_package_json".to_string());

    let workspace_paths: Vec<PathBuf> = root
        .workspaces
        .as_ref()
        .map(|w| match w {
            Workspaces::Array(arr) => resolve_workspace_globs(repo_root, arr),
        })
        .unwrap_or_default();

    let packages: Vec<(PathBuf, PackageJson)> = if workspace_paths.is_empty() {
        let name = root.name.as_deref().unwrap_or("root").to_string();
        let pkg = PackageJson {
            name: Some(name.clone()),
            workspaces: None,
            dependencies: root.dependencies.clone(),
            dev_dependencies: root.dev_dependencies.clone(),
        };
        vec![(repo_root.to_path_buf(), pkg)]
    } else {
        let mut out = Vec::new();
        for p in &workspace_paths {
            let manifest_path = p.join("package.json");
            let Ok(s) = std::fs::read_to_string(&manifest_path) else {
                continue;
            };
            let pkg: PackageJson = serde_json::from_str(&s).unwrap_or_default();
            out.push((p.clone(), pkg));
        }
        out
    };

    let name_to_id: HashMap<String, String> = packages
        .iter()
        .filter_map(|(_path, pkg)| {
            let name = pkg.name.as_ref()?;
            let id = format!("npm:{}", name);
            Some((name.clone(), id))
        })
        .collect();

    for (rel_base, pkg) in &packages {
        let name = pkg.name.as_deref().unwrap_or("(anonymous)").to_string();
        let id = format!("npm:{}", name);
        let rel_path = rel_base
            .strip_prefix(repo_root)
            .unwrap_or(rel_base)
            .join("package.json");
        let path_str = rel_path.to_string_lossy().to_string();

        let mut node = Node {
            id: id.clone(),
            kind: NodeKind::Module,
            label: name,
            technology: Some("Node.js".to_string()),
            path: Some(path_str),
            metadata: HashMap::new(),
        };

        for (dep_name, _) in pkg
            .dependencies
            .iter()
            .flatten()
            .chain(pkg.dev_dependencies.iter().flatten())
        {
            if name_to_id.contains_key(dep_name) {
                node.metadata
                    .insert("hint:workspace_dep".to_string(), "true".to_string());
                break;
            }
        }
        graph.nodes.push(node);
    }

    let mut seen_edges: HashSet<(String, String)> = HashSet::new();
    for (pkg_path, pkg) in &packages {
        let source_name = match &pkg.name {
            Some(n) => n,
            None => continue,
        };
        let source_id = match name_to_id.get(source_name) {
            Some(id) => id.clone(),
            None => continue,
        };
        let deps: Vec<&String> = pkg
            .dependencies
            .iter()
            .flatten()
            .chain(pkg.dev_dependencies.iter().flatten())
            .map(|(k, _)| k)
            .collect();
        for dep_name in deps {
            let Some(target_id) = name_to_id.get(dep_name) else {
                continue;
            };
            if source_id == *target_id {
                continue;
            }
            if !seen_edges.insert((source_id.clone(), target_id.clone())) {
                continue;
            }
            graph.edges.push(Edge {
                source: source_id.clone(),
                target: target_id.clone(),
                kind: EdgeKind::Calls,
                evidence: vec![EdgeEvidence {
                    rule: "package_json_dep".to_string(),
                    file: Some(pkg_path.join("package.json").to_string_lossy().to_string()),
                    line: None,
                    detail: None,
                }],
            });
        }
    }

    Ok(graph)
}
