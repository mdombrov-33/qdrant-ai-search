"""Summarization related exceptions."""

from .base import QdrantSearchError


class SummarizationError(QdrantSearchError):
    """General summarization error."""

    pass


class SummaryGenerationError(SummarizationError):
    """Error generating summary."""

    pass
