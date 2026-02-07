//! # PolarisDB
//!
//! **A pure-Rust embedded vector database for local AI and RAG workloads.**
//!
//! PolarisDB provides fast, in-process vector similarity search optimized for:
//!
//! - **RAG applications** — Semantic retrieval for LLM context
//! - **Semantic search** — Find similar documents, images, or audio
//! - **Recommendations** — Content-based filtering with embeddings
//! - **Edge AI** — Local inference without cloud dependencies
//!
//! ## Features
//!
//! | Feature | Description |
//! |---------|-------------|
//! | **Dual Index Types** | BruteForce (exact) and HNSW (approximate) |
//! | **Distance Metrics** | Euclidean, Cosine, DotProduct, Hamming |
//! | **Filtered Search** | Combine similarity with metadata conditions |
//! | **Persistence** | WAL-based crash-safe durability |
//! | **Async API** | Tokio-compatible async operations (feature flag) |
//!
//! ## Quick Start
//!
//! ### In-Memory Index
//!
//! For quick prototyping or small datasets:
//!
//! ```rust
//! use polarisdb::prelude::*;
//!
//! // Create a 384-dimensional index (common embedding size)
//! let mut index = BruteForceIndex::new(DistanceMetric::Cosine, 384);
//!
//! // Insert vectors with metadata
//! let embedding = vec![0.1; 384];
//! let payload = Payload::new()
//!     .with_field("title", "Introduction to Rust")
//!     .with_field("category", "programming");
//! index.insert(1, embedding, payload).unwrap();
//!
//! // Search for similar vectors
//! let query = vec![0.1; 384];
//! let results = index.search(&query, 10, None);
//!
//! assert!(!results.is_empty());
//! ```
//!
//! ### Persistent Collection
//!
//! For production use with crash-safe durability:
//!
//! ```no_run
//! use polarisdb::prelude::*;
//!
//! fn main() -> Result<()> {
//!     // Open or create a persistent collection
//!     let config = CollectionConfig::new(384, DistanceMetric::Cosine);
//!     let collection = Collection::open_or_create("./my_vectors", config)?;
//!
//!     // Insert with automatic ID generation
//!     let id = collection.insert_auto(
//!         vec![0.1; 384],
//!         Payload::new().with_field("doc", "example"),
//!     )?;
//!
//!     // Search with filters
//!     let filter = Filter::field("doc").eq("example");
//!     let results = collection.search(&[0.1; 384], 10, Some(filter));
//!
//!     // Flush to ensure durability
//!     collection.flush()?;
//!     Ok(())
//! }
//! ```
//!
//! ### HNSW for Large Datasets
//!
//! For millions of vectors with fast approximate search:
//!
//! ```rust
//! use polarisdb::prelude::*;
//!
//! let config = HnswConfig {
//!     m: 16,              // Connections per node (higher = better recall)
//!     m_max0: 32,         // Layer 0 connections
//!     ef_construction: 100, // Build-time beam width
//!     ef_search: 50,      // Search-time beam width
//! };
//!
//! let mut index = HnswIndex::new(DistanceMetric::Cosine, 128, config);
//!
//! // Insert 1000 vectors
//! for i in 0..1000 {
//!     let v: Vec<f32> = (0..128).map(|j| ((i * 128 + j) as f32).sin()).collect();
//!     index.insert(i, v, Payload::new()).unwrap();
//! }
//!
//! // Search is ~10x faster than brute-force
//! let query: Vec<f32> = (0..128).map(|j| (j as f32).cos()).collect();
//! let results = index.search(&query, 10, None, None);
//! ```
//!
//! ### Filtered Search
//!
//! Combine vector similarity with metadata conditions:
//!
//! ```rust
//! use polarisdb::prelude::*;
//!
//! let mut index = BruteForceIndex::new(DistanceMetric::Cosine, 3);
//!
//! // Insert vectors with categories
//! index.insert(1, vec![1.0, 0.0, 0.0], Payload::new().with_field("category", "A")).unwrap();
//! index.insert(2, vec![0.9, 0.1, 0.0], Payload::new().with_field("category", "B")).unwrap();
//! index.insert(3, vec![0.8, 0.2, 0.0], Payload::new().with_field("category", "A")).unwrap();
//!
//! // Search only category A
//! let filter = Filter::field("category").eq("A");
//! let results = index.search(&[1.0, 0.0, 0.0], 10, Some(filter));
//!
//! assert_eq!(results.len(), 2); // Only category A results
//! ```
//!
//! ## Crate Features
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `async` | Enables `AsyncCollection` for tokio compatibility |
//!
//! Enable features in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! polarisdb = { version = "0.1", features = ["async"] }
//! ```
//!
//! ## Architecture
//!
//! PolarisDB is organized into two crates:
//!
//! - **`polarisdb-core`** — Core library with no async runtime dependency
//! - **`polarisdb`** — Main crate that re-exports everything
//!
//! ### Core Components
//!
//! - [`BruteForceIndex`] — Exact nearest neighbor search (O(n))
//! - [`HnswIndex`] — Approximate nearest neighbor search (O(log n))
//! - [`Collection`] — Persistent storage with WAL durability
//! - [`Filter`] — Metadata filter expressions
//! - [`BitmapIndex`] — Roaring bitmap for fast pre-filtering
//! - [`Payload`] — JSON-like metadata for vectors
//!
//! ## Performance
//!
//! Benchmarked on 10,000 128-dimensional vectors:
//!
//! | Index | Search Time | Recall |
//! |-------|-------------|--------|
//! | BruteForce | 2.8 ms | 100% |
//! | HNSW | 300 µs | 99%+ |
//!
//! ## Error Handling
//!
//! All fallible operations return [`Result<T>`](crate::Result), which uses
//! the [`Error`] enum for error types.
//!
//! ## Thread Safety
//!
//! - [`BruteForceIndex`] and [`HnswIndex`] are not thread-safe; use external synchronization
//! - [`Collection`] uses internal `RwLock` for thread-safe operations
//! - [`AsyncCollection`] is `Clone` and safe to share across tasks

// Re-export everything from core
pub use polarisdb_core::*;
