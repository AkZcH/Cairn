# /search Load Test Results

**Hardware:** Windows laptop, 12 logical CPUs, Docker Desktop, CPU only, no GPU.  
**Setup:** single uvicorn process, query embedding via fastembed (bge-small, ONNX, threads=1) in a thread pool, 25 chunks in Postgres 16 + pgvector.  
**Tool:** k6 v2.4.0 on the same machine, 20s per point, one run per point.  
**Script:** `loadtest/search.js` (6 rotating natural-language queries).

## Results

| Users | Median |   p95 | req/s | Failures |
| ----: | -----: | ----: | ----: | -------: |
|     1 |   28ms |  40ms |    33 |        0 |
|    10 |  116ms | 166ms |    81 |        0 |
|    25 |  291ms | 409ms |    81 |        0 |
|    50 |  455ms | 1.45s |    85 |        0 |
|   100 |  1.01s | 3.58s |    81 |        0 |

## Search latency during a large upload

Question: does uploading a large document stall search for other users?

Setup: 10 concurrent search users (k6, 20s) while one 400-chunk document (223 KB, 400 paragraphs) was uploaded via `POST /upload/text`. Baseline corpus: 25 chunks. Hardware: Docker Desktop, 12 CPUs, CPU only, ONNX threads=1. One run per row.

|                 | No upload (baseline) | Upload, embedding on event loop | Upload, embedding in worker thread |
| --------------- | -------------------- | ------------------------------- | ---------------------------------- |
| Median          | 116ms                | 606ms                           | 142ms                              |
| p95             | 166ms                | 1.5s                            | 227ms                              |
| Max             | n/a                  | 2.19s                           | 740ms                              |
| Throughput      | ~81 req/s            | 13 req/s                        | 64 req/s                           |
| Failures        | 0                    | 0                               | 0                                  |
| Upload duration | n/a                  | 108s                            | 107s                               |

Cause: `_ingest_text` called `embed_text` synchronously inside an async function, once per chunk, so each embedding held the event loop. Fix: `await asyncio.to_thread(embed_text, content)` (commit d1b3b0b).

What the fix does and does not do:

- Search during an upload recovered from 5x slower to about 1.2x baseline median. The remaining gap is likely CPU contention with the embedding thread (not isolated).
- The upload itself is not faster (about 270ms per chunk, still one chunk at a time). Batching the embedding calls would address that and is not done.

Limits: single run per column. k6 covered only the first 20s of the ~107s upload. 25-chunk search corpus. Numbers apply to this hardware only.
