//! Sruja Extraction Framework
//!
//! Automatic discovery of architectural artifacts (OpenAPI specs, Kubernetes
//! manifests, Dockerfiles, Terraform configs, Helm charts, Protobuf schemas,
//! GraphQL schemas, AsyncAPI specs, and more) from a codebase.
//!
//! The framework is designed around a pluggable [`Extractor`] trait with a
//! [`FileContext`] that lazily reads file content to avoid redundant I/O
//! across multiple extractors scanning the same file.
//!
//! # Quick Start
//!
//! ```no_run
//! use sruja_extract::ExtractionEngine;
//! use std::path::Path;
//!
//! let engine = ExtractionEngine::default();
//! let report = engine.discover(Path::new("."));
//! println!("Found {} artifacts in {}ms", report.sources.len(), report.stats.duration_ms);
//! ```
//!
//! # Custom Configuration
//!
//! ```no_run
//! use sruja_extract::{ExtractionEngine, ExtractionConfig};
//! use std::path::Path;
//!
//! let config = ExtractionConfig {
//!     min_confidence: 0.5,
//!     max_file_size: 5 * 1024 * 1024,
//!     ..Default::default()
//! };
//! let engine = ExtractionEngine::with_config(config);
//! let report = engine.discover(Path::new("."));
//! ```

pub mod alias;
pub mod asyncapi;
pub mod config;
pub mod dependency;
pub mod dockerfile;
pub mod docs;
pub mod graphql;
pub mod helm;
pub mod kubernetes;
pub mod openapi;
pub mod proto;
pub mod terraform;
pub mod utils;

use sruja_language::ast::SourceBinding;
use std::cell::OnceCell;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("IO error reading {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },

    #[error("Parse error in {path}: {message}")]
    Parse { path: String, message: String },

    #[error("Discovery error: {0}")]
    Discovery(String),
}

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// A discovered source binding with context about which architectural element
/// it likely belongs to.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscoveredSource {
    pub binding: SourceBinding,
    /// Suggested element FQN or ID this source belongs to (if inferable).
    pub suggested_element: Option<String>,
    /// Confidence score in `[0.0, 1.0]`.
    pub confidence: f32,
}

/// Identity comparison for dedup: two sources are "the same" if they point
/// to the same file + kind + element, regardless of confidence or description.
impl PartialEq for DiscoveredSource {
    fn eq(&self, other: &Self) -> bool {
        self.binding.path == other.binding.path
            && self.binding.kind == other.binding.kind
            && self.suggested_element == other.suggested_element
    }
}

impl std::fmt::Display for DiscoveredSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} (element: {}, confidence: {:.0}%)",
            self.binding.kind.as_str(),
            self.binding.path,
            self.suggested_element.as_deref().unwrap_or("unknown"),
            self.confidence * 100.0,
        )
    }
}

// ---------------------------------------------------------------------------
// FileContext – lazy, cached file I/O shared across extractors
// ---------------------------------------------------------------------------

/// Context for file inspection. Lazily reads and caches file content so
/// multiple extractors inspecting the same file only trigger one read.
pub struct FileContext<'a> {
    pub path: &'a Path,
    pub repo_root: &'a Path,
    relative_path: String,
    content: OnceCell<Option<String>>,
}

impl<'a> FileContext<'a> {
    pub fn new(path: &'a Path, repo_root: &'a Path) -> Self {
        let relative_path = path
            .strip_prefix(repo_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        Self {
            path,
            repo_root,
            relative_path,
            content: OnceCell::new(),
        }
    }

    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }

    pub fn file_name(&self) -> &str {
        self.path.file_name().and_then(|n| n.to_str()).unwrap_or("")
    }

    pub fn file_name_lower(&self) -> String {
        self.file_name().to_lowercase()
    }

    pub fn extension(&self) -> &str {
        self.path.extension().and_then(|e| e.to_str()).unwrap_or("")
    }

    /// Read and cache the file content. Returns `None` for binary/unreadable files.
    pub fn content(&self) -> Option<&str> {
        self.content
            .get_or_init(|| std::fs::read_to_string(self.path).ok())
            .as_deref()
    }

    pub fn parent_dir_name(&self) -> Option<&str> {
        self.path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
    }
}

// ---------------------------------------------------------------------------
// Extractor trait
// ---------------------------------------------------------------------------

/// Trait for automatic discovery of architectural artifacts from files.
///
/// Implementors should return an empty `Vec` for files they don't recognize
/// rather than erroring. Reserve `Err` for genuine I/O or parse failures that
/// the caller should know about.
pub trait Extractor: Send + Sync {
    /// Human-readable name for this extractor (e.g. `"kubernetes"`, `"openapi"`).
    fn name(&self) -> &'static str;

    /// Inspect a single file and return any discovered architectural sources.
    fn check_file(&self, ctx: &FileContext) -> Result<Vec<DiscoveredSource>, ExtractError>;

