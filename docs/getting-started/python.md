# Python Bindings

PolarisDB provides native Python bindings via PyO3.

## Installation

```bash
pip install polarisdb
```

## Classes

### Index

In-memory brute-force index for exact nearest neighbor search.

```python
from polarisdb import Index

# Create index
index = Index(metric: str, dimension: int)
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `metric` | `str` | Distance metric: `"cosine"`, `"euclidean"`, `"dot"` |
| `dimension` | `int` | Vector dimensionality |

**Methods:**

| Method | Description |
|--------|-------------|
| `insert(id, vector)` | Insert a vector with ID |
| `insert_batch(ids, vectors)` | Insert multiple vectors |
| `search(query, k)` | Find k nearest neighbors |
| `search_batch(queries, k)` | Search for multiple queries |

### Collection

Persistent collection with WAL durability.

```python
from polarisdb import Collection

# Open or create
collection = Collection.open_or_create(path: str, dimension: int, metric: str)
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `path` | `str` | Directory for storage |
| `dimension` | `int` | Vector dimensionality |
| `metric` | `str` | Distance metric |

**Methods:**

| Method | Description |
|--------|-------------|
| `insert(id, vector)` | Insert a vector |
| `insert_batch(ids, vectors)` | Batch insert |
| `search(query, k)` | Find k nearest neighbors |
| `search_batch(queries, k)` | Batch search |
| `flush()` | Persist to disk |

## LangChain Integration

```python
from polarisdb.langchain import PolarisDBVectorStore
```

See [LangChain Guide](../guides/langchain.md) for details.
