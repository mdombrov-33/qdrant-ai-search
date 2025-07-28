"""Package initialization for exceptions module."""

from .base import QdrantSearchError
from .file_processing import (
    FileProcessingError,
    FileTooLargeError,
    UnsupportedFileTypeError,
    EmptyFileError,
    FileExtractionError,
)
from .search import SearchError, InvalidQueryError, VectorSearchError
from .embedding import EmbeddingError, EmbeddingGenerationError, EmbeddingServiceError
from .external_service import (
    ExternalServiceError,
    RustServiceError,
    QdrantServiceError,
    OpenAIServiceError,
)
from .summarization import SummarizationError, SummaryGenerationError

__all__ = [
    "QdrantSearchError",
    "FileProcessingError",
    "FileTooLargeError",
    "UnsupportedFileTypeError",
    "EmptyFileError",
    "FileExtractionError",
    "SearchError",
    "InvalidQueryError",
    "VectorSearchError",
    "EmbeddingError",
    "EmbeddingGenerationError",
    "EmbeddingServiceError",
    "ExternalServiceError",
    "RustServiceError",
    "QdrantServiceError",
    "OpenAIServiceError",
    "SummarizationError",
    "SummaryGenerationError",
]
