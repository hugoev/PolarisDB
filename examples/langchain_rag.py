#!/usr/bin/env python3
"""
LangChain RAG Example with PolarisDB

This example demonstrates how to use PolarisDB with LangChain for
Retrieval-Augmented Generation (RAG).

Requirements:
    pip install langchain-openai langchain-core polarisdb

Usage:
    export OPENAI_API_KEY="your-api-key"
    python langchain_rag.py
"""

import os
from typing import List

# Check for API key
if not os.getenv("OPENAI_API_KEY"):
    print("Please set OPENAI_API_KEY environment variable")
    print("Example: export OPENAI_API_KEY='sk-...'")
    exit(1)

from langchain_openai import OpenAIEmbeddings, ChatOpenAI
from langchain_core.prompts import ChatPromptTemplate
from langchain_core.output_parsers import StrOutputParser
from langchain_core.runnables import RunnablePassthrough

# Import PolarisDB LangChain integration
from polarisdb.langchain import PolarisDBVectorStore


def format_docs(docs: List) -> str:
    """Format documents for the prompt."""
    return "\n\n".join(doc.page_content for doc in docs)


def main():
    # Sample documents about PolarisDB
    documents = [
        "PolarisDB is a high-performance embedded vector database written in Rust.",
        "PolarisDB supports HNSW indexing for approximate nearest neighbor search.",
        "PolarisDB uses Write-Ahead Logging (WAL) for crash recovery and durability.",
        "PolarisDB provides Python bindings via PyO3 and is available on PyPI.",
        "PolarisDB supports Cosine, Euclidean, and Dot Product distance metrics.",
        "PolarisDB can be used for RAG applications with LangChain integration.",
        "PolarisDB uses bitmap indexing for efficient metadata filtering.",
        "PolarisDB is designed for local AI workloads without external dependencies.",
    ]

    print("Initializing PolarisDB + LangChain RAG pipeline...\n")

    # Initialize embedding model
    embeddings = OpenAIEmbeddings(model="text-embedding-3-small")

    # Create vector store with documents
    vectorstore = PolarisDBVectorStore.from_texts(
        texts=documents,
        embedding=embeddings,
        collection_path="./rag_demo_collection",
        metric="cosine",
    )

    print(f"[OK] Indexed {len(documents)} documents\n")

    # Create retriever
    retriever = vectorstore.as_retriever(search_kwargs={"k": 3})

    # Create RAG prompt
    prompt = ChatPromptTemplate.from_template("""
Answer the question based only on the following context:

{context}

Question: {question}

Answer:""")

    # Create LLM
    llm = ChatOpenAI(model="gpt-3.5-turbo", temperature=0)

    # Build RAG chain
    rag_chain = (
        {"context": retriever | format_docs, "question": RunnablePassthrough()}
        | prompt
        | llm
        | StrOutputParser()
    )

    # Example queries
    queries = [
        "What indexing algorithm does PolarisDB use?",
        "How does PolarisDB handle crash recovery?",
        "What distance metrics are supported?",
    ]

    print("=" * 60)
    print("Running RAG queries...")
    print("=" * 60)

    for query in queries:
        print(f"\nQuestion: {query}")
        answer = rag_chain.invoke(query)
        print(f"Answer: {answer}")

    print("\n" + "=" * 60)
    print("[OK] RAG demo complete!")
    print("=" * 60)


if __name__ == "__main__":
    main()
