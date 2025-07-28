"""Exception handling utilities for API routes."""

from fastapi import HTTPException
from exceptions import (
    QdrantSearchError,
    FileTooLargeError,
    EmptyFileError,
    UnsupportedFileTypeError,
    FileExtractionError,
    InvalidQueryError,
    VectorSearchError,
    EmbeddingGenerationError,
    EmbeddingServiceError,
    RustServiceError,
    QdrantServiceError,
    OpenAIServiceError,
    SummarizationError,
    SummaryGenerationError,
)


def handle_custom_exception(e: Exception) -> HTTPException:
    """
    Convert custom exceptions to appropriate HTTP exceptions.

    Args:
        e: Custom exception from the application

    Returns:
        HTTPException with appropriate status code and message
    """
    # Client errors (4xx)
    if isinstance(e, (FileTooLargeError, EmptyFileError, UnsupportedFileTypeError)):
        return HTTPException(status_code=400, detail=str(e))

    if isinstance(e, (FileExtractionError, InvalidQueryError)):
        return HTTPException(status_code=400, detail=str(e))

    # Server errors (5xx)
    if isinstance(e, (EmbeddingGenerationError, EmbeddingServiceError)):
        return HTTPException(
            status_code=500, detail=f"Embedding service error: {str(e)}"
        )

    if isinstance(e, (VectorSearchError, QdrantServiceError)):
        return HTTPException(status_code=500, detail=f"Database error: {str(e)}")

    if isinstance(e, OpenAIServiceError):
        return HTTPException(status_code=500, detail=f"AI service error: {str(e)}")

    if isinstance(e, RustServiceError):
        return HTTPException(status_code=500, detail=f"Ranking service error: {str(e)}")

    if isinstance(e, (SummarizationError, SummaryGenerationError)):
        return HTTPException(status_code=500, detail=f"Summarization error: {str(e)}")

    # Generic custom exceptions
    if isinstance(e, QdrantSearchError):
        return HTTPException(status_code=500, detail=f"Application error: {str(e)}")

    # Unknown exceptions
    return HTTPException(status_code=500, detail="Internal server error")
