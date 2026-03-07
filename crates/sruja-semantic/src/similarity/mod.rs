//! Similarity metrics for semantic comparison.

mod cosine;

pub use cosine::{cosine_similarity, pairwise_cosine};
