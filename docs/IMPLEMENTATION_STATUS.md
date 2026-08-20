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

Implemented foundations:

- translation-core
- memory-engine
- character-engine
- quality-engine
- document-engine
- CLI entry point

## Next Engineering Milestones

### Phase 1 — Real Input Pipeline

- PDF extraction
- EPUB parsing
- DOCX parsing
- chapter segmentation
- metadata extraction

### Phase 2 — Literary Intelligence

- character bible generation
- voice profiles
- relationship tracking
- terminology memory

### Phase 3 — Translation Workflow

- context retrieval
- translation jobs
- review passes
- consistency checks

### Phase 4 — Production

- persistent storage
- API service
- desktop CLI workflow
- export system

## Non-Negotiable Constraints

- Rust remains the core language.
- No simple machine translation wrapper.
- Preserve author intent and narrative structure.
- Build reusable translation memory across projects.
