"""Embedding related exceptions."""

from .base import QdrantSearchError


class EmbeddingError(QdrantSearchError):
    """General embedding error."""

    pass


class EmbeddingGenerationError(EmbeddingError):
    """Error generating embeddings."""

    pass


class EmbeddingServiceError(EmbeddingError):
    """Embedding service unavailable."""

    pass
