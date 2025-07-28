import io
import pdfplumber
from docx import Document
from exceptions import UnsupportedFileTypeError, FileExtractionError


def extract_text_from_pdf(file_bytes: bytes) -> str:
    try:
        text = ""
        with pdfplumber.open(io.BytesIO(file_bytes)) as pdf:
            for page in pdf.pages:
                text += page.extract_text() + "\n"
        return text.strip()
    except Exception as e:
        raise FileExtractionError(f"Failed to extract text from PDF: {str(e)}")


def extract_text_from_txt(file_bytes: bytes) -> str:
    try:
        return file_bytes.decode("utf-8", errors="ignore").strip()
    except Exception as e:
        raise FileExtractionError(f"Failed to extract text from TXT: {str(e)}")


def extract_text_from_docx(file_bytes: bytes) -> str:
    try:
        text = ""
        file_stream = io.BytesIO(file_bytes)
        doc = Document(file_stream)
        for para in doc.paragraphs:
            text += para.text + "\n"
        return text.strip()
    except Exception as e:
        raise FileExtractionError(f"Failed to extract text from DOCX: {str(e)}")


def extract_text(file_bytes: bytes, content_type: str) -> str:
    if content_type == "application/pdf":
        return extract_text_from_pdf(file_bytes)
    elif content_type == "text/plain":
        return extract_text_from_txt(file_bytes)
    elif (
        content_type
        == "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    ):
        return extract_text_from_docx(file_bytes)
    else:
        raise UnsupportedFileTypeError(f"Unsupported file type: {content_type}")
