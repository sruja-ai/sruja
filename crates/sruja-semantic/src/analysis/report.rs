//! Semantic analysis report types.

use super::coupling::SemanticCouplingReport;
use crate::cluster::{BoundedContext, DomainCluster};
use crate::vocabulary::VocabularyGraph;

/// Full semantic analysis report.
#[derive(Debug, Clone)]
pub struct SemanticReport {
    pub contexts: Vec<BoundedContext>,
    pub clusters: Vec<DomainCluster>,
    pub coupling: SemanticCouplingReport,
    pub vocabulary_leaks: Vec<VocabularyLeak>,
    pub summary: SemanticSummary,
}

/// A term that leaks across bounded contexts.
#[derive(Debug, Clone)]
pub struct VocabularyLeak {
    pub term: String,
    pub contexts: Vec<String>,
    pub leakage_confidence: f32,
    pub is_intentional: bool,
}

/// High-level summary for CLI output.
#[derive(Debug, Clone)]
pub struct SemanticSummary {
    pub component_count: usize,
    pub context_count: usize,
    pub hidden_coupling_count: usize,
    pub vocabulary_leak_count: usize,
    pub health_score: u8,
}

impl SemanticReport {
    pub fn from_parts(
        contexts: Vec<BoundedContext>,
        clusters: Vec<DomainCluster>,
        coupling: SemanticCouplingReport,
        vocabulary: &VocabularyGraph,
    ) -> Self {
        let mut vocabulary_leaks = Vec::new();
        for ctx in &contexts {
            for st in &ctx.shared_vocabulary {
                vocabulary_leaks.push(VocabularyLeak {
                    term: st.term.clone(),
                    contexts: st.contexts.clone(),
                    leakage_confidence: st.leakage_confidence,
                    is_intentional: st.is_intentional,
                });
            }
        }
        vocabulary_leaks.sort_by(|a, b| {
            b.leakage_confidence
                .partial_cmp(&a.leakage_confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let component_count = vocabulary.component_count();
        let context_count = contexts.len();
        let hidden_coupling_count = coupling.hidden_couplings.len();
        let vocabulary_leak_count = vocabulary_leaks.len();

        let health_score = compute_health_score(
            component_count,
            hidden_coupling_count,
            vocabulary_leak_count,
        );

        Self {
            contexts,
            clusters,
            coupling,
            vocabulary_leaks,
            summary: SemanticSummary {
                component_count,
                context_count,
                hidden_coupling_count,
                vocabulary_leak_count,
                health_score,
            },
        }
    }
}

fn compute_health_score(
    component_count: usize,
    hidden_coupling_count: usize,
    vocabulary_leak_count: usize,
) -> u8 {
    if component_count == 0 {
        return 100;
    }
    let penalty_hidden = (hidden_coupling_count as f32 * 3.0).min(30.0);
    let penalty_leak = (vocabulary_leak_count as f32 * 2.0).min(20.0);
    let score = 100.0 - penalty_hidden - penalty_leak;
    score.clamp(0.0, 100.0) as u8
}
