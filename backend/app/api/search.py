from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Query
from pydantic import BaseModel

from app.retrieval import hybrid_search

# from app.db import get_pool
# from app.embedder import embed_query, to_pgvector_literal

router = APIRouter()

class SearchResult(BaseModel):
    chunk_id: UUID
    document_id: UUID
    chunk_index: int
    content: str
    title: Optional[str]
    source: str
    score: float


@router.get("/search", response_model=list[SearchResult])
async def search(q: str = Query(..., min_length=1), limit: int = Query(10, ge=1, le=50)):
    return await hybrid_search(q, limit)