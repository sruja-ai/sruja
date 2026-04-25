use crate::{DiscoveredSource, Extractor};
use sruja_language::ast::{SourceBinding, SourceKind};
use std::path::Path;

pub struct OpenApiExtractor;

impl Default for OpenApiExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenApiExtractor {
    pub fn new() -> Self {
        Self
    }

    fn is_openapi_file(path: &Path) -> bool {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !name.ends_with(".yaml") && !name.ends_with(".yml") && !name.ends_with(".json") {
            return false;
        }

        // common names
        if name.contains("openapi") || name.contains("swagger") || name.contains("api-spec") {
            return true;
        }

        // check content for "openapi:" or "\"openapi\":" (minimal check)
        if let Ok(content) = std::fs::read_to_string(path) {
            content.contains("openapi:")
                || content.contains("\"openapi\":")
                || content.contains("swagger:")
                || content.contains("\"swagger\":")
        } else {
            false
        }
    }
}

impl Extractor for OpenApiExtractor {
    fn name(&self) -> &'static str {
        "openapi"
    }

    fn check_file(&self, path: &Path, repo_root: &Path) -> Vec<DiscoveredSource> {
        let mut results = Vec::new();

        if Self::is_openapi_file(path) {
            let relative_path = path
                .strip_prefix(repo_root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            // Heuristic for suggested element: use parent directory name
            let suggested_element = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|s| s.to_string());

            results.push(DiscoveredSource {
                binding: SourceBinding {
                    kind: SourceKind::OpenApi,
                    path: relative_path,
                    description: Some("Discovered OpenAPI specification".to_string()),
                },
                suggested_element,
                confidence: 0.8,
            });
        }

        results
    }
}
