# Rust API Reference

For complete API documentation, see [docs.rs/polarisdb](https://docs.rs/polarisdb).

## Quick Reference

### Core Types

```rust
use polarisdb::prelude::*;
```

| Type | Description |
|------|-------------|
| `BruteForceIndex` | Exact nearest neighbor search |
| `HnswIndex` | Approximate search with HNSW |
| `Collection` | Persistent storage with WAL |
| `AsyncCollection` | Async version (requires `async` feature) |
| `Payload` | JSON-like metadata |
| `Filter` | Filter expressions |
| `DistanceMetric` | Cosine, Euclidean, DotProduct, Hamming |

### Example

```rust
use polarisdb::prelude::*;

fn main() -> Result<()> {
    // Create collection
    let config = CollectionConfig::new(384, DistanceMetric::Cosine);
    let collection = Collection::open_or_create("./my_vectors", config)?;

    // Insert
    let payload = Payload::new()
        .with_field("category", "docs")
        .with_field("year", 2024);
    collection.insert(1, vec![0.1; 384], payload)?;

    // Search
    let results = collection.search(&vec![0.1; 384], 10, None);

    // Flush
    collection.flush()?;
    Ok(())
}
```

## Features

Enable in `Cargo.toml`:

```toml
[dependencies]
polarisdb = { version = "0.1", features = ["async"] }
```

| Feature | Description |
|---------|-------------|
| `async` | Tokio-based async API |
