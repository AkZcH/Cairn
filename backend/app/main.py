from contextlib import asynccontextmanager
from fastapi import FastAPI
from app.api.search import router as search_router
from app.db import close_pool, get_pool
from app.api.chat import router as chat_router
from app.api.conversations import router as conversations_router
from app.api.auth import router as auth_router
from app.api.documents import router as documents_router
from fastapi.middleware.cors import CORSMiddleware
from app.api.upload import router as upload_router
from prometheus_fastapi_instrumentator import Instrumentator


@asynccontextmanager
async def lifespan(app: FastAPI):
    await get_pool()
    yield
    await close_pool()

app = FastAPI(title="Cairn", version="0.1.0", lifespan=lifespan)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

Instrumentator().instrument(app).expose(app)

app.include_router(search_router)
app.include_router(chat_router)
app.include_router(conversations_router)
app.include_router(auth_router)
app.include_router(documents_router)
app.include_router(upload_router)

@app.get("/health")
async def health():
    pool = await get_pool()
    async with pool.acquire() as conn:
        await conn.execute("SELECT 1")
    return {"status": "ok", "service": "cairn-backend", "db": "connected"}