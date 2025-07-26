from fastapi import APIRouter
from config import settings

router = APIRouter()


@router.get("/config")
async def config():
    return {
        "openai_api_key_set": bool(settings.OPENAI_API_KEY),
        "qdrant_url": settings.QDRANT_URL,
        "rust_service_url": settings.RUST_SERVICE_URL,
    }
