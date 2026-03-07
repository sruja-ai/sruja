//! Cosine similarity for embedding vectors.

use crate::EmbeddingVector;

/// Cosine similarity between two unit vectors (range -1..1; 1 = identical).
///
/// Assumes inputs are L2-normalized. For non-normalized vectors, behavior
/// is still correct but magnitude affects the result.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na <= 0.0 || nb <= 0.0 {
        return 0.0;
    }
    (dot / (na * nb)).clamp(-1.0, 1.0)
}

/// Pairwise cosine similarities for a list of vectors.
/// Returns upper-triangle as flat vec: (0,1), (0,2), (1,2), ...
pub fn pairwise_cosine(vectors: &[EmbeddingVector]) -> Vec<f32> {
    let mut out = Vec::new();
    for i in 0..vectors.len() {
        for j in (i + 1)..vectors.len() {
            out.push(cosine_similarity(&vectors[i], &vectors[j]));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_identical() {
        let v = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &b).abs() < 1e-5);
    }

    #[test]
    fn cosine_opposite() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) + 1.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_mismatched_len() {
        assert_eq!(cosine_similarity(&[1.0], &[1.0, 0.0]), 0.0);
    }
}
