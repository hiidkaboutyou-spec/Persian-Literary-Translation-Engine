# Phase 19 Research — Literary Fidelity & Persian Naturalness Review

Status: implementation branch research record. The code described here becomes canonical only after the Phase 19 pull request is merged and post-merge CI is verified on `main`.

## Goal

Phase 19 adds a review stack for mistakes that generic machine-translation metrics miss: omission/addition, semantic drift, character voice, relationship/register, Persian naturalness, dialogue/subtext, terminology, and long-form continuity.

The review stack is evidence-producing and non-mutating. It cannot approve text, mutate canon, or silently rewrite the translation. Human review remains the final authority.

## Selected Architecture

### Native Rust review domain

`engine/crates/literary-review-engine` owns typed review dimensions, findings, severity, provenance, revision proposals, provider contracts, alignment contracts, and validation.

The application layer persists one literary-review artifact per translated chapter. Every artifact records source, translation, and translation-context fingerprints so manual edits or changed context make prior evidence explicitly stale.

### Independent review dimensions

- omission/addition
- semantic fidelity / narrative-intent preservation
- character voice
- relationship/register consistency
- Persian naturalness
- dialogue rhythm / emotional subtext
- terminology / continuity

A dimension that was not actually evaluated stays unevaluated. Tool or provider failure must never be converted into a silent pass.

### Provider critic

The critic API is provider-neutral. The optional OpenAI adapter uses a bounded, paragraph-indexed request and structured response validation. Findings must cite valid source/target paragraph indices. Invented indices and findings outside the requested dimensions are rejected.

Provider findings are advisory evidence. Suggested rewrites remain revision proposals and are never auto-applied.

### Native semantic alignment

Phase 19 uses a native bounded monotonic dynamic-programming aligner rather than importing an external Python alignment runtime.

Supported alignment shapes include 1:1, 1:N, N:1, N:M, source-only gaps, and target-only gaps. Source-only gaps are omission evidence candidates; target-only gaps are addition evidence candidates. Low semantic similarity alone is not treated as proof of a translation error.

The optional `tools/literary-alignment` process boundary reuses the BGE-M3 model boundary already approved for Phase 18. The Rust domain validates returned unit IDs, indices, coverage, monotonic order, finite similarity values, and schema structure before attaching evidence.

The alignment tool is optional. Missing, failed, malformed, or timed-out alignment must not block translation or mutate canon.

## External Research

### BGE-M3 — selected model boundary reuse

Upstream: `FlagOpen/FlagEmbedding`

BGE-M3 is multilingual, supports more than 100 languages, and supports inputs up to 8192 tokens. Phase 19 reuses the already-approved Rust/FastEmbed integration rather than adding a second multilingual embedding stack.

Rules:

- model download is explicit and never triggered by normal compilation/default CI;
- deterministic native review works without BGE;
- BGE produces alignment/retrieval evidence only;
- BGE never owns canon or approval.

### Vecalign — reference only

Upstream: `thompsonb/vecalign`
License: Apache-2.0 for Vecalign core.

Vecalign is a strong design reference for multilingual monotonic alignment and 1-many/many-1 matching. It requires a Cython/C extension and external embeddings. Its repository also includes Bleualign development/test data under GPL, so datasets must never be copied blindly.

Decision: do not install Vecalign while the native Rust aligner plus the existing BGE boundary satisfies the measured requirement.

### SentWeave 0.3.3 — audited reference, not dependency

Upstream release provenance: `amajdalawi/sentweave` tag `v0.3.3`, release commit `61e9af7086a4339448ab4b2c25b8a2071961d123`.
License: Apache-2.0.
PyPI sdist SHA-256: `ef6414bdd1d7fa4064f31fdf1b446c7f2601955777fcdf64988fc09bca9d2940`.

SentWeave provides in-memory VecAlign-style monotonic alignment and leaves the embedding encoder to the caller. A one-off audit successfully installed the hash-pinned package, ran its algorithm without an external model, audited the dependency graph, and exercised Linux and Apple Silicon.

Decision: reference only. It is a new/small Python package with one maintainer, and adding its Python/Cython surface is unnecessary when the native Rust aligner can reuse the model boundary already present in the project.

### Hazm 0.12.1 — blocked by dependency security

Upstream: `roshan-research/hazm`
License: MIT.
Python requirement: `>=3.12,<3.14`.
Mandatory dependency: `nltk ^3.9.0`.

A real isolated install/smoke was evaluated, but the dependency audit exposed GitHub-reviewed advisory `GHSA-8mgp-746c-j5xp` / `CVE-2026-81726` in NLTK through 3.10.3. As of the Phase 19 research date, the advisory lists no patched version.

Decision: do not add Hazm and do not waive the advisory. Reconsider only after a patched compatible NLTK release exists and a fresh install/security audit passes.

### DadmaTools — deferred

DadmaTools remains conditional. It may be evaluated only if a benchmark demonstrates a concrete Persian NER/POS/dependency capability gap after the native review stack. Do not add it merely because it provides more NLP features.

### TransAgents / Armenian literary-agent projects — architecture references only

Useful idea: separate critics for different literary dimensions and keep critic evidence distinct from translation execution.

Decision: do not import their agent orchestration/memory architecture. This repository already owns translation execution, project memory, character canon, review lifecycle, and persistence.

## Application Boundary

`ApplicationService` exposes literary review as a post-translation operation. CLI/product surfaces call the same application API; they do not read or write review JSON directly.

Default behavior is credential-free:

- provider critic: `none`
- semantic alignment: optional; absence is recorded as not configured/not requested rather than success
- native structural review: always available

Review artifacts are stored separately from deterministic quality-gate state and human-intelligence review/canon state.

## Validation Requirements

Before Phase 19 can merge:

- engine lockfile freshness
- rustfmt
- Clippy with `-D warnings`
- `literary-review-engine` unit tests
- application-level artifact/staleness regression tests
- CLI/application review tests without credentials
- full normal Rust CI and security workflow
- alignment protocol tests without a model
- optional BGE adapter compile on Linux and Apple Silicon arm64
- no model weights downloaded during compile/default CI
- RustSec audit for engine and alignment-tool lockfiles
- no temporary write-enabled validation workflow left in the final diff

## Post-Phase-19 Supporting-Tool Shortlist

These are not Phase 19 runtime dependencies.

### OpenDataLoader PDF — benchmark candidate

Potential use: optional PDF-ingestion sidecar for complex layout, reading order, tables, bounding boxes, and scanned/OCR-heavy PDFs. It must be benchmarked against the existing `document-engine` PDF path using project-owned fixtures before adoption. Deterministic local mode is preferred; hybrid/AI mode must never become a hidden requirement.

### ripwire — developer-only candidate

Potential use: local codebase map/call-graph/MCP support for coding agents. It must remain developer tooling and must never be imported by production Rust or become required for build/translation/review/export.

### Headroom — conditional developer/research candidate

Potential use: compress tool outputs/context for coding or research agents. Never place it between literary runtime context and the translation/review provider without a separate fidelity benchmark, because lossy context compression could alter literary evidence.

## Durable Decision Rule

Before adding any future GitHub dependency/tool:

1. prove a concrete capability gap;
2. inspect current license, release provenance, maintenance, dependencies, model/data licenses, privacy, and failure modes;
3. prefer the native Rust capability when it is reasonably small and safer;
4. isolate heavy/model-backed tools behind explicit optional boundaries;
5. preserve deterministic fallback;
6. benchmark on project-owned fixtures;
7. require Linux/macOS compatibility where relevant;
8. run security audits;
9. never let automated evidence become canon or human approval.
