from typing import Optional
from uuid import UUID

from fastapi import APIRouter
from pydantic import BaseModel

from app.conversations import create_conversation, get_recent_messages, insert_message
from app.db import get_pool
from app.llm import call_groq
from app.retrieval import hybrid_search

router = APIRouter()


class ChatRequest(BaseModel):
    question: str
    conversation_id: Optional[UUID] = None
    limit: int = 5


class Citation(BaseModel):
    chunk_id: UUID
    document_id: UUID
    title: Optional[str]
    content: str


class ChatResponse(BaseModel):
    conversation_id: UUID
    answer: str
    citations: list[Citation]


SYSTEM_PROMPT = """You are Cairn, a personal knowledge base assistant. Answer the
user's question using ONLY the numbered context passages provided with it.
Every claim you make must be traceable to one of these passages, referenced
inline as [1], [2], etc. You may also refer back to earlier turns in this
conversation for continuity.

If the passages don't contain enough information to answer, say so plainly
instead of guessing or using outside knowledge."""


@router.post("/chat", response_model=ChatResponse)
async def chat(request: ChatRequest):
    pool = await get_pool()

    conversation_id = request.conversation_id
    if conversation_id is None:
        conversation_id = await create_conversation(pool)

    # Retrieval always uses just the current question, not the full history,
    # searching on an entire conversation's accumulated text would dilute
    # the query and hurt relevance.
    chunks = await hybrid_search(request.question, request.limit)

    history = await get_recent_messages(pool, conversation_id, limit=10)

    if not chunks:
        answer = "I don't have any notes covering that yet."
        citations: list[Citation] = []
    else:
        context_block = "\n\n".join(
            f"[{i + 1}] ({chunk['title'] or 'untitled'}): {chunk['content']}"
            for i, chunk in enumerate(chunks)
        )
        user_turn = f"Context passages:\n\n{context_block}\n\nQuestion: {request.question}"

        messages = [{"role": "system", "content": SYSTEM_PROMPT}]
        messages.extend(history)
        messages.append({"role": "user", "content": user_turn})

        answer = await call_groq(messages)
        citations = [
            Citation(
                chunk_id=chunk["chunk_id"],
                document_id=chunk["document_id"],
                title=chunk["title"],
                content=chunk["content"],
            )
            for chunk in chunks
        ]

    # Store the raw question (not the context-stuffed version) so history
    # stays readable and re-sendable on future turns.
    await insert_message(pool, conversation_id, "user", request.question)
    await insert_message(
        pool, conversation_id, "assistant", answer,
        citations=[c.model_dump(mode="json") for c in citations],
    )

    return ChatResponse(conversation_id=conversation_id, answer=answer, citations=citations)