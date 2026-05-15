//! Stable rule identifiers for intent-vs-reality drift findings.

use crate::compare::DriftKind;

#[must_use]
pub fn rule_id_for_drift_kind(kind: DriftKind) -> &'static str {
    match kind {
        DriftKind::UndocumentedComponent => "SRUJA-INTENT-DOC-001",
        DriftKind::MissingComponent => "SRUJA-INTENT-MISS-001",
        DriftKind::BoundaryViolation => "SRUJA-INTENT-BOUNDARY-001",
        DriftKind::UndocumentedRelationship => "SRUJA-INTENT-REL-001",
        DriftKind::MissingRelationship => "SRUJA-INTENT-REL-002",
        DriftKind::TechnologyMismatch => "SRUJA-INTENT-TECH-001",
        DriftKind::PolicyViolation => "SRUJA-INTENT-POLICY-001",
        DriftKind::ConstraintViolation => "SRUJA-INTENT-CONSTRAINT-001",
        DriftKind::SchemaViolation => "SRUJA-INTENT-SCHEMA-001",
        DriftKind::TaxonomyMismatch => "SRUJA-INTENT-TAX-001",
        DriftKind::UnproposedChange => "SRUJA-INTENT-PROPOSAL-001",
    }
}
