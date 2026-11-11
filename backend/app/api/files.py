from fastapi import APIRouter, Depends, HTTPException, status, Query
from sqlalchemy.orm import Session
from typing import Optional, List
from app.db.session import get_db
from app.models.file import File
from app.models.user import User
from app.schemas.file import FileResponse, FileListResponse, FolderCreate, FileUpdate
from app.api.auth import get_current_user
from app.services.storage import StorageService

router = APIRouter(prefix="/files", tags=["files"])

@router.get("", response_model=FileListResponse)
def list_files(
    parent_id: Optional[int] = None,
    page: int = Query(1, ge=1),
    page_size: int = Query(20, ge=1, le=100),
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    query = db.query(File).filter(
        File.owner_id == current_user.id,
        File.parent_id == parent_id,
        File.deleted_at.is_(None)
    ).order_by(File.name)

    total = query.count()
    items = query.offset((page - 1) * page_size).limit(page_size).all()
    total_pages = (total + page_size - 1) // page_size

    responses = [
        FileResponse(
            id=f.id,
            owner_id=f.owner_id,
            parent_id=f.parent_id,
            name=f.name,
            storage_key=f.storage_key,
            mime_type=f.mime_type,
            size_bytes=f.size_bytes,
            checksum=f.checksum,
            version=f.version,
            is_folder=(f.mime_type == "application/vnd.ar3mis.folder"),
            created_at=f.created_at,
            updated_at=f.updated_at
        ) for f in items
    ]
    return FileListResponse(items=responses, total=total, page=page, page_size=page_size, total_pages=total_pages)

@router.post("", response_model=FileResponse, status_code=status.HTTP_201_CREATED)
def create_folder(
    folder_data: FolderCreate,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    folder = File(
        owner_id=current_user.id,
        parent_id=folder_data.parent_id,
        name=folder_data.name,
        storage_key="folder", # Placeholder
        mime_type="application/vnd.ar3mis.folder",
        size_bytes=0,
        checksum="none"
    )
    db.add(folder)
    db.commit()
    db.refresh(folder)
    return folder

@router.delete("/{file_id}")
def delete_file(
    file_id: int,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    file = db.query(File).filter(File.id == file_id, File.owner_id == current_user.id).first()
    if not file:
        raise HTTPException(status_code=404, detail="File not found")

    file.deleted_at = datetime.utcnow()
    db.commit()
    return {"message": "File moved to trash"}
