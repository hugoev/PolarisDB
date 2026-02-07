# LangChain Integration

PolarisDB provides a native LangChain `VectorStore` implementation for seamless integration with RAG pipelines.

## Installation

```bash
pip install polarisdb langchain-core langchain-openai
```

## Quick Start

```python
from polarisdb.langchain import PolarisDBVectorStore
from langchain_openai import OpenAIEmbeddings

# Create vector store from texts
vectorstore = PolarisDBVectorStore.from_texts(
    texts=[
        "PolarisDB is a high-performance vector database.",
        "It supports HNSW indexing for fast search.",
        "PolarisDB runs locally without external services.",
    ],
    embedding=OpenAIEmbeddings(),
    collection_path="./my_vectors",
    metric="cosine",
)

# Similarity search
docs = vectorstore.similarity_search("fast database", k=2)
for doc in docs:
    print(doc.page_content)
```

## API Reference

### PolarisDBVectorStore

```python
from polarisdb.langchain import PolarisDBVectorStore
```

#### Constructor

```python
PolarisDBVectorStore(
    embedding: Embeddings,
    collection_path: Optional[str] = None,  # None = in-memory
    dimension: Optional[int] = None,        # Auto-detected from embedding
    metric: str = "cosine",
)
```

#### Factory Methods

```python
# From texts
PolarisDBVectorStore.from_texts(
    texts: List[str],
    embedding: Embeddings,
    metadatas: Optional[List[dict]] = None,
    collection_path: Optional[str] = None,
    metric: str = "cosine",
) -> PolarisDBVectorStore

# From documents
PolarisDBVectorStore.from_documents(
    documents: List[Document],
    embedding: Embeddings,
    collection_path: Optional[str] = None,
    metric: str = "cosine",
) -> PolarisDBVectorStore
```

#### Methods

| Method | Description |
|--------|-------------|
| `add_texts(texts, metadatas)` | Add texts with optional metadata |
| `similarity_search(query, k)` | Find k similar documents |
| `similarity_search_with_score(query, k)` | With distance scores |
| `as_retriever(**kwargs)` | Create LangChain Retriever |

## RAG Example

Complete RAG pipeline with OpenAI:

```python
from polarisdb.langchain import PolarisDBVectorStore
from langchain_openai import OpenAIEmbeddings, ChatOpenAI
from langchain_core.prompts import ChatPromptTemplate
from langchain_core.output_parsers import StrOutputParser
from langchain_core.runnables import RunnablePassthrough

# 1. Create vector store
vectorstore = PolarisDBVectorStore.from_texts(
    texts=your_documents,
    embedding=OpenAIEmbeddings(),
    collection_path="./rag_db",
)

# 2. Create retriever
retriever = vectorstore.as_retriever(search_kwargs={"k": 3})

# 3. Build RAG chain
prompt = ChatPromptTemplate.from_template("""
Answer based on the context:

{context}

Question: {question}
""")

def format_docs(docs):
    return "\n\n".join(doc.page_content for doc in docs)

rag_chain = (
    {"context": retriever | format_docs, "question": RunnablePassthrough()}
    | prompt
    | ChatOpenAI(model="gpt-3.5-turbo")
    | StrOutputParser()
)

# 4. Query
answer = rag_chain.invoke("What is PolarisDB?")
print(answer)
```

## Supported Embedding Models

PolarisDB works with any LangChain-compatible embedding model:

| Provider | Model | Dimension |
|----------|-------|-----------|
| OpenAI | `text-embedding-3-small` | 1536 |
| OpenAI | `text-embedding-3-large` | 3072 |
| HuggingFace | `sentence-transformers/*` | Varies |
| Cohere | `embed-english-v3.0` | 1024 |
| Local | Ollama embeddings | Varies |

## Persistence

```python
# Persistent (recommended for production)
vectorstore = PolarisDBVectorStore.from_texts(
    texts=documents,
    embedding=embeddings,
    collection_path="./my_vectors",  # Data persisted here
)

# In-memory (for testing)
vectorstore = PolarisDBVectorStore.from_texts(
    texts=documents,
    embedding=embeddings,
    collection_path=None,  # Data in memory only
)
```
