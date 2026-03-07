//! Domain concept clustering from embeddings.

use super::dbscan;
use crate::similarity::cosine_similarity;
use crate::vocabulary::VocabularyGraph;
use crate::EmbeddingVector;
use std::collections::HashMap;

/// A cluster of semantically similar components.
#[derive(Debug, Clone)]
pub struct DomainCluster {
    pub id: String,
    pub centroid_terms: Vec<String>,
    pub components: Vec<String>,
    pub coherence: f32,
    pub suggested_context: String,
}

/// Clusters components by embedding similarity.
#[derive(Debug, Clone)]
pub struct DomainClusterer {
    min_cluster_size: usize,
    epsilon: f32,
}

impl Default for DomainClusterer {
    fn default() -> Self {
        Self {
            min_cluster_size: 2,
            epsilon: 0.7,
        }
    }
}

impl DomainClusterer {
    pub fn new(min_cluster_size: usize, epsilon: f32) -> Self {
        Self {
            min_cluster_size,
            epsilon,
        }
    }

    /// Cluster components by their embeddings.
    ///
    /// `component_order` must match the order of `embeddings`.
    pub fn cluster(
        &self,
        component_ids: &[String],
        embeddings: &[EmbeddingVector],
        vocabulary: &VocabularyGraph,
    ) -> Vec<DomainCluster> {
        if component_ids.len() != embeddings.len() || component_ids.is_empty() {
            return vec![];
        }

        let labels = dbscan::dbscan(embeddings, self.epsilon, self.min_cluster_size);

        let mut clusters: HashMap<usize, Vec<usize>> = HashMap::new();
        for (i, label) in labels.into_iter().enumerate() {
            if let Some(cid) = label {
                clusters.entry(cid).or_default().push(i);
            }
        }

        clusters
            .into_iter()
            .filter(|(_, indices)| indices.len() >= self.min_cluster_size)
            .map(|(cid, indices)| {
                let components: Vec<String> =
                    indices.iter().map(|&i| component_ids[i].clone()).collect();
                let centroid_terms = self.centroid_terms(&components, vocabulary);
                let coherence = self.compute_coherence(embeddings, &indices);
                let suggested_context = Self::suggest_context_name(&centroid_terms);
                DomainCluster {
                    id: format!("cluster_{}", cid),
                    centroid_terms,
                    components,
                    coherence,
                    suggested_context,
                }
            })
            .collect()
    }

    fn centroid_terms(
        &self,
        component_ids: &[String],
        vocabulary: &VocabularyGraph,
    ) -> Vec<String> {
        let mut term_counts: HashMap<String, usize> = HashMap::new();
        for id in component_ids {
            for term in vocabulary.terms_for_component(id) {
                *term_counts.entry(term.clone()).or_insert(0) += 1;
            }
        }
        let mut terms: Vec<_> = term_counts.into_iter().collect();
        terms.sort_by(|a, b| b.1.cmp(&a.1));
        terms.into_iter().take(10).map(|(t, _)| t).collect()
    }

    fn compute_coherence(&self, embeddings: &[EmbeddingVector], indices: &[usize]) -> f32 {
        if indices.len() < 2 {
            return 1.0;
        }
        let mut sum = 0.0f32;
        let mut count = 0;
        for i in 0..indices.len() {
            for j in (i + 1)..indices.len() {
                sum += cosine_similarity(&embeddings[indices[i]], &embeddings[indices[j]]);
                count += 1;
            }
        }
        if count == 0 {
            1.0
        } else {
            (sum / count as f32).clamp(0.0, 1.0)
        }
    }

    fn suggest_context_name(terms: &[String]) -> String {
        if terms.is_empty() {
            "UnknownContext".to_string()
        } else {
            let first = &terms[0];
            let mut s = first
                .chars()
                .next()
                .unwrap_or('X')
                .to_uppercase()
                .to_string();
            s.push_str(&first[1..]);
            s.push_str("Context");
            s
        }
    }
}
