# Implementation Status

## Vision

A production-grade English-to-Persian literary translation engine with a Rust core that preserves author intent, character voice, relationship/register, terminology, emotional subtext, continuity, and natural Persian while keeping human review as the final authority.

## Canonical Main State

`main` is verified through Phase 18.

### Core crates and application boundaries

- **translation-core** — provider-neutral translate → revise → quality pipeline with deterministic EchoProvider and production OpenAIProvider, plus bounded oversized-passage handling.
- **document-engine** — TXT/Markdown/DOCX/EPUB/text-PDF ingestion, structured `Manuscript → Book → Chapter → Scene → Paragraph`, provenance, parser registry, Unicode-safe segmentation, Persian RTL DOCX export, and strict revision-pinned BookForge EPUB ingestion.
- **memory-engine** — durable translation memory/glossary, deterministic lexical/polarity/diversity retrieval, Context Packet v2 types/provenance/fingerprints, and optional semantic candidate fusion boundaries.
- **character-engine** — character profiles, aliases, relationships, word-boundary-aware relevance, and canonical JSON persistence.
- **quality-engine** — deterministic blocking checks plus optional advisory COMET and English/Persian Lingua diagnostics; probabilistic evidence never equals approval.
- **literary-intelligence-engine** — deterministic manuscript analysis, chapter maps, continuity hooks, entity/terminology seeds, observed literary metrics, evidence-backed initialization proposals, and shared Context Packet v2 assembly.
- **advanced-literary-analysis** — optional bounded provider-assisted literary findings with cache/resume, structured validation, stable evidence, and review-only output.
- **human-review-workflow** — versioned review ledger, stable IDs, lifecycle validation, typed conflicts, promotion plans, audit lineage, and explicit human decisions.
- **project-engine** — manifests, atomic persistence/recovery, application orchestration, translation lifecycle, review/canon integration, checkpoint/fingerprint safety, manual revisions, export, history, and UI-ready snapshots.
- **literary-reference-knowledge** — editorial/reference sources and validation rules.
- **text-normalization** — shared Persian/Arabic normalization, matching, negation, and similarity helpers.

### Phase 17 — Controlled External Integrations — canonical

PR #90; merge commit `6a4b8807d8c54878f1f10db5cab1f1290fcc60fb`.

Canonical integrations include revision-pinned BookForge EPUB parsing and the optional isolated COMET quality-evidence sidecar. Safe supporting tooling added after Phase 17 includes optional Lingua English/Persian diagnostics, checksum-pinned EPUBCheck tooling, and isolated projectmem developer memory.

### Phase 18 — Context Packet v2 & Selective Long-Novel Retrieval — canonical

PR #94; merge commit `2af408a19b8f69db93aff8e6896eaf189c4d69ae`.

Delivered:

- typed/budgeted Context Packet v2 with provenance, authority, selection reasons, stable IDs, and SHA-256 packet fingerprints;
- relevant glossary, character/relationship canon, translation memory, reviewed literary findings, manuscript intelligence, and local continuity in one shared assembly policy;
- deterministic lexical retrieval remains the fallback/safety floor;
- optional Rust-native FastEmbed/BGE-M3 semantic retrieval/reranking boundary;
- sidecar timeout/failure fallback and rejection of unknown semantic IDs;
- context-aware resume invalidation through packet fingerprints;
- Linux and Apple Silicon arm64 validation without model downloads during normal build/CI.

## Phase 19 Branch State — Literary Fidelity & Persian Naturalness Review

Branch: `phase-19-literary-review-stack`.

This work is **not canonical until its pull request is merged and post-merge CI is verified**.

Implemented and validated so far:

