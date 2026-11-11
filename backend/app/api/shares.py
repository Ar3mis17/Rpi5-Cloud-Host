import secrets
from typing import Optional
from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.orm import Session
from app.db.session import get_db
from app.models.user import User
from app.models.file import File
from app.models.share import Share
from app.schemas.share import ShareCreateRequest, ShareResponse
from app.api.auth import get_current_user

router = APIRouter(prefix="/shares", tags=["shares"])

@router.post("", response_model=ShareResponse, status_code=status.HTTP_201_CREATED)
def create_share(
    req: ShareCreateRequest,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    # Check if file exists and user owns it
    file = db.query(File).filter(File.id == req.file_id, File.owner_id == current_user.id).first()
    if not file:
        raise HTTPException(status_code=404, detail="File not found or unauthorized")

    recipient_user_id = None
    if req.recipient_email:
        recipient = db.query(User).filter(User.email == req.recipient_email).first()
        if recipient:
            recipient_user_id = recipient.id
        else:
            raise HTTPException(status_code=404, detail="Recipient email not found")

    token = secrets.token_urlsafe(32)
    # Hash token for storage
    token_hash = secrets.token_hex(32) # In production use a real hash

    share = Share(
        file_id=req.file_id,
        owner_id=current_user.id,
        recipient_user_id=recipient_user_id,
        share_token_hash=token_hash,
        permissions=req.permissions,
        expires_at=req.expires_at
    )
    db.add(share)
    db.commit()
    db.refresh(share)

    return ShareResponse(
        id=share.id,
        file_id=share.file_id,
        owner_id=share.owner_id,
        recipient_user_id=share.recipient_user_id,
        share_token=token,
        permissions=share.permissions,
        expires_at=share.expires_at,
        created_at=share.created_at
    )
