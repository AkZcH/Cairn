# Cairn

A local-first, ever-evolving personal knowledge base (RAG). Ingests notes,
LLM conversations, and articles; retrieves with citations; grows over time.

## Stack

- **Ingestion/chunking**: Rust (`ingest/`)
- **Retrieval + orchestration**: FastAPI (`backend/`)
- **Storage**: Postgres + pgvector
- **UI**: Streamlit (Phase 1)

## Running locally

1. `cp infra/.env.example infra/.env` and fill in real values
2. `cd infra && docker compose up --build`
3. Backend health check: `curl http://localhost:8000/health`

## Status

Phase 1 (MVP) — skeleton stage.
