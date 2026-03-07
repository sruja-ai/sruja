//! Semantic architecture intelligence for Sruja.
//!
//! Provides embedding-backed analysis: domain clustering, bounded context detection,
//! semantic coupling, and vocabulary leakage. No API key required for structural
//! analysis; embedding providers are optional for semantic features.

pub mod analysis;
pub mod cluster;
pub mod embedding;
pub mod similarity;
pub mod vocabulary;

pub use analysis::{
    SemanticCoupling, SemanticCouplingAnalyzer, SemanticCouplingReport, SemanticReport,
    SemanticSummary, VocabularyLeak,
};
pub use cluster::{BoundedContext, BoundedContextDetector, DomainCluster, DomainClusterer};
pub use embedding::{EmbeddingError, EmbeddingProvider, EmbeddingVector};
pub use similarity::{cosine_similarity, pairwise_cosine};
pub use vocabulary::{ComponentVocabulary, VocabularyExtractor, VocabularyGraph};

/// Configuration for semantic analysis.
#[derive(Debug, Clone)]
pub struct SemanticConfig {
    /// Minimum similarity threshold for coupling detection (0.0–1.0).
    pub coupling_threshold: f32,
    /// Minimum cluster size for bounded context.
    pub min_context_size: usize,
    /// Vocabulary frequency threshold for inclusion.
    pub vocab_min_frequency: usize,
}

impl Default for SemanticConfig {
    fn default() -> Self {
        Self {
            coupling_threshold: 0.7,
            min_context_size: 2,
            vocab_min_frequency: 2,
        }
    }
}

/// Run full semantic analysis on components.
///
/// `components` are (id, text) pairs; `structural_edges` are (source, target) from scan.
/// Uses the provided `EmbeddingProvider`; for zero-config, use `StubEmbeddingProvider`.
pub async fn analyze(
    components: &[(String, String)],
    structural_edges: &[(String, String)],
    provider: &dyn EmbeddingProvider,
    config: Option<SemanticConfig>,
) -> Result<SemanticReport, EmbeddingError> {
    let config = config.unwrap_or_default();
    if components.is_empty() {
        return Ok(SemanticReport::from_parts(
            vec![],
            vec![],
            SemanticCouplingReport {
                couplings: vec![],
                hidden_couplings: vec![],
                semantic_hubs: vec![],
                recommendations: vec![],
            },
            &VocabularyGraph::new(),
        ));
    }

    let extractor = VocabularyExtractor::with_defaults();
    let comp_vocs = extractor.extract_components(
        &components
            .iter()
            .map(|(id, text)| (id.as_str(), text.as_str()))
            .collect::<Vec<_>>(),
    );
    let vocabulary = VocabularyGraph::from_component_vocabularies(&comp_vocs);

    let ids: Vec<String> = components.iter().map(|(id, _)| id.clone()).collect();
    let texts: Vec<&str> = components.iter().map(|(_, t)| t.as_str()).collect();
    let embeddings = provider.embed_batch(&texts).await?;

    let clusterer = DomainClusterer::new(config.min_context_size, config.coupling_threshold);
    let clusters = clusterer.cluster(&ids, &embeddings, &vocabulary);

    let context_detector =
        BoundedContextDetector::new(config.min_context_size, config.coupling_threshold);
    let contexts = context_detector.detect(&clusters, &vocabulary);

    let coupling_analyzer = SemanticCouplingAnalyzer::new(config.coupling_threshold);
    let coupling = coupling_analyzer.analyze(&ids, &embeddings, &vocabulary, structural_edges);

    Ok(SemanticReport::from_parts(
        contexts,
        clusters,
        coupling,
        &vocabulary,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::StubEmbeddingProvider;

    #[tokio::test]
    async fn test_analyze_with_stub_provider() {
        let components: Vec<(String, String)> = vec![
            (
                "order-service".to_string(),
                "order service process orders".to_string(),
            ),
            (
                "payment-service".to_string(),
                "payment service handles payments".to_string(),
            ),
            (
                "inventory".to_string(),
                "inventory stock warehouse".to_string(),
            ),
        ];
        let structural_edges: Vec<(String, String)> = vec![
            ("order-service".to_string(), "payment-service".to_string()),
            ("order-service".to_string(), "inventory".to_string()),
        ];
        let provider = StubEmbeddingProvider::new();

        let result = analyze(&components, &structural_edges, &provider, None).await;

        assert!(result.is_ok(), "analyze should succeed with stub provider");
        let report = result.unwrap();
        assert_eq!(report.summary.component_count, 3);
        assert!(report.summary.health_score <= 100);
        // context_count, hidden_coupling_count, vocabulary_leak_count are usize (non-negative)
    }
}
