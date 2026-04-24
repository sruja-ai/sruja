use std::path::Path;
use sruja_language::ast::{SourceBinding, SourceKind};
use crate::{DiscoveredSource, Extractor};

pub struct AliasExtractor;

impl Default for AliasExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl AliasExtractor {
    pub fn new() -> Self {
        Self
    }

    fn is_docker_compose(path: &Path) -> bool {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        name == "docker-compose.yaml" || name == "docker-compose.yml" || name == "compose.yaml" || name == "compose.yml"
    }

    #[allow(dead_code)]
    fn is_helm_values(path: &Path) -> bool {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        name == "values.yaml" || name == "values.yml"
    }
}

impl Extractor for AliasExtractor {
    fn name(&self) -> &'static str {
        "alias"
    }

    fn check_file(&self, path: &Path, repo_root: &Path) -> Vec<DiscoveredSource> {
        let mut results = Vec::new();

        if Self::is_docker_compose(path) {
            if let Ok(content) = std::fs::read_to_string(path) {
                // Very simple heuristic to find service names in docker-compose
                // services:
                //   my-service:
                let mut in_services = false;
                for line in content.lines() {
                    let trimmed = line.trim_start();
                    let indent = line.len() - trimmed.len();
                    
                    if trimmed.starts_with("services:") {
                        in_services = true;
                        continue;
                    }

                    if in_services && indent == 2 && trimmed.ends_with(':') {
                        let service_name = trimmed.trim_matches(':').trim().to_string();
                        let relative_path = path.strip_prefix(repo_root)
                            .unwrap_or(path)
                            .to_string_lossy()
                            .to_string();

                        results.push(DiscoveredSource {
                            binding: SourceBinding {
                                kind: SourceKind::Custom("docker-compose".to_string()),
                                path: relative_path.clone(),
                                description: Some(format!("Discovered service alias: {}", service_name)),
                            },
                            suggested_element: Some(service_name),
                            confidence: 0.9,
                        });
                    } else if in_services && indent == 0 && !trimmed.is_empty() {
                        in_services = false;
                    }
                }
            }
        }

        results
    }
}
