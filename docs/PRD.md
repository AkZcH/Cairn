# Cairn — Product Requirements (Current)

**Version 2.0 — reflects the shipped system, not the original plan.** The original PRD (`docs/archive/PRD-original-2026-07.md`) proposed a heavier stack, LlamaIndex, LangGraph, Neo4j, Streamlit, Chroma. Most of that was deliberately cut in favor of a leaner, more defensible build, documented honestly below rather than left stale.

## Vision

A local-first, ever-evolving personal knowledge base. Ingest notes, retrieve with hybrid search, get answers grounded in citations, not guesses, with a working proof, not a claim, that the retrieval actually works.

## What's actually built

**Ingestion**

- Rust CLI (`ingest/`), structure-aware markdown/plaintext chunking (splits on headings, packs paragraphs, handles markdown list items correctly)
- Local embeddings via `candle` (BAAI/bge-small-en-v1.5), CPU-only, no GPU, no cloud ML calls
- Whole-folder batch ingestion with per-file error isolation
- Path-based document identity with content-hash change detection, editing a file replaces its old chunks, never forks the knowledge base
- Uploads to the backend over an authenticated HTTP API (API key), not a direct database connection

**Web ingestion** (`backend/app/upload.py`)

- Paste-text and drag-and-drop file upload through the dashboard
- Server-side chunking and embedding via `fastembed` (same model, same vector space as the CLI path)
- Same edit-detection guarantee as the CLI path

**Retrieval**

- Postgres full-text search + pgvector cosine similarity (HNSW index), fused with Reciprocal Rank Fusion
- Measured, not assumed: a reproducible evaluation harness (`backend/eval/`) compares four retrieval modes on a labeled question set, hybrid achieves 1.00 recall@5 / 0.90 MRR against 0.00 for full-text alone, full methodology and findings in `backend/eval/README.md`

**Chat**

- Citation-grounded, multi-turn, via Groq (`openai/gpt-oss-20b`)
- Conversations persist indefinitely in Postgres; bounded context window sent per LLM call, not the full history every time
- Refuses to answer when retrieval finds nothing relevant, rather than guessing

**Multi-tenancy & auth**

- Real accounts: signup/login, JWT for the web app, API keys for the CLI
- Every retrieval query scoped by `user_id`, including inside the ranking CTEs, not just the outer filter
- Isolation verified with two real, separate accounts, not assumed

**Frontend**

- Next.js landing page and authenticated dashboard
- Real signup/login pages, chat UI with clickable citations, conversation history sidebar, upload modal

**Operations**

- CI (GitHub Actions): Rust build/test/fmt, backend + retrieval eval against a reproducible dataset, fails the build on a measured recall regression
- Observability: Prometheus + Grafana, retrieval/embedding/LLM latency histograms, real token usage and cost tracking (from Groq's own `usage` field, not estimated)

## Deliberately not built (and why)

- **Neo4j / knowledge graph** — the original plan's most ambitious piece, cut early. A self-updating knowledge graph that reconciles concepts across sources is a genuine, unsolved research problem, not a weekend feature. Revisit only if there's a real, specific need for it, not because it sounds impressive.
- **LangGraph / agentic pipelines** — added complexity with no concrete use case yet. Hybrid retrieval plus grounded chat covers the actual current need.
- **Kubernetes, Terraform, Ansible** — considered explicitly, rejected. Single-service, single-database workload; these tools solve problems of scale and fleet management that don't exist here yet. Revisit if that changes.
- **Hosted deployment** — deliberately last. No point standing up public infrastructure for a product that wasn't finished, tested, or measured yet.

## Tech stack

| Layer                 | Choice                                      |
| --------------------- | ------------------------------------------- |
| Ingestion & embedding | Rust, `candle`, `pulldown-cmark`, `reqwest` |
| Storage               | PostgreSQL 16 + pgvector                    |
| API & auth            | Python, FastAPI, `asyncpg`, JWT + API keys  |
| LLM                   | Groq (`openai/gpt-oss-20b`)                 |
| Frontend              | Next.js, Tailwind, TypeScript               |
| Observability         | Prometheus, Grafana                         |
| CI                    | GitHub Actions                              |

## Known limitations

- No hosted deployment yet, local Docker Compose only
- Web-upload chunking (Python) is simpler than the Rust CLI's chunker, not yet feature-parity
- No password reset, email verification, or LLM-call rate limiting
- Single-instance Postgres, no backups or read replicas
