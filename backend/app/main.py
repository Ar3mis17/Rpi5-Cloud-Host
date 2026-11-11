from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.api import api_router
from app.core.config import get_settings

SETTINGS = get_settings()
app = FastAPI(
    title="Ar3mis Cloud API",
    version="0.1.0",
    description="Self-hosted cloud storage platform backend",
    docs_url="/docs",
    redoc_url=None,
    openapi_url="/openapi.json",
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=SETTINGS.CORS_ALLOW_ORIGINS,
    allow_credentials=True,
    allow_methods=["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"],
    allow_headers=["*"],
    expose_headers=["*"],
)

# Include API routes
app.include_router(api_router, prefix="/api/v1")

# Root endpoint
@app.get("/")
def root():
    return {
        "name": "Ar3mis Cloud",
        "version": "0.1.0",
        "status": "running",
        "documentation": "/docs",
    }

# Health endpoint
@app.get("/health")
def health():
    from app.db.session import test_connection
    db_status = "connected" if test_connection() else "disconnected"
    return {
        "status": "healthy" if test_connection() else "unhealthy",
        "service": "ar3mis-cloud-api",
        "database": db_status,
    }