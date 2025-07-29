from pydantic_settings import BaseSettings
import os


class Settings(BaseSettings):
    OPENAI_API_KEY: str = "test-key" if os.getenv("TESTING") else ""
    QDRANT_URL: str = "http://localhost:6333" if os.getenv("TESTING") else ""
    RUST_SERVICE_URL: str = "http://localhost:5000" if os.getenv("TESTING") else ""
    QDRANT_COLLECTION_NAME: str = "documents"

    # Chat completion settings
    OPENAI_CHAT_MODEL: str = "gpt-4o"
    MAX_SUMMARY_TOKENS: int = 500
    SUMMARY_TEMPERATURE: float = 0.3

    class Config:
        env_file = ".env"  # used only for local development when running without Docker


settings = Settings()
