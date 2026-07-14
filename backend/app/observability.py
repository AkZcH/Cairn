"""
Custom Prometheus metrics for the RAG pipeline specifically, separate from
generic HTTP metrics (which prometheus-fastapi-instrumentator already
provides automatically). These track the things an AI engineer actually
cares about: how long retrieval and generation take, and what they cost.
"""

from prometheus_client import Counter, Histogram

# Buckets tuned for this workload: retrieval should be tens of milliseconds,
# LLM calls are typically hundreds of ms to a few seconds on Groq's hardware.
retrieval_duration_seconds = Histogram(
    "cairn_retrieval_duration_seconds",
    "Time spent in hybrid_search (embedding query + both SQL retrieval paths)",
    buckets=(0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0),
)

embedding_duration_seconds = Histogram(
    "cairn_embedding_duration_seconds",
    "Time spent computing a single embedding via fastembed",
    buckets=(0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5),
)

llm_duration_seconds = Histogram(
    "cairn_llm_duration_seconds",
    "Time spent waiting on the Groq chat completion call",
    buckets=(0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0),
)

llm_tokens_total = Counter(
    "cairn_llm_tokens_total",
    "Total tokens sent to/received from the LLM",
    labelnames=["direction"],  # "prompt" or "completion"
)

# USD per 1M tokens for openai/gpt-oss-20b on Groq, per Groq's own official
# pricing announcement (groq.com/blog/day-zero-support-for-openai-open-models).
# Third-party trackers show slightly different figures ($0.075/$0.30 vs
# Groq's own $0.10/$0.50) — using Groq's primary source, but pricing changes,
# verify against console.groq.com/settings/billing before trusting this for
# real budgeting, this is directional, not a billing-grade calculation.
GROQ_INPUT_PRICE_PER_1M = 0.10
GROQ_OUTPUT_PRICE_PER_1M = 0.50

llm_cost_usd_total = Counter(
    "cairn_llm_cost_usd_total",
    "Estimated USD cost of LLM calls, based on a hardcoded price constant, verify before trusting for real billing",
)


def record_llm_usage(prompt_tokens: int, completion_tokens: int) -> None:
    llm_tokens_total.labels(direction="prompt").inc(prompt_tokens)
    llm_tokens_total.labels(direction="completion").inc(completion_tokens)
    cost = (prompt_tokens / 1_000_000 * GROQ_INPUT_PRICE_PER_1M) + (
        completion_tokens / 1_000_000 * GROQ_OUTPUT_PRICE_PER_1M
    )
    llm_cost_usd_total.inc(cost)