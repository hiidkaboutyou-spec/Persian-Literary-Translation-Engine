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

The current `main` branch already contains the production foundation represented by:

- structured TXT/Markdown/DOCX/EPUB/text-PDF ingestion and source provenance
- translation memory, glossary, character bible, relationship context, and Persian-aware normalization/retrieval
- provider-neutral multi-pass translation runtime with bounded requests and resumable checkpoints
- deterministic quality gates and cross-chapter consistency checks
- manuscript intelligence and evidence-backed context seeding
- human review ledger, conflict handling, preview/apply canon promotion, audit lineage, and atomic recovery
- optional bounded model-assisted literary analysis whose findings remain review-controlled
- project-oriented `ApplicationService` orchestration for Import -> Analyze -> Review -> Translate -> Edit -> Export
- CLI/JSON interfaces, release/security CI, large-book regression coverage, and Persian RTL DOCX output

## Verified Recent Phases

### Phase 13 — Manuscript Intelligence & Literary Context Seeding — merged

Deterministic pre-translation analysis produces evidence-backed character/relationship seeds, chapter maps, terminology candidates, literary metrics, stable IDs, and non-mutating initialization proposals.

### Phase 14 — Literary Intelligence Review & Canon Promotion — merged

Human-controlled review lifecycle, deterministic proposal reconciliation, typed canon conflicts, preview/apply promotion, atomic multi-file mutation, rollback/recovery, and audit lineage.

### Phase 15 — Advanced Model-Assisted Literary Analysis — merged

Optional bounded provider analysis with structured validation, stable evidence, cache/resume, injection resistance, review-only literary findings, and chapter-scoped translation context.

### Phase 16 — Application Orchestration Layer — merged

One project-oriented application boundary above the domain engines. Provides project lifecycle, snapshots, analysis/review/promotion orchestration, translation progress/pause/resume, manual edit revisions, export, typed errors, events, locking, staleness detection, and UI-ready JSON models.

### Phase 17 — Controlled External Integrations — in progress

Branch: `phase-17-external-integrations`
PR: #90
Implementation state: implemented on branch, not canonical until merged.

Scope:

- pin and integrate BookForge for validated deterministic EPUB ingestion while mapping back into native `document-engine` models
- preserve an explicit legacy-parser compatibility build path; never silently fall back after a BookForge validation failure
- add an optional isolated Rust <-> Python COMET/XCOMET/DocCOMET quality-evidence sidecar
- keep external scores advisory; they cannot mark a translation human-approved
- keep ContextWeaver, TranslateBooksWithLLMs, TransAgents, and similar projects as design references when native capabilities already cover the need
- record dependency, license, privacy, and upgrade boundaries
- keep the lockfile, CI, audit, release build, and smoke tests reproducible

Phase 17 completion gate:

1. latest PR head passes lockfile check, rustfmt, Clippy `-D warnings`, compatibility build, COMET-sidecar syntax/protocol checks, full workspace tests, cargo audit, release build, and CLI smoke test
2. PR #90 is merged into `main`
3. post-merge verification confirms `main` contains the integration and documentation

## Forward Roadmap

### Phase 18 — Context Packet v2 & Selective Long-Novel Retrieval

Goal: make every translation unit receive the smallest sufficient, highest-value context instead of broad prompt stuffing.

Planned work:

- stable context-packet contract per translation unit
- passage-relevant glossary selection only
- relevant character/relationship state only
- previous translation decisions and local continuity handoff
- scene/chapter neighbor context with strict budgets
- retrieval provenance explaining why each memory item was included
- deterministic context fingerprints so changed canon/context invalidates stale checkpoints
- regression tests for very long novels, repeated names, polarity, timeline changes, and conflicting terminology

### Phase 19 — Literary Fidelity & Persian Naturalness Review Stack

Goal: catch errors that generic MT metrics miss.

Planned independent review dimensions:

- omission/addition detection
- semantic faithfulness and narrative-intent preservation
- character-voice consistency
- relationship/register consistency
- Persian naturalness and non-literal fluency
- dialogue rhythm and emotional-subtext preservation
- terminology/continuity consistency

These reviewers produce evidence and revision proposals. They do not silently mutate canon and do not bypass human review.

### Phase 20 — Publication-Grade EPUB Round Trip

Goal: produce a translated EPUB while preserving the source book's structure and assets deterministically.

Planned work:

- block/segment identity through translation and rebuild
- preserve XHTML structure, links, footnotes/endnotes, navigation, images, styles, and non-translatable resources
- RTL/Persian metadata and language handling
- structural round-trip validation before publication
- deterministic output checks and hostile/corrupt EPUB fixtures
- keep DOCX publishing path intact

### Phase 21 — Literary Evaluation Corpus & Benchmarking

Goal: measure whether changes improve actual Persian literary translation quality rather than only passing unit tests.

Planned work:

- curated EN -> FA literary test corpus with rights-safe/project-owned fixtures
- expected terminology, voice, relationship, omission, and continuity assertions
- deterministic regression suite plus optional COMET/XCOMET evidence
- human-review scorecards for naturalness, voice, fidelity, subtext, and readability
- compare model/provider/prompt/runtime changes without making one metric the final judge

### Phase 22 — Product Surface & Distribution Hardening

Goal: expose the stable application layer through a usable product without moving domain logic into the UI.

Possible surfaces can call `ApplicationService` directly and should support project home/status, analysis review, canon editing, translation progress, manual revision history, export, recovery hints, and provider configuration checks.

## Next Action Rule

Always finish and verify the current phase before starting a new numbered phase. Do not treat branch-only work as merged. When a phase changes architecture or persistence contracts, update `IMPLEMENTATION_STATUS.md`, this roadmap, engineering decisions, and project-memory notes together.
