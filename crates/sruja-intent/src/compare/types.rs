//! Drift Detection
//!
//! Compares declared architectural intent against actual implementation
//! to detect boundary drift, intent violations, and undocumented changes.

use serde::{Deserialize, Serialize};

pub struct DriftDetector {
    #[allow(dead_code)]
    pub(crate) config: DriftConfig,
}

#[derive(Debug, Clone)]
pub struct DriftConfig {
    pub use_semantic_similarity: bool,
    pub min_confidence: f32,
    pub severity_threshold: Severity,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            use_semantic_similarity: false,
            min_confidence: 0.5,
            severity_threshold: Severity::Info,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub intent_source: String,
    pub reality_source: String,
    pub drifts: Vec<Drift>,
    pub drift_score: u8,
    pub health: DriftHealth,
    pub summary: DriftSummary,
}

impl DriftReport {
    /// Recompute summary counts from drifts and update drift_score and health.
    /// Call after appending additional drifts (e.g. policy violations).
    pub fn recompute_summary_and_score(&mut self) {
        self.summary.undocumented_components = self
            .drifts
            .iter()
            .filter(|d| d.kind == DriftKind::UndocumentedComponent)
            .count();
        self.summary.missing_components = self
            .drifts
            .iter()
            .filter(|d| d.kind == DriftKind::MissingComponent)
            .count();
        self.summary.undocumented_relationships = self
            .drifts
            .iter()
            .filter(|d| d.kind == DriftKind::UndocumentedRelationship)
            .count();
        self.summary.boundary_violations = self
            .drifts
            .iter()
            .filter(|d| d.kind == DriftKind::BoundaryViolation)
            .count();
        self.summary.policy_violations = self
            .drifts
            .iter()
            .filter(|d| d.kind == DriftKind::PolicyViolation)
            .count();
        self.summary.schema_violations = self
            .drifts
            .iter()
            .filter(|d| d.kind == DriftKind::SchemaViolation)
            .count();
        self.summary.taxonomy_mismatches = self
            .drifts
            .iter()
            .filter(|d| d.kind == DriftKind::TaxonomyMismatch)
            .count();
        self.summary.unproposed_changes = self
            .drifts
            .iter()
            .filter(|d| d.kind == DriftKind::UnproposedChange)
            .count();
        self.drift_score = DriftDetector::compute_drift_score(&self.summary, &self.drifts);
        self.health = DriftDetector::classify_health(self.drift_score);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftSummary {
    pub total_components_declared: usize,
    pub total_components_discovered: usize,
    pub undocumented_components: usize,
    pub missing_components: usize,
    pub undocumented_relationships: usize,
    pub boundary_violations: usize,
    pub policy_violations: usize,
    pub schema_violations: usize,
    pub taxonomy_mismatches: usize,
    pub unproposed_changes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drift {
    pub kind: DriftKind,
    pub severity: Severity,
    pub description: String,
    pub evidence: Vec<Evidence>,
    pub intent_ref: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftKind {
    UndocumentedComponent,
    MissingComponent,
    BoundaryViolation,
    UndocumentedRelationship,
    MissingRelationship,
    TechnologyMismatch,
    PolicyViolation,
    ConstraintViolation,
    SchemaViolation,
    TaxonomyMismatch,
    UnproposedChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftHealth {
    Healthy,
    MinorDrift,
    SignificantDrift,
    CriticalDrift,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub source: String,
    pub location: Option<String>,
    pub detail: String,
}

impl Default for DriftDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DriftHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriftHealth::Healthy => write!(f, "Healthy"),
            DriftHealth::MinorDrift => write!(f, "Minor Drift"),
            DriftHealth::SignificantDrift => write!(f, "Significant Drift"),
            DriftHealth::CriticalDrift => write!(f, "Critical Drift"),
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::High => write!(f, "HIGH"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::Low => write!(f, "LOW"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

impl std::fmt::Display for DriftKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriftKind::UndocumentedComponent => write!(f, "Undocumented Component"),
            DriftKind::MissingComponent => write!(f, "Missing Component"),
            DriftKind::BoundaryViolation => write!(f, "Boundary Violation"),
            DriftKind::UndocumentedRelationship => write!(f, "Undocumented Relationship"),
            DriftKind::MissingRelationship => write!(f, "Missing Relationship"),
            DriftKind::TechnologyMismatch => write!(f, "Technology Mismatch"),
            DriftKind::PolicyViolation => write!(f, "Policy Violation"),
            DriftKind::ConstraintViolation => write!(f, "Constraint Violation"),
            DriftKind::SchemaViolation => write!(f, "Schema Violation"),
            DriftKind::TaxonomyMismatch => write!(f, "Taxonomy Mismatch"),
            DriftKind::UnproposedChange => write!(f, "Unproposed Change"),
        }
    }
}
