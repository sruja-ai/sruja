//! Semantic coupling detection between components.

use crate::similarity::cosine_similarity;
use crate::vocabulary::VocabularyGraph;
use crate::EmbeddingVector;
use std::collections::HashSet;

/// A semantic coupling between two components.
#[derive(Debug, Clone)]
pub struct SemanticCoupling {
    pub source: String,
    pub target: String,
    pub similarity: f32,
    pub shared_concepts: Vec<String>,
    pub has_structural_coupling: bool,
    pub coupling_type: SemanticCouplingType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticCouplingType {
    SharedVocabulary,
    BusinessLogic,
    DataModel,
    ErrorHandling,
    Configuration,
}

/// A component with high semantic fan-out (coupled to many others).
#[derive(Debug, Clone)]
pub struct SemanticHub {
    pub component: String,
    pub coupling_count: usize,
    pub avg_similarity: f32,
}

/// Report from semantic coupling analysis.
#[derive(Debug, Clone)]
pub struct SemanticCouplingReport {
    pub couplings: Vec<SemanticCoupling>,
    pub hidden_couplings: Vec<SemanticCoupling>,
    pub semantic_hubs: Vec<SemanticHub>,
    pub recommendations: Vec<String>,
}

/// Analyzes semantic coupling between components.
#[derive(Debug, Clone)]
pub struct SemanticCouplingAnalyzer {
    similarity_threshold: f32,
}

impl Default for SemanticCouplingAnalyzer {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.7,
        }
    }
}

impl SemanticCouplingAnalyzer {
    pub fn new(similarity_threshold: f32) -> Self {
        Self {
            similarity_threshold,
        }
    }

    /// Analyze semantic coupling between components.
    ///
    /// `component_ids` and `embeddings` must have matching order.
    /// `structural_edges` is a set of (source, target) pairs for structural relationships.
    pub fn analyze(
        &self,
        component_ids: &[String],
        embeddings: &[EmbeddingVector],
        vocabulary: &VocabularyGraph,
        structural_edges: &[(String, String)],
    ) -> SemanticCouplingReport {
        if component_ids.len() != embeddings.len() || component_ids.is_empty() {
            return SemanticCouplingReport {
                couplings: vec![],
                hidden_couplings: vec![],
                semantic_hubs: vec![],
                recommendations: vec![],
            };
        }

        let structural: HashSet<(String, String)> = structural_edges
            .iter()
            .map(|(a, b)| (a.clone(), b.clone()))
            .chain(structural_edges.iter().map(|(a, b)| (b.clone(), a.clone())))
            .collect();

        let mut couplings = Vec::new();
        let mut hidden_couplings = Vec::new();
        let mut hub_counts: std::collections::HashMap<String, (usize, f32)> =
            std::collections::HashMap::new();

        for i in 0..component_ids.len() {
            for j in (i + 1)..component_ids.len() {
                let sim = cosine_similarity(&embeddings[i], &embeddings[j]);
                if sim < self.similarity_threshold {
                    continue;
                }
                let src = &component_ids[i];
                let tgt = &component_ids[j];
                let shared = vocabulary.shared_terms(src, tgt);
                let has_structural = structural.contains(&(src.clone(), tgt.clone()))
                    || structural.contains(&(tgt.clone(), src.clone()));
                let coupling_type = infer_coupling_type(&shared);
                let c = SemanticCoupling {
                    source: src.clone(),
                    target: tgt.clone(),
                    similarity: sim,
                    shared_concepts: shared,
                    has_structural_coupling: has_structural,
                    coupling_type,
                };
                couplings.push(c.clone());
                if !has_structural {
                    hidden_couplings.push(c);
                }
                {
                    let e = hub_counts.entry(src.clone()).or_insert((0, 0.0));
                    e.0 += 1;
                    e.1 += sim;
                }
                {
                    let e = hub_counts.entry(tgt.clone()).or_insert((0, 0.0));
                    e.0 += 1;
                    e.1 += sim;
                }
            }
        }

        let semantic_hubs: Vec<SemanticHub> = hub_counts
            .into_iter()
            .filter(|(_, (cnt, _))| *cnt >= 3)
            .map(|(comp, (cnt, sum))| SemanticHub {
                component: comp,
                coupling_count: cnt,
                avg_similarity: if cnt > 0 { sum / cnt as f32 } else { 0.0 },
            })
            .collect();

        let mut recommendations = Vec::new();
        if !hidden_couplings.is_empty() {
            recommendations.push(format!(
                "{} hidden coupling(s) detected (high semantic similarity without structural dependency). Consider explicit interfaces or shared contracts.",
                hidden_couplings.len()
            ));
        }
        for hub in &semantic_hubs {
            if hub.coupling_count >= 5 {
                recommendations.push(format!(
                    "Component '{}' is a semantic hub (coupled to {} others). Consider extracting shared domain concepts.",
                    hub.component, hub.coupling_count
                ));
            }
        }

        SemanticCouplingReport {
            couplings,
            hidden_couplings,
            semantic_hubs,
            recommendations,
        }
    }
}

fn infer_coupling_type(shared: &[String]) -> SemanticCouplingType {
    let terms: HashSet<&str> = shared.iter().map(|s| s.as_str()).collect();
    if terms.iter().any(|t| {
        ["error", "err", "exception", "fail", "panic"]
            .iter()
            .any(|x| t.contains(x))
    }) {
        SemanticCouplingType::ErrorHandling
    } else if terms.iter().any(|t| {
        ["config", "env", "setting", "option"]
            .iter()
            .any(|x| t.contains(x))
    }) {
        SemanticCouplingType::Configuration
    } else if terms.iter().any(|t| {
        ["model", "entity", "schema", "record"]
            .iter()
            .any(|x| t.contains(x))
    }) {
        SemanticCouplingType::DataModel
    } else if !shared.is_empty() {
        SemanticCouplingType::SharedVocabulary
    } else {
        SemanticCouplingType::BusinessLogic
    }
}
