# Implementation Status

## Vision

A production-grade English-to-Persian literary translation engine with a Rust core that preserves author intent, character voice, relationship/register, terminology, emotional subtext, continuity, and natural Persian while keeping human review as the final authority.

## Canonical Main State

`main` is verified through Phase 19.

Current verified `main` merge head after Phase 19:

```text
d073dab10c0965197745a6cbc7b8e56c946835e8
```

PR #95 merged Phase 19 and post-merge Rust/Phase 19/Phase 18 workflows were verified green.

### Core crates and application boundaries

- **translation-core** — provider-neutral translate -> revise -> quality pipeline with deterministic EchoProvider and production OpenAIProvider, bounded oversized-passage handling, immutable structural-marker guidance, and explicit translation style profiles.
- **document-engine** — TXT/Markdown/DOCX/EPUB/text-PDF ingestion, structured `Manuscript -> Book -> Chapter -> Scene -> Paragraph`, provenance, parser registry, Unicode-safe segmentation, Persian RTL DOCX export, and strict revision-pinned BookForge EPUB ingestion.
- **memory-engine** — durable translation memory/glossary, deterministic lexical/polarity/diversity retrieval, Context Packet v2 types/provenance/fingerprints, and optional semantic candidate fusion boundaries.
- **character-engine** — character profiles, aliases, relationships, word-boundary-aware relevance, and canonical JSON persistence.
- **quality-engine** — deterministic blocking checks plus optional advisory COMET and English/Persian Lingua diagnostics; probabilistic evidence never equals approval.
- **literary-intelligence-engine** — deterministic manuscript analysis, chapter maps, continuity hooks, entity/terminology seeds, observed literary metrics, evidence-backed initialization proposals, and shared Context Packet v2 assembly.
- **advanced-literary-analysis** — optional bounded provider-assisted literary findings with cache/resume, structured validation, stable evidence, and review-only output.
- **literary-review-engine** — Phase 19 post-translation review evidence for omission/addition, semantic fidelity, character voice, relationship/register, Persian naturalness, dialogue/subtext, terminology/continuity, plus profile-specific dimensions such as intimacy fidelity; automated review never auto-applies or human-approves revisions.
- **human-review-workflow** — versioned review ledger, stable IDs, lifecycle validation, typed conflicts, promotion plans, audit lineage, and explicit human decisions.
- **project-engine** — manifests, atomic persistence/recovery, application orchestration, translation lifecycle, literary-review/canon integration, checkpoint/fingerprint safety, manual revisions, export, history, and UI-ready snapshots.
- **literary-reference-knowledge** — editorial/reference sources and validation rules.
- **text-normalization** — shared Persian/Arabic normalization, matching, negation, and similarity helpers.

### Phase 17 — Controlled External Integrations — canonical

PR #90; merge commit `6a4b8807d8c54878f1f10db5cab1f1290fcc60fb`.

Revision-pinned BookForge EPUB parsing and optional isolated COMET evidence became approved boundaries. Safe supporting tooling added afterward includes optional Lingua diagnostics, checksum-pinned EPUBCheck tooling, and isolated projectmem developer memory.

### Phase 18 — Context Packet v2 & Selective Long-Novel Retrieval — canonical

PR #94; merge commit `2af408a19b8f69db93aff8e6896eaf189c4d69ae`.

Delivered typed/budgeted Context Packet v2, explicit authority/provenance/fingerprints, deterministic fallback retrieval, optional bounded BGE-M3/FastEmbed semantic ranking, timeout/failure fallback, context-aware resume invalidation, and Linux/Apple Silicon validation without model downloads in normal CI.

### Phase 19 — Literary Fidelity & Persian Naturalness Review — canonical

PR #95; merge commit `d073dab10c0965197745a6cbc7b8e56c946835e8`.

Delivered:

- `literary-review-engine` with independent evidence dimensions for omission/addition, semantic fidelity, character voice, relationship/register, Persian naturalness, dialogue/subtext, and terminology/continuity;
- explicit unevaluated state instead of treating unavailable evidence as a pass;
- provider-neutral bounded critic contracts with paragraph-index/evidence validation and revision proposals that never auto-apply;
- native bounded monotonic Rust alignment covering 1:1, 1:N, N:1, N:M and source/target gaps;
- optional BGE-M3 alignment adapter reusing the approved Phase 18 model boundary;
- strict alignment response validation and deterministic behavior when optional tools are absent/failing;
- persisted per-chapter literary-review artifacts with source/translation/context fingerprints and manual-edit staleness detection;
- `ApplicationService` plus CLI `project review-translation` boundary;
- permanent Linux/Apple Silicon Phase 19 validation.

Dependency decisions remain: Hazm blocked while its mandatory NLTK dependency is affected by the recorded unpatched advisory; Vecalign/SentWeave remain references while native Rust satisfies the measured need; DadmaTools remains conditional on a demonstrated Persian NLP gap.

## Current Branch — Phase 20 Publication-Grade EPUB Round Trip

Branch: `phase-20-publication-epub-roundtrip`.

This branch is **not canonical until its PR is merged and post-merge CI is verified**.

