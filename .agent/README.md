# PolarisDB - AI Agent Development Guide

This directory contains context and instructions for AI coding agents working on PolarisDB.

## Quick Context

**PolarisDB** is a pure-Rust embedded vector database for local AI and RAG workloads.

### Tech Stack
- **Language**: Rust (edition 2021)
- **Build System**: Cargo with workspace
- **Key Dependencies**: serde, thiserror, memmap2, roaring, parking_lot, tokio (optional)

### Repository Structure

```
polarisdb/
├── polarisdb-core/          # Core library (no runtime deps)
│   └── src/
│       ├── lib.rs           # Crate root, exports
│       ├── collection.rs    # Persistent Collection + AsyncCollection
│       ├── distance.rs      # Distance metrics (Euclidean, Cosine, etc.)
│       ├── error.rs         # Error types
│       ├── payload.rs       # JSON-like metadata
│       ├── vector.rs        # Vector types
│       ├── filter/          # Filter expressions
│       │   ├── mod.rs       # Filter DSL
│       │   └── bitmap_index.rs  # Roaring bitmap pre-filtering
│       ├── index/           # Index implementations
│       │   ├── mod.rs
│       │   ├── brute_force.rs   # Exact NN (O(n))
│       │   └── hnsw.rs          # Approximate NN (O(log n))
│       └── storage/         # Persistence layer
│           ├── mod.rs
│           ├── wal.rs       # Write-ahead log
│           └── data_file.rs # Vector storage
│
├── polarisdb/               # Main crate (re-exports core)
│   ├── src/lib.rs
│   └── examples/            # Usage examples
│       ├── hnsw_demo.rs
│       ├── async_demo.rs
│       ├── prefilter_demo.rs
│       └── ollama_rag.rs
│
└── .agent/                  # AI agent documentation (you are here)
```

## Key Concepts

### Vector Storage Model

```
VectorId (u64) → Vector + Payload + Index Position
```

- **VectorId**: Unique u64 identifier
- **Vector**: `Vec<f32>` of fixed dimension (set at collection creation)
- **Payload**: JSON-like metadata (`serde_json::Value` backed)

### Index Types

| Type | File | Complexity | Use Case |
|------|------|------------|----------|
| `BruteForceIndex` | `index/brute_force.rs` | O(n) exact | <10K vectors |
| `HnswIndex` | `index/hnsw.rs` | O(log n) approx | Millions of vectors |

### Persistence Model

```
Collection Directory/
├── metadata.json      # Config (dimension, metric)
├── data.bin           # Append-only vector storage
└── wal.bin            # Write-ahead log for crash safety
```

**WAL Protocol**:
1. Write operation to WAL
2. Apply to in-memory index
3. Periodically checkpoint to data file

### Thread Safety

- `Collection`: Thread-safe via `RwLock`
- `BruteForceIndex`/`HnswIndex`: NOT thread-safe, wrap in mutex if shared
- `AsyncCollection`: Clone-safe, uses `Arc<Collection>` internally

## Development Workflows

See `.agent/workflows/` for specific task workflows:
- `add-feature.md` - Adding new features
- `fix-bug.md` - Bug fixing process
- `add-index.md` - Adding new index types
- `run-tests.md` - Testing procedures

## Common Commands

```bash
# Build
cargo build --workspace

# Test everything
cargo test --workspace --all-features

# Test with output
cargo test --workspace -- --nocapture

# Specific test
cargo test test_name

# Clippy (must pass)
cargo clippy --workspace --all-features -- -D warnings

# Format
cargo fmt --all

# Generate docs
cargo doc --workspace --no-deps --open

# Run example
cargo run --release --example hnsw_demo

# Run async example
cargo run --release --example async_demo --features async

# Benchmarks
cargo bench
```

## Code Style

### Error Handling
- Use `Result<T>` from `error.rs` for fallible operations
- Add new error variants to `Error` enum if needed
- Never panic in library code

### Documentation
- All public items need rustdoc
- Include `# Example` sections with runnable code
- Use `/// # Errors` section for fallible functions

### Testing
- Unit tests in `#[cfg(test)]` modules within each file
- Doc tests for examples
- Integration tests in `tests/` directory

### Naming Conventions
- Types: `PascalCase`
- Functions/methods: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

## Architecture Decisions

### Why Brute Force + HNSW?
- BruteForce: Simple, exact, good for small datasets
- HNSW: Proven approximate NN algorithm, sublinear search

### Why WAL?
- Crash safety without full sync on every write
- Enables batch operations with durability guarantee

### Why Roaring Bitmaps?
- Compressed representation for ID sets
- Fast set operations (AND, OR, NOT)
- Proven in databases (Apache Druid, ClickHouse)

### Why No Async in Core?
- Keeps core dependency-free
- AsyncCollection wraps sync ops with `spawn_blocking`
- Users can choose their async runtime

## Adding New Features

1. Design API in `polarisdb-core/src/`
2. Export in `lib.rs`
3. Add tests
4. Update docs
5. Add example if user-facing
6. Update CHANGELOG.md
