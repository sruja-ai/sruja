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
use std::collections::HashSet;
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

/// Scan scope metadata for skill-facing JSON outputs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScanScope {
    /// Top-level directory names that contain at least one file that was scanned (included).
    pub included: Vec<String>,

    /// Relative paths of directories that were excluded: we did not descend into these,
    /// so no file under them was counted. Authoritative for "what was left out."
    pub excluded: Vec<String>,

    /// Total number of files scanned (included).
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
/// Walks the repo, applies default exclude patterns, and populates
/// `ScanScope::included`, `excluded`, and `total_files` for skill-facing JSON.
pub fn resolve_scan_scope(repo_root: &Path) -> (ScanConfig, ScanScope) {
    let config = ScanConfig::default();
    let scope = build_scope_from_walk(repo_root);
    (config, scope)
}

/// Walk repo and classify paths to build ScanScope metadata.
/// - included: top-level segment of any dir that contains at least one scanned file.
/// - excluded: full relative path of each directory we skipped (did not descend into).
fn build_scope_from_walk(repo_root: &Path) -> ScanScope {
    let mut included_top_level: HashSet<String> = HashSet::new();
    let mut excluded_rel_paths: HashSet<String> = HashSet::new();
    let mut total_files = 0usize;

    fn walk(
        dir: &Path,
        repo_root: &Path,
        included_top_level: &mut HashSet<String>,
        excluded_rel_paths: &mut HashSet<String>,
        total_files: &mut usize,
    ) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name.starts_with('.') {
                continue;
            }

            let rel: Option<String> = path
                .strip_prefix(repo_root)
                .ok()
                .map(|p| p.to_string_lossy().replace('\\', "/"));
            let first_seg = rel
                .as_ref()
                .and_then(|r| r.split('/').next().map(String::from));

            if path.is_dir() {
                if should_exclude(&path) {
                    if let Some(ref r) = rel {
                        excluded_rel_paths.insert(r.clone());
                    }
                    continue;
                }
                walk(
                    &path,
                    repo_root,
                    included_top_level,
                    excluded_rel_paths,
                    total_files,
                );
            } else if !should_exclude(&path) {
                *total_files += 1;
                if let Some(ref seg) = first_seg {
                    included_top_level.insert(seg.clone());
                }
            }
        }
    }

    walk(
        repo_root,
        repo_root,
        &mut included_top_level,
        &mut excluded_rel_paths,
        &mut total_files,
    );

    let mut included: Vec<String> = included_top_level.into_iter().collect();
    included.sort();
    let mut excluded: Vec<String> = excluded_rel_paths.into_iter().collect();
    excluded.sort();

    ScanScope {
        included,
        excluded,
        total_files,
        exclude_patterns: DEFAULT_EXCLUDE_PATTERNS
            .iter()
            .map(|s| s.to_string())
            .collect(),
    }
}

/// Check if a path should be excluded based on default patterns.
///
/// Returns true if the path matches any exclude pattern.
/// Handles: exact names, directory names, suffix wildcards (*.min.js),
/// and infix+suffix wildcards (*test*.rs = filename contains "test" and ends with ".rs").
pub fn should_exclude(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    for pattern in DEFAULT_EXCLUDE_PATTERNS {
        if pattern.starts_with('*') {
            let rest = pattern.trim_start_matches('*');
            if let Some((mid, suffix)) = rest.split_once('*') {
                // *mid*suffix e.g. *test*.rs => filename contains "test" and ends with ".rs"
                if !suffix.is_empty() && file_name.ends_with(suffix) && file_name.contains(mid) {
                    return true;
                }
            } else if file_name.ends_with(rest) {
                // *.suffix e.g. *.min.js
                return true;
            }
        } else if pattern.ends_with('/') {
            // Directory pattern (e.g., "tests/")
            if path_str.contains(pattern.trim_end_matches('/')) {
                return true;
            }
        } else if pattern.ends_with('*') && !pattern.starts_with('*') {
            // Prefix wildcard (e.g. ".env*")
            let prefix = pattern.trim_end_matches('*');
            if file_name.starts_with(prefix) {
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
