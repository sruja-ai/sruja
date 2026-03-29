//! Architecture graph diffing and comparison.
//!
//! This crate provides functionality to compare architecture graphs,
//! identifying differences, new components, missing elements, and potential violations.

mod compare;
mod convert;
mod drift;
mod git_mapper;
mod health;
mod source_ref;
mod types;

pub use compare::compare_graphs;
pub use convert::program_to_graph;
pub use drift::{
    detect_architectural_drift, detect_architectural_drift_with_config, find_circular_dependencies,
    find_orphan_modules,
};
pub use git_mapper::map_git_diff;
pub use health::calculate_health_score_from_violations;
pub use types::{
    ComponentDiff, DiffEdge, DiffError, DiffNode, DiffResult, DiffSummary, DriftConfig,
    DriftReport, EdgeDiff, HealthScoreBreakdown, HealthScorePenalties, NodeDiff, NodeMatch,
    Severity, SourceRef, TruthStatus, Violation, ViolationKind,
};

#[cfg(test)]
mod tests;
