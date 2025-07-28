from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    OPENAI_API_KEY: str
    QDRANT_URL: str
    RUST_SERVICE_URL: str
    QDRANT_COLLECTION_NAME: str = "documents"

    # Chat completion settings
    OPENAI_CHAT_MODEL: str = "gpt-4o"
    MAX_SUMMARY_TOKENS: int = 500
    SUMMARY_TEMPERATURE: float = 0.3

    class Config:
        env_file = ".env"  # used only for local development when running without Docker


settings = Settings()
