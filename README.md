<p align="center">
  <img src="https://raw.githubusercontent.com/hugoev/polarisdb/main/assets/logo.svg" alt="PolarisDB Logo" width="200">
</p>

<h1 align="center">PolarisDB</h1>

<p align="center">
  <strong>A pure-Rust embedded vector database for local AI and RAG workloads</strong>
</p>

<p align="center">
  <a href="https://github.com/hugoev/polarisdb/actions/workflows/ci.yml">
    <img src="https://github.com/hugoev/polarisdb/actions/workflows/ci.yml/badge.svg" alt="CI">
  </a>
  <a href="https://crates.io/crates/polarisdb">
    <img src="https://img.shields.io/crates/v/polarisdb.svg" alt="Crates.io">
  </a>
  <a href="https://docs.rs/polarisdb">
    <img src="https://docs.rs/polarisdb/badge.svg" alt="Documentation">
  </a>
  <a href="https://github.com/hugoev/polarisdb/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg" alt="License">
  </a>
  <a href="https://pypi.org/project/polarisdb">
    <img src="https://img.shields.io/pypi/v/polarisdb.svg" alt="PyPI">
  </a>
  <a href="https://github.com/hugoev/polarisdb">
    <img src="https://img.shields.io/github/stars/hugoev/polarisdb?style=social" alt="GitHub Stars">
  </a>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#examples">Examples</a> •
  <a href="#performance">Performance</a> •
  <a href="#documentation">Docs</a> •
  <a href="#contributing">Contributing</a>
</p>

---

## Why PolarisDB?

PolarisDB is built for developers who need **fast, local vector search** without the complexity of external services.

| Feature | PolarisDB | Cloud Solutions |
|---------|-----------|-----------------|
| 🏠 **Runs locally** | ✅ | ❌ Requires internet |
| 🔒 **Data privacy** | ✅ Your machine | ❌ Third-party servers |
| ⚡ **Zero latency** | ✅ In-process | ❌ Network overhead |
| 💰 **Cost** | ✅ Free | 💵 Pay per query |
| 🦀 **Pure Rust** | ✅ No FFI | ⚠️ Often C++ bindings |

**Perfect for:**
- 🤖 RAG applications with LLMs
- 🔍 Semantic search engines  
- 💡 Recommendation systems
- 📱 Mobile/edge AI applications
- 🎮 Game AI with embeddings

## Features

### 🚀 High-Performance Indexing

| Index Type | Use Case | Complexity |
|------------|----------|------------|
| **BruteForce** | Small datasets (<10K vectors) | O(n) exact |
| **HNSW** | Large datasets (millions) | O(log n) approximate |

### 📏 Distance Metrics

```rust
DistanceMetric::Euclidean   // L2 distance
DistanceMetric::Cosine      // Angular similarity (text embeddings)
DistanceMetric::DotProduct  // Maximum inner product
DistanceMetric::Hamming     // Binary vectors
```

### 🎯 Powerful Filtering

Combine vector similarity with metadata conditions:

```rust
// Find similar documents from 2024 in the "AI" category
let filter = Filter::field("category").eq("AI")
    .and(Filter::field("year").gte(2024));

let results = index.search(&query_embedding, 10, Some(filter));
```

### 💾 Durable Persistence

- **Write-Ahead Log (WAL)** for crash safety
- **Automatic recovery** on restart
- **Memory-mapped files** for efficient disk access

### ⚡ Async-Ready

```rust
// Enable with: polarisdb = { version = "0.1", features = ["async"] }
let collection = AsyncCollection::open_or_create("./data", config).await?;
collection.insert(id, embedding, payload).await?;
let results = collection.search(&query, 10, None).await;
```

### 🐍 Python Bindings

```python
import polarisdb

# Persistent Collection
col = polarisdb.Collection.open_or_create("./data/my_col", 384, "cosine")
col.insert(1, [0.1, 0.2, ...])
results = col.search([0.1, 0.2, ...], 5)
```

### 🦜 LangChain Integration

Use PolarisDB as a vector store in your RAG pipelines:

```python
from polarisdb.langchain import PolarisDBVectorStore
from langchain_openai import OpenAIEmbeddings

# Create vector store from documents
vectorstore = PolarisDBVectorStore.from_texts(
    texts=["Document 1", "Document 2", "Document 3"],
    embedding=OpenAIEmbeddings(),
    collection_path="./my_vectors",
)

# Similarity search
docs = vectorstore.similarity_search("query", k=3)

# Use as retriever in RAG chain
retriever = vectorstore.as_retriever()
```

See [`examples/langchain_rag.py`](./examples/langchain_rag.py) for a complete RAG example.

### 🌐 HTTP Server

Run the standalone server:

```bash
cargo run -p polarisdb-server
```

Integrate via REST API:

```bash
curl -X POST http://localhost:8080/collections/my_col/search \
  -d '{"vector": [0.1, ...], "k": 5}'
```

## Quick Start

Add PolarisDB to your `Cargo.toml`:

```toml
[dependencies]
polarisdb = "0.1"
```

### Basic Usage

```rust
use polarisdb::prelude::*;

fn main() -> Result<()> {
    // Create a collection for 384-dimensional embeddings
    let config = CollectionConfig::new(384, DistanceMetric::Cosine);
    let collection = Collection::open_or_create("./my_vectors", config)?;

    // Insert vectors with metadata
    let embedding = get_embedding("Introduction to Rust"); // Your embedding function
    let payload = Payload::new()
        .with_field("title", "Introduction to Rust")
        .with_field("category", "programming")
        .with_field("year", 2024);
    
    collection.insert(1, embedding, payload)?;

    // Search for similar vectors
    let query = get_embedding("Rust programming tutorial");
    let results = collection.search(&query, 5, None);

    for result in results {
        if let Some(payload) = &result.payload {
            println!(
                "Found: {} (distance: {:.4})",
                payload.get_str("title").unwrap_or("Unknown"),
                result.distance
            );
        }
    }

    collection.flush()?; // Ensure durability
    Ok(())
}
```

