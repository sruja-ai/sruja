use std::path::Path;
use sruja_language::ast::{SourceBinding, SourceKind};
use crate::{DiscoveredSource, Extractor};

pub struct DependencyExtractor;

impl Default for DependencyExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyExtractor {
    pub fn new() -> Self {
        Self
    }

    fn is_source_code(path: &Path) -> bool {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        matches!(ext, "rs" | "go" | "ts" | "js" | "py" | "java" | "yaml" | "yml")
    }
}

impl Extractor for DependencyExtractor {
    fn name(&self) -> &'static str {
        "dependency"
    }

    fn check_file(&self, path: &Path, repo_root: &Path) -> Vec<DiscoveredSource> {
        let mut results = Vec::new();

        if Self::is_source_code(path) {
            if let Ok(content) = std::fs::read_to_string(path) {
                // Heuristic: look for SERVICE_URL, API_URL, etc.
                for line in content.lines() {
                    let line_upper = line.to_uppercase();
                    if line_upper.contains("_URL") || line_upper.contains("_SERVICE") || line_upper.contains("_HOST") {
                        // Extract potential service name from the variable
                        // e.g., PAYMENTS_SERVICE_URL -> PAYMENTS
                        if let Some(start) = line.find(|c: char| c.is_alphabetic()) {
                            let part = &line[start..];
                            if let Some(end) = part.find(|c: char| !c.is_alphanumeric() && c != '_') {
                                let var_name = &part[..end];
                                if var_name.contains("_URL") || var_name.contains("_SERVICE") || var_name.contains("_HOST") {
                                    let service_name = var_name
                                        .replace("_URL", "")
                                        .replace("_SERVICE", "")
                                        .replace("_HOST", "")
                                        .replace("_", "")
                                        .to_lowercase();

                                    if service_name.len() > 2 {
                                        let relative_path = path.strip_prefix(repo_root)
                                            .unwrap_or(path)
                                            .to_string_lossy()
                                            .to_string();

                                        results.push(DiscoveredSource {
                                            binding: SourceBinding {
                                                kind: SourceKind::Custom("dependency_signal".to_string()),
                                                path: relative_path,
                                                description: Some(format!("Signal for dependency on: {}", service_name)),
                                            },
                                            suggested_element: Some(service_name),
                                            confidence: 0.5,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        results
    }
}
