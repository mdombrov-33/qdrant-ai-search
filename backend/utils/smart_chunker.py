"""
Modern semantic chunking using established libraries.

This implementation uses:
- LangChain RecursiveCharacterTextSplitter for intelligent text splitting
- NLTK for sentence tokenization and text processing
- spaCy for advanced document structure analysis
"""

from typing import List, Optional
from dataclasses import dataclass

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

        # Create the LangChain text splitter
        self.text_splitter = RecursiveCharacterTextSplitter(
            chunk_size=self.config.target_chunk_size * 6,
            chunk_overlap=self.config.overlap_size * 6,
            separators=["\n\n", "\n", ". ", "! ", "? ", " ", ""],
        )

    def chunk_document(self, text: str, document_name: str = "document") -> List[dict]:
        # Use spaCy for better text preprocessing
        doc = self.nlp(text)

        # Use spaCy's sentence segmentation for cleaner text
        sentences = [sent.text.strip() for sent in doc.sents if sent.text.strip()]
        cleaned_text = " ".join(sentences)

        # Use LangChain to split the cleaned text
        chunks = self.text_splitter.split_text(cleaned_text)

        # Convert to the format expected by upload.py
        result = []
        for i, chunk in enumerate(chunks):
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
