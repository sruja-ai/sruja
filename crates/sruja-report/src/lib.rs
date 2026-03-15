//! Compliance report schema for Sruja.
//!
//! This crate defines compliance DTOs used by CLI for JSON output.

pub mod compliance;

pub use compliance::{ComplianceReport, ComplianceStatus, DriftEntry, PolicyViolationEntry};
