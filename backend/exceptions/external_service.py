"""External service related exceptions."""

from .base import QdrantSearchError


class ExternalServiceError(QdrantSearchError):
    """General external service error."""

    pass


class RustServiceError(ExternalServiceError):
    """Rust service communication error."""

    pass


class QdrantServiceError(ExternalServiceError):
    """Qdrant vector database error."""

    pass


class OpenAIServiceError(ExternalServiceError):
    """OpenAI API service error."""

    pass
