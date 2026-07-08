from contextlib import asynccontextmanager
from fastapi import FastAPI
from app.api.search import router as search_router
from app.db import close_pool, get_pool
from app.api.chat import router as chat_router


@asynccontextmanager
async def lifespan(app: FastAPI):
    await get_pool()
    yield
    await close_pool()

app = FastAPI(title="Cairn", version="0.1.0", lifespan=lifespan)
app.include_router(search_router)
app.include_router(chat_router)

@app.get("/health")
async def health():
    pool = await get_pool()
    async with pool.acquire() as conn:
        await conn.execute("SELECT 1")
    return {"status": "ok", "service": "cairn-backend", "db": "connected"}