//! Persistent vector collection with crash-safe storage.
//!
//! A `Collection` combines the in-memory index with durable storage:
//! - WAL for atomic operations
//! - Append-only data file for vectors
//! - Automatic recovery on open

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use parking_lot::RwLock;

use crate::distance::DistanceMetric;
use crate::error::{Error, Result};
use crate::filter::Filter;
use crate::index::brute_force::{BruteForceIndex, SearchResult};
use crate::payload::Payload;
use crate::storage::data_file::DataFile;
use crate::storage::wal::{SyncMode, Wal, WalEntry, WalEntryKind};
use crate::vector::VectorId;

/// Configuration for a collection.
#[derive(Debug, Clone)]
pub struct CollectionConfig {
    /// Dimensionality of vectors.
    pub dimension: usize,
    /// Distance metric.
    pub metric: DistanceMetric,
    /// WAL sync mode.
    pub sync_mode: SyncMode,
}

impl CollectionConfig {
    /// Creates a new config with the given dimension and metric.
    pub fn new(dimension: usize, metric: DistanceMetric) -> Self {
        Self {
            dimension,
            metric,
            sync_mode: SyncMode::Batched,
        }
    }

    /// Sets the sync mode. Chainable.
    pub fn with_sync_mode(mut self, mode: SyncMode) -> Self {
        self.sync_mode = mode;
        self
    }
}

/// Metadata for the collection, persisted to disk.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CollectionMeta {
    dimension: usize,
    metric: String,
    vector_count: u64,
    next_id: u64,
}

/// A persistent vector collection.
///
/// The collection provides durable storage with automatic crash recovery.
/// All mutations are first written to a WAL, then applied to the in-memory
/// index. On open, the WAL is replayed to restore state.
///
/// # Example
///
/// ```no_run
/// use polarisdb_core::{Collection, CollectionConfig, DistanceMetric, Payload};
///
/// // Open or create a collection
/// let config = CollectionConfig::new(384, DistanceMetric::Cosine);
/// let collection = Collection::open_or_create("./my_vectors", config).unwrap();
///
/// // Insert vectors
/// collection.insert(1, vec![0.1; 384], Payload::new()).unwrap();
///
/// // Search
/// let query = vec![0.1; 384];
/// let results = collection.search(&query, 10, None);
///
/// // Flush to ensure durability
/// collection.flush().unwrap();
/// ```
pub struct Collection {
    /// Path to collection directory.
    path: PathBuf,
    /// Configuration.
    config: CollectionConfig,
    /// In-memory index.
    index: RwLock<BruteForceIndex>,
    /// Write-ahead log.
    wal: RwLock<Wal>,
    /// Data file for vector storage.
    data_file: RwLock<DataFile>,
    /// Mapping from vector ID to data file offset.
    offsets: RwLock<HashMap<VectorId, u64>>,
    /// Next auto-generated ID.
    next_id: RwLock<u64>,
}

