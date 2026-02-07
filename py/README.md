# PolarisDB Python Bindings

**PolarisDB** is a high-performance, embedded vector database written in Rust. This package provides Python bindings for efficient local vector search and storage.

## Features

- 🚀 **Fast**: Built on Rust with SIMD optimizations.
- 💾 **Embedded**: Runs locally without a separate server process.
- 📦 **Simple**: Easy-to-use Python API for managing collections and indexes.
- 🔍 **Search**: Supports Euclidean, Cosine, and Dot Product distance metrics.
- 🛡️ **Durable**: WAL-based persistence for data safety (Collection API).

## Installation

```bash
pip install polarisdb
```

## Quick Start

### In-Memory Index (Brute Force)

Perfect for small datasets or exact search requirements.

```python
import numpy as np
from polarisdb import Index

# Create an index with Cosine similarity and 128 dimensions
index = Index("cosine", 128)

# Insert vectors (ID, Vector)
# Vectors can be lists or numpy arrays
v1 = np.random.rand(128).astype(np.float32)
index.insert(1, v1)

v2 = np.random.rand(128).astype(np.float32)
index.insert(2, v2)

# Search for nearest neighbors
query = np.random.rand(128).astype(np.float32)
results = index.search(query, k=5)

for id, distance in results:
    print(f"Found ID: {id}, Distance: {distance}")
```

### Persistent Collection

Store vectors on disk with crash recovery.

```python
from polarisdb import Collection

# Open or create a collection at the specified path
collection = Collection.open_or_create("./my_collection", 128, "euclidean")

# Insert vectors
v1 = [1.0] * 128
collection.insert(1, v1)

# Persist changes to disk (WAL checkpoint)
collection.flush()

# Search
results = collection.search(v1, k=1)
print(results)
```

## API Reference

### `Index(metric: str, dimension: int)`

Creates an in-memory brute-force index.

- **metric**: "cosine", "euclidean", or "dot".
- **dimension**: Dimension of vectors (e.g., 128, 768).

### `Collection.open_or_create(path: str, dimension: int, metric: str)`

Opens an existing collection or creates a new one.

- **path**: Directory path for storage.
- **dimension**: Vector dimension.
- **metric**: Distance metric.

## LangChain Integration

Use PolarisDB as a vector store in LangChain RAG pipelines:

```python
from polarisdb.langchain import PolarisDBVectorStore
from langchain_openai import OpenAIEmbeddings

# Create from texts
vectorstore = PolarisDBVectorStore.from_texts(
    texts=["Hello world", "Goodbye world"],
    embedding=OpenAIEmbeddings(),
    collection_path="./my_vectors",
)

# Similarity search
docs = vectorstore.similarity_search("Hello", k=1)

# Use as retriever
retriever = vectorstore.as_retriever()
```

**Requirements**: `pip install langchain-core langchain-openai`

## Batch Operations

For efficient bulk operations:

```python
from polarisdb import Index

index = Index("cosine", 128)

# Insert many vectors at once
ids = [1, 2, 3, 4, 5]
vectors = [[0.1] * 128 for _ in range(5)]
index.insert_batch(ids, vectors)

# Search multiple queries
queries = [[0.1] * 128, [0.2] * 128]
results = index.search_batch(queries, k=3)
```

## License

MIT
