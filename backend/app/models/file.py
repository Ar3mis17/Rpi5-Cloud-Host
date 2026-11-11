from sqlalchemy import Column, Integer, String, BigInteger, DateTime, text, ForeignKey
from sqlalchemy.orm import relationship
from app.db.session import Base
class File(Base):
    __tablename__ = "files"

    id = Column(Integer, primary_key=True, index=True)
    owner_id = Column(Integer, ForeignKey("users.id"), nullable=False)
    parent_id = Column(Integer, ForeignKey("files.id"), nullable=True)
    name = Column(String(255), nullable=False)
    storage_key = Column(String(255), unique=True, nullable=False)
    mime_type = Column(String(100))
    size_bytes = Column(BigInteger, nullable=False)
    checksum = Column(String(64), nullable=False)
    version = Column(Integer, default=1, nullable=False)
    created_at = Column(DateTime, server_default=text("CURRENT_TIMESTAMP"))
    updated_at = Column(
        DateTime, server_default=text("CURRENT_TIMESTAMP"), onupdate=text("CURRENT_TIMESTAMP")
    )
    deleted_at = Column(DateTime, nullable=True)

    owner = relationship("User", back_populates="files")
    parent = relationship("File", remote_side=[id], backref="children")
    shares = relationship("Share", back_populates="file")