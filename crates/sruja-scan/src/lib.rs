//! Repo scanner that infers an architecture graph.
//!
//! This crate extracts architecture information from source code using Tree-sitter.
//! Supports multiple programming languages and can also parse package manifests.

mod cargo;
pub mod confidence;
pub mod graph;
pub mod npm;
pub mod repomap;
pub mod scan_scope;
mod manifests;
mod scip_ingest;
pub mod tree_sitter;

use std::path::Path;

use thiserror::Error;

pub use graph::{
    BlastRadiusDirection, BlastRadiusNode, BlastRadiusResult, Edge, EdgeEvidence, EdgeKind, Graph,
    Node, NodeKind,
};
pub use repomap::{generate_repomap, generate_repomap_from_graph, RepoMapOptions};
pub use scan_scope::{
    is_path_production_relevant, should_exclude_with_config, ScanScope, DEFAULT_EXCLUDE_PATTERNS,
};
pub use tree_sitter::{detect_language, parse_file, scan_with_tree_sitter, ScanConfig};

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("no supported source files found in {path}")]
    UnsupportedRepo { path: String },

    #[error("cargo metadata failed: {message}")]
    CargoMetadata { message: String },

    #[error("parse error: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("walk error: {0}")]
    Walk(String),
}

/// Scan a repository and return an inferred architecture graph.
///
/// Uses Tree-sitter to parse source code files and extract:
/// - Modules/packages from file structure
/// - Dependencies from import statements
/// - Public interfaces from exports
pub fn scan_repo(repo_root: &Path) -> Result<Graph, ScanError> {
    let config = ScanConfig::default();
    let mut graph = scan_with_tree_sitter(repo_root, &config)?;

    // Merge manifest data (technology labels, monorepo packages, manifests)
    if let Ok(manifest_graph) = scan_repo_manifests(repo_root) {
        graph.merge(manifest_graph);
    }

    // Calculate discovery confidence
    confidence::ConfidenceScorer::score_graph(&mut graph);

    // 4. Enrich with SCIP if available
    if let Ok(scip_graph) = scip_ingest::enrich_with_scip(repo_root) {
        graph.merge(scip_graph);
    }

    Ok(graph)
}

/// Scan a repository with custom configuration.
pub fn scan_repo_with_config(repo_root: &Path, config: &ScanConfig) -> Result<Graph, ScanError> {
    scan_with_tree_sitter(repo_root, config)
}

/// Scan a repository using all available package and deployment manifests.
/// This merges results from package.json, Cargo.toml, OpenAPI, Docker, and Kubernetes.
pub fn scan_repo_manifests(repo_root: &Path) -> Result<Graph, ScanError> {
    let mut final_graph = Graph::new();
    let mut found = false;

    if repo_root.join("package.json").exists() {
        if let Ok(g) = npm::scan_npm_repo(repo_root) {
            final_graph.merge(g);
            found = true;
        }
    }

    if repo_root.join("Cargo.toml").exists() {
        if let Ok(g) = cargo::scan_cargo_repo(repo_root) {
            final_graph.merge(g);
            found = true;
        }
    }

    // New manifests (OpenAPI, Docker, K8s)
    if let Ok(g) = manifests::scan_other_manifests(repo_root) {
        final_graph.merge(g);
        found = true;
    }

    if found {
        Ok(final_graph)
    } else {
        Err(ScanError::UnsupportedRepo {
            path: repo_root.display().to_string(),
        })
    }
}