- new `literary-review-engine` crate with typed dimensions for omission/addition, semantic fidelity, character voice, relationship/register, Persian naturalness, dialogue/subtext, and terminology/continuity;
- deterministic native findings that do not pretend unevaluated dimensions passed;
- provider-neutral literary critic contract with bounded paragraph-indexed input, structured output validation, evidence-index validation, requested-dimension enforcement, and revision proposals that are never auto-applied;
- optional OpenAI critic adapter while credential-free mock/native review remains available;
- native bounded monotonic alignment with 1:1, 1:N, N:1, N:M, source-only and target-only gaps;
- optional `tools/literary-alignment` Rust/FastEmbed process adapter reusing BGE-M3 rather than adding a second embedding stack;
- strict alignment response validation for unit identity, index ranges, monotonicity, complete coverage, finite values, and schema shape;
- post-translation `ApplicationService::review_translation` plus persisted per-chapter literary-review artifacts;
- artifact staleness detection from source, translated-text, and translation-context fingerprints;
- CLI `project review-translation` adapter over the same application API;
- permanent Phase 19 CI covering native review, application regression, alignment protocol, Linux BGE compile, Apple Silicon arm64 BGE compile, lockfiles, and security audits;
- application-level regression coverage proving literary review does not mutate the human-intelligence review/canon lifecycle and becomes stale after a manual translation edit.

### Phase 19 dependency decisions

- **BGE-M3/FastEmbed** — reuse the already-approved optional Phase 18 model boundary; no model download during normal compilation/default CI.
- **Hazm 0.12.1** — blocked. It requires NLTK, and the current compatible NLTK line is affected by unpatched High-severity `GHSA-8mgp-746c-j5xp` / `CVE-2026-81726`. No advisory waiver is allowed merely to enable Hazm.
- **Vecalign** — Apache-2.0 design reference, but not installed because its Python/Cython/C-compiler surface is unnecessary for the current native aligner; bundled Bleualign dev/test data has separate GPL licensing.
- **SentWeave 0.3.3** — release provenance/hash/platform/dependency audit completed successfully as research, but it remains reference-only because the native Rust aligner satisfies the same measured need with a smaller dependency surface.
- **DadmaTools** — deferred unless a concrete Persian NLP gap remains after the native Phase 19 stack.

Detailed research is in `docs/PHASE_19_RESEARCH.md`.

## CLI

Current canonical commands include:

- `inspect`
- `analyze`
- `analyze-advanced`
- review lifecycle commands
- `prepare`
- `run`
- `resume`
- `project create/import/status/analyze/analyze-advanced/review/translate/resume/progress/export/history`

The Phase 19 branch additionally exposes `project review-translation` through `ApplicationService`; it becomes canonical only after Phase 19 merge.

## CI/CD

Canonical CI includes reproducible lockfile checks, rustfmt, Clippy `-D warnings`, workspace tests, optional-feature compatibility tests, strict EPUB compatibility paths, wrapper/sidecar checks, 120-chapter regression, CLI smoke tests, Cargo audit, Security workflow, Dependabot, projectmem safe-init CI, and Phase 18 Linux/Apple Silicon semantic-tool validation.

Phase 19 adds a dedicated read-only workflow for review-engine/application tests and optional BGE alignment compilation on Linux and Apple Silicon without fetching model weights.

## Non-Negotiable Constraints

- Rust remains the core language.
- No simple machine-translation wrapper replaces literary translation logic.
- Preserve author intent and narrative structure.
- Human review remains the approval/canon boundary.
- External/model metrics remain evidence, not authority.
- Optional tools/models must not become hidden runtime requirements.
- Review/provider/alignment failures must remain explicit; absence must never be interpreted as a clean literary review.
- No secrets, proprietary manuscripts, generated translations, or private reviewer material are committed to Git or developer-memory systems.

## Test Coverage

Coverage includes document ingestion, runtime translation, glossary/character/relationship memory, Context Packet v2 retrieval/fingerprints, deterministic quality gates, review lifecycle/promotion/conflicts, provider contracts, cache/resume, source/canon staleness, manual edits, atomic persistence, CLI JSON, Unicode/Persian cases, EPUB boundaries, optional quality evidence, and large-book regressions.

Phase 19 branch coverage additionally validates native review dimensions, provider evidence indices/dimension scope, monotonic alignment and gaps, sidecar schema/failure boundaries, persisted review artifacts, manual-edit staleness, non-mutation of human review/canon state, and credential-free operation.
