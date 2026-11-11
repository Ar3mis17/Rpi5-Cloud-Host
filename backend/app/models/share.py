from sqlalchemy import Column, Integer, String, DateTime, text, ForeignKey
from sqlalchemy.orm import relationship
from datetime import datetime
from app.db.session import Base
class Share(Base):
    __tablename__ = "shares"

    id = Column(Integer, primary_key=True, index=True)
    file_id = Column(Integer, ForeignKey("files.id"), nullable=False)
    owner_id = Column(Integer, ForeignKey("users.id"), nullable=False)
    recipient_user_id = Column(Integer, ForeignKey("users.id"), nullable=True)
    share_token_hash = Column(String(255), nullable=False)
    permissions = Column(String(50), default="read", nullable=False)
    expires_at = Column(DateTime, nullable=True)
    created_at = Column(DateTime, server_default=text("CURRENT_TIMESTAMP"))

    file = relationship("File", back_populates="shares")
    owner = relationship("User", foreign_keys=[owner_id], back_populates="shares")
    recipient = relationship("User", foreign_keys=[recipient_user_id])
    shared_by = relationship("User", foreign_keys=[owner_id])