impl Collection {
    /// Opens an existing collection or creates a new one.
    pub fn open_or_create<P: AsRef<Path>>(path: P, config: CollectionConfig) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Create directory if needed
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|e| Error::CollectionError(format!("create dir failed: {}", e)))?;
        }

        let meta_path = path.join("meta.json");
        let wal_path = path.join("wal.log");
        let data_path = path.join("data.pdb");

        // Load or create metadata
        let (meta, is_new) = if meta_path.exists() {
            let content = fs::read_to_string(&meta_path)
                .map_err(|e| Error::CollectionError(format!("read meta failed: {}", e)))?;
            let meta: CollectionMeta = serde_json::from_str(&content)
                .map_err(|e| Error::CollectionError(format!("parse meta failed: {}", e)))?;
            (meta, false)
        } else {
            let meta = CollectionMeta {
                dimension: config.dimension,
                metric: format!("{:?}", config.metric),
                vector_count: 0,
                next_id: 1,
            };
            (meta, true)
        };

        // Verify dimension matches
        if !is_new && meta.dimension != config.dimension {
            return Err(Error::CollectionError(format!(
                "dimension mismatch: collection has {}, config has {}",
                meta.dimension, config.dimension
            )));
        }

        // Open WAL and data file
        let wal = Wal::open(&wal_path, config.sync_mode)?;
        let data_file = DataFile::open(&data_path)?;

        // Create in-memory index
        let index = BruteForceIndex::new(config.metric, config.dimension);

        let collection = Self {
            path: path.clone(),
            config,
            index: RwLock::new(index),
            wal: RwLock::new(wal),
            data_file: RwLock::new(data_file),
            offsets: RwLock::new(HashMap::new()),
            next_id: RwLock::new(meta.next_id),
        };

        // Recover from data file and WAL
        collection.recover()?;

        // Save metadata if new
        if is_new {
            collection.save_meta()?;
        }

        Ok(collection)
    }

    /// Recovers state by reading the data file and replaying the WAL.
    fn recover(&self) -> Result<()> {
        // First, load all active records from data file
        let records = {
            let df = self.data_file.read();
            df.iter_active()?
        };

        {
            let mut index = self.index.write();
            let mut offsets = self.offsets.write();
            let mut max_id = 0u64;

            for record in records {
                let _ = index.insert(record.id, record.vector.clone(), record.payload.clone());
                offsets.insert(record.id, record.offset);
                max_id = max_id.max(record.id);
            }

            *self.next_id.write() = max_id + 1;
        }

        // Then replay WAL (may have newer operations)
        let wal_path = self.path.join("wal.log");
        let entries = Wal::read_all(&wal_path)?;

        for entry in entries {
            match entry.kind {
                WalEntryKind::Insert => {
                    self.apply_insert_no_wal(entry.id, entry.vector, entry.payload)?;
                }
                WalEntryKind::Update => {
                    self.apply_update_no_wal(entry.id, entry.vector, entry.payload)?;
                }
                WalEntryKind::Delete => {
                    self.apply_delete_no_wal(entry.id)?;
                }
                WalEntryKind::Checkpoint => {
                    // Checkpoint entries are just markers
                }
            }
        }

        Ok(())
    }

    /// Inserts a vector with the given ID.
    pub fn insert(&self, id: VectorId, vector: Vec<f32>, payload: Payload) -> Result<()> {
        // Write to WAL first
        {
            let mut wal = self.wal.write();
            wal.append(&WalEntry::insert(id, vector.clone(), payload.clone()))?;
        }

        // Apply to index and data file
        self.apply_insert_no_wal(id, vector, payload)
    }

    /// Inserts with auto-generated ID. Returns the ID.
    pub fn insert_auto(&self, vector: Vec<f32>, payload: Payload) -> Result<VectorId> {
        let id = {
            let mut next_id = self.next_id.write();
            let id = *next_id;
            *next_id += 1;
            id
        };

        self.insert(id, vector, payload)?;
        Ok(id)
    }

    /// Updates an existing vector.
    pub fn update(&self, id: VectorId, vector: Vec<f32>, payload: Payload) -> Result<()> {
        // Write to WAL first
        {
            let mut wal = self.wal.write();
            wal.append(&WalEntry::update(id, vector.clone(), payload.clone()))?;
        }

        self.apply_update_no_wal(id, vector, payload)
    }

    /// Deletes a vector.
    pub fn delete(&self, id: VectorId) -> Result<bool> {
        // Write to WAL first
        {
            let mut wal = self.wal.write();
            wal.append(&WalEntry::delete(id))?;
        }

        self.apply_delete_no_wal(id)
    }

    /// Searches for similar vectors.
    pub fn search(&self, query: &[f32], k: usize, filter: Option<Filter>) -> Vec<SearchResult> {
        let index = self.index.read();
        index.search(query, k, filter)
    }

    /// Gets a vector by ID.
    pub fn get(&self, id: VectorId) -> Option<(Vec<f32>, Payload)> {
        let index = self.index.read();
        index
            .get(id)
            .map(|(v, p)| (v.as_slice().to_vec(), p.clone()))
    }

    /// Returns the number of vectors.
    pub fn len(&self) -> usize {
        self.index.read().len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.index.read().is_empty()
    }

    /// Flushes all pending writes and performs a checkpoint.
    pub fn flush(&self) -> Result<()> {
        // Flush data file
        {
            let mut df = self.data_file.write();
            df.flush()?;
        }

        // Checkpoint WAL
        {
            let mut wal = self.wal.write();
            wal.checkpoint()?;
        }

        // Save metadata
        self.save_meta()?;

        Ok(())
    }

    /// Returns the collection path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    // Internal: apply insert without writing to WAL
    fn apply_insert_no_wal(&self, id: VectorId, vector: Vec<f32>, payload: Payload) -> Result<()> {
        // Write to data file
        let offset = {
            let mut df = self.data_file.write();
            df.append(id, &vector, &payload)?
        };

        // Update index
        {
            let mut index = self.index.write();
            // Remove if exists (for recovery idempotence)
            index.delete(id);
            index.insert(id, vector, payload)?;
        }

        // Track offset
        {
            let mut offsets = self.offsets.write();
            offsets.insert(id, offset);
        }

        // Update next_id
        {
            let mut next_id = self.next_id.write();
            *next_id = (*next_id).max(id + 1);
        }

        Ok(())
    }

    // Internal: apply update without writing to WAL
    fn apply_update_no_wal(&self, id: VectorId, vector: Vec<f32>, payload: Payload) -> Result<()> {
        // Mark old record as deleted
        {
            let offsets = self.offsets.read();
            if let Some(&offset) = offsets.get(&id) {
                let df = self.data_file.read();
                df.mark_deleted(offset)?;
            }
        }

        // Write new record to data file
        let offset = {
            let mut df = self.data_file.write();
            df.append(id, &vector, &payload)?
        };

        // Update index
        {
            let mut index = self.index.write();
            index.delete(id);
            index.insert(id, vector, payload)?;
        }

        // Track new offset
        {
            let mut offsets = self.offsets.write();
            offsets.insert(id, offset);
        }

        Ok(())
    }

    // Internal: apply delete without writing to WAL
    fn apply_delete_no_wal(&self, id: VectorId) -> Result<bool> {
        // Mark record as deleted in data file
        {
            let offsets = self.offsets.read();
            if let Some(&offset) = offsets.get(&id) {
                let df = self.data_file.read();
                df.mark_deleted(offset)?;
            }
        }

        // Remove from index
        let deleted = {
            let mut index = self.index.write();
            index.delete(id)
        };

        // Remove offset tracking
        {
            let mut offsets = self.offsets.write();
            offsets.remove(&id);
        }

        Ok(deleted)
    }

    // Saves metadata to disk
    fn save_meta(&self) -> Result<()> {
        let meta = CollectionMeta {
            dimension: self.config.dimension,
            metric: format!("{:?}", self.config.metric),
            vector_count: self.len() as u64,
            next_id: *self.next_id.read(),
        };

        let content = serde_json::to_string_pretty(&meta)
            .map_err(|e| Error::CollectionError(format!("serialize meta failed: {}", e)))?;

        let meta_path = self.path.join("meta.json");
        fs::write(&meta_path, content)
            .map_err(|e| Error::CollectionError(format!("write meta failed: {}", e)))?;

        Ok(())
    }
}

