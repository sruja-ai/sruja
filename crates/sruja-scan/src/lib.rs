//! Repo scanner that infers an architecture graph.
//!
//! This crate extracts architecture information from source code using Tree-sitter.
//! Supports multiple programming languages and can also parse package manifests.

mod cargo;
pub mod graph;
mod npm;
pub mod tree_sitter;

use std::path::Path;

use thiserror::Error;

pub use graph::{Edge, EdgeEvidence, EdgeKind, Graph, Node, NodeKind};
pub use tree_sitter::{scan_with_tree_sitter, ScanConfig};

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
    scan_with_tree_sitter(repo_root, &config)
}

/// Scan a repository with custom configuration.
pub fn scan_repo_with_config(repo_root: &Path, config: &ScanConfig) -> Result<Graph, ScanError> {
    scan_with_tree_sitter(repo_root, config)
}

/// Scan a repository using only package manifests (package.json, Cargo.toml).
/// This is a fallback for cases where source code parsing is not needed.
pub fn scan_repo_manifests(repo_root: &Path) -> Result<Graph, ScanError> {
    if repo_root.join("package.json").exists() {
        if let Ok(g) = npm::scan_npm_repo(repo_root) {
            return Ok(g);
        }
    }
    if repo_root.join("Cargo.toml").exists() {
        if let Ok(g) = cargo::scan_cargo_repo(repo_root) {
            return Ok(g);
        }
    }

    Err(ScanError::UnsupportedRepo {
        path: repo_root.display().to_string(),
    })
}
