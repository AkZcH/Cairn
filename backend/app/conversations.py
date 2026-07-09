import json
from uuid import UUID

from asyncpg import Pool
from fastapi import HTTPException


async def create_conversation(pool: Pool, user_id: UUID, title: str | None = None) -> UUID:
    row = await pool.fetchrow(
        "INSERT INTO conversations (user_id, title) VALUES ($1, $2) RETURNING id",
        user_id,
        title,
    )
    return row["id"]


async def verify_conversation_owner(pool: Pool, user_id: UUID, conversation_id: UUID) -> None:
    """Prevents a user from reading or appending to someone else's
    conversation by guessing/reusing a UUID, this is the actual security
    check, not just a convenience lookup."""
    row = await pool.fetchrow(
        "SELECT id FROM conversations WHERE id = $1 AND user_id = $2",
        conversation_id,
        user_id,
    )
    if row is None:
        raise HTTPException(status_code=404, detail="Conversation not found")


async def get_recent_messages(pool: Pool, conversation_id: UUID, limit: int = 10) -> list[dict]:
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
        "INSERT INTO messages (conversation_id, role, content, citations) VALUES ($1, $2, $3, $4)",
        conversation_id,
        role,
        content,
        json.dumps(citations) if citations else None,
    )
    await pool.execute("UPDATE conversations SET updated_at = now() WHERE id = $1", conversation_id)