// Async API when tokio feature is enabled
#[cfg(feature = "async")]
mod async_api {
    use super::*;
    use std::sync::Arc;

    /// Async wrapper for Collection.
    ///
    /// Provides async versions of Collection methods using `spawn_blocking`
    /// for compatibility with async runtimes like Tokio.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use polarisdb_core::{AsyncCollection, CollectionConfig, DistanceMetric, Payload};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = CollectionConfig::new(384, DistanceMetric::Cosine);
    ///     let collection = AsyncCollection::open_or_create("./my_vectors", config).await.unwrap();
    ///
    ///     collection.insert(1, vec![0.1; 384], Payload::new()).await.unwrap();
    ///     let results = collection.search(&[0.1; 384], 10, None).await;
    /// }
    /// ```
    #[derive(Clone)]
    pub struct AsyncCollection {
        inner: Arc<Collection>,
    }

    impl AsyncCollection {
        /// Opens or creates a collection asynchronously.
        pub async fn open_or_create<P: AsRef<std::path::Path> + Send + 'static>(
            path: P,
            config: CollectionConfig,
        ) -> Result<Self> {
            let path = path.as_ref().to_path_buf();
            let collection =
                tokio::task::spawn_blocking(move || Collection::open_or_create(path, config))
                    .await
                    .map_err(|e| {
                        Error::CollectionError(format!("spawn_blocking failed: {}", e))
                    })??;

            Ok(Self {
                inner: Arc::new(collection),
            })
        }