### High-Performance HNSW Index

For millions of vectors, use the HNSW index:

```rust
let config = HnswConfig {
    m: 16,              // Connections per node
    m_max0: 32,         // Connections at layer 0
    ef_construction: 100, // Build-time beam width
    ef_search: 50,      // Search-time beam width
};

let mut index = HnswIndex::new(DistanceMetric::Cosine, 384, config);

// Insert vectors
for (id, embedding, metadata) in documents {
    index.insert(id, embedding, metadata)?;
}

// Search with ~9x speedup over brute-force
let results = index.search(&query, 10, None, None);
```

### Pre-Filtered Search with Bitmap Index

For highly selective filters:

```rust
// Build a bitmap index alongside your vector index
let mut bitmap = BitmapIndex::new();
let mut hnsw = HnswIndex::new(DistanceMetric::Cosine, 384, config);

for (id, embedding, payload) in documents {
    hnsw.insert(id, embedding.clone(), payload.clone())?;
    bitmap.insert(id, &payload);
}

// Query with bitmap pre-filtering
let filter = Filter::field("category").eq("AI");
let valid_ids = bitmap.query(&filter);
let results = hnsw.search_with_bitmap(&query, 10, None, &valid_ids);
```

## Examples

Run the included examples:

```bash
# HNSW performance benchmark (9x speedup demo)
cargo run --release --example hnsw_demo

# Async concurrent insertions
cargo run --release --example async_demo --features async

# Pre-filtering benchmark
cargo run --release --example prefilter_demo

# Ollama RAG integration (requires Ollama running)
cargo run --release --example ollama_rag
```

## Performance

Benchmarked on M1 MacBook Pro with 128-dimensional vectors (Cosine distance):

| Operation | Vectors | Time | Throughput |
|-----------|---------|------|------------|
| **Brute Force Search** | 1,000 | 325 µs | 3.1M elem/s |
| **Brute Force Search** | 10,000 | 5.5 ms | 1.8M elem/s |
| **Brute Force Search** | 50,000 | 34 ms | 1.5M elem/s |

### Distance Calculations (SIMD-optimized)

| Dimension | Dot Product | Throughput |
|-----------|-------------|------------|
| 128 | 81 ns | 1.6 Gelem/s |
| 384 | 155 ns | 2.5 Gelem/s |
| 768 | 154 ns | 5.0 Gelem/s |
| 1536 | 304 ns | 5.1 Gelem/s |

### Scaling Projections

| Vectors | HNSW Search Time | Memory |
|---------|------------------|--------|
| 10K | ~500 µs | 12 MB |
| 100K | ~600 µs | 120 MB |
| 1M | ~800 µs | 1.2 GB |

*HNSW search time scales logarithmically. Brute force scales linearly.*

## Documentation

- 📖 **[API Reference](https://docs.rs/polarisdb)** — Complete rustdoc documentation
- 📚 **[Examples](./polarisdb/examples/)** — Working code examples
- 🔧 **[CONTRIBUTING.md](./CONTRIBUTING.md)** — Development guide
- 📝 **[CHANGELOG.md](./CHANGELOG.md)** — Version history

## Architecture

```
polarisdb/
├── polarisdb-core/          # Core library (distance, indexing, storage)
│   ├── index/               # BruteForce, HNSW implementations
│   ├── storage/             # WAL, persistence layer
│   └── filter/              # Bitmap filtering
│
├── polarisdb/               # Main crate (convenient re-exports)
├── polarisdb-server/        # HTTP API server (axum)
└── py/                      # Python bindings (pyo3 + maturin)
```

## Roadmap

- [x] **v0.1** — Core functionality, brute-force search, filtering
- [x] **v0.2** — WAL persistence, crash recovery
- [x] **v0.3** — HNSW approximate nearest neighbor
- [x] **v0.4** — Bitmap pre-filtering, async API, SIMD acceleration
- [x] **v0.5** — Python bindings, PyPI release
- [ ] **v0.6** — LangChain integration, multi-vector queries
- [ ] **v1.0** — Stable API, product quantization, hybrid search

## Comparison

| Feature | PolarisDB | LanceDB | Chroma | Qdrant |
|---------|-----------|---------|--------|--------|
| Language | Rust | Rust/Python | Python | Rust |
| Embedded | ✅ | ✅ | ⚠️ | ❌ |
| Python Bindings | ✅ PyPI | ✅ | ✅ Native | ✅ |
| HNSW | ✅ | ✅ | ✅ | ✅ |
| Persistence | ✅ WAL | ✅ Lance | ✅ SQLite | ✅ Raft |
| Filtering | ✅ Bitmap | ✅ | ✅ | ✅ |
| Async | ✅ | ✅ | ❌ | ✅ |
| SIMD | ✅ | ✅ | ❌ | ✅ |

## Contributing

We welcome contributions! See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

```bash
# Clone and build
git clone https://github.com/hugoev/polarisdb.git
cd polarisdb
cargo build

# Run tests
cargo test --workspace --all-features

# Run clippy
cargo clippy -- -D warnings
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- [HNSW Paper](https://arxiv.org/abs/1603.09320) — Hierarchical Navigable Small World graphs
- [Roaring Bitmaps](https://roaringbitmap.org/) — Compressed bitmap data structure
- The Rust community 🦀

---

<p align="center">
  <sub>Built with ❤️ by the PolarisDB contributors</sub>
</p>