Implemented on the branch:

- native `SourceLocation.block_id` provenance for structured source-format block identity;
- EPUB paragraph/heading provenance carried into translated chapter artifacts;
- explicit `BookForge block ID -> translated text` reconstruction instead of positional guessing;
- fail-closed rejection of unknown, duplicate, missing, empty, or mismatched EPUB block translations;
- BookForge marker-aware structural reconstruction and validation using pinned revision `23f8c9d3c97a06f48e13424698441bfb4b037844`;
- source-aware preservation of XHTML structure and non-translated resources instead of regenerating the book from plain translated chapter text;
- target language rewriting through BookForge plus native `dir="rtl"` and OPF `page-progression-direction="rtl"` for Persian/other RTL targets;
- deterministic publication ZIP output and source non-mutation contract;
- application/CLI `project export ... --export-format docx|epub` while the existing DOCX path remains intact;
- application capability declaration for DOCX and EPUB exports;
- `literary` default translation style plus explicit opt-in `adult-intimacy` fidelity profile with mandatory caller confirmation that every participant in sexual content is an adult; automated intimacy-fidelity evidence remains non-canonical;
- permanent read-only `Phase 20 EPUB Round Trip` workflow with a generated rights-safe EPUB 3.3 fixture, EPUBCheck 5.3.0 before/after validation, source checksum verification, repeated-export byte comparison, representative CSS/image/link/inline-markup preservation checks, Linux validation, no-default-features document compatibility, and Apple Silicon arm64 compile/tests.

### Phase 20 standards/dependency decisions

- **EPUB 3.3** is the publication target because it is the current W3C Recommendation.
- **EPUB 3.4** remains deferred while it is a Candidate Recommendation; do not silently redefine the product standard.
- **EPUBCheck 5.3.0** remains the authoritative Phase 20 conformance gate because it validates EPUB 3.3.
- **EPUBCheck 5.4.x** is a future-compatibility signal while it validates EPUB 3 content against EPUB 3.4 rules; adoption requires a deliberate standards migration.
- No second EPUB framework, model, vector store, or publication database is added; Phase 20 reuses the approved BookForge boundary and the existing checksum-pinned validator tooling.

Detailed research and completion gates are in `docs/PHASE_20_RESEARCH.md`.

## CLI

Canonical commands include `inspect`, `analyze`, `analyze-advanced`, review lifecycle commands, `prepare`, `run`, `resume`, and project-oriented create/import/status/analyze/analyze-advanced/review/review-translation/translate/resume/progress/export/history flows.

The Phase 20 branch extends project export selection to `--export-format docx|epub` and translation configuration to `--style-profile literary|adult-intimacy` plus explicit adult-character confirmation for the latter.

## CI/CD

Canonical CI includes reproducible lockfile checks, rustfmt, Clippy `-D warnings`, workspace tests, optional-feature compatibility tests, strict EPUB compatibility paths, wrapper/sidecar checks, large-book regression, CLI smoke, Cargo audit, Security workflow, Dependabot, developer-memory validation, and Phase 18/19 Linux plus Apple Silicon gates.

Phase 20 adds a permanent publication workflow that may install the checksum-pinned EPUBCheck 5.3.0 distribution specifically for rights-safe publication validation. Normal ingestion/translation/DOCX runtime does not depend on Java/EPUBCheck.

## Non-Negotiable Constraints

- Rust remains the core language.
- No simple machine-translation wrapper replaces literary translation logic.
- Preserve author intent and narrative structure.
- Human review remains the approval/canon boundary.
- External/model metrics remain evidence, not authority.
- Optional tools/models must not become hidden runtime requirements.
- Review/provider/alignment failures remain explicit; absence is never interpreted as a clean review.
- EPUB publication provenance must be explicit; incomplete/mismatched block identity fails closed rather than guessing.
- No secrets, proprietary manuscripts, generated translations, or private reviewer material are committed to Git or developer-memory systems.

## Test Coverage

Coverage includes document ingestion, runtime translation, glossary/character/relationship memory, Context Packet v2 retrieval/fingerprints, deterministic quality gates, literary review/alignment, review lifecycle/promotion/conflicts, provider contracts, cache/resume, source/canon/context staleness, manual edits, atomic persistence, CLI JSON, Unicode/Persian cases, EPUB boundaries, optional quality evidence, and large-book regressions.

Phase 20 branch coverage additionally validates block-provenance reconstruction, fail-closed publication mapping, RTL metadata patching, EPUBCheck 3.3 conformance on synthetic fixtures, asset/link/inline-markup survival, deterministic repeated export, source non-mutation, and Apple Silicon publication compatibility.

## Immediate Exit Gate

Before Phase 20 can be called complete/canonical:

1. final branch dedicated Phase 20 workflow must be green;
2. no temporary write-enabled Phase 20 one-shot workflows/scripts may remain;
3. open the Phase 20 PR to `main`;
4. normal Rust/security/release checks on the PR head must be green;
5. merge only after those checks pass;
6. verify post-merge Phase 20 and normal main checks;
7. only then advance the roadmap to Phase 21.
