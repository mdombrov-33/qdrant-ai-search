from fastapi import FastAPI
from routes import health, config_routes, upload
from qdrant_service import create_collection, client
from contextlib import asynccontextmanager
from config import settings


@asynccontextmanager
async def lifespan(app: FastAPI):
    # Runs before startup
    create_collection(client, settings.QDRANT_COLLECTION_NAME, vector_size=1536)
    yield
    # Runs on shutdown (if needed)


app = FastAPI(lifespan=lifespan)

app.include_router(health.router)
app.include_router(config_routes.router)
app.include_router(upload.router, prefix="/api")
