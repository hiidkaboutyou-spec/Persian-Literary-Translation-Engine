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
- **literary-intelligence-engine** — Literary decision models (characters, scenes, relationships, translation decisions, decision traces) with validation.
- **literary-reference-knowledge** — Reference sources, editorial guidelines, and validation rules with UUID identity.
- **text-normalization** — Shared Persian/Arabic text normalization, negation detection, and similarity scoring. Eliminates duplication across memory, quality, and character engines.

### CLI
- `inspect` — Document analysis with text and JSON output
- `prepare` — Chapter preparation with text and JSON output
- `run` — Full pipeline: ingest → segment → context → translate → quality → export
- `resume` — Checkpoint-based resume with source fingerprinting
- `--format json` — Machine-readable JSON output for all commands

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
- Unit tests: ~115 across all crates
- Integration tests: 6 (pipeline, resume, JSON output, inspect JSON, quality gate, large-book regression)
- Doc-tests: 1 (text-normalization)
