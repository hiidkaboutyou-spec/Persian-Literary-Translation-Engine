# External Translation Integrations

This document records approved external integrations for the Persian Literary Translation Engine, the boundary each dependency is allowed to cross, and researched candidates that are intentionally deferred.

## Principles

- Rust remains the product core and owns project state, literary intelligence, translation orchestration, deterministic quality gates, review state, and publishing.
- External projects are integrated only where they provide a concrete capability stronger or safer than duplicating the capability natively.
- External model scores and probabilistic diagnostics are evidence, never human approval and never permission to silently rewrite a manuscript.
- Exact upstream revisions or package versions are pinned where practical. Upgrades require tests and provenance/security review.
- Heavy models and optional tooling are not downloaded by normal CI unless a dedicated reproducible validation explicitly requires them.
- Manuscript text, translations, reviewer notes, credentials, and project memory are not logged or sent anywhere except to a provider explicitly selected for that operation.

## BookForge — active EPUB ingestion and Phase 20 reconstruction dependency

Upstream: `JunjoSick/bookforge`

Approved revision:

```text
23f8c9d3c97a06f48e13424698441bfb4b037844
```

License: MIT.

`document-engine` enables the `bookforge-epub` feature by default and maps BookForge IR back into native document types. BookForge types must not leak into project persistence, translation memory, literary intelligence, review ledgers, or public application contracts.

The previous built-in EPUB reader remains an explicit compatibility build path, not a silent fallback. When BookForge is enabled and rejects an EPUB, ingestion returns an actionable error.

Phase 20 also uses BookForge for source-aware reconstruction. The product persists only native stable source provenance (`block_id`) and supplies an explicit block-ID-to-translation map at export. BookForge owns inline-marker-aware XHTML reconstruction, target primary `dc:language`, and XHTML `lang`/`xml:lang` rewriting. The native document layer adds only the RTL publication metadata not supplied by that boundary (`dir="rtl"` and OPF spine `page-progression-direction="rtl"`).

Publication export must fail closed when expected source block provenance is missing, unknown, duplicated, empty, or mismatched. Do not regenerate EPUB from flattened translated chapter text and do not guess block alignment by paragraph order.

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

## EPUBCheck — Phase 20 publication conformance validator

Upstream: `w3c/epubcheck`

Authoritative Phase 20 package:

```text
EPUBCheck 5.3.0
SHA-256 6c07e68584b2e2ce2f89fe06e1246dfead3eb36b46b340e7d93524f29dcff6c5
```

License: BSD-3-Clause.

Phase 20 targets the W3C EPUB 3.3 Recommendation. EPUBCheck 5.3.0 explicitly validates EPUB 3.3 and is therefore the publication-conformance gate for this phase. The distribution is not vendored; the checksum-pinned installer places it only under ignored local tool storage. Java/EPUBCheck must not become a requirement for ingestion, translation, literary review, or DOCX export.

EPUBCheck 5.4.0 is newer, but its EPUB 3 validation tracks EPUB 3.4. EPUB 3.4 is still a Candidate Recommendation at the Phase 20 research date (2026-09-17). Treat 5.4.x as future-compatibility evidence; do not silently migrate the product standard until EPUB 3.4 is stable or a deliberate migration is approved and tested.

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

## Phase 19 Native Literary Alignment — canonical

Phase 19 implements monotonic alignment natively in Rust and exposes optional embedding-backed execution through `tools/literary-alignment`.

Supported alignment shapes include 1:1, 1:N, N:1, N:M and source/target gaps. The Rust core validates returned unit identity, index bounds, complete ordered coverage, finite values, and schema before evidence is accepted.

A missing/failing/malformed aligner does not block translation and does not become a clean review result. Review artifacts record whether evidence was requested, unavailable, completed, or failed.

Phase 19 is canonical via PR #95, merge `d073dab10c0965197745a6cbc7b8e56c946835e8`.

## Hazm — blocked by unpatched dependency advisory

Upstream: `roshan-research/hazm`
Current evaluated version: `0.12.1`
License: MIT.
Python requirement: `>=3.12,<3.14`.
Mandatory dependency includes `nltk ^3.9.0`.

Potential value: Persian normalization/tokenization/POS/syntax diagnostics.

