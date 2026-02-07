use polarisdb_core::{BruteForceIndex, DistanceMetric, Payload};
use pyo3::prelude::*;

#[pyclass]
struct Index {
    inner: BruteForceIndex,
}

#[pymethods]
impl Index {
    #[new]
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

    fn insert(&mut self, id: u64, vector: Vec<f32>) -> PyResult<()> {
        // Simple payload for now
        self.inner
            .insert(id, vector, Payload::new())
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    fn search(&self, query: Vec<f32>, k: usize) -> PyResult<Vec<(u64, f32)>> {
        let results = self.inner.search(&query, k, None);
        Ok(results.into_iter().map(|r| (r.id, r.distance)).collect())
    }
}

#[pyclass]
struct Collection {
    inner: polarisdb_core::Collection,
}

#[pymethods]
impl Collection {
    #[staticmethod]
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

    fn insert(&mut self, id: u64, vector: Vec<f32>) -> PyResult<()> {
        self.inner
            .insert(id, vector, Payload::new())
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    fn search(&self, query: Vec<f32>, k: usize) -> PyResult<Vec<(u64, f32)>> {
        let results = self.inner.search(&query, k, None);
        Ok(results.into_iter().map(|r| (r.id, r.distance)).collect())
    }

    fn flush(&mut self) -> PyResult<()> {
        self.inner
            .flush()
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
    }
}

#[pymodule]
fn _polarisdb(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Index>()?;
    m.add_class::<Collection>()?;
    Ok(())
}
