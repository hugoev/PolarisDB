//! Brute-force (flat) index for exact nearest neighbor search.
//!
//! This is the simplest index type that computes distances to all vectors
//! during search. While O(n) in complexity, it provides:
//! - 100% recall (exact results)
//! - Baseline for benchmarking other indexes
//! - Suitable for small datasets (< 100k vectors)

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::distance::DistanceMetric;
use crate::error::{Error, Result};
use crate::filter::Filter;
use crate::payload::Payload;
use crate::vector::{Vector, VectorId};

/// A single search result with ID, distance, and optional payload.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The ID of the matched vector.
    pub id: VectorId,
    /// The distance from the query vector (lower = more similar).
    pub distance: f32,
    /// The payload attached to this vector, if requested.
    pub payload: Option<Payload>,
}

impl SearchResult {
    /// Creates a new search result.
    pub fn new(id: VectorId, distance: f32, payload: Option<Payload>) -> Self {
        Self {
            id,
            distance,
            payload,
        }
    }
}

impl PartialEq for SearchResult {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && (self.distance - other.distance).abs() < f32::EPSILON
    }
}

impl Eq for SearchResult {}

impl PartialOrd for SearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchResult {
    fn cmp(&self, other: &Self) -> Ordering {
        // Lower distance = better, so we compare in natural order
        self.distance
            .partial_cmp(&other.distance)
            .unwrap_or(Ordering::Equal)
    }
}

/// A stored vector entry with its metadata.
#[derive(Debug, Clone)]
struct VectorEntry {
    vector: Vector,
    payload: Payload,
}

/// Brute-force index that computes exact distances to all vectors.
///
/// This index provides 100% recall at the cost of O(n) search complexity.
/// Suitable for datasets up to ~100k vectors or as a baseline for benchmarks.
///
/// # Example
///
/// ```
/// use polarisdb_core::{BruteForceIndex, DistanceMetric, Payload, Filter};
///
/// let mut index = BruteForceIndex::new(DistanceMetric::Cosine, 3);
///
/// // Insert some vectors
/// index.insert(1, vec![1.0, 0.0, 0.0], Payload::new().with_field("type", "a")).unwrap();
/// index.insert(2, vec![0.0, 1.0, 0.0], Payload::new().with_field("type", "b")).unwrap();
/// index.insert(3, vec![0.9, 0.1, 0.0], Payload::new().with_field("type", "a")).unwrap();
///
/// // Search for similar vectors
/// let query = vec![1.0, 0.0, 0.0];
/// let results = index.search(&query, 2, None);
/// assert_eq!(results[0].id, 1); // Exact match
/// assert_eq!(results[1].id, 3); // Close match
///
/// // Search with filter
/// let filter = Filter::field("type").eq("a");
/// let results = index.search(&query, 10, Some(filter));
/// assert!(results.iter().all(|r| r.payload.as_ref().unwrap().get_str("type") == Some("a")));
/// ```
#[derive(Debug)]
pub struct BruteForceIndex {
    /// The dimension of vectors in this index.
    dimension: usize,
    /// The distance metric to use.
    metric: DistanceMetric,
    /// Stored vectors indexed by ID.
    vectors: HashMap<VectorId, VectorEntry>,
    /// Next auto-generated ID (if not specified).
    next_id: VectorId,
}

impl BruteForceIndex {
    /// Creates a new brute-force index with the specified metric and dimension.
    ///
    /// # Arguments
    ///
    /// * `metric` - The distance metric to use for similarity computation.
    /// * `dimension` - The dimensionality of vectors (e.g., 384 for sentence-transformers).
    pub fn new(metric: DistanceMetric, dimension: usize) -> Self {
        Self {
            dimension,
            metric,
            vectors: HashMap::new(),
            next_id: 1,
        }
    }

    /// Returns the dimension of vectors in this index.
    #[inline]
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Returns the distance metric used by this index.
    #[inline]
    pub fn metric(&self) -> DistanceMetric {
        self.metric
    }

    /// Returns the number of vectors in the index.
    #[inline]
    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    /// Returns true if the index contains no vectors.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }

    /// Inserts a vector with the given ID and payload.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The vector dimension doesn't match the index dimension.
    /// - A vector with the same ID already exists.
    pub fn insert<V: Into<Vector>>(
        &mut self,
        id: VectorId,
        vector: V,
        payload: Payload,
    ) -> Result<()> {
        let vector = vector.into();

        if vector.dimension() != self.dimension {
            return Err(Error::DimensionMismatch {
                expected: self.dimension,
                got: vector.dimension(),
            });
        }

        if self.vectors.contains_key(&id) {
            return Err(Error::DuplicateId(id));
        }

        self.vectors.insert(id, VectorEntry { vector, payload });
        self.next_id = self.next_id.max(id + 1);

        Ok(())
    }

