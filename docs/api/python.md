# Python API Reference

Complete API documentation for PolarisDB Python bindings.

## polarisdb.Index

In-memory brute-force index for exact nearest neighbor search.

### Constructor

```python
Index(metric: str, dimension: int)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `metric` | `str` | `"cosine"`, `"euclidean"`, or `"dot"` |
| `dimension` | `int` | Vector dimensionality |

### Methods

#### insert

```python
insert(id: int, vector: list[float]) -> None
```

Insert a single vector.

#### insert_batch

```python
insert_batch(ids: list[int], vectors: list[list[float]]) -> None
```

Insert multiple vectors efficiently.

#### search

```python
search(query: list[float], k: int) -> list[tuple[int, float]]
```

Find k nearest neighbors. Returns list of `(id, distance)` tuples.

#### search_batch

```python
search_batch(queries: list[list[float]], k: int) -> list[list[tuple[int, float]]]
```

Search for multiple queries at once.

---

## polarisdb.Collection

Persistent collection with WAL durability.

### Factory Method

```python
Collection.open_or_create(path: str, dimension: int, metric: str) -> Collection
```

Opens an existing collection or creates a new one.

| Parameter | Type | Description |
|-----------|------|-------------|
| `path` | `str` | Directory path for storage |
| `dimension` | `int` | Vector dimensionality |
| `metric` | `str` | Distance metric |

### Methods

#### insert

```python
insert(id: int, vector: list[float]) -> None
```

#### insert_batch

```python
insert_batch(ids: list[int], vectors: list[list[float]]) -> None
```

#### search

```python
search(query: list[float], k: int) -> list[tuple[int, float]]
```

#### search_batch

```python
search_batch(queries: list[list[float]], k: int) -> list[list[tuple[int, float]]]
```

#### flush

```python
flush() -> None
```

Persist all pending writes to disk (WAL checkpoint).

---

## polarisdb.langchain.PolarisDBVectorStore

LangChain VectorStore implementation.

See [LangChain Integration Guide](../guides/langchain.md) for full documentation.
