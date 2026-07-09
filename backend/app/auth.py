import hashlib
import os
import secrets
from datetime import datetime, timedelta, timezone
from uuid import UUID

import jwt
from fastapi import Depends, HTTPException
from fastapi.security import HTTPAuthorizationCredentials, HTTPBearer
# from passlib.context import CryptContext
import bcrypt

from app.db import get_pool

# pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")
JWT_SECRET = os.environ.get("JWT_SECRET")
if not JWT_SECRET:
    raise RuntimeError("JWT_SECRET must be set (see infra/.env.example)")
JWT_ALGORITHM = "HS256"
JWT_EXPIRE_MINUTES = 60 * 24 * 7  # 1 week

security = HTTPBearer()

def hash_password(password: str) -> str:
    # bcrypt has a genuine 72-byte input limit (not the false one passlib
    # was hitting), truncating here is the standard, safe handling, not a
    # workaround, real-world passwords essentially never approach this.
    return bcrypt.hashpw(password.encode("utf-8")[:72], bcrypt.gensalt()).decode("utf-8")


def verify_password(password: str, password_hash: str) -> bool:
    return bcrypt.checkpw(password.encode("utf-8")[:72], password_hash.encode("utf-8"))


# def hash_password(password: str) -> str:
#     return pwd_context.hash(password)


# def verify_password(password: str, password_hash: str) -> bool:
#     return pwd_context.verify(password, password_hash)


def generate_api_key() -> str:
    return "cairn_" + secrets.token_urlsafe(32)


def hash_api_key(api_key: str) -> str:
    # API keys are long, random, and high-entropy already, unlike passwords,
    # so a fast hash (not bcrypt) is fine here, we're not defending against
    # brute-forcing a low-entropy secret.
    return hashlib.sha256(api_key.encode()).hexdigest()


def create_jwt(user_id: UUID) -> str:
    payload = {
        "sub": str(user_id),
        "exp": datetime.now(timezone.utc) + timedelta(minutes=JWT_EXPIRE_MINUTES),
    }
    return jwt.encode(payload, JWT_SECRET, algorithm=JWT_ALGORITHM)


async def get_current_user(
    credentials: HTTPAuthorizationCredentials = Depends(security),
) -> UUID:
    """Frontend-facing endpoints: verifies a short-lived JWT."""
    try:
        payload = jwt.decode(credentials.credentials, JWT_SECRET, algorithms=[JWT_ALGORITHM])
        return UUID(payload["sub"])
    except jwt.PyJWTError:
        raise HTTPException(status_code=401, detail="Invalid or expired token")


async def get_current_user_from_api_key(
    credentials: HTTPAuthorizationCredentials = Depends(security),
) -> UUID:
    """cairn-ingest: verifies a long-lived API key, not a JWT, a CLI tool
    shouldn't have to re-login every week."""
    pool = await get_pool()
    key_hash = hash_api_key(credentials.credentials)
    row = await pool.fetchrow("SELECT id FROM users WHERE api_key_hash = $1", key_hash)
    if row is None:
        raise HTTPException(status_code=401, detail="Invalid API key")
    return row["id"]