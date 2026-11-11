from pydantic import BaseModel, Field
from typing import Optional, List
from datetime import datetime

class FileBase(BaseModel):
    name: str = Field(..., min_length=1, max_length=255)
    parent_id: Optional[int] = None

class FolderCreate(FileBase):
    pass

class FileUpdate(BaseModel):
    name: Optional[str] = Field(None, min_length=1, max_length=255)

class FileResponse(BaseModel):
    id: int
    owner_id: int
    parent_id: Optional[int]
    name: str
    storage_key: str
    mime_type: Optional[str]
    size_bytes: int
    checksum: str
    version: int
    is_folder: bool = False
    created_at: datetime
    updated_at: datetime

    class Config:
        from_attributes = True

class FileListResponse(BaseModel):
    items: List[FileResponse]
    total: int
    page: int
    page_size: int
    total_pages: int

# Upload Schemas
class UploadInitiateRequest(BaseModel):
    filename: str = Field(..., min_length=1, max_length=255)
    size_bytes: int = Field(..., gt=0)
    mime_type: Optional[str] = "application/octet-stream"
    parent_id: Optional[int] = None

class UploadInitiateResponse(BaseModel):
    upload_id: str
    chunk_size_bytes: int
    total_chunks: int

class UploadStatusResponse(BaseModel):
    upload_id: str
    received_chunks: List[int]
    total_chunks: int
    status: str # "pending", "completed", "failed"
