//! Shared analysis pipeline: scan + compare + violation filtering.
//!
//! This module extracts the common analysis steps used by `status`, `health`,
//! `review`, `sync`, and `drift` commands into a single reusable pipeline.
//!
//! ## Pipeline steps
//! 1. Scan repo → `Graph`
//! 2. Resolve architecture path → `Option<PathBuf>`
//! 3. Parse architecture → `Option<Program>`
//! 4. Compare graphs (or structural-only drift) → violations + truth status + health score
//! 5. Filter violations (baseline suppression, production-relevance)

use std::path::{Path, PathBuf};

use super::violation_shared::*;
use super::CliError;
use crate::utils::architecture_path;

/// Result of the shared analysis pipeline.
pub struct AnalysisResult {
    /// The scanned code graph.
    pub graph: sruja_scan::Graph,
    /// Truth status from comparing scan vs architecture.
    pub truth_status: String,
    /// Health score (0–100) from violations.
    pub health_score: u8,
    /// Active (non-suppressed) violations.
    pub active_violations: Vec<sruja_diff::Violation>,
    /// Baseline-suppressed violations.
    pub suppressed_violations: Vec<sruja_diff::Violation>,
}

/// Options for the analysis pipeline.
pub struct AnalysisOptions {
    /// If true, skip architecture comparison (structural-only analysis).
    pub structural_only: bool,
    /// If true, suppress orphan INFO findings.
    pub advisory: bool,
    /// Path to violations baseline file for suppression.
    pub violations_baseline_path: Option<PathBuf>,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            structural_only: false,
            advisory: false,
            violations_baseline_path: None,
        }
    }
}

/// Run the full analysis pipeline: scan → compare → filter violations.
///
/// This is the single source of truth for the analysis steps shared by
/// `status`, `health`, `review`, `sync`, and `drift` commands.
pub fn run_analysis(repo_path: &Path, opts: &AnalysisOptions) -> Result<AnalysisResult, CliError> {
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_path.display()),
        )));
    }

    // Step 1: Scan repo
    let graph = sruja_scan::scan_repo(repo_path)?;

    // Step 2: Resolve architecture path
    let baseline_path = if opts.structural_only {
        None
    } else {
        architecture_path::resolve_architecture_path(repo_path)
    };

    // Step 3 & 4: Compare or detect drift
    let (truth_status, raw_violations, health_score) =
        if let Some(ref arch_path) = baseline_path {
            let content = std::fs::read_to_string(arch_path)?;
            let parser =
                sruja_language::Parser::new(arch_path.to_string_lossy().as_ref());
            let program = parser.parse(&content).map_err(|diags| {
                CliError::parse_with_diagnostics(
                    arch_path.to_string_lossy().to_string(),
                    diags,
                )
            })?;
            let proposed_graph = sruja_diff::program_to_graph(&program);
            let diff = sruja_diff::compare_graphs(&graph, &proposed_graph);
            let truth = match diff.truth_status {
                sruja_diff::TruthStatus::Reviewed => "reviewed",
                sruja_diff::TruthStatus::Drifted => "drifted",
                sruja_diff::TruthStatus::Unknown => "unknown",
            };
            (
                truth.to_string(),
                diff.violations,
                diff.summary.health_score,
            )
        } else {
            let drift = sruja_diff::detect_architectural_drift(&graph);
            let truth = match drift.truth_status {
                sruja_diff::TruthStatus::Reviewed => "reviewed",
                sruja_diff::TruthStatus::Drifted => "drifted",
                sruja_diff::TruthStatus::Unknown => "unknown",
            };
            (
                truth.to_string(),
                drift.violations,
                drift.health_score,
            )
        };

    // Step 5: Filter violations
    let violations: Vec<sruja_diff::Violation> = if opts.advisory {
        raw_violations
            .into_iter()
            .filter(|v| {
                !matches!(v.kind, sruja_diff::ViolationKind::OrphanComponent)
                    || v.severity != sruja_diff::Severity::Info
            })
            .collect()
    } else {
        raw_violations
            .into_iter()
            .filter(is_production_relevant)
            .collect()
    };

    // Apply baseline suppression if available
    let baseline_set: Option<std::collections::HashSet<String>> =
        if let Some(ref bp) = opts.violations_baseline_path {
            if bp.exists() {
                Some(load_violations_baseline(bp)?.fingerprints)
            } else {
                None
            }
        } else {
            None
        };

    let (active_violations, suppressed_violations): (
        Vec<sruja_diff::Violation>,
        Vec<sruja_diff::Violation>,
    ) = if let Some(ref set) = baseline_set {
        violations
            .into_iter()
            .map(|mut v| {
                let suppressed = set.contains(&fingerprint_violation(&v));
                v.suppressed = Some(suppressed);
                v.baseline_delta =
                    Some(if suppressed { "baseline" } else { "new" }.to_string());
                v
            })
            .partition(|v| v.suppressed != Some(true))
    } else {
        (violations, Vec::new())
    };

    Ok(AnalysisResult {
        graph,
        truth_status,
        health_score,
        active_violations,
        suppressed_violations,
    })
}

/// Run analysis with default options (no structural-only, no advisory, no baseline).
pub fn run_analysis_default(repo_path: &Path) -> Result<AnalysisResult, CliError> {
    run_analysis(repo_path, &AnalysisOptions::default())
}
