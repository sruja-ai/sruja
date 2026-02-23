//! DBSCAN clustering for embedding vectors.

use crate::similarity::cosine_similarity;
use crate::EmbeddingVector;
use std::collections::HashSet;

/// DBSCAN cluster labels: Some(id) = cluster id, None = noise.
pub type ClusterLabels = Vec<Option<usize>>;

/// DBSCAN on embedding vectors using cosine similarity.
///
/// Epsilon is the minimum similarity (0..1) for points to be neighbors.
/// MinPts is the minimum points to form a cluster.
pub fn dbscan(vectors: &[EmbeddingVector], epsilon: f32, min_pts: usize) -> ClusterLabels {
    let n = vectors.len();
    let mut labels: ClusterLabels = vec![None; n];
    let mut cluster_id = 0usize;

    for i in 0..n {
        if labels[i].is_some() {
            continue;
        }
        let neighbors = region_query(vectors, i, epsilon);
        if neighbors.len() < min_pts {
            continue; // noise
        }
        expand_cluster(
            vectors,
            &mut labels,
            i,
            &neighbors,
            cluster_id,
            epsilon,
            min_pts,
        );
        cluster_id += 1;
    }

    labels
}

fn region_query(vectors: &[EmbeddingVector], i: usize, epsilon: f32) -> Vec<usize> {
    let mut out = Vec::new();
    for j in 0..vectors.len() {
        if cosine_similarity(&vectors[i], &vectors[j]) >= epsilon {
            out.push(j);
        }
    }
    out
}

fn expand_cluster(
    vectors: &[EmbeddingVector],
    labels: &mut ClusterLabels,
    seed: usize,
    seed_neighbors: &[usize],
    cluster_id: usize,
    epsilon: f32,
    min_pts: usize,
) {
    let mut stack: Vec<usize> = seed_neighbors.to_vec();
    let mut visited = HashSet::new();
    visited.insert(seed);

    while let Some(i) = stack.pop() {
        if labels[i].is_some() {
            continue;
        }
        labels[i] = Some(cluster_id);
        visited.insert(i);

        let neighbors = region_query(vectors, i, epsilon);
        if neighbors.len() >= min_pts {
            for j in neighbors {
                if !visited.contains(&j) {
                    stack.push(j);
                    visited.insert(j);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vector(values: &[f32]) -> EmbeddingVector {
        values.to_vec()
    }

    #[test]
    fn test_empty_input() {
        let labels = dbscan(&[], 0.5, 2);
        assert!(labels.is_empty());
    }

    #[test]
    fn test_single_point() {
        let vectors = vec![make_vector(&[1.0, 0.0, 0.0])];
        let labels = dbscan(&vectors, 0.5, 2);
        assert_eq!(labels.len(), 1);
        assert!(
            labels[0].is_none(),
            "Single point with min_pts=2 should be noise"
        );
    }

    #[test]
    fn test_two_points_cluster() {
        let vectors = vec![
            make_vector(&[1.0, 0.0, 0.0]),
            make_vector(&[0.99, 0.0, 0.0]),
        ];
        let labels = dbscan(&vectors, 0.9, 2);
        assert_eq!(labels.len(), 2);
        assert!(labels[0].is_some(), "Should form a cluster");
        assert!(labels[1].is_some(), "Should form a cluster");
        assert_eq!(labels[0], labels[1], "Both should be in same cluster");
    }

    #[test]
    fn test_two_distant_points_noise() {
        let vectors = vec![make_vector(&[1.0, 0.0, 0.0]), make_vector(&[0.0, 1.0, 0.0])];
        let labels = dbscan(&vectors, 0.9, 2);
        assert_eq!(labels.len(), 2);
        assert!(labels[0].is_none(), "Distant points should be noise");
        assert!(labels[1].is_none(), "Distant points should be noise");
    }

    #[test]
    fn test_two_clusters() {
        let vectors = vec![
            make_vector(&[1.0, 0.0, 0.0]),
            make_vector(&[0.99, 0.0, 0.0]),
            make_vector(&[0.0, 1.0, 0.0]),
            make_vector(&[0.0, 0.99, 0.0]),
        ];
        let labels = dbscan(&vectors, 0.9, 2);

        let clusters: std::collections::HashSet<_> = labels.iter().filter_map(|l| *l).collect();
        assert_eq!(clusters.len(), 2, "Should have 2 clusters");

        assert_eq!(labels[0], labels[1], "Points 0,1 should be in same cluster");
        assert_eq!(labels[2], labels[3], "Points 2,3 should be in same cluster");
        assert_ne!(
            labels[0], labels[2],
            "Different clusters should have different IDs"
        );
    }

    #[test]
    fn test_all_noise() {
        let vectors = vec![
            make_vector(&[1.0, 0.0, 0.0]),
            make_vector(&[0.0, 1.0, 0.0]),
            make_vector(&[0.0, 0.0, 1.0]),
        ];
        let labels = dbscan(&vectors, 0.99, 2);

        for (i, label) in labels.iter().enumerate() {
            assert!(label.is_none(), "Point {} should be noise", i);
        }
    }
}
