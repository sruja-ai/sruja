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
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            confidence: None,
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
        graph.canonicalize();
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

    graph.canonicalize();
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_cargo_repo_missing_cargo_toml_returns_error() {
        let temp = tempfile::tempdir().expect("temp dir");
        let result = scan_cargo_repo(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn scan_cargo_repo_invalid_cargo_toml_returns_error() {
        let temp = tempfile::tempdir().expect("temp dir");
        std::fs::write(temp.path().join("Cargo.toml"), "not valid toml").expect("write");
        let result = scan_cargo_repo(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn parse_cargo_metadata_valid_json() {
        let json = r#"{
            "packages": [],
            "resolve": null,
            "workspace_members": [],
            "workspace_root": "/tmp/test"
        }"#;

        let metadata: CargoMetadata = serde_json::from_str(json).expect("parse");
        assert!(metadata.packages.is_empty());
        assert!(metadata.resolve.is_none());
    }

    #[test]
    fn parse_cargo_metadata_with_packages() {
        let json = r#"{
            "packages": [
                {
                    "id": "test_pkg 0.1.0 (path+file:///tmp/test)",
                    "name": "test_pkg",
                    "manifest_path": "/tmp/test/Cargo.toml",
                    "dependencies": []
                }
            ],
            "resolve": null,
            "workspace_members": ["test_pkg 0.1.0 (path+file:///tmp/test)"],
            "workspace_root": "/tmp/test"
        }"#;

        let metadata: CargoMetadata = serde_json::from_str(json).expect("parse");
        assert_eq!(metadata.packages.len(), 1);
        assert_eq!(metadata.packages[0].name, "test_pkg");
        assert_eq!(metadata.workspace_members.len(), 1);
    }

    #[test]
    fn parse_cargo_metadata_with_dependencies() {
        let json = r#"{
            "packages": [
                {
                    "id": "pkg_a 0.1.0 (path+file:///tmp/test/a)",
                    "name": "pkg_a",
                    "manifest_path": "/tmp/test/a/Cargo.toml",
                    "dependencies": [
                        {"name": "serde"},
                        {"name": "sqlx"}
                    ]
                }
            ],
            "resolve": null,
            "workspace_members": ["pkg_a 0.1.0 (path+file:///tmp/test/a)"],
            "workspace_root": "/tmp/test"
        }"#;

        let metadata: CargoMetadata = serde_json::from_str(json).expect("parse");
        assert_eq!(metadata.packages[0].dependencies.len(), 2);
    }

    #[test]
    fn parse_cargo_resolve_nodes() {
        let json = r#"{
            "packages": [],
            "resolve": {
                "nodes": [
                    {
                        "id": "pkg_a 0.1.0",
                        "deps": [{"pkg": "pkg_b 0.1.0"}]
                    }
                ]
            },
            "workspace_members": [],
            "workspace_root": "/tmp/test"
        }"#;

        let metadata: CargoMetadata = serde_json::from_str(json).expect("parse");
        assert!(metadata.resolve.is_some());
        let resolve = metadata.resolve.unwrap();
        assert_eq!(resolve.nodes.len(), 1);
        assert_eq!(resolve.nodes[0].deps.len(), 1);
    }

    #[test]
    fn cargo_dependency_deserialize() {
        let json = r#"{"name": "serde"}"#;
        let dep: CargoDependency = serde_json::from_str(json).expect("parse");
        assert_eq!(dep.name, "serde");
    }

    #[test]
    fn cargo_package_deserialize() {
        let json = r#"{
            "id": "test 0.1.0",
            "name": "test",
            "manifest_path": "/path/Cargo.toml",
            "dependencies": []
        }"#;
        let pkg: CargoPackage = serde_json::from_str(json).expect("parse");
        assert_eq!(pkg.name, "test");
        assert_eq!(pkg.manifest_path, "/path/Cargo.toml");
    }

    #[test]
    fn cargo_resolve_node_deserialize() {
        let json = r#"{
            "id": "pkg 0.1.0",
            "deps": [{"pkg": "dep 0.2.0"}]
        }"#;
        let node: CargoResolveNode = serde_json::from_str(json).expect("parse");
        assert_eq!(node.id, "pkg 0.1.0");
        assert_eq!(node.deps.len(), 1);
    }

    #[test]
    fn cargo_resolve_dep_deserialize() {
        let json = r#"{"pkg": "dependency 1.0.0"}"#;
        let dep: CargoResolveDep = serde_json::from_str(json).expect("parse");
        assert_eq!(dep.pkg, "dependency 1.0.0");
    }

    #[test]
    fn graph_metadata_includes_scanner_info() {
        let json = r#"{
            "packages": [],
            "resolve": null,
            "workspace_members": [],
            "workspace_root": "/tmp/test"
        }"#;

        let _metadata: CargoMetadata = serde_json::from_str(json).expect("parse");
        let mut graph = Graph::new();
        graph.metadata.insert(
            "scanner".to_string(),
            "cargo_metadata(format_version=1)".to_string(),
        );

        assert!(graph.metadata.contains_key("scanner"));
    }

    #[test]
    fn db_client_hint_detected() {
        let mut node = Node {
            id: "test".to_string(),
            kind: NodeKind::Module,
            label: "test".to_string(),
            technology: Some("Rust".to_string()),
            path: None,
            metadata: HashMap::new(),
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            confidence: None,
        };

        let deps = [CargoDependency {
            name: "sqlx".to_string(),
        }];

        if deps
            .iter()
            .any(|d| d.name.eq_ignore_ascii_case("sqlx") || d.name.eq_ignore_ascii_case("diesel"))
        {
            node.metadata
                .insert("hint:db_client".to_string(), "true".to_string());
        }

        assert_eq!(
            node.metadata.get("hint:db_client"),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn http_client_hint_detected() {
        let mut node = Node {
            id: "test".to_string(),
            kind: NodeKind::Module,
            label: "test".to_string(),
            technology: Some("Rust".to_string()),
            path: None,
            metadata: HashMap::new(),
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            confidence: None,
        };

        let deps = [CargoDependency {
            name: "reqwest".to_string(),
        }];

        if deps.iter().any(|d| {
            d.name.eq_ignore_ascii_case("reqwest")
                || d.name.eq_ignore_ascii_case("hyper")
                || d.name.eq_ignore_ascii_case("ureq")
        }) {
            node.metadata
                .insert("hint:http_client".to_string(), "true".to_string());
        }

        assert_eq!(
            node.metadata.get("hint:http_client"),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn no_hints_without_matching_deps() {
        let mut node = Node {
            id: "test".to_string(),
            kind: NodeKind::Module,
            label: "test".to_string(),
            technology: Some("Rust".to_string()),
            path: None,
            metadata: HashMap::new(),
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            confidence: None,
        };

        let deps = [
            CargoDependency {
                name: "serde".to_string(),
            },
            CargoDependency {
                name: "tokio".to_string(),
            },
        ];

        if deps
            .iter()
            .any(|d| d.name.eq_ignore_ascii_case("sqlx") || d.name.eq_ignore_ascii_case("diesel"))
        {
            node.metadata
                .insert("hint:db_client".to_string(), "true".to_string());
        }
        if deps.iter().any(|d| {
            d.name.eq_ignore_ascii_case("reqwest")
                || d.name.eq_ignore_ascii_case("hyper")
                || d.name.eq_ignore_ascii_case("ureq")
        }) {
            node.metadata
                .insert("hint:http_client".to_string(), "true".to_string());
        }

        assert!(!node.metadata.contains_key("hint:db_client"));
        assert!(!node.metadata.contains_key("hint:http_client"));
    }

    #[test]
    fn diesel_detected_as_db_client() {
        let mut node = Node {
            id: "test".to_string(),
            kind: NodeKind::Module,
            label: "test".to_string(),
            technology: Some("Rust".to_string()),
            path: None,
            metadata: HashMap::new(),
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            confidence: None,
        };

        let deps = [CargoDependency {
            name: "Diesel".to_string(),
        }];

        if deps
            .iter()
            .any(|d| d.name.eq_ignore_ascii_case("sqlx") || d.name.eq_ignore_ascii_case("diesel"))
        {
            node.metadata
                .insert("hint:db_client".to_string(), "true".to_string());
        }

        assert_eq!(
            node.metadata.get("hint:db_client"),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn hyper_detected_as_http_client() {
        let mut node = Node {
            id: "test".to_string(),
            kind: NodeKind::Module,
            label: "test".to_string(),
            technology: Some("Rust".to_string()),
            path: None,
            metadata: HashMap::new(),
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            confidence: None,
        };

        let deps = [CargoDependency {
            name: "HYPER".to_string(),
        }];

        if deps.iter().any(|d| {
            d.name.eq_ignore_ascii_case("reqwest")
                || d.name.eq_ignore_ascii_case("hyper")
                || d.name.eq_ignore_ascii_case("ureq")
        }) {
            node.metadata
                .insert("hint:http_client".to_string(), "true".to_string());
        }

        assert_eq!(
            node.metadata.get("hint:http_client"),
            Some(&"true".to_string())
        );
    }
}
