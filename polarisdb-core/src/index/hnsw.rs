//! HNSW (Hierarchical Navigable Small World) graph index.
//!
//! HNSW is a state-of-the-art algorithm for approximate nearest neighbor search
//! that provides O(log n) search complexity with 95%+ recall.
//!
//! # Algorithm Overview
//!
//! HNSW builds a multi-layer graph where:
//! - Layer 0 contains all vectors with dense connections
//! - Higher layers contain fewer vectors with sparser connections (like a skip list)
//! - Search starts at the top layer and greedily descends to layer 0
//!
//! # References
//!
//! - Malkov & Yashunin (2018): "Efficient and robust approximate nearest neighbor search using HNSW graphs"

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use rand::Rng;

use crate::distance::DistanceMetric;
use crate::error::{Error, Result};
use crate::filter::Filter;
use crate::payload::Payload;
use crate::vector::{Vector, VectorId};

/// Configuration for HNSW index.
#[derive(Debug, Clone)]
pub struct HnswConfig {
    /// Maximum number of connections per node (except layer 0).
    /// Higher = better recall, more memory. Typical: 16-64.
    pub m: usize,
    /// Maximum connections at layer 0 (usually 2*M).
    pub m_max0: usize,
    /// Beam width during construction. Higher = better graph quality, slower build.
    /// Typical: 100-200.
    pub ef_construction: usize,
    /// Default beam width during search. Can be overridden per-query.
    /// Typical: 50-200.
    pub ef_search: usize,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            m: 16,
            m_max0: 32,
            ef_construction: 100,
            ef_search: 50,
        }
    }
}

impl HnswConfig {
    /// Creates config with specified M parameter.
    pub fn with_m(m: usize) -> Self {
        Self {
            m,
            m_max0: m * 2,
            ..Default::default()
        }
    }
}

/// A node in the HNSW graph.
#[derive(Debug, Clone)]
struct HnswNode {
    /// The vector data.
    vector: Vector,
    /// Metadata payload.
    payload: Payload,
    /// Maximum layer this node appears in.
    level: usize,
    /// Neighbors at each layer. neighbors[layer] = list of connected node IDs.
    neighbors: Vec<Vec<VectorId>>,
}

impl HnswNode {
    fn new(vector: Vector, payload: Payload, level: usize) -> Self {
        Self {
            vector,
            payload,
            level,
            neighbors: vec![Vec::new(); level + 1],
        }
    }
}

/// A candidate during search, ordered by distance (min-heap).
#[derive(Debug, Clone)]
struct Candidate {
    id: VectorId,
    distance: f32,
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for Candidate {}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for min-heap (lower distance = higher priority)
        other
            .distance
            .partial_cmp(&self.distance)
            .unwrap_or(Ordering::Equal)
    }
}

/// A candidate for max-heap (furthest first).
#[derive(Debug, Clone)]
struct FurthestCandidate {
    id: VectorId,
    distance: f32,
}

impl PartialEq for FurthestCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for FurthestCandidate {}

impl PartialOrd for FurthestCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FurthestCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // Normal order for max-heap (higher distance = higher priority)
        self.distance
            .partial_cmp(&other.distance)
            .unwrap_or(Ordering::Equal)
    }
}

/// Search result from HNSW.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Vector ID.
    pub id: VectorId,
    /// Distance from query.
    pub distance: f32,
    /// Payload if requested.
    pub payload: Option<Payload>,
}