    /// Inserts a vector with an auto-generated ID.
    ///
    /// Returns the generated ID.
    pub fn insert_auto<V: Into<Vector>>(
        &mut self,
        vector: V,
        payload: Payload,
    ) -> Result<VectorId> {
        let id = self.next_id;
        self.insert(id, vector, payload)?;
        Ok(id)
    }

    /// Updates an existing vector's data and/or payload.
    ///
    /// # Errors
    ///
    /// Returns an error if the vector doesn't exist or dimension mismatches.
    pub fn update<V: Into<Vector>>(
        &mut self,
        id: VectorId,
        vector: V,
        payload: Payload,
    ) -> Result<()> {
        let vector = vector.into();

        if vector.dimension() != self.dimension {
            return Err(Error::DimensionMismatch {
                expected: self.dimension,
                got: vector.dimension(),
            });
        }

        if !self.vectors.contains_key(&id) {
            return Err(Error::NotFound(id));
        }

        self.vectors.insert(id, VectorEntry { vector, payload });
        Ok(())
    }

    /// Deletes a vector by ID.
    ///
    /// Returns true if the vector was deleted, false if it didn't exist.
    pub fn delete(&mut self, id: VectorId) -> bool {
        self.vectors.remove(&id).is_some()
    }

    /// Gets a vector by ID.
    ///
    /// Returns the vector and payload if found.
    pub fn get(&self, id: VectorId) -> Option<(&Vector, &Payload)> {
        self.vectors.get(&id).map(|e| (&e.vector, &e.payload))
    }

    /// Searches for the k nearest neighbors to the query vector.
    ///
    /// # Arguments
    ///
    /// * `query` - The query vector to search for.
    /// * `k` - The number of results to return.
    /// * `filter` - Optional filter to apply to payloads.
    ///
    /// # Returns
    ///
    /// A vector of search results sorted by distance (ascending).
    pub fn search<V: AsRef<[f32]>>(
        &self,
        query: V,
        k: usize,
        filter: Option<Filter>,
    ) -> Vec<SearchResult> {
        let query = query.as_ref();

        if query.len() != self.dimension {
            return Vec::new();
        }

        // Collect all matching vectors with their distances
        let mut candidates: Vec<SearchResult> = self
            .vectors
            .iter()
            .filter(|(_, entry)| {
                filter
                    .as_ref()
                    .map(|f| f.matches(&entry.payload))
                    .unwrap_or(true)
            })
            .map(|(&id, entry)| {
                let distance = self.metric.compute(query, entry.vector.as_slice());
                SearchResult::new(id, distance, Some(entry.payload.clone()))
            })
            .collect();

        // Sort by distance (ascending - lower is better)
        candidates.sort();

        // Return top-k
        candidates.truncate(k);
        candidates
    }

    /// Returns an iterator over all vector IDs in the index.
    pub fn ids(&self) -> impl Iterator<Item = VectorId> + '_ {
        self.vectors.keys().copied()
    }

    /// Clears all vectors from the index.
    pub fn clear(&mut self) {
        self.vectors.clear();
        self.next_id = 1;
    }
}

