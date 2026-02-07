# PolarisDB 🌟

**A pure-Rust embedded vector database for local AI and RAG workloads.**

[![CI](https://github.com/yourusername/polarisdb/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/polarisdb/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/polarisdb.svg)](https://crates.io/crates/polarisdb)
[![Documentation](https://docs.rs/polarisdb/badge.svg)](https://docs.rs/polarisdb)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

PolarisDB is designed for:

- 🏠 **Local-first AI** — Runs entirely on-device, no server required
- 🔒 **Privacy** — Your embeddings never leave your machine
- ⚡ **Performance** — HNSW index with 9x speedup over brute-force
- 💾 **Persistence** — WAL-based durability with crash recovery
- 🎯 **Simplicity** — Single crate, minimal dependencies

Perfect for RAG applications, semantic search, recommendation systems, and AI-powered local apps.

## Quick Start

```toml
[dependencies]
polarisdb = "0.1"
```

```rust
use polarisdb::prelude::*;

fn main() -> Result<()> {
    // Create a persistent collection
    let config = CollectionConfig::new(384, DistanceMetric::Cosine);
    let collection = Collection::open_or_create("./my_vectors", config)?;

    // Insert vectors with metadata
    let embedding = vec![0.1; 384];
    let payload = Payload::new()
        .with_field("title", "Introduction to Rust")
        .with_field("category", "programming");
    
    collection.insert(1, embedding, payload)?;

    // Search for similar vectors
    let query = vec![0.1; 384];
    let results = collection.search(&query, 5, None);

    for result in results {
        println!("ID: {}, Distance: {:.4}", result.id, result.distance);
    }

    collection.flush()?;
    Ok(())
}
```

## Features

### 🔍 Index Types

| Index | Use Case | Performance |
|-------|----------|-------------|
| `BruteForceIndex` | Small datasets (<10K) | O(n) exact search |
| `HnswIndex` | Large datasets | O(log n) approximate search |

### 📏 Distance Metrics

- **Euclidean (L2)** — Standard geometric distance
- **Cosine** — Angular similarity (best for text embeddings)
- **Dot Product** — Inner product similarity
- **Hamming** — For binary vectors

### 🎯 Filtered Search

Combine vector similarity with metadata filters:

```rust
let filter = Filter::field("category").eq("documentation")
    .and(Filter::field("year").gte(2024));

let results = index.search(&query, 10, Some(filter));
```

### ⚡ Pre-filtering with Bitmap Index

For highly selective filters, use bitmap-accelerated search:

```rust
let mut bitmap = BitmapIndex::new();
bitmap.insert(id, &payload);

let valid_ids = bitmap.query(&filter);
let results = hnsw.search_with_bitmap(&query, 10, None, &valid_ids);
```

### 🔄 Async API

Enable the `async` feature for tokio compatibility:

```toml
polarisdb = { version = "0.1", features = ["async"] }
```

```rust
use polarisdb::AsyncCollection;

#[tokio::main]
async fn main() {
    let collection = AsyncCollection::open_or_create("./data", config).await?;
    collection.insert(1, embedding, payload).await?;
    let results = collection.search(&query, 10, None).await;
}
```

## Examples

```bash
# HNSW performance demo
cargo run --release --example hnsw_demo

# Async concurrent insertion
cargo run --release --example async_demo --features async

# Pre-filtering benchmark
cargo run --release --example prefilter_demo

# Ollama RAG integration (requires Ollama)
cargo run --release --example ollama_rag
```

## Performance

Benchmarked on 10,000 128-dimensional vectors:

| Operation | Time |
|-----------|------|
| HNSW Insert (batch) | 45ms |
| HNSW Search (k=10) | 299µs |
| Brute-force Search | 2.8ms |
| **Speedup** | **9.4x** |

## Architecture

```
polarisdb/                 # Main crate (re-exports)
├── examples/              # Usage examples
└── src/lib.rs

polarisdb-core/            # Core implementation
├── src/
│   ├── collection.rs      # Persistent collection API
│   ├── distance.rs        # Distance metrics
│   ├── filter/            # Filter expressions + bitmap index
│   ├── index/             # BruteForce + HNSW indexes
│   ├── payload.rs         # JSON-like metadata
│   ├── storage/           # WAL + data files
│   └── vector.rs          # Vector types
```

## Roadmap

- [x] **v0.1** — Brute-force index, distance metrics, filtered search
- [x] **v0.2** — On-disk persistence, WAL, crash recovery
- [x] **v0.3** — HNSW index for approximate nearest neighbor search
- [x] **v0.4** — Bitmap pre-filtering, async API
- [ ] **v0.5** — Product quantization, WASM support
- [ ] **v1.0** — Stable API, comprehensive benchmarks

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE](LICENSE))

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Run tests (`cargo test --workspace`)
4. Commit your changes (`git commit -m 'Add amazing feature'`)
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request
