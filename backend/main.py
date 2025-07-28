from fastapi import FastAPI
from routes import health, config_routes, upload, search, summarize
from qdrant_service import create_collection, client
from contextlib import asynccontextmanager
from config import settings
from utils.logging_config import logger


@asynccontextmanager
async def lifespan(app: FastAPI):
    try:
        create_collection(client, settings.QDRANT_COLLECTION_NAME, vector_size=1536)
    except Exception as e:
        logger.warning(f"Failed to create collection on startup: {e}")
    yield


app = FastAPI(lifespan=lifespan)

app.include_router(health.router)
app.include_router(config_routes.router)
app.include_router(upload.router, prefix="/api")
app.include_router(search.router, prefix="/api")
app.include_router(summarize.router, prefix="/api")
