//! MCP Error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP server error: {0}")]
    Http(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Path traversal detected: {path}")]
    PathTraversal { path: String },

    #[error("Scan error: {0}")]
    Scan(#[from] sruja_scan::ScanError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
