use std::path::Path;
use walkdir::WalkDir;
use sruja_language::ast::{SourceBinding, SourceKind};
use crate::{DiscoveredSource, ExtractError, Extractor};

pub struct KubernetesExtractor;

impl KubernetesExtractor {
    pub fn new() -> Self {
        Self
    }

    fn is_k8s_file(path: &Path) -> bool {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        if !name.ends_with(".yaml") && !name.ends_with(".yml") {
            return false;
        }
        
        // check content for "apiVersion:" (minimal check)
        if let Ok(content) = std::fs::read_to_string(path) {
            content.contains("apiVersion:") && (content.contains("kind: Deployment") || content.contains("kind: Service") || content.contains("kind: StatefulSet"))
        } else {
            false
        }
    }
}

impl Extractor for KubernetesExtractor {
    fn name(&self) -> &'static str {
        "kubernetes"
    }

    fn check_file(&self, path: &Path, repo_root: &Path) -> Vec<DiscoveredSource> {
        let mut results = Vec::new();

        if Self::is_k8s_file(path) {
            let relative_path = path.strip_prefix(repo_root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
            
            // Heuristic: try to find 'name:' in metadata
            let mut suggested_element = None;
            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines() {
                    if line.trim().starts_with("name:") {
                        suggested_element = Some(line.trim()["name:".len()..].trim().trim_matches('"').to_string());
                        break;
                    }
                }
            }

            results.push(DiscoveredSource {
                binding: SourceBinding {
                    kind: SourceKind::Kubernetes,
                    path: relative_path,
                    description: Some("Discovered Kubernetes manifest".to_string()),
                },
                suggested_element,
                confidence: 0.7,
            });
        }

        results
    }
}
