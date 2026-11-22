# Retrieval evaluation harness

Compares four retrieval strategies against a labeled question set: full-text (AND), full-text (websearch), vector-only, and the production hybrid (RRF) search.

## Datasets

| File                        | Purpose                                                     | Corpus needed                                                                                    |
| --------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `dataset.json`              | Full set, source of the headline numbers in the main README | The author's personal notes, ingested under your account. Not reproducible from a clean checkout |
| `dataset_ci.json`           | Reproducible set used by CI                                 | `README.md` and `docs/PRD.md`, loaded by `seed_ci.py`                                            |
| `dataset_chunking_bug.json` | Before/after set for the list-chunking bug                  | See `docs/evidence/chunking-bug-before-after.md`                                                 |

## Running it

Full dataset (the account must already have ingested content matching the titles referenced in `dataset.json`):

```bash
docker compose exec backend python -m eval.run_eval --email you@example.com --runs 5
```

CI dataset, as CI runs it (from a full checkout, in the `backend/` directory, with Postgres reachable):

```bash
python -m eval.seed_ci
python -m eval.run_eval \
  --email ci-eval@cairn.local \
  --dataset dataset_ci.json \
  --runs 3 \
  --min-recall 0.9
```

`--min-recall` makes the run exit non-zero if hybrid recall falls below the threshold, which is what fails the CI build.

`seed_ci.py` finds the repo root relative to its own location, so it works from a full checkout but not inside the backend container, where only `app/`, `db/` and `eval/` are present. To run it in the container, copy `README.md` to `/README.md` and `docs/PRD.md` to `/docs/PRD.md` first. The seeded account (`ci-eval@cairn.local`) is inserted directly into the database. The API rejects `.local` addresses at login, so that account can't be used through the HTTP API.

## Findings

Both `plainto_tsquery` and `websearch_to_tsquery` AND all significant query terms together by default (confirmed against PostgreSQL's own documentation: `websearch_to_tsquery`'s unquoted-text behavior is documented as equivalent to `plainto_tsquery`, and OR only triggers on the literal word "or" in the query text). Natural-language questions essentially never share every word with the passage that answers them, so both full-text modes scored 0.00 recall on these datasets. This is a structural limitation of keyword matching, not a parser configuration issue. Semantic (vector) retrieval, which matches on meaning rather than token overlap, is what closes the gap.

Most recent CI-dataset run (3 runs, README and PRD corpus): hybrid and vector-only both reached recall@5 1.00 and MRR 1.00, and both full-text modes scored 0.00.

## Honest caveats

- `dataset.json` is a starter set. Verify and expand it against your real corpus before treating results as conclusive.
- `dataset_ci.json` runs over two documents, so a perfect score is easy to reach. Treat it as a regression gate, not a quality claim.
- Ground truth is labeled at the document level, not at exact chunk level.
- The README is part of the CI corpus, so editing it changes what the CI eval searches. Re-run the eval after README changes.
- Results are written to `eval/results/`, timestamped and kept as history. Inside the container that folder is written to the host through a bind mount, so new files appear in your working tree as untracked.
