from fastapi import APIRouter, File, UploadFile, HTTPException
from starlette.status import HTTP_400_BAD_REQUEST

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
    return {
        "filename": file.filename,
        "content_type": file.content_type,
        "size": len(content),
        "message": "File uploaded successfully.",
    }
