//! Comprehensive report types for four-layer architecture intelligence.
//!
//! DTO and recommendation-building from structural/semantic/intent results.
//! Callers (CLI, MCP) build ComprehensiveReport using build_recommendations.

use serde::{Deserialize, Serialize};
use sruja_diff::{DriftReport, Severity, ViolationKind};

/// Layer 1: Structural analysis summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralSection {
    pub modules: usize,
    pub services: usize,
    pub databases: usize,
    pub dependencies: usize,
    pub health_score: u8,
    pub violations_count: usize,
}

/// Layer 2: Semantic analysis summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSection {
    pub component_count: usize,
    pub context_count: usize,
    pub hidden_coupling_count: usize,
    pub vocabulary_leak_count: usize,
    pub health_score: u8,
}

/// Layer 3: Intent vs reality summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentSection {
    pub drift_score: u8,
    pub health: String,
    pub components_declared: usize,
    pub components_discovered: usize,
    pub undocumented_count: usize,
    pub missing_count: usize,
}

/// Layer 4: Runtime analysis summary (minimal fields to avoid depending on sruja-runtime).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSection {
    pub trace_count: usize,
    pub total_spans: usize,
    pub max_depth: usize,
    pub total_duration_ms: i64,
    pub emergent_cycle_count: usize,
    pub hotspot_count: usize,
}

/// Source layer for a recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Layer {
    Structural,
    Semantic,
    Intent,
    Runtime,
}

/// Priority for a recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Category of recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationCategory {
    Cycle,
    Orphan,
    LayerViolation,
    GodModule,
    SemanticCoupling,
    VocabularyLeak,
    IntentDrift,
    BoundaryViolation,
    RuntimeCycle,
    Hotspot,
    Other,
}

/// Estimated effort for a recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
    Low,
    Medium,
    High,
}

/// A single actionable recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: Priority,
    pub category: RecommendationCategory,
    pub description: String,
    pub affected_components: Vec<String>,
    pub source_layer: Layer,
    pub estimated_effort: Effort,
}

/// Schema version for report compatibility. Increment when the report shape changes in a breaking way.
pub const REPORT_SCHEMA_VERSION: u16 = 1;

/// Aggregated report across all layers.
///
/// When serializing (e.g. JSON), include `schema_version` so consumers can detect compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveReport {
    /// Report schema version; use for compatibility checks when parsing.
    #[serde(default = "default_schema_version")]
    pub schema_version: u16,
    pub structural: StructuralSection,
    pub semantic: SemanticSection,
    pub intent: Option<IntentSection>,
    pub runtime: Option<RuntimeSection>,
    pub overall_health: u8,
    pub recommendations: Vec<Recommendation>,
}

fn default_schema_version() -> u16 {
    REPORT_SCHEMA_VERSION
}

/// Build recommendations from structural violations, semantic and intent suggestions,
/// with real category and source_layer. Deduplicates by description and caps at `limit`.
pub fn build_recommendations(
    structural_report: &DriftReport,
    semantic_recommendations: &[String],
    intent_suggestions: &[String],
    limit: usize,
) -> Vec<Recommendation> {
    use std::collections::HashSet;

    fn violation_kind_to_category(kind: ViolationKind) -> RecommendationCategory {
        match kind {
            ViolationKind::CircularDependency => RecommendationCategory::Cycle,
            ViolationKind::OrphanComponent => RecommendationCategory::Orphan,
            ViolationKind::LayerViolation => RecommendationCategory::LayerViolation,
            ViolationKind::GodModule => RecommendationCategory::GodModule,
            ViolationKind::MissingDependency
            | ViolationKind::UndocumentedComponent
            | ViolationKind::PatternMismatch => RecommendationCategory::Other,
        }
    }

    fn severity_to_priority(s: Severity) -> Priority {
        match s {
            Severity::Error => Priority::Critical,
            Severity::Warning => Priority::Medium,
            Severity::Info => Priority::Low,
        }
    }

    let mut out: Vec<Recommendation> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for v in &structural_report.violations {
        let desc = v.message.clone();
        if seen.insert(desc.clone()) {
            out.push(Recommendation {
                priority: severity_to_priority(v.severity),
                category: violation_kind_to_category(v.kind),
                description: desc,
                affected_components: v
                    .location
                    .as_ref()
                    .map(|l| vec![l.clone()])
                    .unwrap_or_default(),
                source_layer: Layer::Structural,
                estimated_effort: Effort::Medium,
            });
        }
    }
    for s in semantic_recommendations {
        if seen.insert(s.clone()) {
            out.push(Recommendation {
                priority: Priority::Medium,
                category: RecommendationCategory::SemanticCoupling,
                description: s.clone(),
                affected_components: vec![],
                source_layer: Layer::Semantic,
                estimated_effort: Effort::Medium,
            });
        }
    }
    for s in intent_suggestions {
        if seen.insert(s.clone()) {
            out.push(Recommendation {
                priority: Priority::Medium,
                category: RecommendationCategory::IntentDrift,
                description: s.clone(),
                affected_components: vec![],
                source_layer: Layer::Intent,
                estimated_effort: Effort::Medium,
            });
        }
    }
    for s in &structural_report.suggestions {
        if seen.insert(s.clone()) {
            out.push(Recommendation {
                priority: Priority::Medium,
                category: RecommendationCategory::Other,
                description: s.clone(),
                affected_components: vec![],
                source_layer: Layer::Structural,
                estimated_effort: Effort::Medium,
            });
        }
    }

    fn priority_order(p: Priority) -> u8 {
        match p {
            Priority::Critical => 0,
            Priority::High => 1,
            Priority::Medium => 2,
            Priority::Low => 3,
        }
    }
    out.sort_by(|a, b| priority_order(a.priority).cmp(&priority_order(b.priority)));
    out.truncate(limit);
    out
}
