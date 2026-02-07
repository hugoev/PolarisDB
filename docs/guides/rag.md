# RAG Applications

Build Retrieval-Augmented Generation (RAG) applications with PolarisDB.

## What is RAG?

RAG combines retrieval from a vector database with LLM generation to answer questions using your own documents.

```
Query → Embed → Search Vector DB → Retrieve Context → Generate Answer
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Your Documents                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Embedding Model                           │
│               (OpenAI, Sentence Transformers)                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                       PolarisDB                              │
│                   (Vector Storage)                           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    LLM (GPT, Claude)                         │
│                  (Answer Generation)                         │
└─────────────────────────────────────────────────────────────┘
```

## Example: Document Q&A

```python
from polarisdb.langchain import PolarisDBVectorStore
from langchain_openai import OpenAIEmbeddings, ChatOpenAI
from langchain_core.prompts import ChatPromptTemplate
from langchain_core.output_parsers import StrOutputParser
from langchain_core.runnables import RunnablePassthrough

# Your documents
documents = [
    "PolarisDB is a high-performance embedded vector database.",
    "It supports HNSW indexing for fast approximate search.",
    "PolarisDB uses WAL for crash recovery.",
    # ... more documents
]

# Create vector store
embeddings = OpenAIEmbeddings()
vectorstore = PolarisDBVectorStore.from_texts(
    texts=documents,
    embedding=embeddings,
    collection_path="./rag_db",
)

# Create retriever
retriever = vectorstore.as_retriever(search_kwargs={"k": 3})

# Build chain
prompt = ChatPromptTemplate.from_template("""
Answer based on the context below:

{context}

Question: {question}
Answer:""")

def format_docs(docs):
    return "\n\n".join(doc.page_content for doc in docs)

rag_chain = (
    {"context": retriever | format_docs, "question": RunnablePassthrough()}
    | prompt
    | ChatOpenAI(model="gpt-3.5-turbo")
    | StrOutputParser()
)

# Ask questions
answer = rag_chain.invoke("What indexing algorithm does PolarisDB use?")
print(answer)
```

## Tips

1. **Chunk your documents** — Split large docs into ~500 token chunks
2. **Use appropriate embeddings** — Match embedding model to your content
3. **Tune k value** — More context = better answers but higher cost
4. **Persist your collection** — Use `collection_path` for production
