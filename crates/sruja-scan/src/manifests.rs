use crate::graph::{Graph, Node, NodeKind};
use crate::ScanError;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct K8sManifest {
    #[serde(rename = "apiVersion")]
    api_version: Option<String>,
    kind: Option<String>,
    metadata: Option<Metadata>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Metadata {
    name: Option<String>,
    namespace: Option<String>,
}

pub fn scan_other_manifests(repo_root: &Path) -> Result<Graph, ScanError> {
    let mut graph = Graph::new();

    // 1. Docker Discovery
    if let Ok(docker_graph) = discover_docker(repo_root) {
        graph.merge(docker_graph);
    }

    // 2. OpenAPI Discovery
    if let Ok(openapi_graph) = discover_openapi(repo_root) {
        graph.merge(openapi_graph);
    }

    // 3. Kubernetes Discovery
    if let Ok(k8s_graph) = discover_k8s(repo_root) {
        graph.merge(k8s_graph);
    }

    Ok(graph)
}

fn discover_docker(repo_root: &Path) -> Result<Graph, ScanError> {
    let mut graph = Graph::new();

    // Check for Dockerfile
    let dockerfile = repo_root.join("Dockerfile");
    if dockerfile.exists() {
        let node = Node {
            id: "docker:Dockerfile".to_string(),
            kind: NodeKind::Container,
            label: "Docker Image".to_string(),
            technology: Some("Docker".to_string()),
            path: Some("Dockerfile".to_string()),
            metadata: {
                let mut m = HashMap::new();
                m.insert("source_manifest".to_string(), "dockerfile".to_string());
                m
            },
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            confidence: None,
        };
        graph.nodes.push(node);
    }

    // Check for docker-compose
    for name in &[
        "docker-compose.yaml",
        "docker-compose.yml",
        "compose.yaml",
        "compose.yml",
    ] {
        let path = repo_root.join(name);
        if path.exists() {
            let node = Node {
                id: format!("docker:{}", name),
                kind: NodeKind::System,
                label: "Docker Compose".to_string(),
                technology: Some("Docker Compose".to_string()),
                path: Some(name.to_string()),
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("source_manifest".to_string(), "docker-compose".to_string());
                    m
                },
                canonical_id: None,
                aliases: Vec::new(),
                owner: None,
                domain: None,
                criticality: None,
                sources: Vec::new(),
                confidence: None,
            };
            graph.nodes.push(node);
        }
    }

    Ok(graph)
}

fn discover_openapi(repo_root: &Path) -> Result<Graph, ScanError> {
    let mut graph = Graph::new();
    let candidates = [
        "openapi.yaml",
        "openapi.yml",
        "openapi.json",
        "swagger.yaml",
        "swagger.yml",
        "swagger.json",
        "docs/openapi.yaml",
        "docs/openapi.json",
    ];

    for name in &candidates {
        let path = repo_root.join(name);
        if path.exists() {
            let id = format!("api:{}", name.replace("/", "_"));
            let node = Node {
                id,
                kind: NodeKind::ExternalApi,
                label: "OpenAPI Spec".to_string(),
                technology: Some("OpenAPI".to_string()),
                path: Some(name.to_string()),
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("source_manifest".to_string(), "openapi".to_string());
                    m
                },
                canonical_id: None,
                aliases: Vec::new(),
                owner: None,
                domain: None,
                criticality: None,
                sources: Vec::new(),
                confidence: None,
            };
            graph.nodes.push(node);
        }
    }

    Ok(graph)
}

fn discover_k8s(repo_root: &Path) -> Result<Graph, ScanError> {
    let mut graph = Graph::new();
    // Simple heuristic for K8s discovery in likely directories
    let k8s_dirs = ["k8s", "kubernetes", "deploy", "deployment"];

    for dir_name in &k8s_dirs {
        let dir_path = repo_root.join(dir_name);
        if !dir_path.is_dir() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .extension()
                    .is_some_and(|ext| ext == "yaml" || ext == "yml")
                {
                    let Ok(content) = std::fs::read_to_string(&path) else {
                        continue;
                    };
                    // Very loose check for K8s manifest
                    if content.contains("apiVersion:") && content.contains("kind:") {
                        let rel_path = path.strip_prefix(repo_root).unwrap_or(&path);
                        let path_str = rel_path.to_string_lossy().to_string();

                        // Try to parse basic info
                        let mut label = "K8s Resource".to_string();
                        if let Ok(k8s) = serde_yaml::from_str::<K8sManifest>(&content) {
                            if let (Some(k), Some(m)) = (k8s.kind, k8s.metadata) {
                                if let Some(n) = m.name {
                                    label = format!("{}: {}", k, n);
                                }
                            }
                        }

                        let node = Node {
                            id: format!("k8s:{}", path_str.replace("/", "_")),
                            kind: NodeKind::System,
                            label,
                            technology: Some("Kubernetes".to_string()),
                            path: Some(path_str),
                            metadata: {
                                let mut m = HashMap::new();
                                m.insert("source_manifest".to_string(), "kubernetes".to_string());
                                m
                            },
                            canonical_id: None,
                            aliases: Vec::new(),
                            owner: None,
                            domain: None,
                            criticality: None,
                            sources: Vec::new(),
                            confidence: None,
                        };
                        graph.nodes.push(node);
                    }
                }
            }
        }
    }

    Ok(graph)
}
