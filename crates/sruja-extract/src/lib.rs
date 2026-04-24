//! Sruja Extraction Framework
//!
//! Handles automatic discovery of architectural artifacts and inference
//! of relationships from codebase signals.

pub mod openapi;
pub mod kubernetes;
pub mod docs;
pub mod alias;
pub mod dependency;

use std::path::Path;
use sruja_language::ast::SourceBinding;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Discovery error: {0}")]
    Discovery(String),
}

/// A discovered source binding with context about which element it might belong to.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscoveredSource {
    pub binding: SourceBinding,
    /// Suggested element FQN or ID this source belongs to (if inferable)
    pub suggested_element: Option<String>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
}

/// Trait for automatic discovery of architectural artifacts.
pub trait Extractor: Send + Sync {
    /// Name of the extractor
    fn name(&self) -> &'static str;
    
    /// Check a specific file for architectural artifacts.
    fn check_file(&self, path: &Path, repo_root: &Path) -> Vec<DiscoveredSource>;

    /// Optional: Finalize discovery after the walk (e.g., for cross-file inference)
    fn finalize(&self) -> Vec<DiscoveredSource> {
        Vec::new()
    }
}

/// Orchestrates multiple extractors to build a complete architectural picture.
pub struct ExtractionEngine {
    extractors: Vec<Box<dyn Extractor>>,
}

impl ExtractionEngine {
    pub fn new() -> Self {
        Self {
            extractors: vec![
                Box::new(openapi::OpenApiExtractor::new()),
                Box::new(kubernetes::KubernetesExtractor::new()),
                Box::new(docs::DocExtractor::new()),
                Box::new(alias::AliasExtractor::new()),
                Box::new(dependency::DependencyExtractor::new()),
            ],
        }
    }

    pub fn discover_all(&self, repo_root: &Path) -> Vec<DiscoveredSource> {
        use walkdir::WalkDir;
        let mut all = Vec::new();

        for entry in WalkDir::new(repo_root)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_str().unwrap_or("");
                // Basic ignore list
                !(name.starts_with('.') || name == "node_modules" || name == "target")
            })
            .filter_map(|e| e.ok()) 
        {
            let path = entry.path();
            if path.is_file() {
                for extractor in &self.extractors {
                    let results = extractor.check_file(path, repo_root);
                    if !results.is_empty() {
                        all.extend(results);
                    }
                }
            }
        }

        // Run finalization for all extractors
        for extractor in &self.extractors {
            all.extend(extractor.finalize());
        }

        all
    }
}

impl Default for ExtractionEngine {
    fn default() -> Self {
        Self::new()
    }
}
