# PolarisDB Architecture Deep Dive

## Overview

PolarisDB follows a layered architecture:

```
┌─────────────────────────────────────────────────────┐
│                   Bindings & API                    │
│      Python (pyo3)     │     HTTP API (axum)        │
├─────────────────────────────────────────────────────┤
│                   User API Layer                     │
│  Collection, AsyncCollection, BruteForceIndex, etc. │
├─────────────────────────────────────────────────────┤
│                   Index Layer                        │
│         BruteForceIndex    │    HnswIndex           │
├─────────────────────────────────────────────────────┤
│                 Filtering Layer                      │
│              Filter    │    BitmapIndex             │
├─────────────────────────────────────────────────────┤
│                  Storage Layer                       │
│              WAL    │    DataFile    │    Metadata  │
├─────────────────────────────────────────────────────┤
│                   Core Types                         │
│       Vector, VectorId, Payload, DistanceMetric     │
└─────────────────────────────────────────────────────┘
```

## Core Types

### VectorId (`vector.rs`)
```rust
pub type VectorId = u64;
```
Simple u64 for maximum compatibility and performance.

### Vector (`vector.rs`)
```rust
pub struct Vector {
    pub id: VectorId,
    pub data: Vec<f32>,
}
```
Owned vector with ID. Used internally; users typically pass `Vec<f32>` directly.

### Payload (`payload.rs`)
```rust
pub struct Payload {
    data: HashMap<String, Value>,  // serde_json::Value
}
```
JSON-like metadata. Supports arbitrary nesting. Used for filtering.

### DistanceMetric (`distance.rs`)
```rust
pub enum DistanceMetric {
    Euclidean,   // L2: sqrt(Σ(a-b)²)
    Cosine,      // 1 - (a·b / (|a||b|))
    DotProduct,  // -(a·b) (negated for min-heap)
    Hamming,     // Σ(a≠b)
}
```

## Index Layer

### BruteForceIndex (`index/brute_force.rs`)

**Data Structure:**
```rust
struct BruteForceIndex {
    dimension: usize,
    metric: DistanceMetric,
    vectors: HashMap<VectorId, (Vec<f32>, Payload)>,
}
```

**Search Algorithm:**
1. Iterate all vectors
2. Calculate distance to query
3. Apply filter (if any)
4. Keep top-k using BinaryHeap

**Complexity:** O(n) search, O(1) insert

### HnswIndex (`index/hnsw.rs`)

**Data Structure:**
```rust
struct HnswIndex {
    nodes: HashMap<VectorId, HnswNode>,
    entry_point: Option<VectorId>,
    max_level: usize,
    config: HnswConfig,
}

struct HnswNode {
    vector: Vec<f32>,
    payload: Payload,
    level: usize,
    neighbors: Vec<Vec<VectorId>>,  // per-layer neighbors
}
```

**Search Algorithm (Greedy):**
1. Start at entry point, top layer
2. Greedy descent: move to closest neighbor
3. When stuck, go down one layer
4. At layer 0, collect k nearest

**Insert Algorithm:**
1. Assign random level (exponential distribution)
2. Search for nearest at each layer
3. Connect to M nearest neighbors
4. Update bidirectional links

**Key Parameters:**
- `m`: Connections per node (default: 16)
- `m_max0`: Layer 0 connections (default: 32)
- `ef_construction`: Build beam width (default: 100)
- `ef_search`: Search beam width (default: 50)

## Filtering Layer

### Filter (`filter/mod.rs`)

**DSL:**
```rust
Filter::field("category").eq("docs")
    .and(Filter::field("year").gte(2024))
    .or(Filter::field("featured").eq(true))
```

**Evaluation:** Post-filter (filter after distance calculation)

### BitmapIndex (`filter/bitmap_index.rs`)

**Data Structure:**
```rust
struct BitmapIndex {
    indexes: HashMap<(String, Value), RoaringBitmap>,
}
```

**Use Case:** Pre-filtering for selective queries
- Build bitmap of matching IDs
- Pass to `search_with_bitmap`
- Index only visits matching candidates

## Storage Layer

### Collection (`collection.rs`)

**Lifecycle:**
```
open_or_create(path) → read metadata → open WAL → recover → ready
```

**Components:**
```
collection_dir/
├── metadata.json   # CollectionConfig
├── data.bin        # Append-only vectors
└── wal.bin         # Write-ahead log
```

### WAL (`storage/wal.rs`)

**Entry Format:**
```rust
struct WalEntry {
    op: Operation,      // Insert, Update, Delete
    id: VectorId,
    vector: Option<Vec<f32>>,
    payload: Option<Payload>,
}
```

**Protocol:**
1. Serialize entry with bincode
2. Write length prefix
3. Write CRC32 checksum
4. fsync (on flush)

### DataFile (`storage/data_file.rs`)

**Format:**
```
[length: u32][vector_id: u64][dim: u32][f32 × dim][payload_len: u32][payload_json]...
```

**Memory-mapped** for efficient reads.

## Async Layer

### AsyncCollection (`collection.rs`)

```rust
pub struct AsyncCollection {
    inner: Arc<Collection>,
}
```

**Pattern:** Wrap sync ops with `tokio::task::spawn_blocking`

```rust
pub async fn insert(&self, id: VectorId, vector: Vec<f32>, payload: Payload) -> Result<()> {
    let inner = Arc::clone(&self.inner);
    tokio::task::spawn_blocking(move || inner.insert(id, vector, payload))
        .await
        .map_err(|e| Error::CollectionError(e.to_string()))?
}
```

## Error Handling

All errors flow through `Error` enum:

```rust
pub enum Error {
    DimensionMismatch { expected, got },
    DuplicateId(u64),
    NotFound(u64),
    InvalidFilter(String),
    PayloadError(String),
    EmptyVector,
    IoError(String),
    WalCorrupted(String),
    CollectionError(String),
}
```

**Convention:** Never panic in library code. Return `Result<T>`.

## Performance Considerations

### Hot Paths
1. Distance calculation (`distance.rs`) - called millions of times
2. HNSW neighbor selection (`hnsw.rs:select_neighbors`)
3. Filter evaluation (`filter/mod.rs:matches`)

### Memory Layout
- Vectors stored contiguously in `Vec<f32>`
- HashMap for O(1) ID lookup
- Roaring bitmaps for compressed ID sets

### Concurrency
- `Collection` uses `RwLock` for read-heavy workloads
- Indexes are single-threaded (wrap in mutex if sharing)
- Async uses thread pool for blocking I/O


## Bindings & API

### Python Bindings (`py/`)
- **Technology**: `pyo3` + `maturin`
- **Pattern**: `polarisdb` Python wrapper package around `_polarisdb` Rust extension module
- **Structure**: `py/python/polarisdb` (wrapper) imports from `src/lib.rs` (extension)
- **Memory**: Vectors passed as `numpy` arrays or lists are converted to `Vec<f32>`

### HTTP Server (`polarisdb-server/`)
- **Technology**: `axum` (web framework) + `tokio`
- **State**: Shared `Arc<RwLock<HashMap<String, AsyncCollection>>>`
- **Endpoints**: JSON-based REST API

