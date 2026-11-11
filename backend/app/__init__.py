# Ar3mis Cloud Backend
import logging
logging.basicConfig(level=logging.INFO)

from app.db.session import SessionLocal, get_db, engine, Base, test_connection
from app.core.config import get_settings
SETTINGS = get_settings()