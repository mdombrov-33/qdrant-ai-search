from fastapi import APIRouter, File, UploadFile, HTTPException
from starlette.status import HTTP_400_BAD_REQUEST
from file_loader import extract_text
from utils.text_cleaner import clean_text

router = APIRouter()


@router.post("/upload")
async def upload_file(file: UploadFile = File(...)):
    if file.content_type not in [
        "application/pdf",
        "text/plain",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    ]:
        raise HTTPException(
            status_code=HTTP_400_BAD_REQUEST,
            detail="Unsupported file type. Please upload a PDF, TXT, or DOCX file.",
        )

    content = await file.read()
    try:
        extracted_text = extract_text(content, file.content_type)
        cleaned_text = clean_text(extracted_text, lowercase=True)
    except ValueError as e:
        raise HTTPException(
            status_code=HTTP_400_BAD_REQUEST,
            detail=str(e),
        )

    return {
        "filename": file.filename,
        "content_type": file.content_type,
        "size": len(content),
        "extracted_text": cleaned_text,
        "message": "File uploaded and text extracted successfully.",
    }
