# Literary Alignment Tool

Optional Phase 19 adapter for source/translation omission-addition evidence.

## Why this exists

The Rust `literary-review-engine` owns the deterministic monotonic block-alignment algorithm and validates every returned path. This tool does only the expensive multilingual embedding step with the same BGE-M3 / `fastembed 6.1.0` family already validated for Phase 18 semantic retrieval.

The design borrows the useful idea from Vecalign-style systems—monotonic alignment over multilingual embeddings—without adopting a second orchestration framework, Python/Cython runtime, bundled datasets, or a second embedding model.

## Safety boundary

- optional executable; not a normal translation-runtime dependency;
- no model weights are downloaded by ordinary engine build, normal CI, or `install.sh`;
- model weights are fetched only on explicit invocation of the BGE-enabled tool;
- source/target segment counts, total text size, block size, costs, indices, similarities, model provenance, and complete monotonic path coverage are validated by Rust;
- malformed output, timeout, unavailable executable/model, or inference failure yields no semantic-alignment evidence rather than changing translation/canon;
- source-only and target-only gaps are **possible omission/addition signals**, not proof;
- matched cosine similarity is not automatically converted into a literary-quality verdict;
- no result can mark a chapter human-approved or apply a revision.

## Build

```bash
tools/literary-alignment/install.sh
```

The installed executable is normally `.tools/bin/literary-alignment`.

## Protocol

Input is bounded JSON with `schema_version`, `unit_id`, source/target segment arrays, and alignment configuration. Output is the versioned `AlignmentResult` contract from `literary-review-engine`.

The optional model adapter creates embeddings for all contiguous spans up to the configured block size, then delegates alignment to the Rust core. Default block size is 3 and either side is bounded to 512 segments.

## Research decision

Evaluated alternatives include upstream Vecalign, SentWeave, BERTAlign-style alignment, and separate SONAR/LaBSE embedding stacks. SentWeave 0.3.3 passed isolated Linux and Apple Silicon installation/smoke/security checks, but remains a reference candidate: adopting it would introduce a new Python/Cython boundary while the project already has a validated Rust BGE stack. The native path therefore remains preferred unless a rights-safe benchmark shows a material quality gap.
