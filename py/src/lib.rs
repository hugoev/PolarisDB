#![allow(clippy::useless_conversion)]
use polarisdb_core::{BruteForceIndex, DistanceMetric, Payload};
use pyo3::prelude::*;

#[pyclass]
/// In-memory brute-force vector index.
///
/// This index stores all vectors in memory and performs exhaustive search
/// for exact nearest neighbors.
struct Index {
    inner: BruteForceIndex,
}

#[pymethods]
impl Index {
    #[new]
    /// Create a new in-memory index.
    ///
    /// Args:
    ///     metric (str): Distance metric ("cosine", "euclidean", "dot").
    ///     dimension (int): Dimension of the vectors.
    fn new(metric: &str, dimension: usize) -> PyResult<Self> {
        let metric = match metric {
            "cosine" => DistanceMetric::Cosine,
            "euclidean" => DistanceMetric::Euclidean,
            "dot" => DistanceMetric::DotProduct,
            _ => return Err(pyo3::exceptions::PyValueError::new_err("Invalid metric")),
        };
        Ok(Index {
            inner: BruteForceIndex::new(metric, dimension),
        })
    }

    /// Insert a vector into the index.
    ///
    /// Args:
    ///     id (int): Unique identifier for the vector.
    ///     vector (list[float]): The vector data.
    fn insert(&mut self, id: u64, vector: Vec<f32>) -> PyResult<()> {
        // Simple payload for now
        self.inner
            .insert(id, vector, Payload::new())
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    /// Search for nearest neighbors.
    ///
    /// Args:
    ///     query (list[float]): The query vector.
    ///     k (int): Number of neighbors to return.
    ///
    /// Returns:
    ///     list[tuple[int, float]]: List of (id, distance) tuples.
    fn search(&self, query: Vec<f32>, k: usize) -> PyResult<Vec<(u64, f32)>> {
        let results = self.inner.search(&query, k, None);
        Ok(results.into_iter().map(|r| (r.id, r.distance)).collect())
    }

    /// Insert multiple vectors at once.
    ///
    /// Args:
    ///     ids (list[int]): List of unique identifiers.
    ///     vectors (list[list[float]]): List of vectors.
    fn insert_batch(&mut self, ids: Vec<u64>, vectors: Vec<Vec<f32>>) -> PyResult<()> {
        if ids.len() != vectors.len() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "ids and vectors must have the same length",
            ));
        }
        for (id, vector) in ids.into_iter().zip(vectors) {
            self.inner
                .insert(id, vector, Payload::new())
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    /// Search for nearest neighbors for multiple queries.
    ///
    /// Args:
    ///     queries (list[list[float]]): List of query vectors.
    ///     k (int): Number of neighbors per query.
    ///
    /// Returns:
    ///     list[list[tuple[int, float]]]: Results for each query.
    fn search_batch(&self, queries: Vec<Vec<f32>>, k: usize) -> PyResult<Vec<Vec<(u64, f32)>>> {
        let mut all_results = Vec::with_capacity(queries.len());
        for query in queries {
            let results = self.inner.search(&query, k, None);
            all_results.push(results.into_iter().map(|r| (r.id, r.distance)).collect());
        }
        Ok(all_results)
    }
}

#[pyclass]
/// Persistent vector collection backed by disk storage.
///
/// Stores vectors and metadata on disk with WAL (Write-Ahead Log) protection
/// for durability. Supports crash recovery.
struct Collection {
    inner: polarisdb_core::Collection,
}

#[pymethods]
impl Collection {
    #[staticmethod]
    /// Open an existing collection or create a new one.
    ///
    /// Args:
    ///     path (str): File system path for the collection.
    ///     dimension (int): Dimension of the vectors (must match existing).
    ///     metric (str): Distance metric ("cosine", "euclidean", "dot").
    fn open_or_create(path: &str, dimension: usize, metric: &str) -> PyResult<Self> {
        let metric = match metric {
            "cosine" => DistanceMetric::Cosine,
            "euclidean" => DistanceMetric::Euclidean,
            "dot" => DistanceMetric::DotProduct,
            _ => return Err(pyo3::exceptions::PyValueError::new_err("Invalid metric")),
        };
        let config = polarisdb_core::CollectionConfig::new(dimension, metric);
        let collection = polarisdb_core::Collection::open_or_create(path, config)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        Ok(Collection { inner: collection })
    }

    /// Insert a vector into the collection.
    ///
    /// Args:
    ///     id (int): Unique identifier.
    ///     vector (list[float]): Vector data.
    fn insert(&mut self, id: u64, vector: Vec<f32>) -> PyResult<()> {
        self.inner
            .insert(id, vector, Payload::new())
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    /// Search for nearest neighbors.
    ///
    /// Args:
    ///     query (list[float]): Query vector.
    ///     k (int): Number of results.
    ///
    /// Returns:
    ///     list[tuple[int, float]]: List of (id, distance) tuples.
    fn search(&self, query: Vec<f32>, k: usize) -> PyResult<Vec<(u64, f32)>> {
        let results = self.inner.search(&query, k, None);
        Ok(results.into_iter().map(|r| (r.id, r.distance)).collect())
    }

    /// Flush all pending writes to disk (checkpoint).
    ///
    /// This ensures all data is durable and truncates the WAL.
    fn flush(&mut self) -> PyResult<()> {
        self.inner
            .flush()
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
    }

    /// Insert multiple vectors at once.
    ///
    /// Args:
    ///     ids (list[int]): List of unique identifiers.
    ///     vectors (list[list[float]]): List of vectors.
    fn insert_batch(&mut self, ids: Vec<u64>, vectors: Vec<Vec<f32>>) -> PyResult<()> {
        if ids.len() != vectors.len() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "ids and vectors must have the same length",
            ));
        }
        for (id, vector) in ids.into_iter().zip(vectors) {
            self.inner
                .insert(id, vector, Payload::new())
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    /// Search for nearest neighbors for multiple queries.
    ///
    /// Args:
    ///     queries (list[list[float]]): List of query vectors.
    ///     k (int): Number of neighbors per query.
    ///
    /// Returns:
    ///     list[list[tuple[int, float]]]: Results for each query.
    fn search_batch(&self, queries: Vec<Vec<f32>>, k: usize) -> PyResult<Vec<Vec<(u64, f32)>>> {
        let mut all_results = Vec::with_capacity(queries.len());
        for query in queries {
            let results = self.inner.search(&query, k, None);
            all_results.push(results.into_iter().map(|r| (r.id, r.distance)).collect());
        }
        Ok(all_results)
    }
}

#[pymodule]
fn _polarisdb(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Index>()?;
    m.add_class::<Collection>()?;
    Ok(())
}
