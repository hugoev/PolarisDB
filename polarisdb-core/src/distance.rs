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
    let mut sum = 0.0;
    let mut chunks_a = a.chunks_exact(16);
    let mut chunks_b = b.chunks_exact(16);

    for (ca, cb) in chunks_a.by_ref().zip(chunks_b.by_ref()) {
        let d0 = ca[0] - cb[0];
        let d1 = ca[1] - cb[1];
        let d2 = ca[2] - cb[2];
        let d3 = ca[3] - cb[3];
        let d4 = ca[4] - cb[4];
        let d5 = ca[5] - cb[5];
        let d6 = ca[6] - cb[6];
        let d7 = ca[7] - cb[7];
        let d8 = ca[8] - cb[8];
        let d9 = ca[9] - cb[9];
        let d10 = ca[10] - cb[10];
        let d11 = ca[11] - cb[11];
        let d12 = ca[12] - cb[12];
        let d13 = ca[13] - cb[13];
        let d14 = ca[14] - cb[14];
        let d15 = ca[15] - cb[15];

        sum += d0 * d0
            + d1 * d1
            + d2 * d2
            + d3 * d3
            + d4 * d4
            + d5 * d5
            + d6 * d6
            + d7 * d7
            + d8 * d8
            + d9 * d9
            + d10 * d10
            + d11 * d11
            + d12 * d12
            + d13 * d13
            + d14 * d14
            + d15 * d15;
    }

    for (x, y) in chunks_a.remainder().iter().zip(chunks_b.remainder()) {
        let diff = x - y;
        sum += diff * diff;
    }
    sum
}

/// Computes cosine distance between two vectors.
///
/// Formula: 1 - (a · b) / (||a|| * ||b||)
/// Range: [0, 2] where 0 = identical direction, 2 = opposite direction
#[inline]
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let dot = dot_product(a, b);
    let norm_a = dot_product(a, a).sqrt();
    let norm_b = dot_product(b, b).sqrt();

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
    let mut sum = 0.0;
    let mut chunks_a = a.chunks_exact(16);
    let mut chunks_b = b.chunks_exact(16);

    for (ca, cb) in chunks_a.by_ref().zip(chunks_b.by_ref()) {
        sum += ca[0] * cb[0]
            + ca[1] * cb[1]
            + ca[2] * cb[2]
            + ca[3] * cb[3]
            + ca[4] * cb[4]
            + ca[5] * cb[5]
            + ca[6] * cb[6]
            + ca[7] * cb[7]
            + ca[8] * cb[8]
            + ca[9] * cb[9]
            + ca[10] * cb[10]
            + ca[11] * cb[11]
            + ca[12] * cb[12]
            + ca[13] * cb[13]
            + ca[14] * cb[14]
            + ca[15] * cb[15];
    }

    for (x, y) in chunks_a.remainder().iter().zip(chunks_b.remainder()) {
        sum += x * y;
    }
    sum
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
