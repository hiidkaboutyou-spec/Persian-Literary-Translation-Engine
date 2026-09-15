# Implementation Roadmap

This file is the current high-level delivery map for the Persian Literary Translation Engine.
The older four-step bootstrap roadmap is superseded by the production architecture that now exists on `main`.

## Product Goal

Build a production-grade English-to-Persian literary translation system that preserves author intent, narrative structure, character voice, relationships, emotional subtext, terminology, and long-novel continuity while keeping human review as the final authority.

## Non-Negotiable Design Rules

- Rust remains the core language and domain/runtime boundary.
- Python is allowed only behind narrow sidecar/tool boundaries when a mature capability is impractical to reproduce in Rust.
- Literary understanding, translation execution, memory, quality evaluation, review, and persistence remain separated by explicit contracts.
- Model inference never becomes canon automatically.
- External quality scores never equal human approval.
- Approved canon must outrank unresolved inference in translation context.
- Long-running work must be resumable and fingerprint-safe.
- External repositories are integrated selectively; do not vendor or replace native architecture wholesale.
- Secrets, credentials, and manuscript content must not be committed to the repository.

## Delivered Foundation

The current `main` branch contains the production foundation represented by:

- structured TXT/Markdown/DOCX/EPUB/text-PDF ingestion and source provenance
- translation memory, glossary, character bible, relationship context, and Persian-aware normalization/retrieval
- provider-neutral multi-pass translation runtime with bounded requests and resumable checkpoints
- deterministic quality gates and cross-chapter consistency checks
- manuscript intelligence and evidence-backed context seeding
- human review ledger, conflict handling, preview/apply canon promotion, audit lineage, and atomic recovery
- optional bounded model-assisted literary analysis whose findings remain review-controlled
- project-oriented `ApplicationService` orchestration for Import -> Analyze -> Review -> Translate -> Edit -> Export
- CLI/JSON interfaces, release/security CI, large-book regression coverage, and Persian RTL DOCX output
- controlled external integrations with pinned provenance and explicit failure boundaries

## Verified Recent Phases

### Phase 13 — Manuscript Intelligence & Literary Context Seeding — merged

Deterministic pre-translation analysis produces evidence-backed character/relationship seeds, chapter maps, terminology candidates, literary metrics, stable IDs, and non-mutating initialization proposals.

### Phase 14 — Literary Intelligence Review & Canon Promotion — merged

Human-controlled review lifecycle, deterministic proposal reconciliation, typed canon conflicts, preview/apply promotion, atomic multi-file mutation, rollback/recovery, and audit lineage.

### Phase 15 — Advanced Model-Assisted Literary Analysis — merged

Optional bounded provider analysis with structured validation, stable evidence, cache/resume, injection resistance, review-only literary findings, and chapter-scoped translation context.

### Phase 16 — Application Orchestration Layer — merged

One project-oriented application boundary above the domain engines. Provides project lifecycle, snapshots, analysis/review/promotion orchestration, translation progress/pause/resume, manual edit revisions, export, typed errors, events, locking, staleness detection, and UI-ready JSON models.

### Phase 17 — Controlled External Integrations — merged

PR: #90
Merge commit: `6a4b8807d8c54878f1f10db5cab1f1290fcc60fb`

Delivered:

- BookForge pinned to a known upstream revision for validated deterministic EPUB ingestion while mapping back into native `document-engine` models
- explicit legacy-parser compatibility build path with no silent fallback after BookForge validation failure
- optional isolated Rust <-> Python COMET/XCOMET/DocCOMET quality-evidence sidecar
- advisory-only external metrics that cannot mark a translation human-approved
- documented licensing, privacy, upgrade, and failure boundaries
- reproducible lockfile/CI/security/audit/release/smoke validation
- post-merge verification of PR #90 on `main`

### Phase 18 — Context Packet v2 & Selective Long-Novel Retrieval — merged

PR: #94
Merge commit: `2af408a19b8f69db93aff8e6896eaf189c4d69ae`

Delivered:

- typed Context Packet v2 with provenance, authority, bounded budgets, stable IDs, and SHA-256 packet fingerprints
- passage-relevant glossary, character/relationship canon, translation memory, literary findings, local neighbor continuity, and hierarchical manuscript-intelligence context
- one shared context assembly policy for CLI and `ApplicationService`
- deterministic lexical retrieval remains the safety floor
- optional Rust-native FastEmbed boundary using BGE-M3 / multilingual reranking only when explicitly enabled
- semantic sidecar timeout/failure handling with deterministic fallback; model output cannot create canon IDs or own project memory
- Linux plus Apple Silicon arm64 compatibility checks without downloading model weights during normal compilation
- context fingerprint integration with resumable checkpoints so canon/context changes invalidate stale reuse safely

## Safe Supporting Tooling

