//! Architecture graph diffing and comparison.
//!
//! This crate provides functionality to compare architecture graphs,
//! identifying differences, new components, missing elements, and potential violations.

pub mod compare;
pub mod convert;
pub mod drift;
pub mod git_mapper;
pub mod health;
pub mod proposal;
pub mod rule_ids;
pub mod source_ref;
pub mod types;

pub use compare::{compare_graphs, compare_graphs_with_options, CompareOptions};
pub use convert::program_to_graph;
pub use drift::{
    detect_architectural_drift, detect_architectural_drift_with_config, find_circular_dependencies,
    find_orphan_modules,
};
pub use git_mapper::{architectural_velocity, map_git_diff, ArchitecturalVelocity};
pub use health::calculate_health_score_from_violations;
pub use proposal::{
    detect_unproposed_changes, Proposal, ProposalChange, ProposalStatus, ProposalValidation,
};
pub use rule_ids::{default_confidence_for_kind, rule_id_for_kind};
pub use types::{
    annotate_violation_metadata, BaselineMode, ComponentDiff, DiffEdge, DiffError, DiffNode,
    DiffResult, DiffSummary, DriftConfig, DriftReport, EdgeDiff, HealthScoreBreakdown,
    HealthScorePenalties, NodeDiff, NodeMatch, Severity, SourceRef, TruthStatus, Violation,
    ViolationKind,
};

#[cfg(test)]
mod tests;
