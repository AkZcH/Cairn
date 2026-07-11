# Cairn

![Cairn](docs/landing.png)

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
- **Multi-tenant** — real accounts, JWT auth, API keys for the ingest CLI,
  data isolation verified with two separate real accounts, not assumed

## Architecture

┌─────────────┐ ┌──────────────┐ ┌─────────────────┐ ┌──────────────┐
│ ingest/ │────▶│ backend/ │◀───▶│ db/ │ │ frontend/ │
│ Rust binary │ │ FastAPI │ │ Postgres + │ │ Next.js │
│ parse+chunk │ │ auth, search,│ │ pgvector │ │ landing page │
│ +embed, │ │ chat (Groq) │ │ │ │ + dashboard │
│ HTTP upload │ │ │ │ │ │ │
└─────────────┘ └──────────────┘ └─────────────────┘ └──────────────┘

- **`ingest/`** — Rust CLI. Parses markdown/plaintext, chunks by section,
  embeds locally via `candle`, uploads to the backend over an authenticated
  API (not a direct DB connection, no shared credentials required).
- **`backend/`** — FastAPI service. `/auth` (signup/login/API keys),
  `/search` (hybrid retrieval), `/chat` (citation-grounded, multi-turn),
  `/conversations` (history), `/documents/upload` (ingest intake).
- **`db/`** — SQL migrations for the full schema: users, documents, chunks,
  conversations, messages, plus a small dependency-free migration runner.
- **`frontend/`** — Next.js landing page and authenticated dashboard
  (chat UI with citations, conversation history).
- **`infra/`** — Docker Compose wiring Postgres + backend together.

## Stack

| Layer                 | Choice                                                   |
| --------------------- | -------------------------------------------------------- |
| Ingestion & embedding | Rust, `candle`, `pulldown-cmark`, `reqwest`              |
| Storage               | PostgreSQL 16 + pgvector (HNSW index, cosine similarity) |
| API & auth            | Python, FastAPI, `asyncpg`, JWT + API keys               |
| LLM                   | Groq (`openai/gpt-oss-20b`)                              |
| Query-time embeddings | `fastembed` (ONNX, same model as ingest)                 |
| Frontend              | Next.js, Tailwind, TypeScript                            |

## Quickstart

```bash
cp infra/.env.example infra/.env
# fill in a real GROQ_API_KEY (free tier at console.groq.com) and JWT_SECRET

cd infra
docker compose up --build -d
docker compose exec backend python db/run_migrations.py
```

Sign up for an account:

```bash
curl -X POST http://localhost:8000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{"email": "you@example.com", "password": "a-real-password"}'
```

Save the `api_key` from the response into `ingest/.env` (see `ingest/.env.example`).

Download the embedding model files (see `ingest/models/README.md`), then ingest a folder:

```bash
cd ingest
cargo run -- /path/to/your/notes/folder
```

Run the frontend:

```bash
cd frontend
npm install
npm run dev
```

## Status

Core pipeline complete: multi-tenant auth, ingest, chunk, embed, hybrid
search, citation-grounded multi-turn chat, folder-batch ingestion, edit
detection, a landing page, and an authenticated dashboard.

Not yet built: real login/signup pages in the frontend (currently a
temporary token-paste bridge), CI/CD, and hosted deployment.
