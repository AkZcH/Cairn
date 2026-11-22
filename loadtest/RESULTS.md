# /search load test results

Hardware: Windows laptop, 12 logical CPUs, Docker Desktop, CPU only, no GPU.
Setup: single uvicorn process, query embedding via fastembed (bge-small, ONNX, threads=1) in a thread pool, 25 chunks in Postgres 16 + pgvector.
Tool: k6 v2.4.0 on the same machine, 20s per point, one run per point.
Script: loadtest/search.js (6 rotating natural-language queries).

Users median p95 req/s failures
1 28ms 40ms 33 0
10 116ms 166ms 81 0
25 291ms 409ms 81 0
50 455ms 1.45s 85 0
100 1.01s 3.58s 81 0
