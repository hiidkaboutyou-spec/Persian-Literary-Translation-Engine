# Persian Literary Translation Engine

A Rust-based workflow for translating fiction and fanfiction into consistent, publication-oriented Persian while preserving character voice, emotional tone, terminology, and translation decisions across chapters.

## Current Status

The project now has a working executable runtime foundation rather than only prompt/documentation scaffolding.

Implemented:

- Rust workspace and CLI
- TXT / Markdown ingestion
- DOCX ingestion
- EPUB ingestion
- text-based PDF ingestion
- chapter segmentation
- provider-neutral translation pipeline
- deterministic `EchoProvider` for credential-free end-to-end testing
- translation-memory / glossary / character-memory foundations
- consistency-quality foundations
- Rust CI, dependency audit, Dependabot, and cross-platform release builds

Not production-complete yet:

- real translation-provider integration
- durable persistence/retrieval for translation, glossary, and character memory
- end-to-end literary quality evaluation
- professional Persian DOCX export with RTL typography and book layout
- large-book bounded-memory/performance regression coverage

## Build

From the repository root:

```bash
cd engine
cargo build --release -p literary-engine
```

Run the full workspace test suite:

```bash
cd engine
cargo test --workspace
```

## CLI

Supported manuscript inputs:

- `.txt`
- `.md`
- `.docx`
- `.epub`
- `.pdf` (text-based PDFs; scanned/image-only PDFs require OCR first)

Inspect a manuscript:

```bash
cd engine
cargo run -p literary-engine -- inspect ../input/original_files/story.epub
```

Prepare chapter translation requests:

```bash
cargo run -p literary-engine -- prepare ../input/original_files/story.epub fa
```

Exercise the current end-to-end runtime pipeline:

```bash
cargo run -p literary-engine -- run ../input/original_files/story.epub fa ../output/runtime
```

The `run` command currently uses the deterministic `EchoProvider`; this validates ingestion → chapter segmentation → pipeline → quality stage → exported chapter files without requiring API credentials.

## Runtime Flow

```text
Source manuscript
      ↓
Document ingestion
      ↓
Chapter segmentation
      ↓
Context / memory hooks
      ↓
Translation provider
      ↓
Quality stage
      ↓
Chapter outputs + manifest
```

## Repository Map

- `engine/` — Rust workspace and executable runtime
- `prompts/` — translation and editing instructions
- `glossary/` — names, terms, and fixed translation decisions
- `character_bible/` — voice/personality references
- `translation_memory/` — previous translation decisions
- `docs/` — architecture, milestones, and production plans
- `.github/workflows/` — CI, security audit, and release automation

## Design Principle

The system prioritizes a natural Persian reading experience over word-for-word translation. Its goal is to combine translation accuracy, literary editing, character-voice preservation, and long-range consistency in one reproducible workflow.

## Launch Tracking

Launch blockers and their current status are tracked in GitHub issue #10.
