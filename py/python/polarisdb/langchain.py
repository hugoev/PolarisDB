"""
LangChain VectorStore integration for PolarisDB.

This module provides a LangChain-compatible VectorStore implementation
that uses PolarisDB as the backend for vector storage and similarity search.

Example:
    >>> from polarisdb.langchain import PolarisDBVectorStore
    >>> from langchain_openai import OpenAIEmbeddings
    >>>
    >>> vectorstore = PolarisDBVectorStore.from_texts(
    ...     texts=["Hello world", "Goodbye world"],
    ...     embedding=OpenAIEmbeddings(),
    ...     collection_path="./my_vectors",
    ... )
    >>> results = vectorstore.similarity_search("Hello", k=1)
"""

from __future__ import annotations

import uuid
from typing import Any, Iterable, List, Optional, Tuple, Type

try:
    from langchain_core.documents import Document
    from langchain_core.embeddings import Embeddings
    from langchain_core.vectorstores import VectorStore
except ImportError:
    raise ImportError(
        "LangChain is required for this module. "
        "Install it with: pip install langchain-core"
    )

from polarisdb import Collection, Index


class PolarisDBVectorStore(VectorStore):
    """LangChain VectorStore backed by PolarisDB.
    
    Supports both in-memory (Index) and persistent (Collection) backends.
    
    Args:
        collection_path: Path for persistent storage. If None, uses in-memory index.
        embedding: Embedding model for encoding texts.
        dimension: Vector dimension (required for in-memory mode).
        metric: Distance metric ("cosine", "euclidean", "dot").
    """

    def __init__(
        self,
        embedding: Embeddings,
        collection_path: Optional[str] = None,
        dimension: Optional[int] = None,
        metric: str = "cosine",
    ):
        self._embedding = embedding
        self._metric = metric
        self._collection_path = collection_path
        self._documents: dict[int, Document] = {}
        self._next_id = 0

        # Determine dimension from embedding if not provided
        if dimension is None:
            test_embedding = self._embedding.embed_query("test")
            dimension = len(test_embedding)
        
        self._dimension = dimension

        # Create backend
        if collection_path:
            self._backend = Collection.open_or_create(
                collection_path, dimension, metric
            )
            self._is_persistent = True
        else:
            self._backend = Index(metric, dimension)
            self._is_persistent = False

    @property
    def embeddings(self) -> Embeddings:
        """Return the embedding model."""
        return self._embedding

    def add_texts(
        self,
        texts: Iterable[str],
        metadatas: Optional[List[dict]] = None,
        **kwargs: Any,
    ) -> List[str]:
        """Add texts to the vector store.
        
        Args:
            texts: Texts to add.
            metadatas: Optional metadata for each text.
            
        Returns:
            List of IDs for the added texts.
        """
        texts_list = list(texts)
        embeddings = self._embedding.embed_documents(texts_list)
        
        if metadatas is None:
            metadatas = [{} for _ in texts_list]
        
        ids = []
        for text, embedding, metadata in zip(texts_list, embeddings, metadatas):
            doc_id = self._next_id
            self._next_id += 1
            
            # Store document for retrieval
            self._documents[doc_id] = Document(
                page_content=text,
                metadata=metadata,
            )
            
            # Insert into PolarisDB
            self._backend.insert(doc_id, embedding)
            ids.append(str(doc_id))
        
        # Flush if persistent
        if self._is_persistent:
            self._backend.flush()
        
        return ids

    def similarity_search(
        self,
        query: str,
        k: int = 4,
        **kwargs: Any,
    ) -> List[Document]:
        """Search for similar documents.
        
        Args:
            query: Query text.
            k: Number of results to return.
            
        Returns:
            List of similar documents.
        """
        results = self.similarity_search_with_score(query, k, **kwargs)
        return [doc for doc, _ in results]

    def similarity_search_with_score(
        self,
        query: str,
        k: int = 4,
        **kwargs: Any,
    ) -> List[Tuple[Document, float]]:
        """Search for similar documents with scores.
        
        Args:
            query: Query text.
            k: Number of results to return.
            
        Returns:
            List of (document, score) tuples.
        """
        query_embedding = self._embedding.embed_query(query)
        results = self._backend.search(query_embedding, k)
        
        documents_with_scores = []
        for doc_id, score in results:
            if doc_id in self._documents:
                documents_with_scores.append((self._documents[doc_id], score))
        
        return documents_with_scores

    @classmethod
    def from_texts(
        cls: Type["PolarisDBVectorStore"],
        texts: List[str],
        embedding: Embeddings,
        metadatas: Optional[List[dict]] = None,
        collection_path: Optional[str] = None,
        metric: str = "cosine",
        **kwargs: Any,
    ) -> "PolarisDBVectorStore":
        """Create a PolarisDBVectorStore from texts.
        
        Args:
            texts: List of texts to add.
            embedding: Embedding model.
            metadatas: Optional metadata for each text.
            collection_path: Path for persistent storage.
            metric: Distance metric.
            
        Returns:
            Initialized PolarisDBVectorStore with texts added.
        """
        store = cls(
            embedding=embedding,
            collection_path=collection_path,
            metric=metric,
        )
        store.add_texts(texts, metadatas)
        return store

    @classmethod
    def from_documents(
        cls: Type["PolarisDBVectorStore"],
        documents: List[Document],
        embedding: Embeddings,
        collection_path: Optional[str] = None,
        metric: str = "cosine",
        **kwargs: Any,
    ) -> "PolarisDBVectorStore":
        """Create a PolarisDBVectorStore from documents.
        
        Args:
            documents: List of documents to add.
            embedding: Embedding model.
            collection_path: Path for persistent storage.
            metric: Distance metric.
            
        Returns:
            Initialized PolarisDBVectorStore with documents added.
        """
        texts = [doc.page_content for doc in documents]
        metadatas = [doc.metadata for doc in documents]
        return cls.from_texts(
            texts=texts,
            embedding=embedding,
            metadatas=metadatas,
            collection_path=collection_path,
            metric=metric,
            **kwargs,
        )
