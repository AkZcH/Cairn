import asyncio
import logging
import re
from typing import Optional
from uuid import UUID

import httpx
from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel

from app.auth import get_current_user
from app.conversations import (
    create_conversation,
    get_recent_messages,
    insert_message,
    verify_conversation_owner,
)
from app.db import get_pool
from app.llm import call_groq
from app.retrieval import hybrid_search

router = APIRouter()
logger = logging.getLogger(__name__)


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

MAX_RETRY_WAIT_SECONDS = 10


async def call_groq_with_retry(messages: list[dict], max_attempts: int = 3) -> str:
    """Call Groq, retrying 429s and mapping upstream failures to clean HTTP errors."""
    for attempt in range(max_attempts):
        try:
            return await call_groq(messages)
        except httpx.HTTPStatusError as e:
            status = e.response.status_code
            if status != 429:
                logger.error("Groq returned HTTP %d", status)
                raise HTTPException(
                    status_code=502,
                    detail="The language model service returned an error.",
                ) from e

            try:
                wait = float(e.response.headers.get("retry-after", 2**attempt))
            except ValueError:
                wait = float(2**attempt)

            if attempt < max_attempts - 1 and wait <= MAX_RETRY_WAIT_SECONDS:
                logger.warning(
                    "Groq 429, retry %d of %d in %.1fs", attempt + 1, max_attempts - 1, wait
                )
                await asyncio.sleep(wait)
                continue

            logger.warning("Groq 429, giving up (retry-after %.1fs)", wait)
            raise HTTPException(
                status_code=429,
                detail="Rate limited by the language model provider. Try again shortly.",
                headers={"Retry-After": str(max(int(wait), 1))},
            ) from e
        except httpx.RequestError as e:
            logger.error("Could not reach Groq: %s", e)
            raise HTTPException(
                status_code=503,
                detail="Could not reach the language model service.",
            ) from e

    # Unreachable, but keeps the return type honest.
    raise HTTPException(status_code=503, detail="Language model unavailable.")


@router.post("/chat", response_model=ChatResponse)
async def chat(request: ChatRequest, user_id: UUID = Depends(get_current_user)):
    pool = await get_pool()

    if request.conversation_id is None:
        conversation_id = await create_conversation(pool, user_id)
    else:
        await verify_conversation_owner(pool, user_id, request.conversation_id)
        conversation_id = request.conversation_id

    chunks = await hybrid_search(user_id, request.question, request.limit)
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

        answer = await call_groq_with_retry(messages)

        # Show only the passages the model actually referenced as [n].
        # A refusal has no markers, so it comes back with no citations.
        used = {int(n) for n in re.findall(r"\[(\d+)\]", answer)}
        citations = [
            Citation(
                chunk_id=c["chunk_id"],
                document_id=c["document_id"],
                title=c["title"],
                content=c["content"],
            )
            for i, c in enumerate(chunks, start=1)
            if i in used
        ]

    await insert_message(pool, conversation_id, "user", request.question)
    await insert_message(
        pool,
        conversation_id,
        "assistant",
        answer,
        citations=[c.model_dump(mode="json") for c in citations],
    )

    return ChatResponse(
        conversation_id=conversation_id, answer=answer, citations=citations
    )