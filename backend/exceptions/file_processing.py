"""File processing related exceptions."""

from .base import QdrantSearchError


class FileProcessingError(QdrantSearchError):
    """General file processing error."""

    pass


class FileTooLargeError(FileProcessingError):
    """File exceeds size limit."""

    pass


class UnsupportedFileTypeError(FileProcessingError):
    """Unsupported file type."""

    pass


class EmptyFileError(FileProcessingError):
    """Empty file error."""

    pass


class FileExtractionError(FileProcessingError):
    """Error extracting text from file."""

    pass
