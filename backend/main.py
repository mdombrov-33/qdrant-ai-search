from fastapi import FastAPI
from config import settings

app = FastAPI()


@app.get("/health")
async def health_check():
    return {"status": "ok"}


@app.get("/config")
async def config():
    return {
        "openrouter_api_key_set": bool(settings.OPENROUTER_API_KEY),
        "qdrant_url": settings.QDRANT_URL,
        "rust_service_url": settings.RUST_SERVICE_URL,
    }
