from fastapi import APIRouter, File, UploadFile, HTTPException
from starlette.status import HTTP_400_BAD_REQUEST
from utils.file_loader import extract_text
from utils.text_cleaner import clean_text
from utils.smart_chunker import SmartChunker, ChunkConfig
from utils.exception_handler import handle_custom_exception
from services.embedding import embed_chunks
from services.qdrant_service import insert_vectors_batch, client
from config import settings
from utils.logging_config import logger
from exceptions import QdrantSearchError
import uuid

router = APIRouter()


@router.post("/upload")
async def upload_file(file: UploadFile = File(...)):
    """
    Upload and process a document with advanced semantic chunking.

    This endpoint uses SmartChunker for better text segmentation that:
    - Preserves sentence and paragraph boundaries
    - Maintains context with overlapping chunks
    - Handles OCR artifacts and document structure
    - Provides better search results through semantic coherence
    """
    try:
        MAX_FILE_SIZE = 50 * 1024 * 1024  # 50MB
        content = await file.read()

        if len(content) > MAX_FILE_SIZE:
            max_size_mb = MAX_FILE_SIZE // (1024 * 1024)
            raise HTTPException(
                status_code=HTTP_400_BAD_REQUEST,
                detail=f"File too large. Maximum size is {max_size_mb}MB",
            )

        if len(content) == 0:
            raise HTTPException(
                status_code=HTTP_400_BAD_REQUEST, detail="Empty file uploaded"
            )

        if file.content_type not in [
            "application/pdf",
            "text/plain",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        ]:
            raise HTTPException(
                status_code=HTTP_400_BAD_REQUEST,
                detail="Unsupported file type. Please upload a PDF, TXT, or DOCX file.",
            )

        extracted_text = extract_text(content, file.content_type)
        logger.info(f"Extracted {len(extracted_text)} characters from {file.filename}")

        # Step 1: Use advanced semantic chunking instead of simple word-based chunking
        cleaned_text = clean_text(
            extracted_text, lowercase=False
        )  # Keep case for better chunking

        # Step 2: Configure chunking for better search results
        chunk_config = ChunkConfig(
            target_chunk_size=250,  # Slightly smaller for more focused results
            min_chunk_size=80,  # Avoid tiny fragments
            max_chunk_size=400,  # Cap very long chunks
            overlap_size=40,  # Good context overlap
            preserve_paragraphs=True,
            sentence_aware=True,
        )

        # Step 3: Create semantic chunks with metadata
        chunker = SmartChunker(chunk_config)
        chunk_data = chunker.chunk_document(cleaned_text, file.filename or "document")

        if not chunk_data:
            raise HTTPException(
                status_code=HTTP_400_BAD_REQUEST,
                detail="No meaningful text chunks found for embedding.",
            )

        logger.info(f"Created {len(chunk_data)} semantic chunks from {file.filename}")

        # Step 4: Extract just the text for embedding (maintain backward compatibility)
        chunks = [chunk["text"] for chunk in chunk_data]
        embedded_chunks = await embed_chunks(chunks)

        if not embedded_chunks:
            raise HTTPException(
                status_code=HTTP_400_BAD_REQUEST,
                detail="No valid text chunks found for embedding.",
            )

        items_to_insert = []
        for i, embed_dict in enumerate(embedded_chunks):
            # Get original chunk metadata
            chunk_metadata = chunk_data[i]["metadata"] if i < len(chunk_data) else {}

            items_to_insert.append(
                {
                    "id": str(uuid.uuid4()),
                    "embedding": embed_dict["embedding"],
                    "payload": {
                        "text": embed_dict["chunk"],
                        "file_name": file.filename,
                        "content_type": file.content_type,
                        # Enhanced metadata from semantic chunking
                        "chunk_index": chunk_metadata.get("chunk_index", i),
                        "word_count": chunk_metadata.get(
                            "word_count", len(embed_dict["chunk"].split())
                        ),
                        "chunk_type": chunk_metadata.get(
                            "chunk_type", "standard_paragraph"
                        ),
                    },
                }
            )

        insert_vectors_batch(
            client=client,
            collection_name=settings.QDRANT_COLLECTION_NAME,
            items=items_to_insert,
        )
        logger.info(
            f"Successfully inserted {len(items_to_insert)} chunks for {file.filename}"
        )

        return {
            "filename": file.filename,
            "content_type": file.content_type,
            "size": len(content),
            "chunks_count": len(chunks),
            "message": "File processed and embedded successfully.",
        }

    except QdrantSearchError as e:
        # Handle our custom exceptions
        raise handle_custom_exception(e)
    except HTTPException:
        # Re-raise HTTP exceptions as-is
        raise
    except Exception as e:
        # Handle unexpected errors
        logger.error(f"Unexpected error in upload: {e}")
        raise HTTPException(
            status_code=500,
            detail="An unexpected error occurred during file processing",
        )
