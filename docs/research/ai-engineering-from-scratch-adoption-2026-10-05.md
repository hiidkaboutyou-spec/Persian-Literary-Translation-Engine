# ai-engineering-from-scratch adoption review — 2026-10-05

## Scope

Reviewed `rohitg00/ai-engineering-from-scratch` against the current Rust-first literary engine. The upstream repository is MIT-licensed. This change adapts standard information-retrieval evaluation concepts; no upstream implementation is vendored.

## Adopted now: retrieval evaluation before retrieval expansion

The upstream Advanced RAG sequence places retrieval evaluation after hybrid retrieval/reranking/query rewriting and measures ranking with Precision@k, Recall@k, MRR and nDCG.

This project already has the more important production foundations:
- deterministic literary-memory retrieval as the safety floor;
- optional semantic BGE/rerank sidecar;
- Reciprocal Rank Fusion without mixing uncalibrated raw scores;
- bounded candidate pools;
- exact fallback to deterministic retrieval when semantic tooling fails;
- dedicated literary translation evaluation and human-review boundaries.

The missing piece was a native, reusable way to prove that retrieval changes actually improve ranking.

Added to `memory-engine`:
- `precision_at_k`;
- `recall_at_k`;
- `reciprocal_rank`;
- `dcg_at_k` / `ndcg_at_k`;
- `evaluate_ranking` and a typed `RetrievalMetrics` result;
- unit tests that reward earlier/high-grade relevant results and fail closed to zero on empty evidence;
- an integration regression that evaluates the current deterministic literary-memory ranking.

These metrics are offline evaluation evidence only. They never alter retrieval order, canon, project memory, translation output, or human-review authority.

## Already present; do not duplicate

### Hybrid retrieval and RRF
The engine already implements bounded deterministic + semantic retrieval and RRF with explicit semantic failure fallback. Copying the upstream toy BM25/dense implementation would be a regression.

### Reranking
The optional semantic sidecar already supports rerank/dense-then-rerank modes. A second cross-encoder path is not justified without a measured gap.

### EPUB/PDF build ideas
The project already has native document ingestion, Book IR, BookForge-backed structured EPUB round-trip, EPUBCheck, RTL publishing, DOCX export and dedicated round-trip CI. The upstream Pandoc book builder is useful as a reference but is not an architectural upgrade here.

### General translation eval
The existing literary-evaluation engine is more domain-specific than generic answer faithfulness metrics: it covers semantic fidelity, Persian naturalness, voice, relationship register, subtext, terminology and long-context continuity with human scorecards.

## Deferred deliberately

### HyDE / multi-query / decomposition
These strategies can improve recall, but generated hypothetical text can also distort character facts, negation, relationship state or terminology. Do not add them to production until a project-owned qrels benchmark shows a real recall/nDCG gain and the literary safety floor remains unchanged.

### Full OpenTelemetry GenAI instrumentation
Current OpenTelemetry GenAI span conventions are still marked development. The project already has strict privacy rules forbidding manuscript/translation content in telemetry. Revisit only behind a content-free Rust adapter when an operational need justifies it.

### Generic agent/tool harness
This engine deliberately keeps translation providers replaceable and human review explicit. A generic model-controlled tool registry or planner would widen the authority surface without solving a current product gap.

## Next admission gate

Any future semantic model, reranker, HyDE or multi-query proposal should report at least:
1. Recall@k against project-owned relevance judgments;
2. MRR for first-useful-memory placement;
3. nDCG@k when relevance is graded;
4. deterministic-fallback behavior when the optional component is absent/fails;
5. latency/resource impact separately from literary-quality outcomes.

A candidate should not be promoted merely because it produces plausible examples.
