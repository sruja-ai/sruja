use crate::{DiscoveredSource, Extractor};
use sruja_language::ast::{SourceBinding, SourceKind};
use std::path::Path;

pub struct DocExtractor;

impl Default for DocExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl DocExtractor {
    pub fn new() -> Self {
        Self
    }

    fn is_doc_file(path: &Path) -> bool {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        name.ends_with(".md") || name.ends_with(".adoc") || name.ends_with(".rst")
    }

    fn source_kind(path: &Path) -> SourceKind {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        if name.starts_with("readme") {
            SourceKind::Readme
        } else {
            SourceKind::Docs
        }
    }
}

impl Extractor for DocExtractor {
    fn name(&self) -> &'static str {
        "docs"
    }

    fn check_file(&self, path: &Path, repo_root: &Path) -> Vec<DiscoveredSource> {
        let mut results = Vec::new();

        if Self::is_doc_file(path) {
            let relative_path = path
                .strip_prefix(repo_root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            // Heuristic: use directory name for suggested element
            let suggested_element = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|s| s.to_string());

            results.push(DiscoveredSource {
                binding: SourceBinding {
                    kind: Self::source_kind(path),
                    path: relative_path,
                    description: Some("Discovered documentation".to_string()),
                },
                suggested_element,
                confidence: 0.6,
            });
        }

        results
    }
}
