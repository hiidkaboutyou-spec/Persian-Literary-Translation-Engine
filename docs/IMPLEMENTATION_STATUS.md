# Implementation Status

## Vision
A production-grade Persian literary translation engine with a Rust core.

The system is designed around:
- document understanding before translation
- character voice preservation
- translation memory
- glossary consistency
- multi-pass quality control
- publication-ready export

## Current Rust Architecture

Implemented and validated:

### Core Crates
- **translation-core** — Provider-neutral translation pipeline with EchoProvider (deterministic testing) and OpenAIProvider (production). Multi-pass: translate → revise → quality review. Bounded passage chunking for oversized chapters.
- **document-engine** — Ingestion of TXT, Markdown, DOCX, EPUB, and text-based PDF files. Structured `Manuscript → Book → Chapter → Scene → Paragraph` model with source provenance. Extensible parser registry. Character-safe text chunking. Persian RTL DOCX export.
- **memory-engine** — Durable translation memory and glossary with JSON persistence. Passage-relevant runtime retrieval using Jaccard similarity with negation polarity detection and diversity filtering.
- **character-engine** — Character bible with profiles, aliases, word-boundary-aware matching, and relationship context. JSON persistence.
- **quality-engine** — Deterministic quality gate: empty output, prompt leakage, truncation, paragraph collapse, and terminology drift detection. Cross-chapter consistency auditing.
- **project-engine** — Project manifest with chapter state tracking, schema versioning, and JSON persistence.
- **human-review-workflow** — Versioned literary-intelligence review ledger, deterministic proposal identity/reconciliation, validated lifecycle transitions, typed canon conflicts/resolutions, promotion plans, and audit lineage. It owns review decisions but not canonical stores or file mutation.
- **project-engine** — Project manifest plus atomic review-ledger writes and recoverable multi-file promotion transactions for Character Bible, Glossary, and review audit state.
- **literary-intelligence-engine** — Literary decision models plus deterministic `Manuscript` analysis. Produces versioned character and relationship seeds, chapter maps, terminology candidates, observed literary-profile metrics, bounded evidence references, conflict reporting, and non-mutating initialization proposals.
- **literary-reference-knowledge** — Reference sources, editorial guidelines, and validation rules with UUID identity.
- **text-normalization** — Shared Persian/Arabic text normalization, negation detection, and similarity scoring. Eliminates duplication across memory, quality, and character engines.

### CLI
- `inspect` — Document analysis with text and JSON output
- `analyze` — Credential-free manuscript intelligence with text summaries and stable schema-versioned JSON
- `review sync/list/show/approve/edit/reject/defer/reopen/promote` — Non-interactive review, reconciliation, conflict preview, and explicit canon promotion with stable JSON outputs
- `prepare` — Chapter preparation with text and JSON output
- `run` — Full pipeline: ingest → segment → context → translate → quality → export
- `resume` — Checkpoint-based resume with source fingerprinting
- `--format json` — Machine-readable JSON output for all commands
- resume checkpoints include source and assembled-context fingerprints, so canon changes cannot silently reuse output generated under stale project knowledge

### CI/CD
- Format checking (rustfmt)
- Clippy with `-D warnings`
- Full workspace test suite
- Cross-crate integration tests (glossary, character bible, memory, quality gates, JSON output, resume)
- 120-chapter end-to-end regression coverage
- CLI smoke tests (text and JSON modes)
- Cargo audit for dependency vulnerability scanning
- Security audit workflow (weekly schedule)
- Dependabot configuration

## Non-Negotiable Constraints
- Rust remains the core language.
- No simple machine translation wrapper.
- Preserve author intent and narrative structure.
- Build reusable translation memory across projects.

## Current Test Count
- Coverage includes Phase 14 lifecycle, reconciliation, stable IDs, conflict resolution, dry-run non-mutation, atomic rollback, idempotent apply, audit lineage, Unicode, CLI JSON, canon-aware resume, manuscript intelligence, full pipeline, and large-book regression behavior
- Doc-tests: 1 (text-normalization)
