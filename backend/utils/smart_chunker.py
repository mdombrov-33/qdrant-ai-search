"""
Modern semantic chunking using established libraries.

This implementation uses:
- LangChain RecursiveCharacterTextSplitter for intelligent text splitting
- NLTK for sentence tokenization and text processing
- spaCy for advanced document structure analysis
"""

from typing import List, Optional
from dataclasses import dataclass
import re
import unicodedata

# Library imports
from langchain_text_splitters import RecursiveCharacterTextSplitter
import nltk
import spacy


@dataclass
class ChunkConfig:
    """Configuration for chunking behavior"""

    target_chunk_size: int = 300  # Target words per chunk
    min_chunk_size: int = 100  # Minimum words to avoid tiny fragments
    max_chunk_size: int = 500  # Maximum words before force-splitting
    overlap_size: int = 50  # Words of overlap between chunks
    preserve_paragraphs: bool = True  # Try to keep paragraphs intact
    sentence_aware: bool = True  # Split at sentence boundaries


class SmartChunker:
    def __init__(self, config: Optional[ChunkConfig] = None):
        self.config = config or ChunkConfig()

        # Setup NLTK
        try:
            nltk.data.find("tokenizers/punkt")
        except LookupError:
            nltk.download("punkt")

        # Setup spaCy
        try:
            self.nlp = spacy.load("en_core_web_sm")
        except OSError:
            # Download the model if not available
            import subprocess

            subprocess.run(["python", "-m", "spacy", "download", "en_core_web_sm"])
            self.nlp = spacy.load("en_core_web_sm")

        # OCR artifact patterns to clean
        self.ocr_fixes = [
            (r"\boffi ce\b", "office"),
            (r"\bthe m\b", "them"),
            (r"\bwith in\b", "within"),
            (r"\bover all\b", "overall"),
            (r"\bfl ying\b", "flying"),
            (r"\bfi ve\b", "five"),
            (r"\bfi eld\b", "field"),
            (r"\bfi nalising\b", "finalising"),
            (r"\s{2,}", " "),  # Multiple spaces
            (r"([a-z])([A-Z])", r"\1 \2"),  # Missing spaces between words
        ]

        # Create the LangChain text splitter
        self.text_splitter = RecursiveCharacterTextSplitter(
            chunk_size=self.config.target_chunk_size * 6,
            chunk_overlap=self.config.overlap_size * 6,
            separators=["\n\n", "\n", ". ", "! ", "? ", " ", ""],
        )

    def chunk_document(self, text: str, document_name: str = "document") -> List[dict]:
        # Step 1: Clean OCR artifacts
        cleaned_text = self._preprocess_text(text)

        # Step 2: Use spaCy for better text preprocessing
        doc = self.nlp(cleaned_text)

        # Use spaCy's sentence segmentation for cleaner text
        sentences = [sent.text.strip() for sent in doc.sents if sent.text.strip()]
        processed_text = " ".join(sentences)

        # Step 3: Use LangChain to split the cleaned text
        chunks = self.text_splitter.split_text(processed_text)

        # Step 4: Apply quality filtering
        result = []
        for i, chunk in enumerate(chunks):
            # Quality check - only keep meaningful chunks
            if not self._is_meaningful_text(chunk):
                continue

            word_count = len(chunk.split())

            # Skip chunks that are too small
            if word_count < self.config.min_chunk_size:
                continue

            # Use spaCy to analyze chunk type
            chunk_doc = self.nlp(chunk)
            chunk_type = self._classify_chunk_type(chunk_doc)

            result.append(
                {
                    "text": chunk,
                    "metadata": {
                        "document_name": document_name,
                        "chunk_index": i,
                        "word_count": word_count,
                        "char_count": len(chunk),
                        "chunk_type": chunk_type,
                    },
                }
            )

        return result

    def _preprocess_text(self, text: str) -> str:
        """Clean text while preserving document structure (from original code)."""
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

    def _is_meaningful_text(self, text: str) -> bool:
        """Check if text chunk has sufficient information content (from original)."""
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

    def _classify_chunk_type(self, doc) -> str:
        """Use spaCy to classify chunk content type"""
        text = doc.text

        # Check for lists
        if any(
            token.text in ["•", "-", "*"] for token in doc
        ) or text.strip().startswith(("1.", "2.", "3.")):
            return "list_item"

        # Check for headers (all caps or short text)
        if len(text.split()) < 10 and any(token.is_title for token in doc):
            return "heading"

        # Check sentence count
        sentences = list(doc.sents)
        if len(sentences) == 1:
            return "single_sentence"
        elif len(sentences) > 5:
            return "long_passage"
        else:
            return "standard_paragraph"


def smart_chunk_text(
    text: str, document_name: str = "document", config: Optional[ChunkConfig] = None
) -> List[str]:
    chunker = SmartChunker(config)
    chunk_data = chunker.chunk_document(text, document_name)
    return [chunk["text"] for chunk in chunk_data]