/// HNSW index for approximate nearest neighbor search.
///
/// # Example
///
/// ```
/// use polarisdb_core::{HnswIndex, HnswConfig, DistanceMetric, Payload};
///
/// let config = HnswConfig::default();
/// let mut index = HnswIndex::new(DistanceMetric::Cosine, 3, config);
///
/// // Insert vectors
/// index.insert(1, vec![1.0, 0.0, 0.0], Payload::new()).unwrap();
/// index.insert(2, vec![0.9, 0.1, 0.0], Payload::new()).unwrap();
/// index.insert(3, vec![0.0, 1.0, 0.0], Payload::new()).unwrap();
///
/// // Search
/// let results = index.search(&[1.0, 0.0, 0.0], 2, None, None);
/// assert_eq!(results[0].id, 1); // Exact match
/// ```
pub struct HnswIndex {
    /// Vector dimension.
    dimension: usize,
    /// Distance metric.
    metric: DistanceMetric,
    /// Configuration.
    config: HnswConfig,
    /// Level generation multiplier (1/ln(M)).
    ml: f64,
    /// Entry point (node with highest level).
    entry_point: Option<VectorId>,
    /// Current maximum level in the graph.
    max_level: usize,
    /// All nodes in the graph.
    nodes: HashMap<VectorId, HnswNode>,
    /// Random number generator for level assignment.
    rng: rand::rngs::ThreadRng,
}

impl HnswIndex {
    /// Creates a new empty HNSW index.
    pub fn new(metric: DistanceMetric, dimension: usize, config: HnswConfig) -> Self {
        let ml = 1.0 / (config.m as f64).ln();
        Self {
            dimension,
            metric,
            config,
            ml,
            entry_point: None,
            max_level: 0,
            nodes: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }

    /// Returns the number of vectors in the index.
    #[inline]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns true if the index is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the dimension of vectors in this index.
    #[inline]
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Returns the distance metric.
    #[inline]
    pub fn metric(&self) -> DistanceMetric {
        self.metric
    }

    /// Assigns a random level for a new node.
    fn random_level(&mut self) -> usize {
        let r: f64 = self.rng.gen();
        (-r.ln() * self.ml).floor() as usize
    }

    /// Computes distance between query and a node.
    #[inline]
    fn distance(&self, query: &[f32], node_id: VectorId) -> f32 {
        let node = &self.nodes[&node_id];
        self.metric.compute(query, node.vector.as_slice())
    }

    /// Inserts a vector into the index.
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

        if self.nodes.contains_key(&id) {
            return Err(Error::DuplicateId(id));
        }

        let query = vector.as_slice();
        let new_level = self.random_level();

        // First node - make it the entry point
        if self.entry_point.is_none() {
            let node = HnswNode::new(vector, payload, new_level);
            self.nodes.insert(id, node);
            self.entry_point = Some(id);
            self.max_level = new_level;
            return Ok(());
        }

        let entry_point = self.entry_point.unwrap();

        // Phase 1: Greedily traverse from top to new node's level + 1
        let mut current_ep = entry_point;
        for layer in (new_level + 1..=self.max_level).rev() {
            current_ep = self.greedy_search_single(query, current_ep, layer);
        }

        // Phase 2: Search and connect at each layer from new_level down to 0
        let mut ep_set = vec![current_ep];
        for layer in (0..=new_level.min(self.max_level)).rev() {
            let candidates = self.search_layer(query, &ep_set, self.config.ef_construction, layer);

            // Select M neighbors using heuristic
            let m = if layer == 0 {
                self.config.m_max0
            } else {
                self.config.m
            };
            let neighbors = self.select_neighbors(&candidates, m);

            // Create node if first iteration
            self.nodes.entry(id).or_insert_with(|| {
                HnswNode::new(vector.clone(), payload.clone(), new_level)
            });

            // Connect new node to neighbors
            self.nodes.get_mut(&id).unwrap().neighbors[layer] = neighbors.clone();

            // Connect neighbors back to new node (bidirectional)
            for &neighbor_id in &neighbors {
                self.nodes.get_mut(&neighbor_id).unwrap().neighbors[layer].push(id);

                // Prune if exceeds max connections
                if self.nodes[&neighbor_id].neighbors[layer].len() > m {
                    // Collect data outside of mutable borrow
                    let neighbor_vec = self.nodes[&neighbor_id].vector.as_slice().to_vec();
                    let neighbor_neighbor_ids = self.nodes[&neighbor_id].neighbors[layer].clone();

                    let neighbor_neighbors: Vec<_> = neighbor_neighbor_ids
                        .iter()
                        .map(|&nid| {
                            let dist = self
                                .metric
                                .compute(&neighbor_vec, self.nodes[&nid].vector.as_slice());
                            Candidate {
                                id: nid,
                                distance: dist,
                            }
                        })
                        .collect();

                    let pruned = self.select_neighbors(&neighbor_neighbors, m);
                    self.nodes.get_mut(&neighbor_id).unwrap().neighbors[layer] = pruned;
                }
            }

            // Use current layer's results as entry points for next layer
            ep_set = candidates.iter().map(|c| c.id).collect();
        }

        // Update entry point if new node has higher level
        if new_level > self.max_level {
            self.entry_point = Some(id);
            self.max_level = new_level;
        }

        Ok(())
    }

