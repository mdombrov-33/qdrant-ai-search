"""Base exception classes for the Qdrant AI Search application."""

from typing import Optional


class QdrantSearchError(Exception):
    """Base exception for all Qdrant AI Search related errors."""

    def __init__(self, message: str, error_code: Optional[str] = None):
        self.message = message
        self.error_code = error_code
        super().__init__(self.message)
