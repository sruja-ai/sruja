//! Repo scanner that infers an architecture graph.
//!

#![warn(missing_docs)]
//! This crate extracts architecture information from source code using Tree-sitter.
//! Supports multiple programming languages and can also parse package manifests.

#[allow(missing_docs)]
mod assets;
#[allow(missing_docs)]
pub mod ast_cache;
#[allow(missing_docs)]
mod cargo;
#[allow(missing_docs)]
pub mod confidence;
#[allow(missing_docs)]
pub mod graph;
#[allow(missing_docs)]
pub mod manifest;
#[allow(missing_docs)]
mod manifests;
#[allow(missing_docs)]
pub mod npm;
#[allow(missing_docs)]
pub mod repomap;
#[allow(missing_docs)]
pub mod scan_scope;
#[allow(missing_docs)]
mod scip_ingest;
#[cfg(not(target_arch = "wasm32"))]
#[allow(missing_docs)]
pub mod tree_sitter;

#[cfg(target_arch = "wasm32")]
#[allow(missing_docs)]
pub mod tree_sitter_wasm;

#[cfg(target_arch = "wasm32")]
pub use tree_sitter_wasm as tree_sitter;

use std::path::{Path, PathBuf};

use thiserror::Error;

pub use graph::{
    detect_communities, summarize_communities, BlastRadiusDirection, BlastRadiusNode,
    BlastRadiusResult, CommunityInfo, Criticality, Edge, EdgeEvidence, EdgeKind, Graph, Incident,
    Node, NodeKind, ResolvedContract, ResolvedError, ResolvedField, ResolvedStateMachine,
    ResolvedTransition,
};
pub use repomap::{generate_repomap, generate_repomap_from_graph, RepoMapOptions};
pub use scan_scope::{
    is_path_production_relevant, should_exclude_with_config, ScanScope, DEFAULT_EXCLUDE_PATTERNS,
};
pub use tree_sitter::{detect_language, parse_file, scan_with_tree_sitter, ScanConfig};

/// Errors that can occur during repository scanning
#[derive(Debug, Error)]
pub enum ScanError {
    /// No supported source files found in the specified path
    #[error("no supported source files found in {path}")]
    UnsupportedRepo {
        /// The path of the unsupported repository
        path: String,
    },

    /// Cargo metadata command failed
    #[error("cargo metadata failed: {message}")]
    CargoMetadata {
        /// The error message returned from cargo metadata
        message: String,
    },

    /// JSON or configuration parsing error
    #[error("parse error: {0}")]
    Parse(#[from] serde_json::Error),

    /// Input/output error
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Directory traversal/walking error
    #[error("walk error: {0}")]
    Walk(String),
}

use std::cell::RefCell;

thread_local! {
    static CLASSIFICATION_RULES_PATH: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// Set the classification rules path for the current thread.
pub fn set_classification_rules_path(path: Option<PathBuf>) {
    CLASSIFICATION_RULES_PATH.with(|p: &RefCell<Option<PathBuf>>| *p.borrow_mut() = path);
}

/// Scan a repository and return an inferred architecture graph.
///
/// Uses Tree-sitter to parse source code files and extract:
/// - Modules/packages from file structure
/// - Dependencies from import statements
/// - Public interfaces from exports
#[tracing::instrument(skip(repo_root))]
pub fn scan_repo(repo_root: &Path) -> Result<Graph, ScanError> {
    tracing::info!("Starting full repository scan in {:?}", repo_root);
    let config = ScanConfig {
        classification_rules_path: CLASSIFICATION_RULES_PATH
            .with(|p: &RefCell<Option<PathBuf>>| p.borrow().clone()),
        ..Default::default()
    };

    let mut graph = scan_with_tree_sitter(repo_root, &config)?;

    // Merge manifest data (technology labels, monorepo packages, manifests)
    if let Ok(manifest_graph) = scan_repo_manifests(repo_root) {
        tracing::debug!("Merging manifest data into graph");
        graph.merge(manifest_graph);
    }

    // Merge docs and assets
    if let Ok(assets_graph) = assets::discover_docs_and_assets(repo_root) {
        tracing::debug!("Merging docs and assets into graph");
        graph.merge(assets_graph);
    }

    // Calculate discovery confidence
    confidence::ConfidenceScorer::score_graph(&mut graph);

    // 4. Enrich with SCIP if available
    if let Ok(scip_graph) = scip_ingest::enrich_with_scip(repo_root) {
        graph.merge(scip_graph);
    }

    Ok(graph)
}

/// Scan a repository incrementally and return an inferred architecture graph.
#[tracing::instrument(skip(repo_root))]
pub fn scan_repo_incremental(repo_root: &Path) -> Result<Graph, ScanError> {
    tracing::info!("Starting incremental repository scan in {:?}", repo_root);
    let config = ScanConfig {
        classification_rules_path: CLASSIFICATION_RULES_PATH
            .with(|p: &RefCell<Option<PathBuf>>| p.borrow().clone()),
        incremental: true,
        ..Default::default()
    };

    let mut graph = scan_with_tree_sitter(repo_root, &config)?;

    // Merge manifest data (technology labels, monorepo packages, manifests)
    if let Ok(manifest_graph) = scan_repo_manifests(repo_root) {
        tracing::debug!("Merging manifest data into graph");
        graph.merge(manifest_graph);
    }

    // Merge docs and assets
    if let Ok(assets_graph) = assets::discover_docs_and_assets(repo_root) {
        tracing::debug!("Merging docs and assets into graph");
        graph.merge(assets_graph);
    }

    // Calculate discovery confidence
    confidence::ConfidenceScorer::score_graph(&mut graph);

    // Enrich with SCIP if available
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
#[tracing::instrument(skip(repo_root))]
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
