"""
Retrieval evaluation harness.

Usage:
    python -m eval.run_eval --email you@example.com [--k 5]

Requires the account to have real ingested content matching eval/dataset.json.
Run inside the backend container, where app.db / app.embedder are importable:
    docker compose exec backend python -m eval.run_eval --email you@example.com
"""

import argparse
import asyncio
import json
from datetime import datetime, timezone
from pathlib import Path

from app.db import get_pool
from app.retrieval import hybrid_search

from eval.retrieval_modes import search_fulltext_only, search_vector_only

from eval.retrieval_modes import search_fulltext_only, search_fulltext_websearch, search_vector_only

from statistics import mean, stdev

DATASET_PATH = Path(__file__).parent / "dataset.json"
RESULTS_DIR = Path(__file__).parent / "results"


def recall_at_k(results: list[dict], relevant_titles: list[str]) -> float:
    retrieved = {r["title"] for r in results}
    return 1.0 if retrieved & set(relevant_titles) else 0.0


def reciprocal_rank(results: list[dict], relevant_titles: list[str]) -> float:
    for i, r in enumerate(results):
        if r["title"] in relevant_titles:
            return 1.0 / (i + 1)
    return 0.0


async def run_mode(name, search_fn, user_id, dataset, k):
    per_question = []
    for item in dataset:
        results = await search_fn(user_id, item["question"], k)
        per_question.append({
            "question": item["question"],
            "relevant_titles": item["relevant_titles"],
            "retrieved_titles": [r["title"] for r in results],
            "recall": recall_at_k(results, item["relevant_titles"]),
            "reciprocal_rank": reciprocal_rank(results, item["relevant_titles"]),
        })

    mean_recall = sum(q["recall"] for q in per_question) / len(per_question)
    mean_mrr = sum(q["reciprocal_rank"] for q in per_question) / len(per_question)
    return {"mode": name, "recall_at_k": mean_recall, "mrr": mean_mrr, "questions": per_question}


async def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--email", required=True)
    parser.add_argument("--k", type=int, default=5)
    parser.add_argument("--runs", type=int, default=1, help="Repeat each mode N times, report mean ± std")
    args = parser.parse_args()

    pool = await get_pool()
    row = await pool.fetchrow("SELECT id FROM users WHERE email = $1", args.email)
    if row is None:
        raise SystemExit(f"No user found with email {args.email}")
    user_id = row["id"]

    dataset = json.loads(DATASET_PATH.read_text())

    async def hybrid_wrapper(user_id, question, k):
        return await hybrid_search(user_id, question, k)

    modes = [
        ("full-text (AND)", search_fulltext_only),
        ("full-text (websearch)", search_fulltext_websearch),
        ("vector-only", search_vector_only),
        ("hybrid", hybrid_wrapper),
    ]

    print(f"\n{'Mode':<24} {'Recall@' + str(args.k):<16} {'MRR':<16}")
    print("-" * 56)

    all_results = []
    for name, fn in modes:
        recalls, mrrs = [], []
        last_result = None
        for _ in range(args.runs):
            last_result = await run_mode(name, fn, user_id, dataset, args.k)
            recalls.append(last_result["recall_at_k"])
            mrrs.append(last_result["mrr"])

        mean_recall, mean_mrr = mean(recalls), mean(mrrs)
        std_recall = stdev(recalls) if len(recalls) > 1 else 0.0
        std_mrr = stdev(mrrs) if len(mrrs) > 1 else 0.0

        recall_str = f"{mean_recall:.2f} ± {std_recall:.2f}" if args.runs > 1 else f"{mean_recall:.2f}"
        mrr_str = f"{mean_mrr:.2f} ± {std_mrr:.2f}" if args.runs > 1 else f"{mean_mrr:.2f}"
        print(f"{name:<24} {recall_str:<16} {mrr_str:<16}")

        all_results.append({
            "mode": name, "runs": args.runs,
            "recall_mean": mean_recall, "recall_std": std_recall,
            "mrr_mean": mean_mrr, "mrr_std": std_mrr,
            "last_run_detail": last_result,
        })

    RESULTS_DIR.mkdir(exist_ok=True)
    timestamp = datetime.now(timezone.utc).strftime("%Y%m%d-%H%M%S")
    out_path = RESULTS_DIR / f"eval-{timestamp}.json"
    out_path.write_text(json.dumps(all_results, indent=2))
    print(f"\nFull results written to {out_path}")


if __name__ == "__main__":
    asyncio.run(main())