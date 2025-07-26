from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    OPENAI_API_KEY: str
    QDRANT_URL: str
    RUST_SERVICE_URL: str
    QDRANT_COLLECTION_NAME: str = "documents"

    class Config:
        env_file = ".env"  # used only for local development when running without Docker


settings = Settings()