    /// Called once after the full walk. Override for cross-file inference.
    fn finalize(&self) -> Result<Vec<DiscoveredSource>, ExtractError> {
        Ok(Vec::new())
    }
}

// ---------------------------------------------------------------------------
// ExtractionConfig – builder-pattern configuration
// ---------------------------------------------------------------------------

/// Configuration for the extraction engine.
#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    /// Discard results below this confidence threshold.
    pub min_confidence: f32,
    /// Maximum file size in bytes to consider (skip larger files).
    pub max_file_size: u64,
    /// Only run these extractors (by name). `None` = all.
    pub enabled_extractors: Option<Vec<String>>,
    /// Additional ignore patterns (glob-style) beyond .gitignore.
    pub extra_ignore_patterns: Vec<String>,
    /// Whether to respect .gitignore files (default: true).
    pub respect_gitignore: bool,
    /// Whether to follow symlinks (default: false).
    pub follow_symlinks: bool,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.0,
            max_file_size: 10 * 1024 * 1024,
            enabled_extractors: None,
            extra_ignore_patterns: Vec::new(),
            respect_gitignore: true,
            follow_symlinks: false,
        }
    }
}

impl ExtractionConfig {
    /// Convenience builder that returns a mutable config with defaults.
    /// Use struct update syntax for more complex cases:
    /// ```
    /// # use sruja_extract::ExtractionConfig;
    /// let config = ExtractionConfig {
    ///     min_confidence: 0.5,
    ///     max_file_size: 5 * 1024 * 1024,
    ///     ..Default::default()
    /// };
    /// ```
    pub fn with_min_confidence(confidence: f32) -> Self {
        Self {
            min_confidence: confidence,
            ..Default::default()
        }
    }
}

// ---------------------------------------------------------------------------
// ExtractionReport – structured output for humans and AI consumers
// ---------------------------------------------------------------------------

