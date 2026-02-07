//! # PolarisDB Core
//!
//! Core library for PolarisDB — a pure-Rust embedded vector database.
//!
//! This crate provides the foundational components for vector storage and similarity search.
//! It is designed to be lightweight with no mandatory runtime dependencies.
//!
//! ## Crate Features
//!
//! - `async` - Enables [`AsyncCollection`] for tokio-compatible async operations
//!
//! ## Core Types
//!
//! ### Indexes
//!
//! - [`BruteForceIndex`] - Exact nearest neighbor search, O(n) complexity
//! - [`HnswIndex`] - Approximate nearest neighbor using HNSW graphs, O(log n)
//!
//! ### Persistence
//!
//! - [`Collection`] - Thread-safe persistent storage with WAL durability
//! - [`AsyncCollection`] - Async wrapper for tokio compatibility (requires `async` feature)
//!
//! ### Filtering
//!
//! - [`Filter`] - Declarative filter expressions for metadata conditions
//! - [`BitmapIndex`] - Roaring bitmap index for fast pre-filtering
//!
//! ### Types
//!
//! - [`Vector`] - Owned vector data
//! - [`VectorId`] - Unique identifier for vectors (u64)
//! - [`Payload`] - JSON-like metadata attached to vectors
//! - [`DistanceMetric`] - Supported distance functions

pub mod collection;
pub mod distance;
pub mod error;
pub mod filter;
pub mod index;
pub mod payload;
pub mod storage;
pub mod vector;

// Re-exports for convenient access
pub use collection::{Collection, CollectionConfig};
#[cfg(feature = "async")]
pub use collection::AsyncCollection;
pub use distance::{Distance, DistanceMetric};
pub use error::{Error, Result};
pub use filter::{BitmapIndex, Filter, FilterCondition};
pub use index::brute_force::{BruteForceIndex, SearchResult};
pub use index::hnsw::{HnswConfig, HnswIndex};
pub use payload::Payload;
pub use vector::{Vector, VectorId};

/// Re-export commonly used types for convenience.
///
/// # Example
///
/// ```rust
/// use polarisdb_core::prelude::*;
///
/// let mut index = BruteForceIndex::new(DistanceMetric::Euclidean, 3);
/// let payload = Payload::new().with_field("key", "value");
/// index.insert(1, vec![1.0, 2.0, 3.0], payload).unwrap();
/// ```
pub mod prelude {
    pub use crate::{
        BitmapIndex, BruteForceIndex, Collection, CollectionConfig, Distance, DistanceMetric,
        Error, Filter, HnswConfig, HnswIndex, Payload, Result, Vector, VectorId,
    };
}
