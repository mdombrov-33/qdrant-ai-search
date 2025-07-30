from fastapi import FastAPI
from routes import health, config_routes, upload, search, summarize
from services.qdrant_service import create_collection, client
from prometheus_fastapi_instrumentator import Instrumentator
from contextlib import asynccontextmanager
from config import settings
from utils.logging_config import logger
from exceptions import QdrantServiceError
import asyncio


@asynccontextmanager
async def lifespan(app: FastAPI):
    retries = 10
    delay = 3
    qdrant_connected = False
    for attempt in range(retries):
        try:
            create_collection(client, settings.QDRANT_COLLECTION_NAME, vector_size=1536)
            logger.info("Qdrant collection created successfully on startup")
            qdrant_connected = True
            break
        except QdrantServiceError as e:
            logger.warning(f"Attempt {attempt + 1} failed to create collection: {e}")
        except Exception as e:
            logger.warning(f"Attempt {attempt + 1} unexpected error: {e}")
        if attempt < retries - 1:
            await asyncio.sleep(delay)
            delay *= 2
    if not qdrant_connected:
        logger.error("Exceeded max retries for creating Qdrant collection on startup.")
    yield


app = FastAPI(lifespan=lifespan)
instrumentator = Instrumentator()
instrumentator.instrument(app).expose(app)

app.include_router(health.router)
app.include_router(config_routes.router)
app.include_router(upload.router, prefix="/api")
app.include_router(search.router, prefix="/api")
app.include_router(summarize.router, prefix="/api")