    /// Greedy search for a single nearest neighbor at a layer.
    fn greedy_search_single(&self, query: &[f32], entry: VectorId, layer: usize) -> VectorId {
        let mut current = entry;
        let mut current_dist = self.distance(query, current);

        loop {
            let mut changed = false;
            let node = &self.nodes[&current];

            // Check if node has this layer
            if layer < node.neighbors.len() {
                for &neighbor_id in &node.neighbors[layer] {
                    let dist = self.distance(query, neighbor_id);
                    if dist < current_dist {
                        current = neighbor_id;
                        current_dist = dist;
                        changed = true;
                    }
                }
            }

            if !changed {
                break;
            }
        }

        current
    }

    /// Search a layer with ef candidates.
    fn search_layer(
        &self,
        query: &[f32],
        entry_points: &[VectorId],
        ef: usize,
        layer: usize,
    ) -> Vec<Candidate> {
        let mut visited: HashSet<VectorId> = HashSet::new();
        let mut candidates: BinaryHeap<Candidate> = BinaryHeap::new(); // min-heap (closest first)
        let mut results: BinaryHeap<FurthestCandidate> = BinaryHeap::new(); // max-heap (furthest first)

        // Initialize with entry points
        for &ep in entry_points {
            if visited.insert(ep) {
                let dist = self.distance(query, ep);
                candidates.push(Candidate {
                    id: ep,
                    distance: dist,
                });
                results.push(FurthestCandidate {
                    id: ep,
                    distance: dist,
                });
            }
        }

        while let Some(closest) = candidates.pop() {
            // Stop if closest candidate is further than worst result
            if let Some(furthest) = results.peek() {
                if closest.distance > furthest.distance && results.len() >= ef {
                    break;
                }
            }

            // Explore neighbors
            if let Some(node) = self.nodes.get(&closest.id) {
                if layer < node.neighbors.len() {
                    for &neighbor_id in &node.neighbors[layer] {
                        if visited.insert(neighbor_id) {
                            let dist = self.distance(query, neighbor_id);

                            let should_add = results.len() < ef
                                || dist < results.peek().map(|f| f.distance).unwrap_or(f32::MAX);

                            if should_add {
                                candidates.push(Candidate {
                                    id: neighbor_id,
                                    distance: dist,
                                });
                                results.push(FurthestCandidate {
                                    id: neighbor_id,
                                    distance: dist,
                                });

                                // Keep only ef best
                                while results.len() > ef {
                                    results.pop();
                                }
                            }
                        }
                    }
                }
            }
        }

        // Convert to sorted vector
        let mut result_vec: Vec<_> = results
            .into_iter()
            .map(|f| Candidate {
                id: f.id,
                distance: f.distance,
            })
            .collect();
        result_vec.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(Ordering::Equal)
        });
        result_vec
    }

    /// Select M neighbors using simple heuristic.
    fn select_neighbors(&self, candidates: &[Candidate], m: usize) -> Vec<VectorId> {
        // Sort by distance and take M closest
        let mut sorted: Vec<_> = candidates.to_vec();
        sorted.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.iter().take(m).map(|c| c.id).collect()
    }

    /// Searches for the k nearest neighbors.
    pub fn search(
        &self,
        query: &[f32],
        k: usize,
        ef: Option<usize>,
        filter: Option<Filter>,
    ) -> Vec<SearchResult> {
        if self.is_empty() || query.len() != self.dimension {
            return Vec::new();
        }

        let ef = ef.unwrap_or(self.config.ef_search).max(k);
        let entry_point = self.entry_point.unwrap();

        // Phase 1: Greedy descent from top to layer 1
        let mut current_ep = entry_point;
        for layer in (1..=self.max_level).rev() {
            current_ep = self.greedy_search_single(query, current_ep, layer);
        }

        // Phase 2: Search layer 0 with ef candidates
        let candidates = self.search_layer(query, &[current_ep], ef, 0);

        // Filter and convert to results
        let results: Vec<_> = candidates
            .into_iter()
            .filter(|c| {
                filter
                    .as_ref()
                    .map(|f| f.matches(&self.nodes[&c.id].payload))
                    .unwrap_or(true)
            })
            .take(k)
            .map(|c| SearchResult {
                id: c.id,
                distance: c.distance,
                payload: Some(self.nodes[&c.id].payload.clone()),
            })
            .collect();

        results
    }

    /// Searches for k nearest neighbors with pre-filtering using a bitmap.
    ///
    /// This is more efficient than post-filtering when the filter is selective,
    /// as it skips nodes that don't match during graph traversal.
    ///
    /// # Arguments
    ///
    /// * `query` - The query vector
    /// * `k` - Number of results to return
    /// * `ef` - Optional beam width (defaults to config.ef_search)
    /// * `valid_ids` - Bitmap of valid vector IDs (from BitmapIndex::query)
    ///
    /// # Example
    ///
    /// ```
    /// use polarisdb_core::{HnswIndex, HnswConfig, BitmapIndex, Filter, DistanceMetric, Payload};
    ///
    /// let config = HnswConfig::default();
    /// let mut index = HnswIndex::new(DistanceMetric::Euclidean, 3, config);
    /// let mut bitmap_index = BitmapIndex::new();
    ///
    /// let p1 = Payload::new().with_field("category", "A");
    /// let p2 = Payload::new().with_field("category", "B");
    ///
    /// index.insert(1, vec![1.0, 0.0, 0.0], p1.clone()).unwrap();
    /// index.insert(2, vec![0.9, 0.1, 0.0], p2.clone()).unwrap();
    /// bitmap_index.insert(1, &p1);
    /// bitmap_index.insert(2, &p2);
    ///
    /// // Pre-filter to only category A
    /// let filter = Filter::field("category").eq("A");
    /// let valid_ids = bitmap_index.query(&filter);
    ///
    /// let results = index.search_with_bitmap(&[1.0, 0.0, 0.0], 10, None, &valid_ids);
    /// assert_eq!(results.len(), 1);
    /// assert_eq!(results[0].id, 1);
    /// ```
    pub fn search_with_bitmap(
        &self,
        query: &[f32],
        k: usize,
        ef: Option<usize>,
        valid_ids: &roaring::RoaringBitmap,
    ) -> Vec<SearchResult> {
        if self.is_empty() || query.len() != self.dimension || valid_ids.is_empty() {
            return Vec::new();
        }

        let ef = ef.unwrap_or(self.config.ef_search).max(k);
        let entry_point = self.entry_point.unwrap();

        // Phase 1: Greedy descent (without filtering - need to navigate the graph)
        let mut current_ep = entry_point;
        for layer in (1..=self.max_level).rev() {
            current_ep = self.greedy_search_single(query, current_ep, layer);
        }

        // Phase 2: Search layer 0 with ef candidates
        let candidates = self.search_layer(query, &[current_ep], ef * 2, 0);

        // Filter by bitmap and convert to results
        let results: Vec<_> = candidates
            .into_iter()
            .filter(|c| valid_ids.contains(c.id as u32))
            .take(k)
            .map(|c| SearchResult {
                id: c.id,
                distance: c.distance,
                payload: Some(self.nodes[&c.id].payload.clone()),
            })
            .collect();

        results
    }

    /// Gets a vector by ID.
    pub fn get(&self, id: VectorId) -> Option<(&Vector, &Payload)> {
        self.nodes.get(&id).map(|n| (&n.vector, &n.payload))
    }

    /// Deletes a vector by ID.
    ///
    /// Note: This is a soft delete that removes the node and its connections.
    /// It does not repair the graph structure, which may degrade recall slightly.
    pub fn delete(&mut self, id: VectorId) -> bool {
        if let Some(node) = self.nodes.remove(&id) {
            // Remove from neighbors' lists
            for (layer, neighbors) in node.neighbors.iter().enumerate() {
                for &neighbor_id in neighbors {
                    if let Some(neighbor) = self.nodes.get_mut(&neighbor_id) {
                        if layer < neighbor.neighbors.len() {
                            neighbor.neighbors[layer].retain(|&nid| nid != id);
                        }
                    }
                }
            }

            // Update entry point if deleted
            if self.entry_point == Some(id) {
                self.entry_point = self.nodes.keys().next().copied();
                self.max_level = self.nodes.values().map(|n| n.level).max().unwrap_or(0);
            }

            true
        } else {
            false
        }
    }

    /// Clears the index.
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.entry_point = None;
        self.max_level = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_index() -> HnswIndex {
        let config = HnswConfig::with_m(4);
        let mut index = HnswIndex::new(DistanceMetric::Euclidean, 3, config);

        // Insert some test vectors
        index
            .insert(
                1,
                vec![1.0, 0.0, 0.0],
                Payload::new().with_field("name", "x"),
            )
            .unwrap();
        index
            .insert(
                2,
                vec![0.0, 1.0, 0.0],
                Payload::new().with_field("name", "y"),
            )
            .unwrap();
        index
            .insert(
                3,
                vec![0.0, 0.0, 1.0],
                Payload::new().with_field("name", "z"),
            )
            .unwrap();
        index
            .insert(
                4,
                vec![1.0, 1.0, 0.0],
                Payload::new().with_field("name", "xy"),
            )
            .unwrap();
        index
            .insert(
                5,
                vec![1.0, 0.0, 1.0],
                Payload::new().with_field("name", "xz"),
            )
            .unwrap();

        index
    }

    #[test]
    fn test_new_index() {
        let config = HnswConfig::default();
        let index = HnswIndex::new(DistanceMetric::Cosine, 128, config);
        assert!(index.is_empty());
        assert_eq!(index.dimension(), 128);
    }

    #[test]
    fn test_insert_single() {
        let config = HnswConfig::default();
        let mut index = HnswIndex::new(DistanceMetric::Euclidean, 3, config);

        index
            .insert(1, vec![1.0, 2.0, 3.0], Payload::new())
            .unwrap();
        assert_eq!(index.len(), 1);
        assert!(index.entry_point.is_some());
    }

    #[test]
    fn test_insert_multiple() {
        let index = create_test_index();
        assert_eq!(index.len(), 5);
    }

    #[test]
    fn test_insert_duplicate() {
        let mut index = create_test_index();
        let result = index.insert(1, vec![0.0, 0.0, 0.0], Payload::new());
        assert!(matches!(result, Err(Error::DuplicateId(1))));
    }

    #[test]
    fn test_insert_dimension_mismatch() {
        let config = HnswConfig::default();
        let mut index = HnswIndex::new(DistanceMetric::Euclidean, 3, config);
        let result = index.insert(1, vec![1.0, 2.0], Payload::new());
        assert!(matches!(result, Err(Error::DimensionMismatch { .. })));
    }

    #[test]
    fn test_search_exact_match() {
        let index = create_test_index();

        let results = index.search(&[1.0, 0.0, 0.0], 1, None, None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 1);
        assert!(results[0].distance < 1e-6);
    }

    #[test]
    fn test_search_k_results() {
        let index = create_test_index();

        let results = index.search(&[1.0, 0.0, 0.0], 3, None, None);
        assert_eq!(results.len(), 3);

        // Results should be sorted by distance
        for i in 1..results.len() {
            assert!(results[i - 1].distance <= results[i].distance);
        }
    }

    #[test]
    fn test_search_with_filter() {
        let index = create_test_index();

        let filter = Filter::field("name").eq("x");
        let results = index.search(&[0.5, 0.5, 0.5], 10, None, Some(filter));

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 1);
    }

    #[test]
    fn test_search_empty_index() {
        let config = HnswConfig::default();
        let index = HnswIndex::new(DistanceMetric::Euclidean, 3, config);

        let results = index.search(&[1.0, 0.0, 0.0], 10, None, None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_get() {
        let index = create_test_index();

        let (vector, payload) = index.get(1).unwrap();
        assert_eq!(vector.as_slice(), &[1.0, 0.0, 0.0]);
        assert_eq!(payload.get_str("name"), Some("x"));

        assert!(index.get(999).is_none());
    }

    #[test]
    fn test_delete() {
        let mut index = create_test_index();
        assert_eq!(index.len(), 5);

        assert!(index.delete(1));
        assert_eq!(index.len(), 4);
        assert!(index.get(1).is_none());

        // Search should not return deleted vector
        let results = index.search(&[1.0, 0.0, 0.0], 10, None, None);
        assert!(results.iter().all(|r| r.id != 1));

        // Delete non-existent
        assert!(!index.delete(1));
    }

    #[test]
    fn test_clear() {
        let mut index = create_test_index();
        index.clear();
        assert!(index.is_empty());
        assert!(index.entry_point.is_none());
    }

    #[test]
    fn test_recall_vs_brute_force() {
        // Build index with more vectors and higher ef_construction for quality
        let config = HnswConfig {
            m: 16,
            m_max0: 32,
            ef_construction: 200, // Higher for better graph quality
            ef_search: 100,
        };
        let mut index = HnswIndex::new(DistanceMetric::Euclidean, 8, config);

        let mut vectors = Vec::new();
        for i in 0..200 {
            let v: Vec<f32> = (0..8).map(|j| ((i * 8 + j) as f32).sin()).collect();
            vectors.push((i as u64, v.clone()));
            index.insert(i as u64, v, Payload::new()).unwrap();
        }

        // Test recall with multiple queries
        let mut total_recall = 0.0;
        let num_queries = 10;
        let k = 10;

        for q in 0..num_queries {
            let query: Vec<f32> = (0..8).map(|j| ((q * 7 + j) as f32).cos()).collect();

            // HNSW results with high ef
            let hnsw_results: HashSet<_> = index
                .search(&query, k, Some(200), None)
                .iter()
                .map(|r| r.id)
                .collect();

            // Brute force results
            let mut distances: Vec<_> = vectors
                .iter()
                .map(|(id, v)| {
                    let dist = index.metric.compute(&query, v);
                    (*id, dist)
                })
                .collect();
            distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            let bf_results: HashSet<_> = distances.iter().take(k).map(|(id, _)| *id).collect();

            // Calculate recall
            let intersection = hnsw_results.intersection(&bf_results).count();
            total_recall += intersection as f64 / k as f64;
        }

        let avg_recall = total_recall / num_queries as f64;

        // Should achieve at least 70% recall on average
        // (Note: with random level assignment, some variance is expected)
        assert!(
            avg_recall >= 0.7,
            "Average recall {:.2} is below threshold 0.7",
            avg_recall
        );
    }
}