Decision: **blocked**, not merely deferred. The mandatory compatible NLTK dependency was affected by the recorded High-severity `GHSA-8mgp-746c-j5xp` / `CVE-2026-81726` at the Phase 19 audit point. Do not add an audit waiver merely to enable Hazm. Re-evaluate only after a patched compatible NLTK release exists and a fresh dependency/security audit passes.

## Vecalign — algorithm/design reference only

Upstream: `thompsonb/vecalign`
Core license: Apache-2.0.

Vecalign is a strong reference for multilingual monotonic sentence alignment, including one-to-many/many-to-one behavior and document-scale alignment. It is not installed because its Python/Cython/compiler surface is unnecessary while the native Rust aligner satisfies the measured requirement; bundled Bleualign dev/test datasets also have separate GPL licensing and must not be copied blindly.

## SentWeave 0.3.3 — audited research reference only

Upstream: `amajdalawi/sentweave`
Release commit: `61e9af7086a4339448ab4b2c25b8a2071961d123`
License: Apache-2.0.
PyPI sdist SHA-256:

```text
ef6414bdd1d7fa4064f31fdf1b446c7f2601955777fcdf64988fc09bca9d2940
```

A one-off research workflow validated hash-pinned installation, dependency audit, algorithm smoke, Linux, and Apple Silicon. Decision: reference only; native Rust covers the need with less operational surface.

## DadmaTools — conditional research candidate

License: Apache-2.0.
Potential value: Persian NER/POS/dependency/ezafe diagnostics.
Decision: defer unless a benchmark demonstrates a specific capability gap after the native Phase 19 stack.

## ContextWeaver — architecture reference, no dependency

ContextWeaver concepts were reviewed for long-form context packets, stable segment identity, revision history, and resumability. Native Context Packet v2 owns those responsibilities.

## TranslateBooksWithLLMs — design reference only

No source code is copied or linked. Selective glossary/context ideas overlap native memory architecture and its licensing does not justify importing code.

## TransAgents — research/agent-role reference only

TransAgents can inform translator/editor/fidelity/voice/naturalness role separation, but its orchestration and memory architecture are not runtime dependencies. Provider judgments remain separate from deterministic quality checks and human review.

## SacreBLEU / chrF++ — Phase 21 candidate

License: Apache-2.0.
Potential value: reproducible reference-based BLEU/chrF/TER benchmarking.
Decision: defer until a rights-safe EN->FA literary reference corpus exists. Reference metrics remain benchmark evidence, never the literary judge.

## Supporting-tool candidates outside translation runtime

### OpenDataLoader PDF — ingestion benchmark candidate

Potential value: structured Markdown/JSON/HTML extraction, reading-order/layout recovery, bounding boxes, tables, and OCR/hybrid handling for difficult PDFs. Do not replace `document-engine`; benchmark project-owned fixtures first and keep any Java/Python/hybrid AI tooling optional behind a narrow ingestion boundary.

### ripwire — developer-only code-intelligence candidate

Potential value: deterministic tree-sitter symbol/call graph, ranked bounded code maps, and MCP for coding agents. If evaluated, it remains developer tooling and never owns runtime/project memory or becomes required for build/translation/review/export.

### Headroom — conditional developer/research candidate

License: Apache-2.0. Potential value is coding/research context compression. Do not place lossy compression between literary evidence/context and translation/review providers without a dedicated fidelity benchmark.

## Other architecture references

- `sukamenev/booktrans` — whole-book scouting/selective context/editor-verifier patterns.
- Tolmach / `KazKozDev/book-translator` — refinement/verifier concepts; reference-only under licensing boundary.
- `madpin/epublate` — EPUB round-trip/glossary lifecycle ideas; no code copied without independent licensing/provenance review.
- ArmenianLitTranslator — critic-role/evaluation-dimension reference only.

## Upgrade / adoption policy

Before adding/upgrading any external repository/package/model/tool:

1. prove a concrete capability gap;
2. read current license, release/security notes, provenance, maintenance state, and model/data licenses;
3. compare public API, runtime cost, privacy, persistence, and failure implications;
4. prefer a reasonably small native Rust implementation when safer and easier to own;
5. keep heavyweight models/downloads optional unless a numbered phase explicitly promotes them;
6. benchmark on rights-safe/project-owned fixtures;
7. run rustfmt, Clippy, affected/full tests, release/CLI smoke where relevant, and vulnerability audits;
8. validate optional tools on Linux/macOS when expected there;
9. never convert external evidence into canon or human approval;
10. record selected/deferred decisions in roadmap/status/PMC/this document before merge.
