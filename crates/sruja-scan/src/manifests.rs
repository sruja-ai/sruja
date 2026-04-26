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
    spec: Option<serde_yaml::Value>,
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
        let mut technology = "Docker".to_string();
        let mut description = String::new();

        if let Ok(content) = std::fs::read_to_string(&dockerfile) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("FROM ") {
                    technology = trimmed.replace("FROM ", "").to_string();
                } else if trimmed.starts_with("EXPOSE ") {
                    description.push_str(&format!("Exposes: {}; ", trimmed.replace("EXPOSE ", "")));
                } else if trimmed.starts_with("ENTRYPOINT ") || trimmed.starts_with("CMD ") {
                    let cmd = trimmed.replace("ENTRYPOINT ", "").replace("CMD ", "");
                    description.push_str(&format!("Runs: {}; ", cmd));
                }
            }
        }

        let mut metadata = HashMap::new();
        metadata.insert("source_manifest".to_string(), "dockerfile".to_string());
        if !description.is_empty() {
            metadata.insert("description".to_string(), description.trim().to_string());
        }

        let node = Node {
            id: "docker:Dockerfile".to_string(),
            kind: NodeKind::Container,
            label: "Docker Image".to_string(),
            technology: Some(technology),
            path: Some("Dockerfile".to_string()),
            metadata,
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            confidence: None,
            ..Default::default()
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
                ..Default::default()
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
                ..Default::default()
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
                        let mut description = String::new();
                        let mut technology = "Kubernetes".to_string();

                        if let Ok(k8s) = serde_yaml::from_str::<K8sManifest>(&content) {
                            if let (Some(k), Some(m)) = (k8s.kind.clone(), k8s.metadata) {
                                if let Some(n) = m.name {
                                    label = format!("{}: {}", k, n);
                                }
                            }

                            // Extract details based on kind
                            if let (Some(kind), Some(spec)) = (k8s.kind, k8s.spec) {
                                match kind.as_str() {
                                    "Deployment" | "StatefulSet" | "DaemonSet" | "Job" => {
                                        // Try to find containers
                                        if let Some(containers) = spec
                                            .get("template")
                                            .and_then(|t| t.get("spec"))
                                            .and_then(|s| s.get("containers"))
                                            .and_then(|c| c.as_sequence())
                                        {
                                            for container in containers {
                                                if let Some(image) =
                                                    container.get("image").and_then(|i| i.as_str())
                                                {
                                                    description
                                                        .push_str(&format!("Image: {}; ", image));
                                                    technology = image.to_string();
                                                }
                                                if let Some(ports) = container
                                                    .get("ports")
                                                    .and_then(|p| p.as_sequence())
                                                {
                                                    for port in ports {
                                                        if let Some(cp) = port
                                                            .get("containerPort")
                                                            .and_then(|p| p.as_u64())
                                                        {
                                                            description.push_str(&format!(
                                                                "Port: {}; ",
                                                                cp
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    "Service" => {
                                        if let Some(ports) =
                                            spec.get("ports").and_then(|p| p.as_sequence())
                                        {
                                            for port in ports {
                                                if let Some(p) =
                                                    port.get("port").and_then(|v| v.as_u64())
                                                {
                                                    description.push_str(&format!(
                                                        "Service Port: {}; ",
                                                        p
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        let mut metadata = HashMap::new();
                        metadata.insert("source_manifest".to_string(), "kubernetes".to_string());
                        if !description.is_empty() {
                            metadata
                                .insert("description".to_string(), description.trim().to_string());
                        }

                        let node = Node {
                            id: format!("k8s:{}", path_str.replace("/", "_")),
                            kind: NodeKind::System,
                            label,
                            technology: Some(technology),
                            path: Some(path_str),
                            metadata,
                            canonical_id: None,
                            aliases: Vec::new(),
                            owner: None,
                            domain: None,
                            criticality: None,
                            sources: Vec::new(),
                            confidence: None,
                            ..Default::default()
                        };
                        graph.nodes.push(node);
                    }
                }
            }
        }
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_discover_docker_parses_content() {
        let dir = tempdir().unwrap();
        let dockerfile = dir.path().join("Dockerfile");
        fs::write(
            &dockerfile,
            "FROM python:3.9-slim\nEXPOSE 8080\nCMD [\"python\", \"app.py\"]\n",
        )
        .unwrap();

        let graph = discover_docker(dir.path()).unwrap();
        assert_eq!(graph.nodes.len(), 1);
        let node = &graph.nodes[0];

        assert_eq!(node.kind, NodeKind::Container);
        assert_eq!(node.technology.as_deref(), Some("python:3.9-slim"));

        let desc = node.metadata.get("description").unwrap();
        assert!(desc.contains("Exposes: 8080"));
        assert!(desc.contains("Runs: [\"python\", \"app.py\"]"));
    }

    #[test]
    fn test_discover_k8s_deployment() {
        let dir = tempdir().unwrap();
        let k8s_dir = dir.path().join("k8s");
        fs::create_dir(&k8s_dir).unwrap();

        let deployment = k8s_dir.join("deploy.yaml");
        fs::write(
            &deployment,
            r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
spec:
  template:
    spec:
      containers:
      - name: app
        image: my-registry/my-app:1.0
        ports:
        - containerPort: 8080
"#,
        )
        .unwrap();

        let graph = discover_k8s(dir.path()).unwrap();
        assert_eq!(graph.nodes.len(), 1);
        let node = &graph.nodes[0];

        assert_eq!(node.kind, NodeKind::System);
        assert_eq!(node.label, "Deployment: my-app");
        assert_eq!(node.technology.as_deref(), Some("my-registry/my-app:1.0"));

        let desc = node.metadata.get("description").unwrap();
        assert!(desc.contains("Image: my-registry/my-app:1.0"));
        assert!(desc.contains("Port: 8080"));
    }
}
