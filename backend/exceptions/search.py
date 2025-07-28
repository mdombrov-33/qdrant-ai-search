"""Search and query related exceptions."""

from .base import QdrantSearchError


class SearchError(QdrantSearchError):
    """General search error."""

    pass


class InvalidQueryError(SearchError):
    """Invalid search query."""

    pass


class VectorSearchError(SearchError):
    """Vector search operation error."""

    pass
