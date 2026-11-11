import os
import shutil
import hashlib
import uuid
from pathlib import Path
from typing import List, Optional
from sqlalchemy.orm import Session
from fastapi import HTTPException, status

from app.core.config import get_settings
from app.models.file import File
from app.models.storage import UserStorage

SETTINGS = get_settings()

class StorageService:
    def __init__(self, user_id: int):
        self.user_id = user_id
        self.base_dir = Path(SETTINGS.STORAGE_ROOT) / "users" / str(user_id)
        self.files_dir = self.base_dir / "files"
        self.temp_dir = self.base_dir / "temp"
        self.trash_dir = self.base_dir / "trash"

        # Ensure directories exist
        os.makedirs(self.files_dir, exist_ok=True)
        os.makedirs(self.temp_dir, exist_ok=True)
        os.makedirs(self.trash_dir, exist_ok=True)

    def check_quota(self, db: Session, additional_bytes: int):
        storage = db.query(UserStorage).filter(UserStorage.user_id == self.user_id).first()
        if not storage:
            # Default quota if not found
            quota = SETTINGS.DEFAULT_DISK_QUOTA_GB * 1024 * 1024 * 1024
            used = 0
        else:
            quota = storage.quota_bytes
            used = storage.used_bytes

        if used + additional_bytes > quota:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Storage quota exceeded"
            )

    def update_used_storage(self, db: Session, bytes_delta: int):
        storage = db.query(UserStorage).filter(UserStorage.user_id == self.user_id).first()
        if storage:
            storage.used_bytes += bytes_delta
            db.commit()

    def initiate_upload(self, file_size: int) -> str:
        upload_id = str(uuid.uuid4())
        upload_path = self.temp_dir / upload_id
        os.makedirs(upload_path, exist_ok=True)
        return upload_id

    def save_chunk(self, upload_id: str, chunk_number: int, chunk_data: bytes):
        upload_path = self.temp_dir / upload_id
        if not upload_path.exists():
            raise HTTPException(status_code=404, detail="Upload session not found")

        chunk_file = upload_path / f"chunk_{chunk_number}"
        with open(chunk_file, "wb") as f:
            f.write(chunk_data)

    def get_received_chunks(self, upload_id: str) -> List[int]:
        upload_path = self.temp_dir / upload_id
        if not upload_path.exists():
            return []

        chunks = []
        for file in os.listdir(upload_path):
            if file.startswith("chunk_"):
                try:
                    chunks.append(int(file.split("_")[1]))
                except ValueError:
                    pass
        return sorted(chunks)

    def assemble_file(self, upload_id: str, total_chunks: int) -> tuple[str, str, int]:
        upload_path = self.temp_dir / upload_id
        if not upload_path.exists():
            raise HTTPException(status_code=404, detail="Upload session not found")

        received = self.get_received_chunks(upload_id)
        if len(received) != total_chunks:
            raise HTTPException(
                status_code=400,
                detail=f"Incomplete upload: received {len(received)} of {total_chunks} chunks"
            )

        storage_key = str(uuid.uuid4())
        final_file_path = self.files_dir / storage_key

        hasher = hashlib.sha256()
        total_bytes = 0

        with open(final_file_path, "wb") as outfile:
            for i in range(1, total_chunks + 1):
                chunk_file = upload_path / f"chunk_{i}"
                if not chunk_file.exists():
                    # Clean up
                    os.remove(final_file_path) if final_file_path.exists() else None
                    raise HTTPException(status_code=400, detail=f"Missing chunk {i}")

                with open(chunk_file, "rb") as infile:
                    data = infile.read()
                    hasher.update(data)
                    outfile.write(data)
                    total_bytes += len(data)

        # Cleanup temp directory
        shutil.rmtree(upload_path, ignore_errors=True)

        return storage_key, hasher.hexdigest(), total_bytes

    def delete_physical_file(self, storage_key: str):
        file_path = self.files_dir / storage_key
        if file_path.exists():
            os.remove(file_path)
