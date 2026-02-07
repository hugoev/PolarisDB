# PolarisDB

<div align="center">
  <h2>A pure-Rust embedded vector database for local AI and RAG workloads</h2>
</div>

## What is PolarisDB?

PolarisDB is a high-performance vector database designed for developers who need **fast, local vector search** without the complexity of external services.

<div class="grid cards" markdown>

-   :rocket: **High Performance**

    ---

    SIMD-optimized distance calculations, HNSW indexing with O(log n) search.

-   :lock: **Local & Private**

    ---

    Runs entirely on your machine. Your data never leaves.

-   :snake: **Python & Rust**

    ---

    Native Rust API with first-class Python bindings on PyPI.

-   :zap: **LangChain Ready**

    ---

    Drop-in VectorStore for RAG applications.

</div>

## Quick Install

=== "Python"

    ```bash
    pip install polarisdb
    ```

=== "Rust"

    ```toml
    [dependencies]
    polarisdb = "0.1"
    ```

## Example

```python
from polarisdb import Collection

# Create a persistent collection
col = Collection.open_or_create("./my_vectors", 384, "cosine")

# Insert vectors
col.insert(1, [0.1] * 384)
col.insert(2, [0.2] * 384)

# Search
results = col.search([0.15] * 384, k=5)
print(results)  # [(1, 0.05), (2, 0.05)]
```

## Next Steps

- [Installation Guide](getting-started/installation.md) — Detailed setup instructions
- [Quick Start](getting-started/quickstart.md) — Your first vector database
- [LangChain Integration](guides/langchain.md) — Use with RAG pipelines
- [API Reference](api/python.md) — Complete API documentation
