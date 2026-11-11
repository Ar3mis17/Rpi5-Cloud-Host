from fastapi import APIRouter
from datetime import datetime
from app.db.session import Base, engine

router = APIRouter()

@router.get("/health")
def health_check():
    """Basic health check endpoint."""
    return {
        "status": "healthy",
        "service": "ar3mis-cloud-api",
        "timestamp": datetime.utcnow().isoformat(),
    }

@router.get("/health/database")
def database_health():
    """Database connectivity check."""
    try:
        Base.metadata.create_all(bind=engine)
        from sqlalchemy import text
        with engine.connect() as conn:
            conn.execute(text("SELECT 1"))
        return {
            "status": "healthy",
            "service": "ar3mis-cloud-api",
            "database": "connected",
            "timestamp": datetime.utcnow().isoformat(),
        }
    except Exception as e:
        return {
            "status": "unhealthy",
            "service": "ar3mis-cloud-api",
            "database": "disconnected",
            "error": str(e),
            "timestamp": datetime.utcnow().isoformat(),
        }

@router.get("/")
def root():
    """API root endpoint."""
    return {
        "name": "Ar3mis Cloud",
        "version": "0.1.0",
        "status": "running",
        "documentation": "/docs",
        "health": "/api/v1/health",
        "api": "/api/v1/health",
    }

# Initialize tables
Base.metadata.create_all(bind=engine)