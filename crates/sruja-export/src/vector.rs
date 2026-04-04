#![cfg(not(target_arch = "wasm32"))]

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorNode {
    pub id: String,
    pub label: String,
    pub description: String,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorIndex {
    pub nodes: Vec<VectorNode>,
}

pub struct SemanticSearcher {
    model: TextEmbedding,
}

impl SemanticSearcher {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::BGESmallENV15).with_show_download_progress(true),
        )?;
        Ok(Self { model })
    }

    pub fn generate_embeddings(
        &mut self,
        inputs: Vec<String>,
    ) -> Result<Vec<Vec<f32>>, Box<dyn Error>> {
        let embeddings = self.model.embed(inputs, None)?;
        Ok(embeddings)
    }

    pub fn index_nodes(
        &mut self,
        nodes: Vec<(String, String, String)>,
    ) -> Result<VectorIndex, Box<dyn Error>> {
        let descriptions: Vec<String> = nodes.iter().map(|(_, _, desc)| desc.clone()).collect();
        let embeddings = self.generate_embeddings(descriptions)?;

        let mut vector_nodes = Vec::new();
        for (i, (id, label, desc)) in nodes.into_iter().enumerate() {
            vector_nodes.push(VectorNode {
                id,
                label,
                description: desc,
                embedding: embeddings[i].clone(),
            });
        }

        Ok(VectorIndex {
            nodes: vector_nodes,
        })
    }

    pub fn search(
        &mut self,
        index: &VectorIndex,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<(String, f32)>, Box<dyn Error>> {
        let query_embedding = self.generate_embeddings(vec![query.to_string()])?[0].clone();

        let mut results = Vec::new();
        for node in &index.nodes {
            let score = cosine_similarity(&query_embedding, &node.embedding);
            results.push((node.id.clone(), score));
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);

        Ok(results)
    }
}

fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
    let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
    let norm1: f32 = v1.iter().map(|a| a * a).sum::<f32>().sqrt();
    let norm2: f32 = v2.iter().map(|a| a * a).sum::<f32>().sqrt();
    if norm1 * norm2 == 0.0 {
        return 0.0;
    }
    dot_product / (norm1 * norm2)
}
