# Filtering

Filter vector search results by metadata conditions.

## Basic Filtering

```python
from polarisdb import Collection, Filter

collection = Collection.open_or_create("./data", 384, "cosine")

# Insert with metadata
collection.insert(1, embedding1, {"category": "docs", "year": 2024})
collection.insert(2, embedding2, {"category": "code", "year": 2023})
collection.insert(3, embedding3, {"category": "docs", "year": 2023})

# Search with filter
filter = Filter.field("category").eq("docs")
results = collection.search(query, k=10, filter=filter)
```

## Filter Operations

| Operation | Example | Description |
|-----------|---------|-------------|
| `eq` | `.eq("value")` | Equals |
| `ne` | `.ne("value")` | Not equals |
| `gt` | `.gt(10)` | Greater than |
| `gte` | `.gte(10)` | Greater or equal |
| `lt` | `.lt(10)` | Less than |
| `lte` | `.lte(10)` | Less or equal |
| `in` | `.contained_in(["a", "b"])` | Value in list |
| `contains` | `.contains("sub")` | String contains |
| `exists` | `.exists()` | Field exists |

## Combining Filters

```python
# AND
filter = Filter.field("category").eq("docs").and(
    Filter.field("year").gte(2024)
)

# OR
filter = Filter.field("category").eq("docs").or(
    Filter.field("category").eq("code")
)

# NOT
filter = Filter.field("category").eq("archived").negate()
```

## Bitmap Index (Pre-Filtering)

For highly selective filters, use bitmap pre-filtering:

```python
from polarisdb import BitmapIndex, HnswIndex

# Build indexes
bitmap = BitmapIndex()
hnsw = HnswIndex::new(DistanceMetric::Cosine, 384, HnswConfig::default())

for id, embedding, payload in documents:
    hnsw.insert(id, embedding, payload)
    bitmap.insert(id, payload)

# Pre-filter using bitmap
filter = Filter.field("category").eq("docs")
valid_ids = bitmap.query(filter)
results = hnsw.search_with_bitmap(query, k=10, None, valid_ids)
```
