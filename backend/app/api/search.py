from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Query
from pydantic import BaseModel

from app.db import get_pool
from app.embedder import embed_query, to_pgvector_literal

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
    pool = await get_pool()

    query_embedding = embed_query(q)
    embedding_literal = to_pgvector_literal(query_embedding)

    # RRF combining full-text rank and vector-similarity rank.
    # Each CTE independently ranks its top 50 candidates; chunks appearing
    # in either (or both) lists get scored and merged below.
    rows = await pool.fetch(
        """
        WITH fts AS (
            SELECT c.id AS chunk_id,
                   ROW_NUMBER() OVER (
                       ORDER BY ts_rank(c.content_tsv, plainto_tsquery('english', $1)) DESC
                   ) AS rank
            FROM chunks c
            WHERE c.content_tsv @@ plainto_tsquery('english', $1)
            LIMIT 50
        ),
        vec AS (
            SELECT c.id AS chunk_id,
                   ROW_NUMBER() OVER (ORDER BY c.embedding <=> $2::vector) AS rank
            FROM chunks c
            WHERE c.embedding IS NOT NULL
            ORDER BY c.embedding <=> $2::vector
            LIMIT 50
        )
        SELECT c.id AS chunk_id, c.document_id, c.chunk_index, c.content,
               d.title, d.source,
               COALESCE(1.0 / (60 + fts.rank), 0)
             + COALESCE(1.0 / (60 + vec.rank), 0) AS score
        FROM chunks c
        JOIN documents d ON d.id = c.document_id
        LEFT JOIN fts ON fts.chunk_id = c.id
        LEFT JOIN vec ON vec.chunk_id = c.id
        WHERE fts.chunk_id IS NOT NULL OR vec.chunk_id IS NOT NULL
        ORDER BY score DESC
        LIMIT $3
        """,
        q,
        embedding_literal,
        limit,
    )
    return [dict(row) for row in rows]