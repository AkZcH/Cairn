import hashlib
from uuid import UUID, uuid4

from fastapi import APIRouter, Depends, File, HTTPException, UploadFile
from pydantic import BaseModel

from app.auth import get_current_user
from app.chunking import chunk_sections, parse_markdown
from app.db import get_pool
from app.embedder import embed_text, to_pgvector_literal

router = APIRouter()


class UploadResponse(BaseModel):
    document_id: UUID
    status: str
    chunks: int


def _content_hash(raw: str) -> str:
    return hashlib.sha256(raw.encode("utf-8")).hexdigest()


async def _ingest_text(user_id: UUID, source_path: str, title: str, raw: str) -> UploadResponse:
    pool = await get_pool()
    content_hash = _content_hash(raw)

    existing = await pool.fetchrow(
        "SELECT id, content_hash FROM documents WHERE user_id = $1 AND source_path = $2",
        user_id, source_path,
    )

    if existing and existing["content_hash"] == content_hash:
        return UploadResponse(document_id=existing["id"], status="unchanged", chunks=0)

    sections = parse_markdown(raw)
    chunks = chunk_sections(sections)

    if existing:
        document_id = existing["id"]
        await pool.execute(
            "UPDATE documents SET content_hash = $1, title = $2, updated_at = now() WHERE id = $3",
            content_hash, title, document_id,
        )
        await pool.execute("DELETE FROM chunks WHERE document_id = $1", document_id)
        status = "updated"
    else:
        row = await pool.fetchrow(
            """INSERT INTO documents (user_id, source, title, content_hash, source_path)
               VALUES ($1, 'web', $2, $3, $4) RETURNING id""",
            user_id, title, content_hash, source_path,
        )
        document_id = row["id"]
        status = "inserted"

    for i, chunk in enumerate(chunks):
        content = f"{chunk['heading']}\n\n{chunk['content']}" if chunk["heading"] else chunk["content"]
        embedding_literal = to_pgvector_literal(embed_text(content))
        await pool.execute(
            """INSERT INTO chunks (document_id, chunk_index, content, embedding)
               VALUES ($1, $2, $3, $4::vector)
               ON CONFLICT (document_id, chunk_index) DO NOTHING""",
            document_id, i, content, embedding_literal,
        )

    return UploadResponse(document_id=document_id, status=status, chunks=len(chunks))


class TextUploadRequest(BaseModel):
    title: str
    content: str


@router.post("/upload/text", response_model=UploadResponse)
async def upload_text(request: TextUploadRequest, user_id: UUID = Depends(get_current_user)):
    if not request.content.strip():
        raise HTTPException(status_code=400, detail="Content is empty")
    source_path = f"web:paste:{uuid4()}"
    return await _ingest_text(user_id, source_path, request.title or "Untitled paste", request.content)


@router.post("/upload/file", response_model=UploadResponse)
async def upload_file(file: UploadFile = File(...), user_id: UUID = Depends(get_current_user)):
    if not (file.filename or "").lower().endswith((".md", ".markdown", ".txt")):
        raise HTTPException(status_code=400, detail="Only .md, .markdown, or .txt files are supported")

    raw_bytes = await file.read()
    try:
        raw = raw_bytes.decode("utf-8")
    except UnicodeDecodeError:
        raise HTTPException(status_code=400, detail="File must be UTF-8 text")

    source_path = f"web:file:{file.filename}"
    return await _ingest_text(user_id, source_path, file.filename, raw)