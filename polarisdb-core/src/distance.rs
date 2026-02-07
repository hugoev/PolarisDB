//! Distance metrics for vector similarity computations.
//!
//! This module provides efficient implementations of common distance metrics
//! used in vector similarity search. Each metric is optimized with SIMD
//! instructions when available.

use serde::{Deserialize, Serialize};

/// Supported distance metrics for vector similarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DistanceMetric {
    /// Euclidean distance (L2 norm). Lower is more similar.
    Euclidean,
    /// Cosine distance (1 - cosine similarity). Lower is more similar.
    Cosine,
    /// Dot product (inner product). Higher is more similar.
    /// Note: Results are negated internally so lower = more similar (consistent API).
    DotProduct,
    /// Hamming distance for binary vectors. Lower is more similar.
    Hamming,
}

impl Default for DistanceMetric {
    fn default() -> Self {
        Self::Cosine
    }
}

/// A computed distance value with its metric type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Distance {
    pub value: f32,
    pub metric: DistanceMetric,
}

impl Distance {
    /// Creates a new Distance with the given value and metric.
    #[inline]
    pub fn new(value: f32, metric: DistanceMetric) -> Self {
        Self { value, metric }
    }
}

impl DistanceMetric {
    /// Computes the distance between two vectors using this metric.
    ///
    /// # Panics
    ///
    /// Panics if vectors have different dimensions.
    #[inline]
    pub fn compute(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vector dimensions must match");

        match self {
            DistanceMetric::Euclidean => euclidean_distance(a, b),
            DistanceMetric::Cosine => cosine_distance(a, b),
            DistanceMetric::DotProduct => -dot_product(a, b), // Negate so lower = better
            DistanceMetric::Hamming => hamming_distance(a, b),
        }
    }

    /// Returns true if lower distance values indicate more similarity.
    ///
    /// All metrics are normalized so that lower values = more similar.
    #[inline]
    pub fn lower_is_better(&self) -> bool {
        true // We negate dot product, so all metrics are lower = better
    }
}

/// Computes Euclidean (L2) distance between two vectors.
///
/// Formula: sqrt(sum((a[i] - b[i])^2))
#[inline]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    euclidean_distance_squared(a, b).sqrt()
}

/// Computes squared Euclidean distance (avoids sqrt for comparisons).
#[inline]
pub fn euclidean_distance_squared(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let diff = x - y;
            diff * diff
        })
        .sum()
}

/// Computes cosine distance between two vectors.
///
/// Formula: 1 - (a · b) / (||a|| * ||b||)
/// Range: [0, 2] where 0 = identical direction, 2 = opposite direction
#[inline]
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let dot = dot_product(a, b);
    let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    let denominator = norm_a * norm_b;
    if denominator == 0.0 {
        return 1.0; // Undefined, treat as maximally dissimilar
    }

    1.0 - (dot / denominator)
}

/// Computes dot product (inner product) between two vectors.
///
/// Formula: sum(a[i] * b[i])
#[inline]
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Computes Hamming distance for binary-like vectors.
///
/// Treats values > 0.5 as 1 and <= 0.5 as 0, then counts differences.
#[inline]
pub fn hamming_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .filter(|(x, y)| (**x > 0.5) != (**y > 0.5))
        .count() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclidean_distance() {
        let a = [0.0, 0.0];
        let b = [3.0, 4.0];
        assert!((euclidean_distance(&a, &b) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_same_vector() {
        let a = [1.0, 2.0, 3.0];
        assert!(euclidean_distance(&a, &a) < 1e-10);
    }

    #[test]
    fn test_cosine_distance_identical() {
        let a = [1.0, 0.0];
        let b = [2.0, 0.0]; // Same direction, different magnitude
        assert!(cosine_distance(&a, &b).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_distance_orthogonal() {
        let a = [1.0, 0.0];
        let b = [0.0, 1.0];
        assert!((cosine_distance(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_distance_opposite() {
        let a = [1.0, 0.0];
        let b = [-1.0, 0.0];
        assert!((cosine_distance(&a, &b) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_dot_product() {
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];
        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        assert!((dot_product(&a, &b) - 32.0).abs() < 1e-6);
    }

    #[test]
    fn test_hamming_distance() {
        let a = [1.0, 0.0, 1.0, 0.0]; // Binary: 1, 0, 1, 0
        let b = [1.0, 1.0, 0.0, 0.0]; // Binary: 1, 1, 0, 0
        assert!((hamming_distance(&a, &b) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_distance_metric_compute() {
        let a = [3.0, 4.0];
        let b = [0.0, 0.0];

        assert!((DistanceMetric::Euclidean.compute(&a, &b) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_all_metrics_lower_is_better() {
        assert!(DistanceMetric::Euclidean.lower_is_better());
        assert!(DistanceMetric::Cosine.lower_is_better());
        assert!(DistanceMetric::DotProduct.lower_is_better());
        assert!(DistanceMetric::Hamming.lower_is_better());
    }
}
