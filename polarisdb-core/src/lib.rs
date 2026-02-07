//! PolarisDB Core - Embedded Vector Database Engine
//!
//! This crate provides the core functionality for PolarisDB, an embedded vector database
//! optimized for local AI and RAG (Retrieval-Augmented Generation) workloads.
//!
//! # Features
//!
//! - **Zero external runtime dependencies** - Pure Rust, embeddable anywhere
//! - **Multiple distance metrics** - Euclidean, Cosine, Dot Product, Hamming
//! - **Flexible payloads** - JSON-like metadata with indexed fields
//! - **Filtered search** - Combine vector similarity with metadata filters
//!
//! # Quick Start
//!
//! ```rust
//! use polarisdb_core::{BruteForceIndex, DistanceMetric, Payload};
//!
//! // Create an index for 384-dimensional vectors (e.g., sentence-transformers)
//! let mut index = BruteForceIndex::new(DistanceMetric::Cosine, 384);
//!
//! // Insert vectors with metadata
//! let embedding = vec![0.1; 384];
//! let payload = Payload::new().with_field("category", "documentation");
//! index.insert(1, embedding, payload).unwrap();
//!
//! // Search for similar vectors
//! let query = vec![0.1; 384];
//! let results = index.search(&query, 10, None);
//! ```

mod collection;
mod distance;
mod error;
mod filter;
mod index;
mod payload;
pub mod storage;
mod vector;

#[cfg(feature = "async")]
pub use collection::AsyncCollection;
pub use collection::{Collection, CollectionConfig};
pub use distance::{Distance, DistanceMetric};
pub use error::{Error, Result};
pub use filter::{BitmapIndex, Filter, FilterCondition};
pub use index::brute_force::BruteForceIndex;
pub use index::hnsw::{HnswConfig, HnswIndex};
pub use payload::Payload;
pub use vector::{Vector, VectorId};

/// Re-export commonly used types for convenience
pub mod prelude {
    pub use crate::{
        BitmapIndex, BruteForceIndex, Collection, CollectionConfig, Distance, DistanceMetric,
        Error, Filter, HnswConfig, HnswIndex, Payload, Result, Vector, VectorId,
    };
}
