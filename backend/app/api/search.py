from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Query
from pydantic import BaseModel

from app.db import get_pool

router = APIRouter()


class SearchResult(BaseModel):
    chunk_id: UUID
    document_id: UUID
    chunk_index: int
    content: str
    title: Optional[str]
    source: str
    rank: float


@router.get("/search", response_model=list[SearchResult])
async def search(q: str = Query(..., min_length=1), limit: int = Query(10, ge=1, le=50)):
    pool = await get_pool()
    rows = await pool.fetch(
        """
        SELECT c.id AS chunk_id, c.document_id, c.chunk_index, c.content,
               d.title, d.source,
               ts_rank(c.content_tsv, plainto_tsquery('english', $1)) AS rank
        FROM chunks c
        JOIN documents d ON d.id = c.document_id
        WHERE c.content_tsv @@ plainto_tsquery('english', $1)
        ORDER BY rank DESC
        LIMIT $2
        """,
        q,
        limit,
    )
    return [dict(row) for row in rows]