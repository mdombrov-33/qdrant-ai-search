from fastapi import FastAPI
from routes import health, config, upload


app = FastAPI()

app.include_router(health.router)
app.include_router(config.router)
app.include_router(upload.router, prefix="/api")