        /// Wraps an existing Collection in an async wrapper.
        pub fn from_sync(collection: Collection) -> Self {
            Self {
                inner: Arc::new(collection),
            }
        }

        /// Inserts a vector asynchronously.
        pub async fn insert(&self, id: VectorId, vector: Vec<f32>, payload: Payload) -> Result<()> {
            let inner = Arc::clone(&self.inner);
            tokio::task::spawn_blocking(move || inner.insert(id, vector, payload))
                .await
                .map_err(|e| Error::CollectionError(format!("spawn_blocking failed: {}", e)))?
        }

        /// Inserts with auto-generated ID asynchronously.
        pub async fn insert_auto(&self, vector: Vec<f32>, payload: Payload) -> Result<VectorId> {
            let inner = Arc::clone(&self.inner);
            tokio::task::spawn_blocking(move || inner.insert_auto(vector, payload))
                .await
                .map_err(|e| Error::CollectionError(format!("spawn_blocking failed: {}", e)))?
        }

        /// Updates a vector asynchronously.
        pub async fn update(&self, id: VectorId, vector: Vec<f32>, payload: Payload) -> Result<()> {
            let inner = Arc::clone(&self.inner);
            tokio::task::spawn_blocking(move || inner.update(id, vector, payload))
                .await
                .map_err(|e| Error::CollectionError(format!("spawn_blocking failed: {}", e)))?
        }

        /// Deletes a vector asynchronously.
        pub async fn delete(&self, id: VectorId) -> Result<bool> {
            let inner = Arc::clone(&self.inner);
            tokio::task::spawn_blocking(move || inner.delete(id))
                .await
                .map_err(|e| Error::CollectionError(format!("spawn_blocking failed: {}", e)))?
        }

        /// Searches for similar vectors asynchronously.
        pub async fn search(
            &self,
            query: &[f32],
            k: usize,
            filter: Option<Filter>,
        ) -> Vec<SearchResult> {
            let inner = Arc::clone(&self.inner);
            let query = query.to_vec();
            tokio::task::spawn_blocking(move || inner.search(&query, k, filter))
                .await
                .unwrap_or_default()
        }

        /// Gets a vector by ID asynchronously.
        pub async fn get(&self, id: VectorId) -> Option<(Vec<f32>, Payload)> {
            let inner = Arc::clone(&self.inner);
            tokio::task::spawn_blocking(move || inner.get(id))
                .await
                .ok()
                .flatten()
        }

        /// Returns the number of vectors.
        pub fn len(&self) -> usize {
            self.inner.len()
        }

        /// Returns true if empty.
        pub fn is_empty(&self) -> bool {
            self.inner.is_empty()
        }

        /// Flushes all pending writes asynchronously.
        pub async fn flush(&self) -> Result<()> {
            let inner = Arc::clone(&self.inner);
            tokio::task::spawn_blocking(move || inner.flush())
                .await
                .map_err(|e| Error::CollectionError(format!("spawn_blocking failed: {}", e)))?
        }

        /// Returns reference to inner sync collection.
        pub fn inner(&self) -> &Collection {
            &self.inner
        }
    }
}

