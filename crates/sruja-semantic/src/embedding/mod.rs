//! Embedding generation for semantic analysis.
//!
//! Provider-agnostic trait allows OpenAI, local models, or custom backends.
//! Uses [rig-core](https://docs.rs/rig-core) when the `rig` feature is enabled.

mod provider;
mod stub;

#[cfg(feature = "rig")]
mod rig_adapter;

pub use provider::{EmbeddingError, EmbeddingProvider, EmbeddingVector};
pub use stub::{StubEmbeddingProvider, STUB_DIMENSION};

#[cfg(feature = "rig")]
pub use rig_adapter::RigEmbeddingAdapter;
