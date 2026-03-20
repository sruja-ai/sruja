//! Shared scan-scope resolver for Sruja CLI commands.
//!
//! This module provides a centralized way to determine what files should be
//! included or excluded when scanning a repository. It is used by:
//! - discover
//! - quickstart
//! - drift
//! - intent flows
//! - context detection
//! - tree-sitter walker

use std::collections::HashSet;
use std::path::Path;

use crate::tree_sitter::ScanConfig;

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

/// Check if a path should be excluded based on ScanConfig options.
///
/// This is the config-aware version of `should_exclude` that respects:
/// - `include_tests`: if false, excludes test files and directories
/// - `include_node_modules`: if false, excludes node_modules
/// - `exclude_examples`: if true, excludes example files/directories
/// - `exclude_benches`: if true, excludes benchmark files/directories
/// - `exclude_fixtures`: if true, excludes fixture files/directories
/// - `exclude_docs`: if true, excludes documentation files/directories
///
/// First applies the base `should_exclude` patterns, then applies config-specific overrides.
pub fn should_exclude_with_config(path: &Path, config: &ScanConfig) -> bool {
    let path_str = path.to_string_lossy();
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // First, apply base exclusions but respect config overrides
    // We need to check each pattern individually to allow overrides

    // Generated/build directories - always exclude
    let always_exclude = [
        "node_modules",
        "target",
        "dist",
        "build",
        ".next",
        "out",
        ".git",
    ];
    if always_exclude
        .iter()
        .any(|p| file_name == *p || path_str.contains(&format!("/{}", p)) || path_str == *p)
    {
        // Allow node_modules override
        if file_name == "node_modules" || path_str.contains("/node_modules/") {
            return !config.include_node_modules;
        }
        return true;
    }

    // Minified files - always exclude
    if file_name.ends_with(".min.js") || file_name.ends_with(".min.css") {
        return true;
    }

    // Vendor directories - always exclude
    let vendor_dirs = ["vendor", "third_party"];
    if vendor_dirs
        .iter()
        .any(|p| file_name == *p || path_str.contains(&format!("/{}", p)))
    {
        return true;
    }

    // Test files and directories - respect include_tests
    if !config.include_tests {
        let is_test_file = file_name.contains("test")
            || file_name.contains("spec")
            || file_name.ends_with("_test.rs")
            || file_name.ends_with("_test.go")
            || file_name.ends_with("_test.py")
            || file_name.ends_with(".test.js")
            || file_name.ends_with(".test.ts")
            || file_name.ends_with(".spec.js")
            || file_name.ends_with(".spec.ts");
        let is_test_dir = file_name == "__tests__" || file_name == "tests" || file_name == "spec";
        let in_test_dir = path_str.contains("/tests/")
            || path_str.contains("/__tests__/")
            || path_str.contains("/spec/");

        if is_test_file || is_test_dir || in_test_dir {
            return true;
        }
    }

    // Examples - respect exclude_examples
    if config.exclude_examples
        && (file_name.contains("example")
            || path_str.contains("/examples/")
            || path_str.contains("/example/"))
    {
        return true;
    }

    // Benchmarks - respect exclude_benches
    if config.exclude_benches
        && (file_name.contains("bench")
            || file_name.contains("benchmark")
            || path_str.contains("/benches/")
            || path_str.contains("/benchmark/"))
    {
        return true;
    }

    // Fixtures - respect exclude_fixtures
    if config.exclude_fixtures
        && (file_name.contains("fixture")
            || path_str.contains("/fixtures/")
            || path_str.contains("/__mocks__/")
            || path_str.contains("/__fixtures__/")
            || path_str.contains("/test_data/"))
    {
        return true;
    }

    // Documentation - respect exclude_docs
    if config.exclude_docs
        && (file_name == "docs"
            || file_name == "documentation"
            || path_str.contains("/docs/")
            || path_str.contains("/documentation/")
            || file_name.ends_with(".md")
            || file_name.ends_with(".rst"))
    {
        return true;
    }

    // Other non-production
    if file_name.starts_with(".env") || file_name.ends_with(".log") {
        return true;
    }

    // Evaluation directories
    if file_name == "evaluation"
        || file_name == "perf"
        || path_str.contains("/evaluation/")
        || path_str.contains("/perf/")
    {
        return true;
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

    #[test]
    fn test_should_exclude_with_config_tests() {
        let config = ScanConfig {
            include_tests: false,
            ..Default::default()
        };
        assert!(should_exclude_with_config(
            Path::new("/project/tests/test.rs"),
            &config
        ));
        assert!(should_exclude_with_config(
            Path::new("/project/src/module_test.rs"),
            &config
        ));

        let config_with_tests = ScanConfig {
            include_tests: true,
            ..Default::default()
        };
        assert!(!should_exclude_with_config(
            Path::new("/project/tests/test.rs"),
            &config_with_tests
        ));
    }

    #[test]
    fn test_should_exclude_with_config_examples() {
        let config = ScanConfig {
            exclude_examples: true,
            ..Default::default()
        };
        assert!(should_exclude_with_config(
            Path::new("/project/examples/demo.rs"),
            &config
        ));

        let config_include_examples = ScanConfig {
            exclude_examples: false,
            ..Default::default()
        };
        assert!(!should_exclude_with_config(
            Path::new("/project/examples/demo.rs"),
            &config_include_examples
        ));
    }

    #[test]
    fn test_should_exclude_with_config_benches() {
        let config = ScanConfig {
            exclude_benches: true,
            ..Default::default()
        };
        assert!(should_exclude_with_config(
            Path::new("/project/benches/benchmark.rs"),
            &config
        ));
    }

    #[test]
    fn test_should_exclude_with_config_node_modules() {
        let config = ScanConfig {
            include_node_modules: false,
            ..Default::default()
        };
        assert!(should_exclude_with_config(
            Path::new("/project/node_modules/pkg/index.js"),
            &config
        ));

        let config_include_node_modules = ScanConfig {
            include_node_modules: true,
            ..Default::default()
        };
        assert!(!should_exclude_with_config(
            Path::new("/project/node_modules/pkg/index.js"),
            &config_include_node_modules
        ));
    }

    #[test]
    fn test_should_exclude_with_config_fixtures() {
        let config = ScanConfig {
            exclude_fixtures: true,
            ..Default::default()
        };
        assert!(should_exclude_with_config(
            Path::new("/project/fixtures/data.json"),
            &config
        ));
        assert!(should_exclude_with_config(
            Path::new("/project/__mocks__/api.ts"),
            &config
        ));
    }

    #[test]
    fn test_should_exclude_with_config_minified() {
        let config = ScanConfig::default();
        assert!(should_exclude_with_config(
            Path::new("/project/app.min.js"),
            &config
        ));
        assert!(should_exclude_with_config(
            Path::new("/project/style.min.css"),
            &config
        ));
    }
}
