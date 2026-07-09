from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel, EmailStr

from app.auth import (
    create_jwt,
    generate_api_key,
    get_current_user,
    hash_api_key,
    hash_password,
    verify_password,
)
from app.db import get_pool

router = APIRouter()


class SignupRequest(BaseModel):
    email: EmailStr
    password: str


class LoginRequest(BaseModel):
    email: EmailStr
    password: str


class AuthResponse(BaseModel):
    token: str
    api_key: str | None = None


@router.post("/auth/signup", response_model=AuthResponse)
async def signup(request: SignupRequest):
    pool = await get_pool()

    existing = await pool.fetchrow("SELECT id FROM users WHERE email = $1", request.email)
    if existing:
        raise HTTPException(status_code=409, detail="Email already registered")

    password_hash = hash_password(request.password)
    api_key = generate_api_key()
    api_key_hash = hash_api_key(api_key)

    row = await pool.fetchrow(
        "INSERT INTO users (email, password_hash, api_key_hash) VALUES ($1, $2, $3) RETURNING id",
        request.email,
        password_hash,
        api_key_hash,
    )

    token = create_jwt(row["id"])
    # api_key is shown exactly once, here, same convention as GitHub/Stripe,
    # we only ever store its hash, if lost, the user must regenerate.
    return AuthResponse(token=token, api_key=api_key)


@router.post("/auth/login", response_model=AuthResponse)
async def login(request: LoginRequest):
    pool = await get_pool()
    row = await pool.fetchrow(
        "SELECT id, password_hash FROM users WHERE email = $1", request.email
    )
    if row is None or not verify_password(request.password, row["password_hash"]):
        raise HTTPException(status_code=401, detail="Invalid email or password")

    return AuthResponse(token=create_jwt(row["id"]))


@router.post("/auth/regenerate-api-key", response_model=AuthResponse)
async def regenerate_api_key(user_id=Depends(get_current_user)):
    pool = await get_pool()
    api_key = generate_api_key()
    await pool.execute(
        "UPDATE users SET api_key_hash = $1 WHERE id = $2", hash_api_key(api_key), user_id
    )
    return AuthResponse(token=create_jwt(user_id), api_key=api_key)