impl FromIterator<(VectorId, Vector, Payload)> for BruteForceIndex {
    fn from_iter<T: IntoIterator<Item = (VectorId, Vector, Payload)>>(iter: T) -> Self {
        let mut iter = iter.into_iter().peekable();

        // Peek to get dimension from first element
        let dimension = iter.peek().map(|(_, v, _)| v.dimension()).unwrap_or(0);
        let mut index = BruteForceIndex::new(DistanceMetric::default(), dimension);

        for (id, vector, payload) in iter {
            let _ = index.insert(id, vector, payload);
        }

        index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_index() -> BruteForceIndex {
        let mut index = BruteForceIndex::new(DistanceMetric::Euclidean, 3);
        index
            .insert(
                1,
                vec![1.0, 0.0, 0.0],
                Payload::new().with_field("type", "a"),
            )
            .unwrap();
        index
            .insert(
                2,
                vec![0.0, 1.0, 0.0],
                Payload::new().with_field("type", "b"),
            )
            .unwrap();
        index
            .insert(
                3,
                vec![0.0, 0.0, 1.0],
                Payload::new().with_field("type", "a"),
            )
            .unwrap();
        index
    }

    #[test]
    fn test_new_index() {
        let index = BruteForceIndex::new(DistanceMetric::Cosine, 384);
        assert_eq!(index.dimension(), 384);
        assert_eq!(index.metric(), DistanceMetric::Cosine);
        assert!(index.is_empty());
    }

    #[test]
    fn test_insert_and_len() {
        let mut index = BruteForceIndex::new(DistanceMetric::Euclidean, 3);
        assert!(index.is_empty());

        index
            .insert(1, vec![1.0, 2.0, 3.0], Payload::new())
            .unwrap();
        assert_eq!(index.len(), 1);

        index
            .insert(2, vec![4.0, 5.0, 6.0], Payload::new())
            .unwrap();
        assert_eq!(index.len(), 2);
    }

    #[test]
    fn test_insert_dimension_mismatch() {
        let mut index = BruteForceIndex::new(DistanceMetric::Euclidean, 3);
        let result = index.insert(1, vec![1.0, 2.0], Payload::new());
        assert!(matches!(result, Err(Error::DimensionMismatch { .. })));
    }

    #[test]
    fn test_insert_duplicate_id() {
        let mut index = BruteForceIndex::new(DistanceMetric::Euclidean, 3);
        index
            .insert(1, vec![1.0, 2.0, 3.0], Payload::new())
            .unwrap();

        let result = index.insert(1, vec![4.0, 5.0, 6.0], Payload::new());
        assert!(matches!(result, Err(Error::DuplicateId(1))));
    }

    #[test]
    fn test_insert_auto() {
        let mut index = BruteForceIndex::new(DistanceMetric::Euclidean, 3);

        let id1 = index
            .insert_auto(vec![1.0, 2.0, 3.0], Payload::new())
            .unwrap();
        let id2 = index
            .insert_auto(vec![4.0, 5.0, 6.0], Payload::new())
            .unwrap();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(index.len(), 2);
    }

    #[test]
    fn test_get() {
        let index = create_test_index();

        let (vector, payload) = index.get(1).unwrap();
        assert_eq!(vector.as_slice(), &[1.0, 0.0, 0.0]);
        assert_eq!(payload.get_str("type"), Some("a"));

        assert!(index.get(999).is_none());
    }

    #[test]
    fn test_update() {
        let mut index = create_test_index();

        index
            .update(
                1,
                vec![0.5, 0.5, 0.0],
                Payload::new().with_field("type", "updated"),
            )
            .unwrap();

        let (vector, payload) = index.get(1).unwrap();
        assert_eq!(vector.as_slice(), &[0.5, 0.5, 0.0]);
        assert_eq!(payload.get_str("type"), Some("updated"));
    }

    #[test]
    fn test_update_not_found() {
        let mut index = create_test_index();
        let result = index.update(999, vec![1.0, 2.0, 3.0], Payload::new());
        assert!(matches!(result, Err(Error::NotFound(999))));
    }

    #[test]
    fn test_delete() {
        let mut index = create_test_index();
        assert_eq!(index.len(), 3);

        assert!(index.delete(1));
        assert_eq!(index.len(), 2);
        assert!(index.get(1).is_none());

        assert!(!index.delete(1)); // Already deleted
    }

    #[test]
    fn test_search_basic() {
        let index = create_test_index();

        // Search for exact match
        let results = index.search([1.0, 0.0, 0.0], 3, None);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].id, 1);
        assert!(results[0].distance < f32::EPSILON);
    }

    #[test]
    fn test_search_k_limit() {
        let index = create_test_index();

        let results = index.search([1.0, 0.0, 0.0], 1, None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 1);
    }

    #[test]
    fn test_search_with_filter() {
        let index = create_test_index();

        let filter = Filter::field("type").eq("a");
        let results = index.search([0.5, 0.5, 0.5], 10, Some(filter));

        assert_eq!(results.len(), 2);
        for result in &results {
            assert_eq!(result.payload.as_ref().unwrap().get_str("type"), Some("a"));
        }
    }

    #[test]
    fn test_search_empty_index() {
        let index = BruteForceIndex::new(DistanceMetric::Euclidean, 3);
        let results = index.search([1.0, 0.0, 0.0], 10, None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut index = create_test_index();
        assert!(!index.is_empty());

        index.clear();
        assert!(index.is_empty());
    }

    #[test]
    fn test_cosine_search() {
        let mut index = BruteForceIndex::new(DistanceMetric::Cosine, 3);
        index
            .insert(1, vec![1.0, 0.0, 0.0], Payload::new())
            .unwrap();
        index
            .insert(2, vec![2.0, 0.0, 0.0], Payload::new())
            .unwrap(); // Same direction

        // Both should have distance ~0 from query (same direction)
        let results = index.search([0.5, 0.0, 0.0], 2, None);
        assert!(results[0].distance < 1e-5);
        assert!(results[1].distance < 1e-5);
    }

    #[test]
    fn test_from_iterator() {
        let data = vec![
            (1, Vector::from_vec(vec![1.0, 0.0, 0.0]), Payload::new()),
            (2, Vector::from_vec(vec![0.0, 1.0, 0.0]), Payload::new()),
        ];

        let index: BruteForceIndex = data.into_iter().collect();
        assert_eq!(index.len(), 2);
        assert_eq!(index.dimension(), 3);
    }
}
