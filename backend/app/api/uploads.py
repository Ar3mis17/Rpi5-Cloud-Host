import math
from typing import List, Optional
from fastapi import APIRouter, Depends, HTTPException, status, UploadFile, File as FastAPIFile, Form
from sqlalchemy.orm import Session
from app.db.session import get_db
from app.models.user import User
from app.models.file import File
from app.schemas.file import (
    UploadInitiateRequest,
    UploadInitiateResponse,
    UploadStatusResponse,
    FileResponse
)
from app.api.auth import get_current_user
from app.services.storage import StorageService
from app.core.config import get_settings

SETTINGS = get_settings()
router = APIRouter(prefix="/uploads", tags=["uploads"])

# In-memory store for upload sessions metadata (can be persisted to DB in production)
# Structure: { upload_id: { "filename": str, "size_bytes": int, "parent_id": Optional[int], "user_id": int, "mime_type": str, "total_chunks": int } }
UPLOAD_SESSIONS = {}

@router.post("/initiate", response_model=UploadInitiateResponse)
def initiate_upload(
    req: UploadInitiateRequest,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    storage_svc = StorageService(current_user.id)
    # 1. Quota check
    storage_svc.check_quota(db, req.size_bytes)

    # 2. Initiate physical directory
    upload_id = storage_svc.initiate_upload(req.size_bytes)

    chunk_size = SETTINGS.UPLOAD_CHUNK_SIZE_MB * 1024 * 1024
    total_chunks = math.ceil(req.size_bytes / chunk_size) if req.size_bytes > 0 else 1

    # 3. Cache upload session details
    UPLOAD_SESSIONS[upload_id] = {
        "filename": req.filename,
        "size_bytes": req.size_bytes,
        "parent_id": req.parent_id,
        "user_id": current_user.id,
        "mime_type": req.mime_type or "application/octet-stream",
        "total_chunks": total_chunks
    }

    return UploadInitiateResponse(
        upload_id=upload_id,
        chunk_size_bytes=chunk_size,
        total_chunks=total_chunks
    )

@router.put("/{upload_id}/chunks/{chunk_number}")
async def upload_chunk(
    upload_id: str,
    chunk_number: int,
    file: UploadFile = FastAPIFile(...),
    current_user: User = Depends(get_current_user)
):
    if upload_id not in UPLOAD_SESSIONS:
        raise HTTPException(status_code=404, detail="Upload session not found")

    session_info = UPLOAD_SESSIONS[upload_id]
    if session_info["user_id"] != current_user.id:
        raise HTTPException(status_code=403, detail="Unauthorized upload session")

    chunk_data = await file.read()
    storage_svc = StorageService(current_user.id)
    storage_svc.save_chunk(upload_id, chunk_number, chunk_data)

    return {"message": f"Chunk {chunk_number} uploaded successfully"}

@router.get("/{upload_id}/status", response_model=UploadStatusResponse)
def get_upload_status(
    upload_id: str,
    current_user: User = Depends(get_current_user)
):
    if upload_id not in UPLOAD_SESSIONS:
        raise HTTPException(status_code=404, detail="Upload session not found")

    session_info = UPLOAD_SESSIONS[upload_id]
    if session_info["user_id"] != current_user.id:
        raise HTTPException(status_code=403, detail="Unauthorized access to upload session")

    storage_svc = StorageService(current_user.id)
    received = storage_svc.get_received_chunks(upload_id)

    status_str = "completed" if len(received) == session_info["total_chunks"] else "pending"

    return UploadStatusResponse(
        upload_id=upload_id,
        received_chunks=received,
        total_chunks=session_info["total_chunks"],
        status=status_str
    )

@router.post("/{upload_id}/complete", response_model=FileResponse)
def complete_upload(
    upload_id: str,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    if upload_id not in UPLOAD_SESSIONS:
        raise HTTPException(status_code=404, detail="Upload session not found")

    session_info = UPLOAD_SESSIONS[upload_id]
    if session_info["user_id"] != current_user.id:
        raise HTTPException(status_code=403, detail="Unauthorized upload session")

    storage_svc = StorageService(current_user.id)

    # Re-check quota before completing
    storage_svc.check_quota(db, session_info["size_bytes"])

    # Assemble file chunks
    storage_key, checksum, total_bytes = storage_svc.assemble_file(
        upload_id=upload_id,
        total_chunks=session_info["total_chunks"]
    )

    # Save File record in PostgreSQL
    db_file = File(
        owner_id=current_user.id,
        parent_id=session_info["parent_id"],
        name=session_info["filename"],
        storage_key=storage_key,
        mime_type=session_info["mime_type"],
        size_bytes=total_bytes,
        checksum=checksum,
        version=1
    )
    db.add(db_file)

    # Update user storage usage
    storage_svc.update_used_storage(db, total_bytes)

    db.commit()
    db.refresh(db_file)

    # Cleanup session cache
    del UPLOAD_SESSIONS[upload_id]

    return FileResponse(
        id=db_file.id,
        owner_id=db_file.owner_id,
        parent_id=db_file.parent_id,
        name=db_file.name,
        storage_key=db_file.storage_key,
        mime_type=db_file.mime_type,
        size_bytes=db_file.size_bytes,
        checksum=db_file.checksum,
        version=db_file.version,
        is_folder=False,
        created_at=db_file.created_at,
        updated_at=db_file.updated_at
    )
