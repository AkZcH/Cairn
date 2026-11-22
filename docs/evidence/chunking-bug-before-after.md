# Chunking bug: before/after evidence

Real chunk content pulled directly from two isolated benchmark accounts,
same source file (`docs/archive/PRD-original-2026-07.md`), ingested with
the parser code from before and after commit `02de3dc`.

## Before (buggy parser, commit `dbf6c32`)

Markdown list items produce no paragraph boundary in `pulldown-cmark`
(a CommonMark "tight list" quirk), so every bullet point in the PRD's
Tech Stack section collapsed into one undifferentiated blob:

Core :
Python (primary) — LlamaIndex (indexing/retrieval) + LangGraph (agents/workflows). Embeddings : BGE or similar local models (Hugging Face). Vector Store : PostgreSQL + pgvector (primary, for infra depth) + Chroma (light fallback). Knowledge Graph : Neo4j (Docker). LLM : Ollama (Llama 3.1 8B/70B or equivalent) + API providers (Groq, Anthropic, xAI) via LiteLLM abstraction. Infrastructure & Deployment :
Containerization : Docker + Docker Compose (multi-service: app, DBs, Ollama, Neo4j). Backend : FastAPI. Frontend : Streamlit (quick iterations) → evolve to Next.js/React + Tailwind. Monitoring : Prometheus + Grafana. Orchestration : LangGraph + optional Airflow/Celery for background jobs. Storage : Local volumes + optional S3-like for backups. Dev Tools :


The fact "Vector Store: PostgreSQL + pgvector" is buried mid-sentence
between two unrelated facts, its embedding represents an average of six
different concepts, not the one a query about databases actually needs.

## After (fixed parser, commit `02de3dc`)

Tech Stack

Core :
Python (primary) — LlamaIndex (indexing/retrieval) + LangGraph (agents/workflows).
Embeddings : BGE or similar local models (Hugging Face).
Vector Store : PostgreSQL + pgvector (primary, for infra depth) + Chroma (light fallback).
Knowledge Graph : Neo4j (Docker).
...


Each fact is now separable, giving "Vector Store" its own focused
embedding instead of one diluted across the whole list.

## Root cause

`pulldown-cmark` doesn't emit `Paragraph` start/end events for items in a
CommonMark "tight list" (the common case: a bullet list with no blank
lines between items). The parser only inserted line breaks on `Paragraph`
boundaries, so tight lists collapsed entirely. Fix: also insert a line
break on `Item` end events. One line, `ingest/src/parser.rs`.