from typing import Optional
from uuid import UUID

from fastapi import APIRouter
from pydantic import BaseModel

from app.llm import call_groq
from app.retrieval import hybrid_search

router = APIRouter()


class ChatRequest(BaseModel):
    question: str
    limit: int = 5


class Citation(BaseModel):
    chunk_id: UUID
    document_id: UUID
    title: Optional[str]
    content: str


class ChatResponse(BaseModel):
    answer: str
    citations: list[Citation]


SYSTEM_PROMPT = """You are Cairn, a personal knowledge base assistant. Answer the
user's question using ONLY the numbered context passages below. Every claim
you make must be traceable to one of these passages, referenced inline as
[1], [2], etc.

If the passages don't contain enough information to answer, say so plainly
instead of guessing or using outside knowledge."""


@router.post("/chat", response_model=ChatResponse)
async def chat(request: ChatRequest):
    chunks = await hybrid_search(request.question, request.limit)

    if not chunks:
        return ChatResponse(
            answer="I don't have any notes covering that yet.",
            citations=[],
        )

    context_block = "\n\n".join(
        f"[{i + 1}] ({chunk['title'] or 'untitled'}): {chunk['content']}"
        for i, chunk in enumerate(chunks)
    )

    user_prompt = f"Context passages:\n\n{context_block}\n\nQuestion: {request.question}"
    answer = await call_groq(SYSTEM_PROMPT, user_prompt)

    citations = [
        Citation(
            chunk_id=chunk["chunk_id"],
            document_id=chunk["document_id"],
            title=chunk["title"],
            content=chunk["content"],
        )
        for chunk in chunks
    ]

    return ChatResponse(answer=answer, citations=citations)