#[cfg(feature = "async")]
pub use async_api::AsyncCollection;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_collection_path() -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir()
            .join("polarisdb_test_col")
            .join(format!("col_{}_{}", std::process::id(), id));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn test_collection_create_and_insert() {
        let path = temp_collection_path();
        let config = CollectionConfig::new(3, DistanceMetric::Euclidean);

        let col = Collection::open_or_create(&path, config).unwrap();
        col.insert(
            1,
            vec![1.0, 2.0, 3.0],
            Payload::new().with_field("key", "val"),
        )
        .unwrap();

        assert_eq!(col.len(), 1);

        let (vec, payload) = col.get(1).unwrap();
        assert_eq!(vec, vec![1.0, 2.0, 3.0]);
        assert_eq!(payload.get_str("key"), Some("val"));

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn test_collection_persistence() {
        let path = temp_collection_path();
        let config = CollectionConfig::new(3, DistanceMetric::Euclidean);

        // Create and insert
        {
            let col = Collection::open_or_create(&path, config.clone()).unwrap();
            col.insert(1, vec![1.0, 2.0, 3.0], Payload::new()).unwrap();
            col.insert(2, vec![4.0, 5.0, 6.0], Payload::new()).unwrap();
            col.flush().unwrap();
        }

        // Reopen and verify
        {
            let col = Collection::open_or_create(&path, config).unwrap();
            assert_eq!(col.len(), 2);
            assert!(col.get(1).is_some());
            assert!(col.get(2).is_some());
        }

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn test_collection_delete() {
        let path = temp_collection_path();
        let config = CollectionConfig::new(3, DistanceMetric::Euclidean);

        let col = Collection::open_or_create(&path, config).unwrap();
        col.insert(1, vec![1.0, 2.0, 3.0], Payload::new()).unwrap();
        assert_eq!(col.len(), 1);

        col.delete(1).unwrap();
        assert_eq!(col.len(), 0);
        assert!(col.get(1).is_none());

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn test_collection_search() {
        let path = temp_collection_path();
        let config = CollectionConfig::new(3, DistanceMetric::Euclidean);

        let col = Collection::open_or_create(&path, config).unwrap();
        col.insert(1, vec![1.0, 0.0, 0.0], Payload::new()).unwrap();
        col.insert(2, vec![0.0, 1.0, 0.0], Payload::new()).unwrap();
        col.insert(3, vec![0.0, 0.0, 1.0], Payload::new()).unwrap();

        let results = col.search(&[1.0, 0.0, 0.0], 1, None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 1);

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn test_collection_update() {
        let path = temp_collection_path();
        let config = CollectionConfig::new(3, DistanceMetric::Euclidean);

        let col = Collection::open_or_create(&path, config).unwrap();
        col.insert(1, vec![1.0, 2.0, 3.0], Payload::new().with_field("v", 1))
            .unwrap();
        col.update(1, vec![4.0, 5.0, 6.0], Payload::new().with_field("v", 2))
            .unwrap();

        let (vec, payload) = col.get(1).unwrap();
        assert_eq!(vec, vec![4.0, 5.0, 6.0]);
        assert_eq!(payload.get_i64("v"), Some(2));

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn test_collection_recovery_after_crash() {
        let path = temp_collection_path();
        let config = CollectionConfig::new(3, DistanceMetric::Euclidean);

        // Simulate writes without checkpoint (mimics crash)
        {
            let col = Collection::open_or_create(&path, config.clone()).unwrap();
            col.insert(1, vec![1.0, 2.0, 3.0], Payload::new()).unwrap();
            col.insert(2, vec![4.0, 5.0, 6.0], Payload::new()).unwrap();
            // No flush() - simulates crash before checkpoint
        }

        // Reopen - should recover from WAL
        {
            let col = Collection::open_or_create(&path, config).unwrap();
            assert_eq!(col.len(), 2);
        }

        let _ = fs::remove_dir_all(&path);
    }
}
