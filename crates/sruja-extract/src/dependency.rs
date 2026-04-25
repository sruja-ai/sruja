use crate::{DiscoveredSource, Extractor};
use sruja_language::ast::{SourceBinding, SourceKind};
use std::path::Path;

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
        matches!(
            ext,
            "rs" | "go" | "ts" | "js" | "py" | "java" | "yaml" | "yml"
        )
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
                // Heuristic: look for uppercase config variables like PAYMENTS_SERVICE_URL
                // Exclude common noise that doesn't represent external dependencies
                let noise_prefixes = [
                    "BASE", "APP", "LOCAL", "CURRENT", "WEB", "SERVER", "MY", "THIS", "API",
                    "PROXY", "TARGET",
                ];

                for line in content.lines() {
                    // Quick check before allocating/parsing
                    if !line.contains("_URL")
                        && !line.contains("_SERVICE")
                        && !line.contains("_HOST")
                        && !line.contains("_API")
                    {
                        continue;
                    }

                    // Extract uppercase tokens that look like config variables
                    let tokens: Vec<&str> = line
                        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                        .filter(|t| !t.is_empty())
                        .collect();

                    for token in tokens {
                        // Must be fully uppercase with underscores to be a reliable config signal
                        if !token
                            .chars()
                            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
                        {
                            continue;
                        }

                        if token.ends_with("_URL")
                            || token.ends_with("_SERVICE")
                            || token.ends_with("_HOST")
                            || token.ends_with("_API")
                        {
                            let service_name_upper = token
                                .replace("_URL", "")
                                .replace("_SERVICE", "")
                                .replace("_HOST", "")
                                .replace("_API", "");

                            // Filter out generic noise (e.g., BASE_URL, APP_HOST)
                            if noise_prefixes.contains(&service_name_upper.as_str()) {
                                continue;
                            }

                            let service_name = service_name_upper.to_lowercase().replace('_', "");

                            if service_name.len() > 2 {
                                let relative_path = path
                                    .strip_prefix(repo_root)
                                    .unwrap_or(path)
                                    .to_string_lossy()
                                    .to_string();

                                results.push(DiscoveredSource {
                                    binding: SourceBinding {
                                        kind: SourceKind::Custom("dependency_signal".to_string()),
                                        path: relative_path,
                                        description: Some(format!(
                                            "Signal for dependency on: {}",
                                            service_name
                                        )),
                                    },
                                    suggested_element: Some(service_name),
                                    // Lower confidence as this is still a heuristic
                                    confidence: 0.3,
                                });
                            }
                        }
                    }
                }
            }
        }

        results
    }
}
