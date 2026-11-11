import os
from functools import lru_cache
from pathlib import Path

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", env_file_encoding="utf-8")

    # PostgreSQL
    POSTGRES_PASSWORD: str = Field(min_length=16)
    POSTGRES_USER: str = "ar3mis_app"
    POSTGRES_DB: str = "ar3mis_cloud"
    DATABASE_URL: str = Field(..., min_length=10)

    # Security
    SECRET_KEY: str = Field(min_length=32)
    ALGORITHM: str = "HS256"
    ACCESS_TOKEN_EXPIRE_MINUTES: int = 15
    REFRESH_TOKEN_EXPIRE_MINUTES: int = 7

    # File upload
    UPLOAD_MAX_SIZE_MB: int = 100
    UPLOAD_CHUNK_SIZE_MB: int = 20
    MAX_CONCURRENT_UPLOADS: int = 5
    DEFAULT_DISK_QUOTA_GB: int = 100

    # Rate limiting
    RATE_LIMIT_REQUESTS_PER_MINUTE: int = 100
    RATE_LIMIT_WINDOW_SECONDS: int = 60

    # CORS
    CORS_ALLOW_ORIGINS: list[str] = [
        "https://app.example.com",
        "http://localhost:5173",
        "http://127.0.0.1:5173",
        "http://localhost:3000",
        "http://localhost:8000",
    ]

    # Security headers
    CSRF_COOKIE_NAME: str = "x-csrf-token"
    SESSION_COOKIE_NAME: str = "session"
    SESSION_COOKIE_SECURE: bool = True
    SESSION_COOKIE_HTTPONLY: bool = True
    SESSION_COOKIE_SAMESITE: str = "lax"
    SESSION_COOKIE_AGE: int = 86400

    # Tokens
    TOKEN_ACCESS_LIFETIME_MINUTES: int = 15
    REFRESH_TOKEN_LIFETIME_MINUTES: int = 7

    # Storage
    UPLOAD_TEMP_DIR: str = "/tmp/uploads"
    STORAGE_ROOT: str = "/mnt/cloud/ar3mis-cloud/files"

    # Logging
    LOG_LEVEL: str = "INFO"
    LOG_FORMAT: str = "json"

    # AWS S3 (optional)
    AWS_ACCESS_KEY_ID: str = ""
    AWS_SECRET_ACCESS_KEY: str = ""
    AWS_BUCKET_NAME: str = ""

    # Redis (optional for future)
    REDIS_URL: str = "redis://localhost:6379/0"

    @property
    def DATABASE_URL_NO_PASSWORD(self) -> str:
        from urllib.parse import urlparse, urlunparse

        parsed = urlparse(self.DATABASE_URL)
        netloc = f"{parsed.username}:****@{parsed.hostname}"
        if parsed.port:
            netloc += f":{parsed.port}"
        return urlunparse(parsed._replace(netloc=netloc))


@lru_cache(maxsize=1)
def get_settings() -> Settings:
    return Settings()