import json
from uuid import UUID

from fastapi import APIRouter, Depends
from pydantic import BaseModel

from app.auth import get_current_user_from_api_key
from app.db import get_pool

router = APIRouter()


class ChunkIn(BaseModel):
    index: int
    content: str
    embedding: list[float]


class UploadRequest(BaseModel):
    source: str
    title: str | None
    source_path: str
    content_hash: str
    chunks: list[ChunkIn]


class UploadResponse(BaseModel):
    document_id: UUID
    status: str  # "inserted" | "updated" | "unchanged"


@router.post("/documents/upload", response_model=UploadResponse)
async def upload_document(request: UploadRequest, user_id: UUID = Depends(get_current_user_from_api_key)):
    pool = await get_pool()

    existing = await pool.fetchrow(
        "SELECT id, content_hash FROM documents WHERE user_id = $1 AND source_path = $2",
        user_id, request.source_path,
    )

    if existing and existing["content_hash"] == request.content_hash:
        return UploadResponse(document_id=existing["id"], status="unchanged")

    if existing:
        document_id = existing["id"]
        await pool.execute(
            "UPDATE documents SET content_hash = $1, title = $2, updated_at = now() WHERE id = $3",
            request.content_hash, request.title, document_id,
        )
        await pool.execute("DELETE FROM chunks WHERE document_id = $1", document_id)
        status = "updated"
    else:
        row = await pool.fetchrow(
            """INSERT INTO documents (user_id, source, title, content_hash, source_path)
               VALUES ($1, $2, $3, $4, $5) RETURNING id""",
            user_id, request.source, request.title, request.content_hash, request.source_path,
        )
        document_id = row["id"]
        status = "inserted"

    for chunk in request.chunks:
        embedding_literal = "[" + ",".join(f"{x:.8f}" for x in chunk.embedding) + "]"
        await pool.execute(
            """INSERT INTO chunks (document_id, chunk_index, content, embedding)
               VALUES ($1, $2, $3, $4::vector)
               ON CONFLICT (document_id, chunk_index) DO NOTHING""",
            document_id, chunk.index, chunk.content, embedding_literal,
        )

    return UploadResponse(document_id=document_id, status=status)