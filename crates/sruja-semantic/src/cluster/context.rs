//! Bounded context detection from domain clusters.

use super::DomainCluster;
use crate::vocabulary::VocabularyGraph;

/// A bounded context with vocabulary and boundaries.
#[derive(Debug, Clone)]
pub struct BoundedContext {
    pub name: String,
    pub components: Vec<String>,
    pub vocabulary: Vec<String>,
    pub shared_vocabulary: Vec<SharedTerm>,
}

/// A term shared across contexts (potential leakage).
#[derive(Debug, Clone)]
pub struct SharedTerm {
    pub term: String,
    pub contexts: Vec<String>,
    pub is_intentional: bool,
    pub leakage_confidence: f32,
}

/// Detects bounded contexts from domain clusters.
#[derive(Debug, Clone)]
pub struct BoundedContextDetector {
    min_size: usize,
    coupling_threshold: f32,
}

impl Default for BoundedContextDetector {
    fn default() -> Self {
        Self {
            min_size: 2,
            coupling_threshold: 0.7,
        }
    }
}

impl BoundedContextDetector {
    pub fn new(min_size: usize, coupling_threshold: f32) -> Self {
        Self {
            min_size,
            coupling_threshold,
        }
    }

    /// Build bounded contexts from clusters and vocabulary.
    pub fn detect(
        &self,
        clusters: &[DomainCluster],
        vocabulary: &VocabularyGraph,
    ) -> Vec<BoundedContext> {
        clusters
            .iter()
            .filter(|c| c.components.len() >= self.min_size)
            .map(|c| {
                let vocab: Vec<String> = c.centroid_terms.clone();
                let shared = self.find_shared_terms(&c.components, clusters, vocabulary);
                BoundedContext {
                    name: c.suggested_context.clone(),
                    components: c.components.clone(),
                    vocabulary: vocab,
                    shared_vocabulary: shared,
                }
            })
            .collect()
    }

    fn find_shared_terms(
        &self,
        components: &[String],
        all_clusters: &[DomainCluster],
        vocabulary: &VocabularyGraph,
    ) -> Vec<SharedTerm> {
        let mut result = Vec::new();
        let context_components: std::collections::HashSet<_> =
            components.iter().map(|s| s.as_str()).collect();

        for comp in components {
            for term in vocabulary.terms_for_component(comp) {
                let users: Vec<&str> = vocabulary
                    .components_using_term(term)
                    .iter()
                    .map(|s| s.as_str())
                    .collect();
                let in_context = users.iter().filter(|u| context_components.contains(*u)).count();
                let out_context = users.len() - in_context;

                if out_context > 0 {
                    let leakage = out_context as f32 / users.len() as f32;
                    if leakage < self.coupling_threshold {
                        continue; // below threshold, likely intentional shared term
                    }
                    let other_contexts: Vec<String> = all_clusters
                        .iter()
                        .filter(|c| c.components.iter().any(|id| users.contains(&id.as_str())))
                        .map(|c| c.suggested_context.clone())
                        .collect();
                    result.push(SharedTerm {
                        term: term.clone(),
                        contexts: other_contexts,
                        is_intentional: leakage < 0.3,
                        leakage_confidence: leakage,
                    });
                }
            }
        }
        result.sort_by(|a, b| b.leakage_confidence.partial_cmp(&a.leakage_confidence).unwrap());
        result.dedup_by(|a, b| a.term == b.term);
        result
    }
}
