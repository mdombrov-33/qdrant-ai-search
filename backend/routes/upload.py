from fastapi import APIRouter, File, UploadFile, HTTPException
from starlette.status import HTTP_400_BAD_REQUEST
from file_loader import extract_text
from utils.text_cleaner import clean_text
from embedding import chunk_text, embed_chunks
from qdrant_service import insert_vectors_batch, client
from config import settings
import uuid

router = APIRouter()


@router.post("/upload")
async def upload_file(file: UploadFile = File(...)):
    MAX_FILE_SIZE = 50 * 1024 * 1024  # 50MB
    content = await file.read()

    if len(content) > MAX_FILE_SIZE:
        raise HTTPException(
            status_code=HTTP_400_BAD_REQUEST,
            detail=f"File too large. Maximum size is {MAX_FILE_SIZE // (1024 * 1024)}",
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

    try:
        extracted_text = extract_text(content, file.content_type)
    except ValueError as e:
        raise HTTPException(
            status_code=HTTP_400_BAD_REQUEST,
            detail=str(e),
        )
    cleaned_text = clean_text(extracted_text, lowercase=True)
    chunks = chunk_text(cleaned_text)
    embedded_chunks = await embed_chunks(chunks)

    if not embedded_chunks:
        raise HTTPException(
            status_code=HTTP_400_BAD_REQUEST,
            detail="No valid text chunks found for embedding.",
        )

    items_to_insert = []
    for embed_dict in embedded_chunks:
        items_to_insert.append(
            {
                "id": str(uuid.uuid4()),
                "embedding": embed_dict["embedding"],
                "payload": {
                    "text": embed_dict["chunk"],
                    "file_name": file.filename,
                    "content_type": file.content_type,
                },
            }
        )

    try:
        insert_vectors_batch(
            client=client,
            collection_name=settings.QDRANT_COLLECTION_NAME,
            items=items_to_insert,
        )
    except Exception as e:
        raise HTTPException(
            status_code=500, detail=f"Failed to store vectors in database: {str(e)}"
        )

    return {
        "filename": file.filename,
        "content_type": file.content_type,
        "size": len(content),
        "chunks_count": len(chunks),
        "message": "File processed and embedded successfully.",
    }
