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
            let Ok(pkg) = serde_json::from_str::<PackageJson>(&s) else {
                continue;
            };
            if pkg.name.is_none() {
                continue;
            }
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
        let Some(name) = pkg.name.as_deref() else {
            continue;
        };
        let name = name.to_string();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_npm_repo_invalid_json_returns_error() {
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path();
        std::fs::write(root.join("package.json"), "not valid json").expect("write");
        let result = scan_npm_repo(root);
        assert!(result.is_err());
    }

    #[test]
    fn scan_npm_repo_missing_file_returns_error() {
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path();
        let result = scan_npm_repo(root);
        assert!(result.is_err());
    }

    #[test]
    fn scan_npm_repo_with_workspaces_creates_nodes_and_edges_with_deduped_deps() {
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path();

        std::fs::create_dir_all(root.join("packages/a")).expect("mkdir");
        std::fs::create_dir_all(root.join("packages/b")).expect("mkdir");

        std::fs::write(
            root.join("package.json"),
            r#"{"name":"root","workspaces":["packages/*"]}"#,
        )
        .expect("write root");

        std::fs::write(
            root.join("packages/a/package.json"),
            r#"{"name":"a","dependencies":{"b":"1.0.0"},"devDependencies":{"b":"1.0.0"}}"#,
        )
        .expect("write a");
        std::fs::write(
            root.join("packages/b/package.json"),
            r#"{"name":"b","dependencies":{}}"#,
        )
        .expect("write b");

        let graph = scan_npm_repo(root).expect("scan");

        assert_eq!(graph.nodes.len(), 2);
        assert!(graph.nodes.iter().any(|n| n.id == "npm:a"));
        assert!(graph.nodes.iter().any(|n| n.id == "npm:b"));

        let a = graph.nodes.iter().find(|n| n.id == "npm:a").expect("a");
        assert_eq!(a.path.as_deref(), Some("packages/a/package.json"));

        assert_eq!(graph.edges.len(), 1);
        let e = &graph.edges[0];
        assert_eq!(e.source, "npm:a");
        assert_eq!(e.target, "npm:b");
        assert_eq!(e.kind, EdgeKind::Calls);
        assert_eq!(e.evidence.len(), 1);
        assert!(e.evidence[0].file.as_ref().is_some());
    }

    #[test]
    fn scan_npm_repo_without_workspaces_creates_single_root_package_node() {
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path();

        std::fs::write(
            root.join("package.json"),
            r#"{"name":"root","dependencies":{"left-pad":"1.0.0"}}"#,
        )
        .expect("write root");

        let graph = scan_npm_repo(root).expect("scan");
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id, "npm:root");
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn resolve_workspace_globs_dedups_and_supports_glob_and_single_path() {
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path();
        std::fs::create_dir_all(root.join("packages/a")).expect("mkdir");
        std::fs::create_dir_all(root.join("packages/b")).expect("mkdir");
        std::fs::write(root.join("packages/a/package.json"), r#"{"name":"a"}"#).expect("write a");
        std::fs::write(root.join("packages/b/package.json"), r#"{"name":"b"}"#).expect("write b");

        let paths =
            resolve_workspace_globs(root, &["packages/*".to_string(), "packages/a".to_string()]);
        assert_eq!(paths.len(), 2);
        assert!(paths.iter().any(|p| p.ends_with("packages/a")));
        assert!(paths.iter().any(|p| p.ends_with("packages/b")));
    }

    #[test]
    fn scan_npm_repo_with_invalid_workspace_package_json_skips_package() {
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path();

        std::fs::create_dir_all(root.join("packages/a")).expect("mkdir");
        std::fs::create_dir_all(root.join("packages/b")).expect("mkdir");

        std::fs::write(
            root.join("package.json"),
            r#"{"name":"root","workspaces":["packages/*"]}"#,
        )
        .expect("write root");

        std::fs::write(root.join("packages/a/package.json"), "not valid json").expect("write a");
        std::fs::write(root.join("packages/b/package.json"), r#"{"name":"b"}"#).expect("write b");

        let graph = scan_npm_repo(root).expect("scan");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id, "npm:b");
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn scan_npm_repo_with_workspace_package_missing_name_skips_package() {
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path();

        std::fs::create_dir_all(root.join("packages/a")).expect("mkdir");
        std::fs::create_dir_all(root.join("packages/b")).expect("mkdir");

        std::fs::write(
            root.join("package.json"),
            r#"{"name":"root","workspaces":["packages/*"]}"#,
        )
        .expect("write root");

        std::fs::write(
            root.join("packages/a/package.json"),
            r#"{"dependencies":{}}"#,
        )
        .expect("write a");
        std::fs::write(root.join("packages/b/package.json"), r#"{"name":"b"}"#).expect("write b");

        let graph = scan_npm_repo(root).expect("scan");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id, "npm:b");
        assert!(graph.edges.is_empty());
    }
}
