from fastapi import APIRouter
from config import settings

router = APIRouter()


@router.get("/config")
async def config():
    return {
        "openrouter_api_key_set": bool(settings.OPENROUTER_API_KEY),
        "qdrant_url": settings.QDRANT_URL,
        "rust_service_url": settings.RUST_SERVICE_URL,
    }
