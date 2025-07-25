from fastapi import FastAPI
from routes import health, config


app = FastAPI()

app.include_router(health.router)
app.include_router(config.router)
