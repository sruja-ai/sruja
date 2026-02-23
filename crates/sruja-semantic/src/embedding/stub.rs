//! Stub embedding provider for deterministic tests and CI.
//!
//! Produces deterministic, L2-normalized vectors from text without any API.
//! Use for testing and when no embedding API is configured.

use super::{EmbeddingError, EmbeddingProvider, EmbeddingVector};
use async_trait::async_trait;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Dimension of stub embeddings (matches all-MiniLM-L6-v2 for consistency).
pub const STUB_DIMENSION: usize = 384;

/// Deterministic embedding provider for tests and zero-config runs.
///
/// Uses text hash to generate reproducible vectors. Similar texts produce
/// similar vectors only if they share substrings; this is not a real
/// semantic model. Use for CI and when no API key is available.
#[derive(Debug, Clone)]
pub struct StubEmbeddingProvider {
    dimension: usize,
}

impl Default for StubEmbeddingProvider {
    fn default() -> Self {
        Self {
            dimension: STUB_DIMENSION,
        }
    }
}

impl StubEmbeddingProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_dimension(dimension: usize) -> Self {
        Self { dimension }
    }

    fn hash_text(text: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }

    fn generate_vector(text: &str, dimension: usize) -> EmbeddingVector {
        let seed = Self::hash_text(text);
        let mut v: Vec<f32> = (0..dimension)
            .map(|i| {
                let x = (seed as f64) * (i as f64 + 1.0) * 0.6180339887;
                (x.sin() * 1000.0) as f32
            })
            .collect();
        normalize_l2(&mut v);
        v
    }
}

#[async_trait]
impl EmbeddingProvider for StubEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<EmbeddingVector, EmbeddingError> {
        if text.is_empty() {
            return Err(EmbeddingError::InvalidInput("empty text".to_string()));
        }
        Ok(Self::generate_vector(text, self.dimension))
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn provider_name(&self) -> &str {
        "stub"
    }
}

fn normalize_l2(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stub_same_text_same_vector() {
        let p = StubEmbeddingProvider::new();
        let a = p.embed("hello").await.unwrap();
        let b = p.embed("hello").await.unwrap();
        assert_eq!(a.len(), STUB_DIMENSION);
        assert_eq!(a, b);
    }

    #[tokio::test]
    async fn stub_different_text_different_vector() {
        let p = StubEmbeddingProvider::new();
        let a = p.embed("hello").await.unwrap();
        let b = p.embed("world").await.unwrap();
        assert_ne!(a, b);
    }

    #[tokio::test]
    async fn stub_normalized() {
        let p = StubEmbeddingProvider::new();
        let v = p.embed("test").await.unwrap();
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    }

    #[tokio::test]
    async fn stub_empty_text_errors() {
        let p = StubEmbeddingProvider::new();
        assert!(p.embed("").await.is_err());
    }
}
