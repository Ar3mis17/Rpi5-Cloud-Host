from passlib.context import CryptContext
from datetime import datetime, timedelta
from typing import Optional
from pydantic import BaseModel
from jose import JWTError, jwt
from fastapi import HTTPException, status
import bcrypt
import secrets
from app.core.config import get_settings
SETTINGS = get_settings()
# Password hashing
PASSWORD_CONTEXT = CryptContext(schemes=["argon2", "bcrypt"], deprecated=["auto"])
# JWT token utilities
class TokenData(BaseModel):
    username: str
    exp: Optional[datetime] = None
def create_access_token(data: dict, expires_delta: Optional[timedelta] = None) -> str:
    to_encode = data.copy()
    if expires_delta:
        expire = datetime.utcnow() + expires_delta
    else:
        expire = datetime.utcnow() + timedelta(minutes=SETTINGS.ACCESS_TOKEN_EXPIRE_MINUTES)
    to_encode.update({"exp": expire})
    encoded_jwt = jwt.encode(to_encode, SETTINGS.SECRET_KEY, algorithm=SETTINGS.ALGORITHM)
    return encoded_jwt
def verify_token(token: str) -> dict:
    try:
        payload = jwt.decode(token, SETTINGS.SECRET_KEY, algorithms=[SETTINGS.ALGORITHM])
        username: str = payload.get("sub")
        if username is None:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="Could not validate credentials",
                headers={"WWW-Authenticate": "Bearer"},
            )
        token_data = TokenData(username=username, exp=payload.get("exp"))
        return payload
    except JWTError:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Could not validate credentials",
            headers={"WWW-Authenticate": "Bearer"},
        )
def generate_secure_password(password: str) -> str:
    """Generate secure password hash using bcrypt by default."""
    return PASSWORD_CONTEXT.hash(password)
def verify_password(password: str, hashed_password: str) -> bool:
    return PASSWORD_CONTEXT.verify(password, hashed_password)
# CSRF protection
CSRF_TOKENS = {}
def generate_csrf_token(session_id: str) -> str:
    token = secrets.token_urlsafe(32)
    CSRF_TOKENS[session_id] = token
    return token
def verify_csrf_token(session_id: str, token: str) -> bool:
    return CSRF_TOKENS.get(session_id) == token
def revoke_csrf_token(session_id: str):
    CSRF_TOKENS.pop(session_id, None)
# Session management
SESSIONS = {}
def create_session(username: str) -> str:
    session_id = secrets.token_urlsafe(32)
    SESSIONS[session_id] = {
        "username": username,
        "created": datetime.utcnow(),
        "user_agent": "",
        "ip_address": "",
        "revoked": False,
    }
    return session_id
def validate_session(session_id: str) -> dict:
    if session_id not in SESSIONS:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid session",
            headers={"WWW-Authenticate": "Bearer"},
        )
    session = SESSIONS[session_id]
    if session["revoked"]:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Session revoked",
            headers={"WWW-Authenticate": "Bearer"},
        )
    return session
def revoke_session(session_id: str):
    if session_id in SESSIONS:
        SESSIONS[session_id]["revoked"] = True
# Password validation
def validate_password_strength(password: str) -> bool:
    if len(password) < 12:
        return False
    if not any(c.isupper() for c in password):
        return False
    if not any(c.islower() for c in password):
        return False
    if not any(c.isdigit() for c in password):
        return False
    if not any(c in "!@#$%^&*()_+-=[]{}|;:,.<>?" for c in password):
        return False
    return True
# Secure random token generation
def generate_secure_token(length: int = 32) -> str:
    return secrets.token_urlsafe(length)