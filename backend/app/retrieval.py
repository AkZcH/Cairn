from uuid import UUID

from app.db import get_pool
from app.embedder import embed_query, to_pgvector_literal

import time

from app.observability import retrieval_duration_seconds

async def hybrid_search(user_id: UUID, query: str, limit: int = 10) -> list[dict]:
    start = time.perf_counter()
    pool = await get_pool()
    query_embedding = embed_query(query)
    embedding_literal = to_pgvector_literal(query_embedding)

    rows = await pool.fetch(
        """
        WITH fts AS (
            SELECT c.id AS chunk_id,
                   ROW_NUMBER() OVER (
                       ORDER BY ts_rank(c.content_tsv, plainto_tsquery('english', $2)) DESC
                   ) AS rank
            FROM chunks c
            JOIN documents d ON d.id = c.document_id
            WHERE d.user_id = $1 AND c.content_tsv @@ plainto_tsquery('english', $2)
            LIMIT 50
        ),
        vec AS (
            SELECT c.id AS chunk_id,
                   ROW_NUMBER() OVER (ORDER BY c.embedding <=> $3::vector) AS rank
            FROM chunks c
            JOIN documents d ON d.id = c.document_id
            WHERE d.user_id = $1 AND c.embedding IS NOT NULL
            ORDER BY c.embedding <=> $3::vector
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
        WHERE d.user_id = $1 AND (fts.chunk_id IS NOT NULL OR vec.chunk_id IS NOT NULL)
        ORDER BY score DESC, c.id
        LIMIT $4
        """,
        user_id,
        query,
        embedding_literal,
        limit,
    )
    retrieval_duration_seconds.observe(time.perf_counter() - start)

    return [dict(row) for row in rows]