//! Adapter that wraps [rig-core](https://docs.rs/rig-core) embedding models.
//!
//! Enables using rig's providers (OpenAI, Ollama, Cohere, etc.) with sruja-semantic.

use super::{EmbeddingError, EmbeddingProvider, EmbeddingVector};
use async_trait::async_trait;
use rig::embeddings::EmbeddingModel;

fn f64_to_f32(v: Vec<f64>) -> EmbeddingVector {
    v.into_iter().map(|x| x as f32).collect()
}

fn map_rig_error(e: rig::embeddings::EmbeddingError) -> EmbeddingError {
    use rig::embeddings::EmbeddingError as RigError;
    match &e {
        RigError::ProviderError(msg) => {
            if msg.to_lowercase().contains("rate") || msg.to_lowercase().contains("429") {
                EmbeddingError::RateLimited
            } else {
                EmbeddingError::Provider(msg.clone())
            }
        }
        RigError::DocumentError(_) | RigError::ResponseError(_) => {
            EmbeddingError::InvalidInput(e.to_string())
        }
        _ => EmbeddingError::Provider(e.to_string()),
    }
}

/// Wraps a rig [EmbeddingModel] to implement our [EmbeddingProvider] trait.
///
/// Use with rig's providers (OpenAI, Ollama, Cohere, etc.). Example with OpenAI:
///
/// ```ignore
/// use sruja_semantic::embedding::{RigEmbeddingAdapter, EmbeddingProvider};
/// use rig::client::ProviderClient;
/// use rig::providers::openai;
///
/// let client = openai::Client::from_env();  // uses OPENAI_API_KEY
/// let model = client.embedding_model(openai::TEXT_EMBEDDING_3_SMALL);
/// let provider = RigEmbeddingAdapter::new(model);
/// let vec = provider.embed("hello world").await?;
/// ```
#[derive(Clone)]
pub struct RigEmbeddingAdapter<M: EmbeddingModel + Clone + Send + Sync> {
    model: M,
}

impl<M: EmbeddingModel + Clone + Send + Sync> RigEmbeddingAdapter<M> {
    pub fn new(model: M) -> Self {
        Self { model }
    }
}

#[async_trait]
impl<M> EmbeddingProvider for RigEmbeddingAdapter<M>
where
    M: EmbeddingModel + Clone + Send + Sync,
    M::Client: Send,
{
    async fn embed(&self, text: &str) -> Result<EmbeddingVector, EmbeddingError> {
        if text.is_empty() {
            return Err(EmbeddingError::InvalidInput("empty text".to_string()));
        }
        let rig_emb = self.model.embed_text(text).await.map_err(map_rig_error)?;
        Ok(f64_to_f32(rig_emb.vec))
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<EmbeddingVector>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }
        let docs: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
        let rig_embs = self.model.embed_texts(docs).await.map_err(map_rig_error)?;
        Ok(rig_embs.into_iter().map(|e| f64_to_f32(e.vec)).collect())
    }

    fn dimension(&self) -> usize {
        self.model.ndims()
    }

    fn provider_name(&self) -> &str {
        "rig"
    }
}
