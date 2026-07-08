from uuid import UUID

from fastapi import APIRouter
from pydantic import BaseModel

from app.db import get_pool

router = APIRouter()


class ConversationSummary(BaseModel):
    id: UUID
    title: str | None
    updated_at: str


class Message(BaseModel):
    role: str
    content: str
    citations: list[dict] | None
    created_at: str


@router.get("/conversations", response_model=list[ConversationSummary])
async def list_conversations():
    pool = await get_pool()
    rows = await pool.fetch(
        "SELECT id, title, updated_at FROM conversations ORDER BY updated_at DESC"
    )
    return [
        ConversationSummary(id=r["id"], title=r["title"], updated_at=r["updated_at"].isoformat())
        for r in rows
    ]


@router.get("/conversations/{conversation_id}/messages", response_model=list[Message])
async def get_conversation_messages(conversation_id: UUID):
    pool = await get_pool()
    rows = await pool.fetch(
        """
        SELECT role, content, citations, created_at FROM messages
        WHERE conversation_id = $1
        ORDER BY created_at ASC
        """,
        conversation_id,
    )
    import json
    return [
        Message(
            role=r["role"],
            content=r["content"],
            citations=json.loads(r["citations"]) if r["citations"] else None,
            created_at=r["created_at"].isoformat(),
        )
        for r in rows
    ]