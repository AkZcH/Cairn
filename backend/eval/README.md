# Retrieval evaluation harness

Compares three retrieval strategies against a labeled question set:
full-text-only, vector-only, and the production hybrid (RRF) search.

## Running it

```bash
docker compose exec backend python -m eval.run_eval --email you@example.com
```

Requires the account to already have real ingested content matching the
titles referenced in `dataset.json`.

## Findings

Both `plainto_tsquery` and `websearch_to_tsquery` AND all significant query
terms together by default (confirmed against PostgreSQL's own
documentation, `websearch_to_tsquery`'s unquoted-text behavior is
explicitly documented as equivalent to `plainto_tsquery`, OR only triggers
on the literal word "or" in the query text). Natural-language questions
essentially never share every word with the passage that answers them, so
both full-text modes scored 0.00 recall on this dataset, a structural
limitation of keyword matching, not a parser configuration issue.
Semantic (vector) retrieval, which matches on meaning rather than token
overlap, is what actually closes this gap.

## Honest caveats

- `dataset.json` is a starter set, verify and expand against your real
  corpus before treating results as conclusive.
- Ground truth is labeled at the document level, not exact chunk-level.
- Results are written to `eval/results/`, timestamped, kept as history.
