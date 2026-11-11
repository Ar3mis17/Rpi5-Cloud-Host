from pydantic import BaseModel
from typing import Optional
from datetime import datetime

class ShareCreateRequest(BaseModel):
    file_id: int
    recipient_email: Optional[str] = None
    permissions: str = "read"
    expires_at: Optional[datetime] = None

class ShareResponse(BaseModel):
    id: int
    file_id: int
    owner_id: int
    recipient_user_id: Optional[int]
    share_token: str # Provided once upon creation
    permissions: str
    expires_at: Optional[datetime]
    created_at: datetime

    class Config:
        from_attributes = True
