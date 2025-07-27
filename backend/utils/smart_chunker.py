"""
Advanced semantic chunking for document processing.

This module implements production-grade text chunking that preserves
semantic boundaries and maintains context between chunks.
"""

import re
import unicodedata
from typing import List, Optional
from dataclasses import dataclass


@dataclass
class ChunkConfig:
    """Configuration for chunking behavior."""

    target_chunk_size: int = 300  # Target words per chunk
    min_chunk_size: int = 100  # Minimum words to avoid tiny fragments
    max_chunk_size: int = 500  # Maximum words before force-splitting
    overlap_size: int = 50  # Words of overlap between chunks
    preserve_paragraphs: bool = True  # Try to keep paragraphs intact
    sentence_aware: bool = True  # Split at sentence boundaries


class SmartChunker:
    """
    Advanced text chunker that preserves semantic structure.

    Key improvements over simple word-based chunking:
    1. Preserves sentence and paragraph boundaries
    2. Maintains overlapping context between chunks
    3. Handles document structure (headings, lists, tables)
    4. Cleans OCR artifacts before chunking
    5. Ensures minimum quality thresholds
    """

    def __init__(self, config: Optional[ChunkConfig] = None):
        self.config = config or ChunkConfig()

        # Sentence boundary patterns (improved for PDFs)
        self.sentence_endings = re.compile(
            r"(?<=[.!?])\s+(?=[A-Z])|"  # Standard sentence endings
            r"(?<=\.)\s+(?=\d+\.)|"  # Numbered lists: "1. Item"
            r"(?<=:]\s)\s*(?=[A-Z])"  # After citations: "[1] Next sentence"
        )

        # Paragraph boundary patterns
        self.paragraph_breaks = re.compile(
            r"\n\s*\n|"  # Double newlines
            r"(?<=\.)\s*\n\s*(?=[A-Z])|"  # Newline after sentence
            r"\n\s*(?=\d+\.)|"  # Before numbered items
            r"\n\s*(?=[•\-\*])"  # Before bullet points
        )

        # OCR artifact patterns to clean
        self.ocr_fixes = [
            (r"\boffi ce\b", "office"),  # Common OCR errors
            (r"\bthe m\b", "them"),
            (r"\bwith in\b", "within"),
            (r"\bover all\b", "overall"),
            (r"\s{2,}", " "),  # Multiple spaces
            (r"([a-z])([A-Z])", r"\1 \2"),  # Missing spaces between words
        ]

    def chunk_document(self, text: str, document_name: str = "document") -> List[dict]:
        """
        Main entry point for chunking a document.

        Returns list of chunk dictionaries with metadata.
        """
        # Step 1: Clean and preprocess
        cleaned_text = self._preprocess_text(text)

        # Step 2: Split into paragraphs first
        paragraphs = self._split_paragraphs(cleaned_text)

        # Step 3: Create semantic chunks
        chunks = self._create_semantic_chunks(paragraphs)

        # Step 4: Add metadata and post-process
        return self._finalize_chunks(chunks, document_name)

    def _preprocess_text(self, text: str) -> str:
        """Clean text while preserving document structure."""
        # Unicode normalization
        text = unicodedata.normalize("NFKC", text)

        # Fix common OCR artifacts
        for pattern, replacement in self.ocr_fixes:
            text = re.sub(pattern, replacement, text)

        # Remove control characters but preserve meaningful whitespace
        text = "".join(char for char in text if char.isprintable() or char in "\n\t")

        # Normalize excessive whitespace but preserve paragraph breaks
        text = re.sub(r"[ \t]+", " ", text)  # Multiple spaces/tabs → single space
        text = re.sub(r"\n[ \t]*\n", "\n\n", text)  # Clean paragraph breaks
        text = re.sub(r"\n{3,}", "\n\n", text)  # Max 2 newlines

        return text.strip()

    def _split_paragraphs(self, text: str) -> List[str]:
        """Split text into meaningful paragraphs."""
        # Split on paragraph boundaries
        paragraphs = self.paragraph_breaks.split(text)

        # Clean and filter paragraphs
        cleaned_paragraphs = []
        for para in paragraphs:
            para = para.strip()
            if len(para) > 20 and self._is_meaningful_text(para):  # Skip tiny fragments
                cleaned_paragraphs.append(para)

        return cleaned_paragraphs

    def _is_meaningful_text(self, text: str) -> bool:
        """Check if text chunk has sufficient information content."""
        words = text.split()

        # Quality filters
        if len(words) < 5:  # Too short
            return False

        # Check alphabetic content ratio
        alpha_chars = sum(1 for c in text if c.isalpha())
        if alpha_chars / len(text) < 0.6:  # Less than 60% letters
            return False

        # Check for repeated patterns (OCR artifacts)
        if len(set(words)) / len(words) < 0.3:  # Too repetitive
            return False

        return True

    def _create_semantic_chunks(self, paragraphs: List[str]) -> List[str]:
        """Create semantically meaningful chunks from paragraphs."""
        chunks = []
        current_chunk = ""
        current_size = 0

        for paragraph in paragraphs:
            para_words = len(paragraph.split())

            # Case 1: Paragraph fits in current chunk
            if current_size + para_words <= self.config.target_chunk_size:
                if current_chunk:
                    current_chunk += "\n\n" + paragraph
                else:
                    current_chunk = paragraph
                current_size += para_words

            # Case 2: Current chunk is substantial, start new chunk
            elif current_size >= self.config.min_chunk_size:
                if current_chunk:
                    chunks.append(current_chunk.strip())

                # Start new chunk with overlap from previous
                overlap_text = self._get_overlap_text(current_chunk)
                current_chunk = overlap_text + paragraph if overlap_text else paragraph
                current_size = len(current_chunk.split())

            # Case 3: Paragraph is too long, split it carefully
            elif para_words > self.config.max_chunk_size:
                # Save current chunk if it exists
                if current_chunk:
                    chunks.append(current_chunk.strip())

                # Split the large paragraph at sentence boundaries
                sub_chunks = self._split_large_paragraph(paragraph)
                chunks.extend(sub_chunks[:-1])  # Add all but last

                # Last sub-chunk becomes the new current chunk
                current_chunk = sub_chunks[-1] if sub_chunks else ""
                current_size = len(current_chunk.split())

            # Case 4: Add paragraph to current chunk (will exceed target but manageable)
            else:
                current_chunk += "\n\n" + paragraph
                current_size += para_words

        # Don't forget the last chunk
        if current_chunk and current_size >= self.config.min_chunk_size:
            chunks.append(current_chunk.strip())

        return chunks

    def _get_overlap_text(self, text: str) -> str:
        """Extract overlap text from the end of a chunk."""
        if not text or not self.config.overlap_size:
            return ""

        words = text.split()
        if len(words) <= self.config.overlap_size:
            return ""

        # Take last N words, but try to end at a sentence boundary
        overlap_words = words[-self.config.overlap_size :]
        overlap_text = " ".join(overlap_words)

        # Try to find a good sentence boundary in the overlap
        sentences = self.sentence_endings.split(overlap_text)
        if len(sentences) > 1:
            # Take the last complete sentence(s)
            return ". ".join(sentences[-2:]).strip() + "."

        return overlap_text

    def _split_large_paragraph(self, paragraph: str) -> List[str]:
        """Split a large paragraph at sentence boundaries."""
        # Split into sentences
        sentences = self.sentence_endings.split(paragraph)
        if len(sentences) <= 1:
            # No sentence boundaries found, force split by words
            return self._force_split_by_words(paragraph)

        chunks = []
        current_chunk = ""
        current_size = 0

        for sentence in sentences:
            sentence = sentence.strip()
            if not sentence:
                continue

            sentence_size = len(sentence.split())

            if current_size + sentence_size <= self.config.target_chunk_size:
                current_chunk += sentence + ". "
                current_size += sentence_size
            else:
                if current_chunk:
                    chunks.append(current_chunk.strip())

                # Start new chunk with this sentence
                current_chunk = sentence + ". "
                current_size = sentence_size

        if current_chunk:
            chunks.append(current_chunk.strip())

        return chunks

    def _force_split_by_words(self, text: str) -> List[str]:
        """Last resort: split by word count when no sentence boundaries exist."""
        words = text.split()
        chunks = []

        for i in range(
            0, len(words), self.config.target_chunk_size - self.config.overlap_size
        ):
            chunk_words = words[i : i + self.config.target_chunk_size]
            chunks.append(" ".join(chunk_words))

        return chunks

    def _finalize_chunks(self, chunks: List[str], document_name: str) -> List[dict]:
        """Add metadata and final validation to chunks."""
        final_chunks = []

        for i, chunk in enumerate(chunks):
            # Final quality check
            if not self._is_meaningful_text(chunk):
                continue

            # Calculate metrics
            word_count = len(chunk.split())
            char_count = len(chunk)

            chunk_data = {
                "text": chunk,
                "metadata": {
                    "document_name": document_name,
                    "chunk_index": i,
                    "word_count": word_count,
                    "char_count": char_count,
                    "chunk_type": self._classify_chunk_type(chunk),
                },
            }

            final_chunks.append(chunk_data)

        return final_chunks

    def _classify_chunk_type(self, text: str) -> str:
        """Classify the type of content in the chunk."""
        # Simple heuristics for content classification
        if re.search(r"^\d+\.|\n\d+\.", text):
            return "numbered_list"
        elif re.search(r"^[•\-\*]|\n[•\-\*]", text):
            return "bullet_list"
        elif re.search(r"[A-Z][A-Z\s]{10,}", text):  # ALL CAPS headers
            return "heading_section"
        elif len(text.split()) < 50:
            return "short_passage"
        elif len(text.split()) > 300:
            return "long_passage"
        else:
            return "standard_paragraph"


# Convenience function for backward compatibility
def smart_chunk_text(
    text: str, document_name: str = "document", config: Optional[ChunkConfig] = None
) -> List[str]:
    """
    Backward-compatible function that returns just the text chunks.

    For new code, use SmartChunker.chunk_document() directly for full metadata.
    """
    chunker = SmartChunker(config)
    chunk_data = chunker.chunk_document(text, document_name)
    return [chunk["text"] for chunk in chunk_data]