Small supporting integrations may land between numbered phases when they do not change the phase architecture or runtime defaults. They must remain optional or advisory and pass the same CI/security gates.

Current supporting tooling:

- optional `lingua-rs` English/Persian language diagnostics for detecting probable untranslated English output; disabled by default and never an automatic approval/rejection decision
- checksum-pinned optional EPUBCheck installer/wrapper for future publication validation; no EPUBCheck binary is vendored into the repository
- optional projectmem developer-side memory tooling; it never owns translation/runtime memory

Future supporting-tool candidates must not interrupt the numbered roadmap. OpenDataLoader PDF is a benchmark candidate for difficult PDF ingestion; ripwire is a developer-only code-intelligence candidate; Headroom is conditional developer/research context compression. None is a Phase 19 runtime dependency.

## Current Phase

### Phase 19 — Literary Fidelity & Persian Naturalness Review Stack — branch validation in progress

Goal: catch errors that generic MT metrics miss without turning probabilistic evidence into approval.

Implemented on the Phase 19 branch and pending final PR/merge verification:

- independent review dimensions for omission/addition, semantic fidelity, character voice, relationship/register, Persian naturalness, dialogue/subtext, and terminology/continuity
- native deterministic review evidence with explicit unevaluated dimensions
- provider-neutral bounded literary critic with validated paragraph citations and revision proposals that are never auto-applied
- post-translation `ApplicationService` review API plus persisted per-chapter review artifacts
- source/translation/context fingerprints that mark old review artifacts stale after manual edits or changed translation context
- native bounded monotonic Rust alignment supporting 1:1, 1:N, N:1, N:M, and source/target gaps
- optional BGE-M3 alignment adapter reusing the existing Phase 18 model boundary; no second embedding stack
- schema/coverage/monotonicity/index validation before alignment evidence enters a review report
- deterministic operation when provider/alignment tooling is absent or fails
- Linux and Apple Silicon arm64 compile validation without model downloads

Dependency decisions:

- Hazm 0.12.1 is blocked while its mandatory NLTK dependency is affected by an unpatched High-severity advisory; do not waive the advisory
- Vecalign is an Apache-2.0 design reference but its Python/Cython runtime is unnecessary for the current native implementation; its bundled Bleualign test/dev data has separate GPL licensing
- SentWeave 0.3.3 was hash/provenance/platform/audit checked as a reference but remains uninstalled because the native Rust aligner satisfies the measured requirement with less dependency surface
- DadmaTools remains conditional on a measured Persian NLP gap after Phase 19

Exit criteria:

- permanent read-only Phase 19 CI
- native review and application artifact/staleness tests
- provider-schema and alignment-protocol tests
- optional BGE compile checks on Linux and Apple Silicon
- RustSec audits
- normal Rust CI/security/release smoke on the final PR head
- no temporary write-enabled validation workflows/scripts in the final diff
- PR merge and post-merge verification before Phase 19 is called canonical

## Forward Roadmap

### Phase 20 — Publication-Grade EPUB Round Trip

Goal: produce a translated EPUB while preserving the source book's structure and assets deterministically.

Planned work:

- block/segment identity through translation and rebuild
- preserve XHTML structure, links, footnotes/endnotes, navigation, images, styles, and non-translatable resources
- RTL/Persian metadata and language handling
- EPUBCheck structural conformance validation before publication
- deterministic output checks and hostile/corrupt EPUB fixtures
- keep DOCX publishing path intact

### Phase 21 — Literary Evaluation Corpus & Benchmarking

Goal: measure whether changes improve actual Persian literary translation quality rather than only passing unit tests.

Planned work:

- curated EN -> FA literary test corpus with rights-safe/project-owned fixtures
- expected terminology, voice, relationship, omission, and continuity assertions
- deterministic regression suite plus optional COMET/XCOMET evidence
- add SacreBLEU/chrF++ only when a suitable reference corpus exists; it is benchmark evidence, never the literary judge
- human-review scorecards for naturalness, voice, fidelity, subtext, and readability
- compare model/provider/prompt/runtime changes without making one metric the final judge

### Phase 22 — Product Surface & Distribution Hardening

Goal: expose the stable application layer through a usable product without moving domain logic into the UI.

Possible surfaces can call `ApplicationService` directly and should support project home/status, analysis review, canon editing, translation progress, literary-review evidence, manual revision history, export, recovery hints, and provider configuration checks.

## Next Action Rule

Always finish and verify the current numbered phase before starting the next numbered phase. Supporting tooling may be added only when it does not change runtime defaults or claim completion of a later phase. Do not treat branch-only work as merged. When a phase changes architecture or persistence contracts, update `IMPLEMENTATION_STATUS.md`, this roadmap, engineering decisions, external-integration notes, and project-memory notes together.
