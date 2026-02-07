# Quick Start

This guide walks you through creating your first vector database with PolarisDB.

## Choose Your Approach

PolarisDB offers two main APIs:

| API | Best For | Persistence |
|-----|----------|-------------|
| `Index` | Small datasets, testing | In-memory only |
| `Collection` | Production use | Disk with WAL |

## In-Memory Index

Perfect for experimentation and small datasets:

```python
from polarisdb import Index
import numpy as np

# Create index with cosine similarity
index = Index("cosine", 384)

# Generate sample embeddings
embeddings = np.random.rand(100, 384).astype(np.float32)

# Insert vectors
for i, emb in enumerate(embeddings):
    index.insert(i, emb)

# Search
query = np.random.rand(384).astype(np.float32)
results = index.search(query, k=5)

for doc_id, distance in results:
    print(f"ID: {doc_id}, Distance: {distance:.4f}")
```

## Persistent Collection

For production use with crash recovery:

```python
from polarisdb import Collection

# Open or create collection
collection = Collection.open_or_create(
    "./my_vectors",  # Storage path
    384,             # Dimension
    "cosine"         # Distance metric
)

# Insert vectors with IDs
collection.insert(1, [0.1] * 384)
collection.insert(2, [0.2] * 384)
collection.insert(3, [0.3] * 384)

# Persist to disk
collection.flush()

# Search
results = collection.search([0.15] * 384, k=2)
print(results)  # [(1, 0.05), (2, 0.05)]
```

## Distance Metrics

| Metric | Use Case | Range |
|--------|----------|-------|
| `cosine` | Text embeddings (OpenAI, Sentence Transformers) | 0-2 |
| `euclidean` | Image embeddings, general purpose | 0-∞ |
| `dot` | Maximum inner product search | -∞ to ∞ |

## Next Steps

- [LangChain Integration](../guides/langchain.md) — Use with RAG
- [Filtering](../guides/filtering.md) — Metadata-based queries
- [Python API Reference](../api/python.md) — Full API docs
