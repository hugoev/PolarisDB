# Changelog

All notable changes to PolarisDB will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Async API with `AsyncCollection` wrapper (behind `async` feature flag)
- Integration examples for Ollama RAG pipeline
- Pre-filtering benchmarks

### Changed
- Renamed `Filter::not()` to `Filter::negate()` for clarity
- Renamed `FieldFilter::is_in()` to `FieldFilter::contained_in()`

## [0.1.0] - 2024-XX-XX

### Added

#### Core Features
- **Vector Storage**: Efficient in-memory storage for high-dimensional vectors
- **Distance Metrics**: Euclidean, Cosine, Dot Product, and Hamming distance
- **Payload Support**: JSON-like metadata attached to each vector
- **Filtered Search**: Combine similarity search with metadata conditions

#### Index Types
- **BruteForceIndex**: Exact nearest neighbor search, ideal for small datasets
- **HnswIndex**: Approximate nearest neighbor with 9x speedup over brute-force

#### Persistence
- **Write-Ahead Log (WAL)**: Crash-safe durability for all operations
- **Collection API**: High-level persistent collection with automatic recovery
- **Memory-mapped Files**: Efficient disk access without loading entire dataset

#### Filtering
- **Filter Expressions**: eq, ne, gt, gte, lt, lte, contained_in, contains, exists
- **Boolean Logic**: Combine filters with AND, OR, NOT
- **BitmapIndex**: Roaring bitmap-based pre-filtering for selective queries

### Performance
- HNSW search: ~300µs for 10K vectors (vs 2.8ms brute-force)
- Concurrent async inserts: 1000 vectors in ~60ms
- Memory-efficient storage with append-only data files

### Documentation
- Comprehensive README with examples
- Rustdoc for all public APIs
- Example integrations (HNSW demo, async demo, Ollama RAG)

[Unreleased]: https://github.com/hugoev/polarisdb/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/hugoev/polarisdb/releases/tag/v0.1.0
