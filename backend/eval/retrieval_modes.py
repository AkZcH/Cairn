"""
Three isolated retrieval modes, split apart from the production
hybrid_search() query, purely for comparison. Production code stays
untouched, this exists only to measure what each strategy contributes
on its own.
"""

from uuid import UUID

from app.db import get_pool
from app.embedder import embed_query, to_pgvector_literal


async def search_fulltext_only(user_id: UUID, query: str, limit: int = 5) -> list[dict]:
    pool = await get_pool()
    rows = await pool.fetch(
        """
        SELECT c.id AS chunk_id, c.document_id, d.title,
               ts_rank(c.content_tsv, plainto_tsquery('english', $2)) AS score
        FROM chunks c
        JOIN documents d ON d.id = c.document_id
        WHERE d.user_id = $1 AND c.content_tsv @@ plainto_tsquery('english', $2)
        ORDER BY score DESC, c.id
        LIMIT $3
        """,
        user_id, query, limit,
    )
    return [dict(r) for r in rows]


async def search_vector_only(user_id: UUID, query: str, limit: int = 5) -> list[dict]:
    pool = await get_pool()
    embedding_literal = to_pgvector_literal(embed_query(query))
    rows = await pool.fetch(
        """
        SELECT c.id AS chunk_id, c.document_id, d.title,
               1 - (c.embedding <=> $2::vector) AS score
        FROM chunks c
        JOIN documents d ON d.id = c.document_id
        WHERE d.user_id = $1 AND c.embedding IS NOT NULL
        ORDER BY c.embedding <=> $2::vector
        LIMIT $3
        """,
        user_id, embedding_literal, limit,
    )
    return [dict(r) for r in rows]

async def search_fulltext_websearch(user_id: UUID, query: str, limit: int = 5) -> list[dict]:
    """Same full-text index, a less brittle query parser: websearch_to_tsquery
    handles implicit OR between terms and quoted phrases, closer to how a
    real search engine behaves, instead of plainto_tsquery's strict AND
    across every word."""
    pool = await get_pool()
    rows = await pool.fetch(
        """
        SELECT c.id AS chunk_id, c.document_id, d.title,
               ts_rank(c.content_tsv, websearch_to_tsquery('english', $2)) AS score
        FROM chunks c
        JOIN documents d ON d.id = c.document_id
        WHERE d.user_id = $1 AND c.content_tsv @@ websearch_to_tsquery('english', $2)
        ORDER BY score DESC, c.id
        LIMIT $3
        """,
        user_id, query, limit,
    )
    return [dict(r) for r in rows]