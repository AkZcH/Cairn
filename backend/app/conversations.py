import json
from uuid import UUID

from asyncpg import Pool


async def create_conversation(pool: Pool, title: str | None = None) -> UUID:
    row = await pool.fetchrow(
        "INSERT INTO conversations (title) VALUES ($1) RETURNING id",
        title,
    )
    return row["id"]


async def get_recent_messages(pool: Pool, conversation_id: UUID, limit: int = 10) -> list[dict]:
    """Returns the last `limit` messages, oldest first, in the shape the
    Groq API expects: {"role": ..., "content": ...}. This is the bounded
    context window, the conversation itself is stored in full, we just
    don't send all of it to the LLM on every turn."""
    rows = await pool.fetch(
        """
        SELECT role, content FROM messages
        WHERE conversation_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        """,
        conversation_id,
        limit,
    )
    return [{"role": r["role"], "content": r["content"]} for r in reversed(rows)]


async def insert_message(
    pool: Pool,
    conversation_id: UUID,
    role: str,
    content: str,
    citations: list[dict] | None = None,
) -> None:
    await pool.execute(
        """
        INSERT INTO messages (conversation_id, role, content, citations)
        VALUES ($1, $2, $3, $4)
        """,
        conversation_id,
        role,
        content,
        json.dumps(citations) if citations else None,
    )
    await pool.execute(
        "UPDATE conversations SET updated_at = now() WHERE id = $1",
        conversation_id,
    )