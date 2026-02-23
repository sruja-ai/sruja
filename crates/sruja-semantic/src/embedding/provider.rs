//! Embedding provider trait and error types.

use async_trait::async_trait;
use thiserror::Error;

/// Dense vector representation of text (e.g. 384 or 1536 dimensions).
pub type EmbeddingVector = Vec<f32>;

/// Errors from embedding providers.
#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Trait for embedding providers so implementations can be swapped (OpenAI, local, custom).
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate an embedding for a single text.
    async fn embed(&self, text: &str) -> Result<EmbeddingVector, EmbeddingError>;

    /// Generate embeddings for multiple texts (batched). Default implementation
    /// calls `embed` in sequence; providers may override for batching.
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<EmbeddingVector>, EmbeddingError> {
        let mut out = Vec::with_capacity(texts.len());
        for t in texts {
            out.push(self.embed(t).await?);
        }
        Ok(out)
    }

    /// Dimensionality of the vectors (e.g. 384 for all-MiniLM-L6-v2, 1536 for text-embedding-3-small).
    fn dimension(&self) -> usize;

    /// Provider name for logging and debugging.
    fn provider_name(&self) -> &str;
}
