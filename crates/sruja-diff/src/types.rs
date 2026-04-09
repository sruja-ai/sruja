//! Core types for diff results, violations, and drift reports.

use serde::{Deserialize, Serialize};
use sruja_scan::{EdgeKind, NodeKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiffError {
    #[error("Graph comparison error: {0}")]
    Comparison(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeMatch {
    pub proposal_id: String,
    pub actual_id: String,
    pub similarity: f32,
    pub kind_match: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeDiff {
    pub added: Vec<DiffNode>,
    pub removed: Vec<DiffNode>,
    pub matched: Vec<NodeMatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeDiff {
    pub added: Vec<DiffEdge>,
    pub removed: Vec<DiffEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffNode {
    pub id: String,
    pub kind: NodeKind,
    pub label: String,
    pub technology: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffEdge {
    pub source: String,
    pub target: String,
    pub kind: EdgeKind,
    pub label: Option<String>,
}

/// Reference to a source location (file, line) for evidence in reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SourceRef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl SourceRef {
    /// Format as a short reference string, e.g. `path/to/file.ts:42` or `path/to/file.ts`.
    #[must_use]
    pub fn display_string(&self) -> String {
        match (&self.file, self.line) {
            (Some(f), Some(l)) => format!("{}:{}", f, l),
            (Some(f), None) => f.clone(),
            _ => self.detail.clone().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Violation {
    pub kind: ViolationKind,
    pub severity: Severity,
    pub message: String,
    pub location: Option<String>,
    pub suggestion: Option<String>,
    /// Source references (file, line) so findings can be traced back to code or docs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<SourceRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub production_relevant: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_delta: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppressed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ViolationKind {
    LayerViolation,
    MissingDependency,
    OrphanComponent,
    CircularDependency,
    UndocumentedComponent,
    PatternMismatch,
    GodModule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Truth state for architecture vs evidence: reviewed (DSL matches), drifted (violations), unknown (no baseline).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TruthStatus {
    Reviewed,
    Drifted,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffResult {
    pub proposal_title: String,
    pub node_diff: NodeDiff,
    pub edge_diff: EdgeDiff,
    pub violations: Vec<Violation>,
    pub suggestions: Vec<String>,
    pub summary: DiffSummary,
    /// reviewed = no violations; drifted = has violations.
    pub truth_status: TruthStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffSummary {
    pub proposed_components: usize,
    pub existing_components: usize,
    pub new_components: usize,
    pub missing_components: usize,
    pub new_dependencies: usize,
    pub removed_dependencies: usize,
    pub health_score: u8,
}

impl DiffResult {
    pub fn is_empty(&self) -> bool {
        self.node_diff.added.is_empty()
            && self.node_diff.removed.is_empty()
            && self.edge_diff.added.is_empty()
            && self.edge_diff.removed.is_empty()
    }

    pub fn has_issues(&self) -> bool {
        !self.violations.is_empty() || !self.node_diff.removed.is_empty()
    }
}

/// Penalties applied per violation severity when computing health score.
#[derive(Debug, Clone, Copy)]
pub struct HealthScorePenalties {
    pub error: u8,
    pub warning: u8,
    pub info: u8,
}

impl Default for HealthScorePenalties {
    fn default() -> Self {
        Self {
            error: 15,
            warning: 5,
            info: 2,
        }
    }
}

/// Configuration for architectural drift detection.
#[derive(Debug, Clone)]
pub struct DriftConfig {
    /// Minimum number of dependencies before a module is flagged as a god module.
    pub god_module_threshold: usize,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            god_module_threshold: 10,
        }
    }
}

/// Per-category penalties for health score (structural only). Exposed so consumers can see why the score is what it is.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HealthScoreBreakdown {
    pub cycle_penalty: u8,
    pub layer_penalty: u8,
    pub god_module_penalty: u8,
    pub orphan_penalty: u8,
    pub other_penalty: u8,
}

/// Result of architectural drift detection (no DSL baseline: structural-only analysis).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    /// Scan scope metadata (what was included/excluded).
    pub scan_scope: sruja_scan::scan_scope::ScanScope,
    pub total_modules: usize,
    pub total_services: usize,
    pub total_databases: usize,
    pub total_dependencies: usize,
    pub circular_dependencies: usize,
    pub orphan_modules: usize,
    pub layer_violations: usize,
    pub violations: Vec<Violation>,
    pub suggestions: Vec<String>,
    pub health_score: u8,
    /// Why the health score is what it is (structural only: cycles, layers, god modules, orphans).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_breakdown: Option<HealthScoreBreakdown>,
    /// No baseline: truth_status is always unknown for structural-only drift.
    pub truth_status: TruthStatus,
}
/// Results of mapping a git diff to architectural components.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentDiff {
    pub component_id: String,
    pub files_changed: Vec<String>,
    pub lines_added: usize,
    pub lines_deleted: usize,
}
