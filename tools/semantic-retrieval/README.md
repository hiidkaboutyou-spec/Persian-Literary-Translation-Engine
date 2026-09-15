# Semantic Retrieval Tool

Optional Phase 18 semantic-ranking companion for the Persian Literary Translation Engine.

## Boundary

This tool is **not** part of the default translation runtime. The Rust application remains fully functional with deterministic retrieval only. This process may reorder only candidate IDs supplied by the core; it cannot create canon, edit memory, approve review items, or write manuscripts.

The protocol is defined by `memory-engine::semantic` and uses JSON on stdin/stdout. The core validates that every returned ID was present in the request before accepting the response.

## Models

The optional `bge` feature pins `fastembed = 6.1.0` and exposes three modes:

- `dense` — quantized BGE-M3 dense semantic similarity (`gpahal/bge-m3-onnx-int8`).
- `rerank` — multilingual BGE reranker (`rozgo/bge-reranker-v2-m3`).
- `dense_then_rerank` — BGE-M3 semantic recall followed by reranking of a bounded shortlist. This is the default protocol mode.

Models are downloaded only on first **actual model execution**, never by the normal workspace build/test path. The default cache is `.tools/models/fastembed`, which is already covered by the repository `.tools/` ignore rule. Override it with `PERSIAN_TRANSLATOR_MODEL_CACHE`.

Model weights are not committed to Git and are not required by default CI.

## Build

Protocol-only build, no ML dependency:

```bash
cargo test --manifest-path tools/semantic-retrieval/Cargo.toml --no-default-features
```

Opt-in semantic build:

```bash
cargo build --release --manifest-path tools/semantic-retrieval/Cargo.toml --features bge
```

Building the `bge` feature resolves/downloads the ONNX Runtime build dependency used by FastEmbed, but it does not download BGE model weights. Model weights are fetched only when the binary is executed in a semantic mode and the requested model is not already cached.

## Safety rules

- Maximum 512 candidates per request.
- Candidate IDs must be unique and non-empty.
- Returned IDs must be a subset of request IDs.
- Semantic scores are advisory. Native deterministic retrieval remains available and is fused by rank rather than by raw model-score calibration.
- Do not send full proprietary manuscripts when a bounded candidate set is sufficient.
- Do not put API credentials, reviewer-private material, or manuscript text in logs.
- A sidecar failure must be recoverable by falling back to deterministic retrieval at the orchestration layer.

## Why not GraphRAG/HippoRAG/RAPTOR as dependencies?

Those projects contain valuable ideas, but importing their full orchestration would duplicate or compete with this repository's native glossary, Character Bible, relationship canon, review ledger, stable IDs, and persistence ownership. Phase 18 therefore adopts the narrow capability that is actually missing: optional multilingual semantic ranking plus native provenance-aware context packets.
