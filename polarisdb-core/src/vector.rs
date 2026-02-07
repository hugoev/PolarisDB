//! Vector types and operations for PolarisDB.

use serde::{Deserialize, Serialize};

/// Unique identifier for a vector in the index.
pub type VectorId = u64;

/// A dense vector of floating-point values.
///
/// This is the primary vector type used throughout PolarisDB.
/// Vectors are stored as `Vec<f32>` for memory efficiency and
/// compatibility with most embedding models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vector {
    data: Vec<f32>,
}

impl Vector {
    /// Creates a new vector from a slice of f32 values.
    ///
    /// # Example
    ///
    /// ```
    /// use polarisdb_core::Vector;
    ///
    /// let v = Vector::new(&[1.0, 2.0, 3.0]);
    /// assert_eq!(v.dimension(), 3);
    /// ```
    #[inline]
    pub fn new(data: &[f32]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    /// Creates a vector from an owned `Vec<f32>`.
    #[inline]
    pub fn from_vec(data: Vec<f32>) -> Self {
        Self { data }
    }

    /// Returns the dimension (length) of the vector.
    #[inline]
    pub fn dimension(&self) -> usize {
        self.data.len()
    }

    /// Returns a slice view of the vector data.
    #[inline]
    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }

    /// Returns a mutable slice view of the vector data.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        &mut self.data
    }

    /// Returns true if the vector has zero elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Computes the L2 (Euclidean) norm of the vector.
    #[inline]
    pub fn norm(&self) -> f32 {
        self.data.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    /// Returns a normalized copy of the vector (unit length).
    ///
    /// Returns None if the vector has zero norm.
    pub fn normalized(&self) -> Option<Self> {
        let norm = self.norm();
        if norm == 0.0 {
            None
        } else {
            Some(Self {
                data: self.data.iter().map(|x| x / norm).collect(),
            })
        }
    }

    /// Consumes the vector and returns the underlying data.
    #[inline]
    pub fn into_inner(self) -> Vec<f32> {
        self.data
    }
}

impl From<Vec<f32>> for Vector {
    fn from(data: Vec<f32>) -> Self {
        Self::from_vec(data)
    }
}

impl From<&[f32]> for Vector {
    fn from(data: &[f32]) -> Self {
        Self::new(data)
    }
}

impl AsRef<[f32]> for Vector {
    fn as_ref(&self) -> &[f32] {
        &self.data
    }
}

impl std::ops::Index<usize> for Vector {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_creation() {
        let v = Vector::new(&[1.0, 2.0, 3.0]);
        assert_eq!(v.dimension(), 3);
        assert_eq!(v[0], 1.0);
        assert_eq!(v[1], 2.0);
        assert_eq!(v[2], 3.0);
    }

    #[test]
    fn test_vector_from_vec() {
        let v = Vector::from_vec(vec![1.0, 2.0]);
        assert_eq!(v.dimension(), 2);
    }

    #[test]
    fn test_vector_norm() {
        let v = Vector::new(&[3.0, 4.0]);
        assert!((v.norm() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_vector_normalized() {
        let v = Vector::new(&[3.0, 4.0]);
        let normalized = v.normalized().unwrap();
        assert!((normalized.norm() - 1.0).abs() < 1e-6);
        assert!((normalized[0] - 0.6).abs() < 1e-6);
        assert!((normalized[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_zero_vector_normalized() {
        let v = Vector::new(&[0.0, 0.0]);
        assert!(v.normalized().is_none());
    }

    #[test]
    fn test_vector_serialization() {
        let v = Vector::new(&[1.0, 2.0, 3.0]);
        let json = serde_json::to_string(&v).unwrap();
        let deserialized: Vector = serde_json::from_str(&json).unwrap();
        assert_eq!(v, deserialized);
    }
}
