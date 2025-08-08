from fastapi import FastAPI
from routes import health, config_routes, upload, search, summarize
from prometheus_fastapi_instrumentator import Instrumentator


app = FastAPI()
instrumentator = Instrumentator()
instrumentator.instrument(app).expose(app)

app.include_router(health.router)
app.include_router(config_routes.router)
app.include_router(upload.router, prefix="/api")
app.include_router(search.router, prefix="/api")
app.include_router(summarize.router, prefix="/api")
