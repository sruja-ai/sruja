//! Semantic importance scoring using embeddings.
//!
//! Scores components by similarity to important architectural concepts.

use sruja_scan::Node;
use sruja_semantic::embedding::EmbeddingProvider;

pub const ARCHITECTURAL_CONCEPTS: &[&str] = &[
    "entry point main handler controller router",
    "api endpoint rest graphql http",
    "database storage persistence repository",
    "service business logic domain core",
    "configuration settings environment",
    "authentication authorization security",
    "logging monitoring observability",
    "external integration api client",
];

pub async fn compute_semantic_importance(node: &Node, provider: &dyn EmbeddingProvider) -> f64 {
    let node_text = build_node_text(node);

    if node_text.is_empty() {
        return 0.0;
    }

    let node_embedding = match provider.embed(&node_text).await {
        Ok(e) => e,
        Err(_) => return 0.0,
    };

    let concept_embeddings = match provider.embed_batch(ARCHITECTURAL_CONCEPTS).await {
        Ok(e) => e,
        Err(_) => return 0.0,
    };

    let max_similarity = concept_embeddings
        .iter()
        .map(|ce| sruja_semantic::cosine_similarity(&node_embedding, ce) as f64)
        .fold(0.0_f64, |a, b| a.max(b));

    max_similarity
}

pub async fn compute_batch_semantic_importance(
    nodes: &[Node],
    provider: &dyn EmbeddingProvider,
) -> Vec<(String, f64)> {
    let texts: Vec<String> = nodes.iter().map(build_node_text).collect();
    let ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();

    let node_embeddings = match provider
        .embed_batch(&texts.iter().map(|s| s.as_str()).collect::<Vec<_>>())
        .await
    {
        Ok(e) => e,
        Err(_) => return nodes.iter().map(|n| (n.id.clone(), 0.0)).collect(),
    };

    let concept_embeddings = match provider.embed_batch(ARCHITECTURAL_CONCEPTS).await {
        Ok(e) => e,
        Err(_) => return nodes.iter().map(|n| (n.id.clone(), 0.0)).collect(),
    };

    ids.into_iter()
        .zip(node_embeddings.into_iter())
        .map(|(id, emb)| {
            let max_sim = concept_embeddings
                .iter()
                .map(|ce| sruja_semantic::cosine_similarity(&emb, ce) as f64)
                .fold(0.0_f64, |a, b| a.max(b));
            (id, max_sim)
        })
        .collect()
}

fn build_node_text(node: &Node) -> String {
    let mut parts = Vec::new();

    if let Some(ref path) = node.path {
        parts.push(path.clone());
    }

    parts.push(node.label.clone());

    if let Some(ref tech) = node.technology {
        parts.push(tech.clone());
    }

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_scan::NodeKind;
    use sruja_semantic::embedding::StubEmbeddingProvider;

    #[tokio::test]
    async fn test_semantic_importance() {
        let node = Node {
            id: "test".to_string(),
            label: "UserService".to_string(),
            path: Some("src/services/user_service.rs".to_string()),
            technology: Some("Rust".to_string()),
            kind: NodeKind::Module,
            metadata: Default::default(),
        };

        let provider = StubEmbeddingProvider::new();
        let score = compute_semantic_importance(&node, &provider).await;

        assert!(score >= 0.0);
        assert!(score <= 1.0);
    }
}
