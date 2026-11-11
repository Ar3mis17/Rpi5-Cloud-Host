from fastapi import APIRouter
from app.api import health, auth, files, uploads, shares

api_router = APIRouter()
api_router.include_router(health.router)
api_router.include_router(auth.router)
api_router.include_router(files.router, prefix="/api/v1")
api_router.include_router(uploads.router, prefix="/api/v1")
api_router.include_router(shares.router, prefix="/api/v1")