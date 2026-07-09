# Cairn

A local-first, ever-evolving personal knowledge base. Point it at your notes, ask it questions, get answers grounded in what you actually wrote, with citations, not hallucinations.

## What it does

- **Ingest a whole notes folder** — recursive walk, markdown/plaintext,
  skips vault noise (`.obsidian`, `.git`, etc.)
- **Structure-aware chunking** — splits on headings, not fixed character
  windows, so retrieved context stays coherent
- **Local embeddings, no GPU** — BAAI/bge-small-en-v1.5, run via `candle`
  in Rust, CPU-only
- **Edits update, not duplicate** — re-ingesting a changed file replaces
  its old chunks rather than forking your knowledge base
- **Hybrid search** — Postgres full-text search + pgvector cosine
  similarity, fused with Reciprocal Rank Fusion, catches both "postgres"
  and "PostgreSQL" as the same concept
- **Citation-grounded chat** — answers are built only from retrieved
  passages, with inline citations back to source notes
- **Multi-turn memory** — conversations persist in Postgres indefinitely,
  with a bounded context window sent per LLM call

## Architecture

┌─────────────┐ ┌──────────────┐ ┌─────────────────┐
│ ingest/ │────▶│ db/ │◀────│ backend/ │
│ Rust binary │ │ Postgres + │ │ FastAPI │
│ parse+chunk │ │ pgvector │ │ hybrid search + │
│ +embed │ │ │ │ chat (Groq LLM) │
└─────────────┘ └──────────────┘ └──────────────────┘

- **`ingest/`** — Rust CLI. Parses markdown/plaintext, chunks by section,
  embeds locally via `candle`, writes to Postgres. Run manually or point
  at a whole folder.
- **`db/`** — SQL migrations for the `documents`, `chunks`, `conversations`,
  and `messages` tables, plus a small dependency-free migration runner.
- **`backend/`** — FastAPI service. `/search` (hybrid retrieval),
  `/chat` (citation-grounded, multi-turn), `/conversations` (history).
- **`infra/`** — Docker Compose wiring Postgres + backend together.

## Stack

| Layer                 | Choice                                                   |
| --------------------- | -------------------------------------------------------- |
| Ingestion & embedding | Rust, `candle`, `pulldown-cmark`, `sqlx`                 |
| Storage               | PostgreSQL 16 + pgvector (HNSW index, cosine similarity) |
| Retrieval & API       | Python, FastAPI, `asyncpg`                               |
| LLM                   | Groq (`openai/gpt-oss-20b`)                              |
| Query-time embeddings | `fastembed` (ONNX, same model as ingest)                 |

## Quickstart

```bash
cp infra/.env.example infra/.env
# fill in a real GROQ_API_KEY (free tier at console.groq.com)

cd infra
docker compose up --build -d

docker compose exec backend python db/run_migrations.py
```

Download the embedding model files (see `ingest/models/README.md`), then:

```bash
cd ingest
cargo run -- /path/to/your/notes/folder
```

Query it:

```bash
curl "http://localhost:8000/search?q=your+topic"

curl -X POST http://localhost:8000/chat \
  -H "Content-Type: application/json" \
  -d '{"question": "what do my notes say about X?"}'
```

## Status

Core pipeline complete: ingest, chunk, embed, hybrid search,
citation-grounded multi-turn chat, folder-batch ingestion, edit detection.

Not yet built: a frontend beyond raw API calls, and installable
distribution (currently requires Docker + Rust toolchain locally).
