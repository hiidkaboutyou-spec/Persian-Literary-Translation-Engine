# Launch Readiness Roadmap

## Goal
Build a production-ready Persian Literary Translation Engine with a Rust core.

## Product Pipeline

Input Documents
-> Document Parsing
-> Chapter Extraction
-> Character Memory
-> Glossary Retrieval
-> Translation Context
-> Translation Generation
-> Quality Review
-> Publishing Export

## Current Foundations

- Rust workspace
- Translation core
- Memory engine
- Character engine
- Document models
- Quality foundations

## Production Milestones

### Milestone 1: Real File Ingestion
- PDF parser
- EPUB parser
- DOCX parser
- chapter detection
- metadata extraction

### Milestone 2: Knowledge Layer
- Supabase persistence
- glossary storage
- character bible storage
- translation memory retrieval

### Milestone 3: Translation Pipeline
- provider abstraction
- context assembly
- chapter processing
- retry and resume support

### Milestone 4: Publishing
- DOCX export
- EPUB export
- manuscript formatting

## Non-negotiable Principles

- Rust remains the core language.
- Preserve author intent and character voice.
- Do not replace literary decisions with raw machine translation.
- Every feature must support real user workflows.
