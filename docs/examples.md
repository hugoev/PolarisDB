# Examples

## Rust Examples

Run examples with:

```bash
cargo run --release --example <name>
```

### HNSW Demo

Demonstrates HNSW index performance vs brute force.

```bash
cargo run --release --example hnsw_demo
```

### Async Demo

Concurrent insertions with async API.

```bash
cargo run --release --example async_demo --features async
```

### Pre-filter Demo

Bitmap pre-filtering for selective queries.

```bash
cargo run --release --example prefilter_demo
```

### Ollama RAG

RAG with local Ollama embeddings.

```bash
cargo run --release --example ollama_rag
```

## Python Examples

### LangChain RAG

```bash
cd examples
pip install langchain-openai
export OPENAI_API_KEY="your-key"
python langchain_rag.py
```

See [`examples/langchain_rag.py`](https://github.com/hugoev/polarisdb/blob/main/examples/langchain_rag.py).

## Quick Snippets

### Basic Search (Python)

```python
from polarisdb import Collection

col = Collection.open_or_create("./data", 384, "cosine")
col.insert(1, [0.1] * 384)
results = col.search([0.1] * 384, k=5)
```

### With LangChain

```python
from polarisdb.langchain import PolarisDBVectorStore
from langchain_openai import OpenAIEmbeddings

vs = PolarisDBVectorStore.from_texts(
    ["Hello", "World"],
    OpenAIEmbeddings(),
)
docs = vs.similarity_search("Hello", k=1)
```
