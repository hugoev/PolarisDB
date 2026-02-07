//! PolarisDB - Embedded Vector Database for Local AI
//!
//! PolarisDB is a pure-Rust, embeddable vector database optimized for
//! local AI and RAG (Retrieval-Augmented Generation) workloads.
//!
//! # Features
//!
//! - **Pure Rust** - No external runtime dependencies
//! - **Embedded** - Runs in-process, no separate server needed
//! - **Fast** - Optimized distance calculations with SIMD support
//! - **Flexible** - JSON-like payloads with filtered search
//!
//! # Quick Start
//!
//! ```rust
//! use polarisdb::prelude::*;
//!
//! // Create an index for 384-dimensional vectors
//! let mut index = BruteForceIndex::new(DistanceMetric::Cosine, 384);
//!
//! // Insert vectors with metadata
//! let embedding = vec![0.1; 384];
//! let payload = Payload::new().with_field("title", "Hello World");
//! index.insert(1, embedding, payload).unwrap();
//!
//! // Search for similar vectors
//! let query = vec![0.1; 384];
//! let results = index.search(&query, 10, None);
//!
//! println!("Found {} results", results.len());
//! ```

// Re-export everything from core
pub use polarisdb_core::*;