/// Rich extraction report with sources, statistics, and diagnostics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExtractionReport {
    pub sources: Vec<DiscoveredSource>,
    pub stats: ExtractionStats,
    pub diagnostics: Vec<ExtractionDiagnostic>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExtractionStats {
    pub files_scanned: usize,
    pub files_matched: usize,
    pub total_sources: usize,
    pub by_extractor: HashMap<String, usize>,
    pub by_kind: HashMap<String, usize>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExtractionDiagnostic {
    pub level: DiagnosticLevel,
    pub extractor: String,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Warning,
    Error,
}

impl std::fmt::Display for ExtractionReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Extraction Report")?;
        writeln!(f, "  Files scanned : {}", self.stats.files_scanned)?;
        writeln!(f, "  Files matched : {}", self.stats.files_matched)?;
        writeln!(f, "  Total sources : {}", self.stats.total_sources)?;
        writeln!(f, "  Duration      : {}ms", self.stats.duration_ms)?;
        if !self.stats.by_kind.is_empty() {
            writeln!(f, "  By kind:")?;
            let mut kinds: Vec<_> = self.stats.by_kind.iter().collect();
            kinds.sort_by_key(|(_, v)| std::cmp::Reverse(**v));
            for (kind, count) in kinds {
                writeln!(f, "    {kind}: {count}")?;
            }
        }
        if !self.diagnostics.is_empty() {
            writeln!(f, "  Diagnostics: {} issues", self.diagnostics.len())?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ExtractionEngine – orchestrates extractors over a file tree
// ---------------------------------------------------------------------------

/// Orchestrates multiple extractors across a repository tree.
///
/// Uses the `ignore` crate for `.gitignore`-aware walking to skip irrelevant
/// files automatically.
pub struct ExtractionEngine {
    extractors: Vec<Box<dyn Extractor>>,
    config: ExtractionConfig,
}

impl ExtractionEngine {
    /// Create an engine with default extractors and default config.
    pub fn new() -> Self {
        Self::with_config(ExtractionConfig::default())
    }

    /// Create an engine with default extractors and custom config.
    pub fn with_config(config: ExtractionConfig) -> Self {
        let mut extractors: Vec<Box<dyn Extractor>> = vec![
            Box::new(openapi::OpenApiExtractor::new()),
            Box::new(asyncapi::AsyncApiExtractor::new()),
            Box::new(kubernetes::KubernetesExtractor::new()),
            Box::new(docs::DocExtractor::new()),
            Box::new(alias::AliasExtractor::new()),
            Box::new(dependency::DependencyExtractor::new()),
            Box::new(dockerfile::DockerfileExtractor::new()),
            Box::new(terraform::TerraformExtractor::new()),
            Box::new(proto::ProtoExtractor::new()),
            Box::new(graphql::GraphqlExtractor::new()),
            Box::new(helm::HelmExtractor::new()),
            Box::new(config::ConfigExtractor::new()),
        ];

        if let Some(ref enabled) = config.enabled_extractors {
            extractors.retain(|e| enabled.iter().any(|n| n == e.name()));
        }

        Self { extractors, config }
    }

    /// Create an engine with only the provided extractors.
    pub fn with_extractors(extractors: Vec<Box<dyn Extractor>>, config: ExtractionConfig) -> Self {
        Self { extractors, config }
    }

    /// Add a custom extractor at runtime.
    pub fn add_extractor(&mut self, extractor: Box<dyn Extractor>) {
        self.extractors.push(extractor);
    }

    /// Run discovery across the entire repository tree.
    pub fn discover(&self, repo_root: &Path) -> ExtractionReport {
        let start = Instant::now();
        let mut all_sources = Vec::new();
        let mut diagnostics = Vec::new();
        let mut files_scanned: usize = 0;
        let mut files_matched: usize = 0;
        let mut by_extractor: HashMap<String, usize> = HashMap::new();

        let walker = self.build_walker(repo_root);

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(err) => {
                    diagnostics.push(ExtractionDiagnostic {
                        level: DiagnosticLevel::Warning,
                        extractor: "engine".to_string(),
                        path: String::new(),
                        message: format!("Walk error: {err}"),
                    });
                    continue;
                }
            };

            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Ok(meta) = path.metadata() {
                if meta.len() > self.config.max_file_size {
                    continue;
                }
            }

            files_scanned += 1;
            let ctx = FileContext::new(path, repo_root);
            let mut file_had_results = false;

            for extractor in &self.extractors {
                match extractor.check_file(&ctx) {
                    Ok(results) => {
                        let count = results.len();
                        if count > 0 {
                            file_had_results = true;
                            *by_extractor
                                .entry(extractor.name().to_string())
                                .or_default() += count;
                            all_sources.extend(results);
                        }
                    }
                    Err(err) => {
                        diagnostics.push(ExtractionDiagnostic {
                            level: DiagnosticLevel::Error,
                            extractor: extractor.name().to_string(),
                            path: ctx.relative_path().to_string(),
                            message: err.to_string(),
                        });
                    }
                }
            }

            if file_had_results {
                files_matched += 1;
            }
        }

        for extractor in &self.extractors {
            match extractor.finalize() {
                Ok(results) => {
                    let count = results.len();
                    if count > 0 {
                        *by_extractor
                            .entry(extractor.name().to_string())
                            .or_default() += count;
                        all_sources.extend(results);
                    }
                }
                Err(err) => {
                    diagnostics.push(ExtractionDiagnostic {
                        level: DiagnosticLevel::Error,
                        extractor: extractor.name().to_string(),
                        path: String::new(),
                        message: format!("Finalize error: {err}"),
                    });
                }
            }
        }

        if self.config.min_confidence > 0.0 {
            all_sources.retain(|s| s.confidence >= self.config.min_confidence);
        }

        let mut by_kind: HashMap<String, usize> = HashMap::new();
        for s in &all_sources {
            *by_kind
                .entry(s.binding.kind.as_str().to_string())
                .or_default() += 1;
        }

        let total_sources = all_sources.len();

        ExtractionReport {
            sources: all_sources,
            stats: ExtractionStats {
                files_scanned,
                files_matched,
                total_sources,
                by_extractor,
                by_kind,
                duration_ms: start.elapsed().as_millis() as u64,
            },
            diagnostics,
        }
    }

    /// Backward-compatible wrapper that returns only the source list.
    pub fn discover_all(&self, repo_root: &Path) -> Vec<DiscoveredSource> {
        self.discover(repo_root).sources
    }

    fn build_walker(&self, repo_root: &Path) -> ignore::Walk {
        let mut builder = ignore::WalkBuilder::new(repo_root);
        builder
            .hidden(true)
            .git_ignore(self.config.respect_gitignore)
            .git_global(self.config.respect_gitignore)
            .git_exclude(self.config.respect_gitignore)
            .follow_links(self.config.follow_symlinks)
            .filter_entry(|e| {
                let name = e.file_name().to_str().unwrap_or("");
                !(name == "node_modules"
                    || name == "target"
                    || name == ".next"
                    || name == "dist"
                    || name == "__pycache__"
                    || name == ".venv"
                    || name == "vendor"
                    || name == ".terraform")
            });

        if !self.config.extra_ignore_patterns.is_empty() {
            let mut ov = ignore::overrides::OverrideBuilder::new(repo_root);
            for pattern in &self.config.extra_ignore_patterns {
                let _ = ov.add(&format!("!{pattern}"));
            }
            if let Ok(overrides) = ov.build() {
                builder.overrides(overrides);
            }
        }

        builder.build()
    }
}

impl Default for ExtractionEngine {
    fn default() -> Self {
        Self::new()
    }
}
