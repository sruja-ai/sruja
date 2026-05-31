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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compare::DriftKind;

    #[test]
    fn rule_ids_are_stable_and_unique() {
        let kinds = [
            DriftKind::UndocumentedComponent,
            DriftKind::MissingComponent,
            DriftKind::BoundaryViolation,
            DriftKind::UndocumentedRelationship,
            DriftKind::MissingRelationship,
            DriftKind::TechnologyMismatch,
            DriftKind::PolicyViolation,
            DriftKind::ConstraintViolation,
            DriftKind::SchemaViolation,
            DriftKind::TaxonomyMismatch,
            DriftKind::UnproposedChange,
        ];
        let ids: Vec<&str> = kinds.iter().map(|k| rule_id_for_drift_kind(*k)).collect();
        assert_eq!(ids.len(), kinds.len());
        for id in &ids {
            assert!(id.starts_with("SRUJA-INTENT-"));
        }
        let unique: std::collections::HashSet<_> = ids.iter().copied().collect();
        assert_eq!(unique.len(), ids.len(), "rule ids must be unique: {ids:?}");
    }

    #[test]
    fn boundary_violation_maps_to_boundary_rule_id() {
        assert_eq!(
            rule_id_for_drift_kind(DriftKind::BoundaryViolation),
            "SRUJA-INTENT-BOUNDARY-001"
        );
    }
}
