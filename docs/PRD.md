## Cairn - Personal Ever-Evolving Knowledge Rift (Second Brain RAG)

## PRD

**Version**: 1.0  
**Date**: July 2026  
**Overview**: Cairn is a local-first, personal AI-powered knowledge base built as a Retrieval-Augmented Generation (RAG) system. It ingests and preserves notes, learning roadmaps, LLM responses (Grok, Claude, ChatGPT), articles, and more. It evolves continuously through incremental updates, hybrid retrieval (vector + knowledge graph), and agentic capabilities. The system emphasizes AI infrastructure best practices: observability, scalability, modularity, and production-grade reliability—perfect for your portfolio and career goals.

**Vision**: Become your ultimate learning companion that never forgets, connects ideas across time, and actively helps synthesize new insights from your growing knowledge base.

**Target User**: You (solo power user) — extensible to other AI-infra enthusiasts.

---

### Functional Requirements

#### Phase 1: Core Ingestion & Basic RAG (MVP)

- Support multiple input formats: Markdown, PDF, TXT, JSON (exported chat histories), plain text.
- Drag-and-drop / folder watch / API endpoint for ingestion.
- Automatic parsing, semantic chunking, embedding, and storage.
- Basic chat interface for querying the KB with source citations.
- Metadata tagging (source, date, topic, LLM-origin).

#### Phase 2: Evolution & Hybrid Intelligence

- Incremental updates & versioning: Detect changes, re-embed only deltas, maintain history.
- Knowledge Graph integration: Auto-extract entities/relations → enable multi-hop reasoning (e.g., roadmap connections).
- Agentic features:
  - Auto-summarize & link new content.
  - "Roadmap Updater" agent: Analyze new notes vs. existing learning paths.
  - Feedback loop: Thumbs up/down refines retrieval.
- Temporal & comparative queries (e.g., "How has my RAG understanding evolved?").

#### Phase 3: Advanced & Polish

- Multi-modal support (images via embeddings/OCR).
- Export/share knowledge packs or generated roadmaps.
- Evaluation dashboard: RAG metrics (faithfulness, relevance) on personal test sets.
- User-defined collections (e.g., "AI Infra", "Roadmaps").
- Proactive suggestions: "Based on your notes, explore X next."

---

### Non-Functional Requirements

- **Performance**: Sub-2s response for most queries (local). Scalable to thousands of documents.
- **Privacy & Security**: Fully local-first. Optional encrypted storage. No data leaves your machine unless explicitly enabled.
- **Reliability**: Robust error handling, retries for ingestion, graceful degradation (vector-only if graph down).
- **Observability**: Logging, metrics (latency, token usage, KB growth, retrieval quality), dashboard.
- **Usability**: Intuitive UI, searchable history, dark mode, mobile-friendly.
- **Maintainability**: Modular, well-documented, containerized, easy to extend (polyglot-friendly).
- **Scalability**: Handle growing KB (GBs of notes) efficiently; support future horizontal scaling.
- **Cost**: Minimal (local models/embeddings). API fallback with cost tracking.
- **Accessibility**: Keyboard shortcuts, export to Markdown/PDF.

---

### Tech Stack

**Core**:

- **Python** (primary) — LlamaIndex (indexing/retrieval) + LangGraph (agents/workflows).
- **Embeddings**: BGE or similar local models (Hugging Face).
- **Vector Store**: PostgreSQL + pgvector (primary, for infra depth) + Chroma (light fallback).
- **Knowledge Graph**: Neo4j (Docker).
- **LLM**: Ollama (Llama 3.1 8B/70B or equivalent) + API providers (Groq, Anthropic, xAI) via LiteLLM abstraction.

**Infrastructure & Deployment**:

- **Containerization**: Docker + Docker Compose (multi-service: app, DBs, Ollama, Neo4j).
- **Backend**: FastAPI.
- **Frontend**: Streamlit (quick iterations) → evolve to Next.js/React + Tailwind.
- **Monitoring**: Prometheus + Grafana.
- **Orchestration**: LangGraph + optional Airflow/Celery for background jobs.
- **Storage**: Local volumes + optional S3-like for backups.

**Dev Tools**:

- Git, Poetry (Python deps), Ruff/Black (linting), pytest.
- Optional future: Rust services for perf-critical parts (embeddings/indexing).

**Why this stack?** Balances rapid development with deep infra learning (DB ops, observability, orchestration, polyglot potential). Fully open-source and self-hostable.

---

### Approximate Build Time (Solo, Part-Time/Full Focus)

**Total Estimated**: 8–14 weeks (realistic, high-quality, iterative). No rush—focus on excellence.

- **Phase 1 (MVP)**: 2–3 weeks (core ingestion + basic chat).
- **Phase 2 (Evolution + Hybrid)**: 3–4 weeks (graph, agents, updates).
- **Phase 3 (Polish + Infra)**: 2–4 weeks (UI/UX, monitoring, evaluation, optimizations).
- **Buffer & Iteration**: 1–3 weeks (testing, refinements, documentation).

This assumes deep dives into components (e.g., custom chunking, hybrid retriever tuning). We can accelerate or slow down based on your schedule.

**Success Metrics**:

- Personal daily usage with high satisfaction.
- Portfolio-worthy: Clean repo, blog posts, metrics showing improvements over naive RAG.
- Extensible & production-grade patterns.

---

This PRD is actionable. We’ll build **feature by feature**, starting with setup and Phase 1.

**Next**: Shall I create the initial GitHub repo structure (folders, Docker Compose, basic README), or jump straight into the ingestion pipeline code? Or refine anything in the PRD first?

Let’s make Cairn legendary. What’s our first move?

