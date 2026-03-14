//! Shared scan-scope resolver for Sruja CLI commands.
//!
//! This module provides a centralized way to determine what files should be
//! included or excluded when scanning a repository. It is used by:
//! - discover
//! - quickstart
//! - drift
//! - intent flows
//! - context detection

use crate::ScanConfig;
use std::path::Path;

/// Default exclude patterns for production-relevant code scanning.
///
/// These patterns exclude generated, vendored, fixture, docs, and evaluation-heavy
/// content to ensure scan results reflect actual production code.
pub const DEFAULT_EXCLUDE_PATTERNS: &[&str] = &[
    // Generated code
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    "out",
    "*.min.js",
    "*.min.css",
    // Vendor directories
    "vendor",
    "third_party",
    // Fixtures and test data
    "fixtures",
    "__mocks__",
    "__fixtures__",
    "test_data",
    // Documentation
    "docs",
    "documentation",
    "*.md",
    "*.rst",
    // Evaluation and benchmarks
    "evaluation",
    "benchmark",
    "bench",
    "perf",
    // Other non-production
    ".git",
    ".gitignore",
    ".env*",
    "*.log",
    // Test files
    "*test*.rs",
    "*test*.js",
    "*test*.ts",
    "*test*.py",
    "*test*.go",
    "*spec*.js",
    "*spec*.ts",
    "tests/",
    "__tests__/",
    "spec/",
];

/// Scan scope metadata that should be included in JSON outputs for skills.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScanScope {
    /// Directories/files that were included in the scan
    pub included: Vec<String>,

    /// Directories/files that were excluded from the scan
    pub excluded: Vec<String>,

    /// Total number of files scanned
    pub total_files: usize,

    /// Patterns used for exclusion
    pub exclude_patterns: Vec<String>,
}

impl Default for ScanScope {
    fn default() -> Self {
        Self {
            included: Vec::new(),
            excluded: Vec::new(),
            total_files: 0,
            exclude_patterns: DEFAULT_EXCLUDE_PATTERNS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

/// Resolve scan scope for a repository.
///
/// Returns a ScanConfig configured with default excludes and ScanScope metadata
/// that can be included in JSON outputs for AI skills.
pub fn resolve_scan_scope(repo_root: &Path) -> (ScanConfig, ScanScope) {
    let config = ScanConfig::default();
    let scope = ScanScope::default();

    // The actual scanning happens in the sruja-scan crate
    // This function returns the config and metadata
    (config, scope)
}

/// Check if a path should be excluded based on default patterns.
///
/// Returns true if the path matches any exclude pattern.
pub fn should_exclude(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    for pattern in DEFAULT_EXCLUDE_PATTERNS {
        if pattern.starts_with('*') {
            // Wildcard pattern (e.g., "*.min.js")
            let pattern = pattern.trim_start_matches('*');
            if file_name.ends_with(pattern) {
                return true;
            }
        } else if pattern.ends_with('/') {
            // Directory pattern (e.g., "tests/")
            if path_str.contains(pattern.trim_end_matches('/')) {
                return true;
            }
        } else if file_name == *pattern
            || path_str.contains(&format!("/{}", pattern))
            || path_str == *pattern
        {
            // Exact match or directory match
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_exclude_node_modules() {
        assert!(should_exclude(Path::new("/project/node_modules")));
    }

    #[test]
    fn test_should_exclude_target() {
        assert!(should_exclude(Path::new("/project/target")));
    }

    #[test]
    fn test_should_exclude_docs() {
        assert!(should_exclude(Path::new("/project/docs")));
    }

    #[test]
    fn test_should_exclude_fixtures() {
        assert!(should_exclude(Path::new("/project/fixtures")));
    }

    #[test]
    fn test_should_exclude_min_js() {
        assert!(should_exclude(Path::new("/project/app.min.js")));
    }

    #[test]
    fn test_should_not_exclude_source() {
        assert!(!should_exclude(Path::new("/project/src/main.rs")));
    }

    #[test]
    fn test_should_exclude_test_files() {
        assert!(should_exclude(Path::new("/project/src/test.rs")));
        assert!(should_exclude(Path::new("/project/tests/module_test.rs")));
    }

    #[test]
    fn test_scan_scope_default() {
        let scope = ScanScope::default();
        assert!(!scope.exclude_patterns.is_empty());
        assert!(scope.exclude_patterns.contains(&"node_modules".to_string()));
    }
}
