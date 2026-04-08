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
//!
//! Supports reading exclusion patterns from:
//! - DEFAULT_EXCLUDE_PATTERNS (hardcoded safe defaults)
//! - .gitignore (repo-specific git ignore rules)
//! - .srujaignore (Sruja-specific scan exclusions)

use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};

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
    "book",
    "site",
    "website",
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

/// Read exclusion patterns from a .gitignore or .srujaignore file.
pub fn read_ignore_file(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|content| {
            content
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        None
                    } else {
                        Some(line.to_string())
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Load all ignore patterns from a repository root.
/// Returns patterns from .gitignore and .srujaignore files.
pub fn load_ignore_patterns(repo_root: &Path) -> Vec<String> {
    let mut patterns = Vec::new();

    if let Some(gitignore) = find_gitignore(repo_root) {
        patterns.extend(read_ignore_file(&gitignore));
    }

    let srujaignore = repo_root.join(".srujaignore");
    if srujaignore.exists() {
        patterns.extend(read_ignore_file(&srujaignore));
    }

    patterns
}

/// Find the nearest .gitignore file, walking up the directory tree.
fn find_gitignore(start: &Path) -> Option<std::path::PathBuf> {
    let mut current = start;
    loop {
        let gitignore = current.join(".gitignore");
        if gitignore.exists() {
            return Some(gitignore);
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => return None,
        }
    }
}

/// Merge default patterns with user-defined ignore patterns.
pub fn merge_ignore_patterns(default: &[&str], user: &[String]) -> Vec<String> {
    let mut merged: Vec<String> = default.iter().map(|s| s.to_string()).collect();
    merged.extend(user.iter().cloned());
    merged.sort();
    merged.dedup();
    merged
}

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

    /// Patterns used for exclusion (defaults + user-defined from .gitignore/.srujaignore)
    pub exclude_patterns: Vec<String>,

    /// User-defined patterns loaded from .gitignore and .srujaignore
    pub user_patterns: Vec<String>,
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
            user_patterns: Vec::new(),
        }
    }
}

/// Resolve scan scope for a repository.
///
/// Walks the repo, applies default exclude patterns plus patterns from
/// .gitignore and .srujaignore files, and populates
/// `ScanScope::included`, `excluded`, and `total_files` for skill-facing JSON.
pub fn resolve_scan_scope(repo_root: &Path) -> (ScanConfig, ScanScope) {
    let config = ScanConfig::default();
    let user_patterns = load_ignore_patterns(repo_root);
    let scope = build_scope_from_walk_with_patterns(repo_root, &user_patterns);
    (config, scope)
}

/// Resolve scan scope with custom user patterns.
pub fn resolve_scan_scope_with_patterns(
    repo_root: &Path,
    user_patterns: &[String],
) -> (ScanConfig, ScanScope) {
    let config = ScanConfig::default();
    let scope = build_scope_from_walk_with_patterns(repo_root, user_patterns);
    (config, scope)
}

/// Walk repo and classify paths to build ScanScope metadata.
/// - included: top-level segment of any dir that contains at least one scanned file.
/// - excluded: full relative path of each directory we skipped (did not descend into).
fn build_scope_from_walk_with_patterns(repo_root: &Path, user_patterns: &[String]) -> ScanScope {
    let mut included_top_level: HashSet<String> = HashSet::new();
    let mut total_files = 0usize;

    let repo_root_buf = repo_root.to_path_buf();
    let excluded_rel_paths: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

    let mut builder = ignore::WalkBuilder::new(repo_root);
    builder
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true);
    builder.add_custom_ignore_filename(".srujaignore");

    let config = ScanConfig::default();
    let config_clone = config.clone();
    let excluded_rel_paths_clone = Arc::clone(&excluded_rel_paths);
    let repo_root_for_filter = repo_root_buf.clone();
    builder.filter_entry(move |e| {
        let path = e.path();
        let excluded = should_exclude_with_config(path, &config_clone);

        if e.file_type().is_some_and(|ft| ft.is_dir()) && excluded {
            if let Ok(rel) = path.strip_prefix(&repo_root_for_filter) {
                let rel = rel.to_string_lossy().replace('\\', "/");
                if let Ok(mut set) = excluded_rel_paths_clone.lock() {
                    set.insert(rel);
                }
            }
            return false;
        }

        !excluded
    });

    let walker = builder.build();
    for entry in walker {
        let Ok(entry) = entry else { continue };
        let path = entry.path();

        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }

        total_files += 1;
        if let Ok(rel) = path.strip_prefix(&repo_root_buf) {
            let rel = rel.to_string_lossy().replace('\\', "/");
            if let Some(seg) = rel.split('/').next() {
                included_top_level.insert(seg.to_string());
            }
        }
    }

    let mut included: Vec<String> = included_top_level.into_iter().collect();
    included.sort();
    let mut excluded: Vec<String> = excluded_rel_paths
        .lock()
        .map(|set| set.iter().cloned().collect())
        .unwrap_or_default();
    excluded.sort();
    let merged_patterns = merge_ignore_patterns(DEFAULT_EXCLUDE_PATTERNS, user_patterns);

    ScanScope {
        included,
        excluded,
        total_files,
        exclude_patterns: merged_patterns,
        user_patterns: user_patterns.to_vec(),
    }
}

/// Check if a path reflects production-relevant code.
///
/// Excludes paths that match non-production segments (docs, evaluation, book, etc.)
/// and documentation-only files (.md, .rst).
///
/// Used to filter drift/violations so PR signals are focused on core logic.
pub fn is_path_production_relevant(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let lower = normalized.to_lowercase();

    // Check against non-production directory segments
    // We look for the segment as a standalone component in the path
    for seg in DEFAULT_EXCLUDE_PATTERNS {
        // Only check directory or explicit file patterns from the exclude list
        if seg.contains('*') {
            continue;
        }

        let s = seg.trim_end_matches('/');
        if lower.contains(&format!("/{}/", s))
            || lower.starts_with(&format!("{}/", s))
            || lower.ends_with(&format!("/{}", s))
            || lower == *s
        {
            return false;
        }
    }

    // Exclude documentation files explicitly
    if lower.ends_with(".md") || lower.ends_with(".rst") {
        return false;
    }

    true
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

    #[test]
    fn test_is_path_production_relevant() {
        assert!(is_path_production_relevant("src/main.rs"));
        assert!(is_path_production_relevant("crates/sruja-cli/src/main.rs"));

        assert!(!is_path_production_relevant("docs/index.md"));
        assert!(!is_path_production_relevant("book/summary.md"));
        assert!(!is_path_production_relevant("evaluation/perf.log"));
        assert!(!is_path_production_relevant("target/debug/sruja"));
        assert!(!is_path_production_relevant("node_modules/lodash/index.js"));

        // Nested patterns
        assert!(!is_path_production_relevant("src/docs/api.md"));
        assert!(!is_path_production_relevant("src/tests/unit.rs"));
        assert!(!is_path_production_relevant("src/test_data/fixture.json"));

        // File extensions
        assert!(!is_path_production_relevant("README.md"));
        assert!(!is_path_production_relevant("ARCHITECTURE.rst"));
    }
}
