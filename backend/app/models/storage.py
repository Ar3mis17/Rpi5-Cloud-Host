from sqlalchemy import Column, Integer, BigInteger, DateTime, text, ForeignKey
from sqlalchemy.orm import relationship
from datetime import datetime
from app.db.session import Base
class UserStorage(Base):
    __tablename__ = "user_storage"

    id = Column(Integer, primary_key=True, index=True)
    user_id = Column(Integer, ForeignKey("users.id"), nullable=False)
    quota_bytes = Column(BigInteger, default=0, nullable=False)
    used_bytes = Column(BigInteger, default=0, nullable=False)
    created_at = Column(DateTime, server_default=text("CURRENT_TIMESTAMP"))
    updated_at = Column(
        DateTime, server_default=text("CURRENT_TIMESTAMP"), onupdate=text("CURRENT_TIMESTAMP")
    )

    user = relationship("User", back_populates="storage")