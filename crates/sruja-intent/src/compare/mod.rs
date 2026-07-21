//! Drift Detection
//!
//! Compares declared architectural intent against actual implementation
//! to detect boundary drift, intent violations, and undocumented changes.

pub mod types;
mod detect;
#[cfg(test)]
mod tests;
mod format;

pub mod mapper;

pub use types::{
    Drift, DriftConfig, DriftDetector, DriftHealth, DriftKind, DriftReport, DriftSummary,
    Evidence, Severity,
};
pub use format::{node_matches_selector, node_matches_selector_strict};
