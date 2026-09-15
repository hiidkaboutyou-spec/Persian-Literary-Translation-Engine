# External Translation Integrations

This document records approved external integrations for the Persian Literary Translation Engine, the boundary each dependency is allowed to cross, and researched candidates that are intentionally deferred.

## Principles

- Rust remains the product core and owns project state, literary intelligence, translation orchestration, deterministic quality gates, review state, and publishing.
- External projects are integrated only where they provide a concrete capability that is stronger than the native implementation.
- External model scores and probabilistic diagnostics are evidence, never human approval and never permission to silently rewrite a manuscript.
- Exact upstream revisions or package versions are pinned where practical. Upgrades require tests and provenance/security review.
- Heavy models and optional tooling are never downloaded by default CI unless a dedicated reproducible test explicitly requires them.
- Manuscript text, translations, reviewer notes, credentials, and project memory must not be logged or sent anywhere except to a provider explicitly selected for that operation.

## BookForge — active EPUB ingestion dependency

Upstream: `JunjoSick/bookforge`

Approved revision:

```text
23f8c9d3c97a06f48e13424698441bfb4b037844
```

License: MIT.

`document-engine` enables the `bookforge-epub` feature by default and maps BookForge IR back into native document types. BookForge types must not leak into project persistence, translation memory, literary intelligence, review ledgers, or public application contracts.

The previous built-in EPUB reader remains an explicit compatibility build path, not a silent fallback. When BookForge is enabled and rejects an EPUB, ingestion returns an actionable error.

## COMET / XCOMET / DocCOMET — optional quality evidence sidecar

Upstream: `Unbabel/COMET`

Pinned package:

```text
unbabel-comet==2.2.7
```

License: Apache-2.0 for the package; individual checkpoints may have separate terms.

COMET remains behind the isolated `quality-engine::comet` process boundary. The installer does not download a model checkpoint. COMET output is advisory evidence only and cannot mark text human-approved.

## Lingua — optional English/Persian language diagnostics

Upstream: `pemistahl/lingua-rs`
Pinned crate: `1.8.0`
License: Apache-2.0.

Only English/Persian model features are enabled and the feature is off by default. Lingua cannot rewrite, approve, or independently reject multilingual literary output.

## EPUBCheck — optional publication conformance validator

Upstream: `w3c/epubcheck`

```text
EPUBCheck 5.3.0
SHA-256 6c07e68584b2e2ce2f89fe06e1246dfead3eb36b46b340e7d93524f29dcff6c5
```

License: BSD-3-Clause.

EPUBCheck is not vendored and absence of Java/EPUBCheck cannot break translation/DOCX/runtime. Phase 20 may promote it to a generated-EPUB publication-conformance gate.

## FastEmbed + BGE-M3 — optional semantic evidence boundary

FastEmbed crate selected by the project:

```text
fastembed = 6.1.0
```

BGE upstream: `FlagOpen/FlagEmbedding`.

BGE-M3 supports multilingual/cross-lingual embeddings and long inputs. The project uses it only through optional Rust tooling for Phase 18 retrieval and Phase 19 alignment evidence.

Rules:

- model weights are not vendored;
- normal compilation/default CI must not download weights;
- deterministic native retrieval/review remains available without BGE;
- BGE output cannot create canon IDs, own project memory, approve text, or silently mutate output;
- model-backed process boundaries must be timeout/failure-safe;
- Linux and Apple Silicon arm64 compatibility are tested.

## Phase 19 Native Literary Alignment — active branch implementation

The Phase 19 branch implements monotonic alignment natively in Rust and exposes optional embedding-backed execution through `tools/literary-alignment`.

Supported alignment shapes include 1:1, 1:N, N:1, N:M and source/target gaps. The Rust core validates returned unit identity, index bounds, complete ordered coverage, finite values, and schema before evidence is accepted.

A missing/failing/malformed aligner does not block translation and does not become a clean review result. Review artifacts record whether evidence was requested, unavailable, completed, or failed.

This integration becomes canonical only after the Phase 19 PR is merged and post-merge validation succeeds.

## Hazm — blocked by unpatched dependency advisory

Upstream: `roshan-research/hazm`
Current evaluated version: `0.12.1`
License: MIT.
Python requirement: `>=3.12,<3.14`.
Mandatory dependency includes `nltk ^3.9.0`.

Potential value: Persian normalization/tokenization/POS/syntax diagnostics.

Decision: **blocked**, not merely deferred. A real isolated evaluation showed the mandatory compatible NLTK dependency is affected by GitHub-reviewed High-severity `GHSA-8mgp-746c-j5xp` / `CVE-2026-81726` through NLTK 3.10.3, with no patched version listed at the Phase 19 research date.

Do not add an audit waiver merely to enable Hazm. Re-evaluate only after a patched compatible NLTK release exists and a fresh dependency/security audit passes.

## Vecalign — algorithm/design reference only

Upstream: `thompsonb/vecalign`
Core license: Apache-2.0.

