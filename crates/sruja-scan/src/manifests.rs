use crate::graph::{AutoContext, Graph, Node, NodeKind};
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
    let repo_canon = repo_root
        .canonicalize()
        .unwrap_or_else(|_| repo_root.to_path_buf());

    if let Ok(entries) = std::fs::read_dir(repo_root) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                let path = entry.path();
                if !crate::is_safe_path(&path, &repo_canon) {
                    continue;
                }
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                if file_name.starts_with("Dockerfile") || file_name.ends_with(".Dockerfile") {
                    if let Some(node) = process_dockerfile(&path, &file_name) {
                        graph.nodes.push(node);
                    }
                }
            }
        }
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
            if !crate::is_safe_path(&path, &repo_canon) {
                continue;
            }
            let node = Node {
                id: format!("docker:{}", name),
                kind: NodeKind::new(NodeKind::SYSTEM),
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

fn process_dockerfile(path: &Path, file_name: &str) -> Option<Node> {
    let mut technology = "Docker".to_string();
    let mut description = String::new();

    if let Ok(content) = std::fs::read_to_string(path) {
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

    Some(Node {
        id: format!("docker:{}", file_name),
        kind: NodeKind::new(NodeKind::CONTAINER),
        label: "Docker Image".to_string(),
        technology: Some(technology),
        path: Some(file_name.to_string()),
        metadata,
        canonical_id: None,
        aliases: Vec::new(),
        owner: None,
        domain: None,
        criticality: None,
        sources: Vec::new(),
        confidence: None,
        ..Default::default()
    })
}

fn discover_openapi(repo_root: &Path) -> Result<Graph, ScanError> {
    let mut graph = Graph::new();
    let repo_canon = repo_root
        .canonicalize()
        .unwrap_or_else(|_| repo_root.to_path_buf());
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
            if !crate::is_safe_path(&path, &repo_canon) {
                continue;
            }
            let id = format!("api:{}", name.replace("/", "_"));
            let node = Node {
                id,
                kind: NodeKind::new(NodeKind::EXTERNAL_API),
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
    let repo_canon = repo_root
        .canonicalize()
        .unwrap_or_else(|_| repo_root.to_path_buf());
    // Simple heuristic for K8s discovery in likely directories
    let k8s_dirs = [
        "k8s",
        "kubernetes",
        "deploy",
        "deployment",
        "manifests",
        "helm",
        "chart",
        "charts",
    ];

    for dir_name in &k8s_dirs {
        let dir_path = repo_root.join(dir_name);
        if !dir_path.is_dir() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !crate::is_safe_path(&path, &repo_canon) {
                    continue;
                }
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
                                    "Deployment" | "StatefulSet" | "DaemonSet" | "Job"
                                    | "CronJob" | "Pod" => {
                                        // Try to find containers
                                        if let Some(containers) = extract_containers(&spec) {
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
                            kind: NodeKind::new(NodeKind::SYSTEM),
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

/// Discover auto-context from repository files (docker-compose, CI, terraform, README).
pub fn discover_auto_context(repo_root: &Path) -> AutoContext {
    let mut ctx = AutoContext::default();

    // docker-compose*.yml
    for name in &[
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
    ] {
        let path = repo_root.join(name);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                ctx.services_from_compose
                    .extend(extract_compose_services(&content));
            }
        }
    }

    // .github/workflows/*.yml
    let workflows_dir = repo_root.join(".github/workflows");
    if workflows_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&workflows_dir) {
            for entry in entries.flatten() {
                if let Ok(entry_type) = entry.file_type() {
                    if entry_type.is_file() {
                        let path = entry.path();
                        if path
                            .extension()
                            .map(|e| e == "yml" || e == "yaml")
                            .unwrap_or(false)
                        {
                            ctx.ci_pipelines
                                .push(entry.file_name().to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    // terraform/*.tf or infra/
    for dir in &["terraform", "infra", "infrastructure"] {
        let tf_dir = repo_root.join(dir);
        if tf_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&tf_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "tf").unwrap_or(false) {
                        ctx.infra_dependencies.push(path.display().to_string());
                    }
                }
            }
        }
    }

    // README.md - extract architecture section
    let readme = repo_root.join("README.md");
    if readme.exists() {
        if let Ok(content) = std::fs::read_to_string(&readme) {
            ctx.readme_summary = extract_architecture_section(&content);
        }
    }

    // .env.example - extract URLs
    let env_example = repo_root.join(".env.example");
    if env_example.exists() {
        if let Ok(content) = std::fs::read_to_string(&env_example) {
            ctx.referenced_urls = extract_urls_from_env(&content);
        }
    }

    ctx
}

fn extract_compose_services(content: &str) -> Vec<String> {
    let mut services = Vec::new();
    let mut in_services = false;
    let mut indent_level = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "services:" {
            in_services = true;
            indent_level = line.len() - line.trim_start().len();
            continue;
        }

        if in_services {
            let current_indent = line.len() - line.trim_start().len();
            // If we're back to the same or lesser indent, we've left the services block
            if current_indent <= indent_level && !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }

            // Service names are at indent_level + 2 and end with ':'
            if current_indent == indent_level + 2
                && trimmed.ends_with(':')
                && !trimmed.starts_with('#')
            {
                let service_name = trimmed.trim_end_matches(':').to_string();
                if !service_name.is_empty() {
                    services.push(service_name);
                }
            }
        }
    }

    services
}

fn extract_architecture_section(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_section = false;
    let mut section_lines = Vec::new();

    for line in &lines {
        let lower = line.to_lowercase();
        if lower.starts_with("## architecture") || lower.starts_with("## overview") {
            in_section = true;
            continue;
        }
        if in_section && line.starts_with("## ") {
            break;
        }
        if in_section {
            section_lines.push(*line);
        }
    }

    if section_lines.is_empty() {
        None
    } else {
        let section = section_lines.join("\n").trim().to_string();
        if section.is_empty() {
            None
        } else {
            Some(section)
        }
    }
}

fn extract_urls_from_env(content: &str) -> Vec<String> {
    let mut urls = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        if let Some(value) = line.split('=').nth(1) {
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if value.starts_with("http://") || value.starts_with("https://") {
                urls.push(value.to_string());
            }
        }
    }
    urls
}

fn extract_containers(spec: &serde_yaml::Value) -> Option<&Vec<serde_yaml::Value>> {
    // Try Pod
    if let Some(containers) = spec.get("containers").and_then(|c| c.as_sequence()) {
        return Some(containers);
    }
    // Try Deployment, etc
    if let Some(containers) = spec
        .get("template")
        .and_then(|t| t.get("spec"))
        .and_then(|s| s.get("containers"))
        .and_then(|c| c.as_sequence())
    {
        return Some(containers);
    }
    // Try CronJob
    if let Some(containers) = spec
        .get("jobTemplate")
        .and_then(|jt| jt.get("spec"))
        .and_then(|t| t.get("template"))
        .and_then(|t| t.get("spec"))
        .and_then(|s| s.get("containers"))
        .and_then(|c| c.as_sequence())
    {
        return Some(containers);
    }
    None
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

        assert_eq!(node.kind, NodeKind::CONTAINER);
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

        assert_eq!(node.kind, NodeKind::SYSTEM);
        assert_eq!(node.label, "Deployment: my-app");
        assert_eq!(node.technology.as_deref(), Some("my-registry/my-app:1.0"));

        let desc = node.metadata.get("description").unwrap();
        assert!(desc.contains("Image: my-registry/my-app:1.0"));
        assert!(desc.contains("Port: 8080"));
    }

    #[test]
    fn test_discover_openapi_finds_spec_at_repo_root() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("openapi.yaml"),
            "openapi: 3.0.0\ninfo:\n  title: Demo\n",
        )
        .unwrap();

        let graph = discover_openapi(dir.path()).unwrap();
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].kind, NodeKind::EXTERNAL_API);
        assert_eq!(graph.nodes[0].technology.as_deref(), Some("OpenAPI"));
    }

    #[test]
    fn test_discover_docker_finds_compose_service() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("docker-compose.yml"),
            "services:\n  api:\n    image: nginx:latest\n    ports:\n      - '8080:80'\n",
        )
        .unwrap();

        let graph = discover_docker(dir.path()).unwrap();
        assert!(
            graph.nodes.iter().any(|n| n.label.contains("Compose")),
            "expected compose node: {:?}",
            graph.nodes
        );
    }

    #[test]
    fn test_scan_other_manifests_merges_docker_openapi_and_k8s() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Dockerfile"), "FROM node:20\n").unwrap();
        fs::write(
            dir.path().join("openapi.json"),
            r#"{"openapi":"3.0.0","info":{"title":"API"}}"#,
        )
        .unwrap();
        let k8s = dir.path().join("k8s");
        fs::create_dir(&k8s).unwrap();
        fs::write(
            k8s.join("svc.yaml"),
            "apiVersion: v1\nkind: Service\nmetadata:\n  name: api\n",
        )
        .unwrap();

        let graph = scan_other_manifests(dir.path()).unwrap();
        assert!(graph.nodes.len() >= 3, "expected merged manifest nodes");
    }

    #[test]
    fn test_discover_auto_context_empty() {
        let dir = tempdir().unwrap();
        let ctx = discover_auto_context(dir.path());
        assert!(ctx.is_empty());
    }

    #[test]
    fn test_discover_auto_context_compose() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("docker-compose.yml"),
            "services:\n  api:\n    image: nginx\n  db:\n    image: postgres\n",
        )
        .unwrap();

        let ctx = discover_auto_context(dir.path());
        assert_eq!(ctx.services_from_compose.len(), 2);
        assert!(ctx.services_from_compose.contains(&"api".to_string()));
        assert!(ctx.services_from_compose.contains(&"db".to_string()));
    }

    #[test]
    fn test_discover_auto_context_ci_pipelines() {
        let dir = tempdir().unwrap();
        let workflows = dir.path().join(".github/workflows");
        fs::create_dir_all(&workflows).unwrap();
        fs::write(workflows.join("ci.yml"), "name: CI\n").unwrap();
        fs::write(workflows.join("deploy.yaml"), "name: Deploy\n").unwrap();

        let ctx = discover_auto_context(dir.path());
        assert_eq!(ctx.ci_pipelines.len(), 2);
    }

    #[test]
    fn test_discover_auto_context_terraform() {
        let dir = tempdir().unwrap();
        let tf_dir = dir.path().join("terraform");
        fs::create_dir(&tf_dir).unwrap();
        fs::write(
            tf_dir.join("main.tf"),
            "resource \"aws_instance\" \"web\" {}\n",
        )
        .unwrap();

        let ctx = discover_auto_context(dir.path());
        assert_eq!(ctx.infra_dependencies.len(), 1);
    }

    #[test]
    fn test_discover_auto_context_readme() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("README.md"),
            "# My Project\n\n## Architecture\n\nThis is a microservices app.\n\n## Usage\n\nRun it.",
        )
        .unwrap();

        let ctx = discover_auto_context(dir.path());
        assert!(ctx.readme_summary.is_some());
        assert!(ctx.readme_summary.unwrap().contains("microservices"));
    }

    #[test]
    fn test_discover_auto_context_env_urls() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join(".env.example"),
            "API_URL=https://api.example.com\nDB_HOST=localhost\nOTHER=https://other.com\n",
        )
        .unwrap();

        let ctx = discover_auto_context(dir.path());
        assert_eq!(ctx.referenced_urls.len(), 2);
        assert!(ctx
            .referenced_urls
            .contains(&"https://api.example.com".to_string()));
    }

    #[test]
    fn test_extract_compose_services() {
        let content = r#"services:
  api:
    image: nginx
    ports:
      - '8080:80'
  db:
    image: postgres
  redis:
    image: redis"#;

        let services = extract_compose_services(content);
        assert_eq!(services.len(), 3);
        assert!(services.contains(&"api".to_string()));
        assert!(services.contains(&"db".to_string()));
        assert!(services.contains(&"redis".to_string()));
    }

    #[test]
    fn test_extract_architecture_section() {
        let content =
            "# Title\n\n## Architecture\n\nSome arch text.\n\nMore details.\n\n## Usage\n\nRun it.";
        let section = extract_architecture_section(content);
        assert!(section.is_some());
        let section = section.unwrap();
        assert!(section.contains("Some arch text"));
        assert!(section.contains("More details"));
        assert!(!section.contains("## Usage"));
    }

    #[test]
    fn test_extract_urls_from_env() {
        let content =
            "# Comment\nAPI_URL=https://api.example.com\nDB=localhost\nOTHER='https://other.com'\n";
        let urls = extract_urls_from_env(content);
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://api.example.com".to_string()));
        assert!(urls.contains(&"https://other.com".to_string()));
    }
}