Vecalign is a strong reference for multilingual monotonic sentence alignment, including one-to-many/many-to-one behavior and document-scale alignment.

Reasons not to install it now:

- Cython/C extension and compiler/Python surface would be added to the project;
- embeddings still need to be supplied separately;
- the native Rust aligner now covers the measured Phase 19 need while reusing the existing BGE boundary;
- bundled Bleualign dev/test datasets have separate GPL licensing and must never be copied blindly.

## SentWeave 0.3.3 — audited research reference only

Upstream: `amajdalawi/sentweave`
Release commit: `61e9af7086a4339448ab4b2c25b8a2071961d123`
License: Apache-2.0.
PyPI sdist SHA-256:

```text
ef6414bdd1d7fa4064f31fdf1b446c7f2601955777fcdf64988fc09bca9d2940
```

SentWeave exposes in-memory VecAlign-style monotonic alignment and leaves the encoder to the caller. A one-off research workflow validated hash-pinned installation, dependency audit, algorithm smoke, Linux, and Apple Silicon.

Decision: reference only. It is a new/small Python package and adding another runtime surface is unnecessary while the native Rust aligner satisfies the same requirement with less operational risk.

## DadmaTools — conditional research candidate

License: Apache-2.0.

Potential value: Persian NER/POS/dependency/ezafe diagnostics.

Decision: defer unless a Phase 19 benchmark demonstrates a specific capability gap after native review. Do not add it just to accumulate NLP features.

## ContextWeaver — architecture reference, no dependency

ContextWeaver concepts were reviewed for long-form context packets, stable segment identity, revision history, and resumability. Native Context Packet v2 now owns those responsibilities.

## TranslateBooksWithLLMs — design reference only

No source code is copied or linked. Selective glossary/context ideas overlap native memory architecture and its licensing does not justify importing code.

## TransAgents — research/agent-role reference only

TransAgents can inform separation of translator/editor/fidelity/voice/naturalness roles, but its orchestration and memory architecture are not runtime dependencies. This repository keeps provider judgments separate from deterministic quality checks and human review.

## SacreBLEU / chrF++ — Phase 21 candidate

License: Apache-2.0.

Potential value: reproducible reference-based BLEU/chrF/TER benchmarking.

Decision: defer until a rights-safe EN→FA literary reference corpus exists. Reference metrics remain benchmark evidence, never the literary judge.

## Supporting-tool candidates outside translation runtime

### OpenDataLoader PDF — ingestion benchmark candidate

Upstream: `opendataloader-project/opendataloader-pdf`.

Potential value: structured Markdown/JSON/HTML extraction, reading-order/layout recovery, bounding boxes, tables, and OCR/hybrid handling for difficult PDFs.

Decision: do not replace `document-engine`. After Phase 19, benchmark deterministic local mode against the current PDF ingestion path using project-owned fixtures. If adopted, keep Java/Python/hybrid AI tooling optional behind a narrow ingestion sidecar and map results back to native document types.

### ripwire — developer-only code-intelligence candidate

Upstream: `redhat-et/ripwire`.

Potential value: deterministic tree-sitter symbol/call graph, ranked bounded code maps, and MCP for coding agents.

Decision: may be evaluated as isolated developer tooling after the numbered phase is complete. It must not be imported into production Rust, own project memory, or become required for build/translation/review/export.

### Headroom — conditional developer/research candidate

Upstream: `headroomlabs-ai/headroom`.
License: Apache-2.0.

Potential value: compressing coding/research-agent tool outputs and context.

Decision: do not place Headroom between literary context/evidence and translation/review providers without a dedicated fidelity benchmark. It may be evaluated only for developer/research workflows first.

## Other architecture references

- `sukamenev/booktrans` — whole-book scouting/selective context/editor-verifier patterns.
- Tolmach / `KazKozDev/book-translator` — useful refinement/verifier concepts; reference-only under licensing boundary.
- `madpin/epublate` — useful EPUB round-trip/glossary lifecycle ideas; do not copy code without independent licensing/provenance review.
- ArmenianLitTranslator — useful critic-role/evaluation dimensions; research reference only.

## Upgrade / adoption policy

Before adding/upgrading any external repository/package/model/tool:

1. prove a concrete capability gap;
2. read current license, release/security notes, provenance, maintenance state, and model/data licenses;
3. compare public API, runtime cost, privacy, persistence, and failure implications;
4. prefer a reasonably small native Rust implementation when it is safer and easier to own;
5. keep heavyweight models/downloads optional unless a numbered phase explicitly promotes them;
6. benchmark on rights-safe/project-owned fixtures;
7. run rustfmt, Clippy, affected/full tests, release/CLI smoke where relevant, and vulnerability audits;
8. validate optional tools on Linux/macOS when they are expected to be developer/product compatible there;
9. never convert external evidence into canon or human approval;
10. record the selected/deferred decision in roadmap/status/PMC/this document before merge.
