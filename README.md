# Persian Literary Translation Engine

A Rust-based workflow for translating fiction and fanfiction into consistent, publication-oriented Persian while preserving character voice, emotional tone, terminology, and translation decisions across chapters.

## Current Status

**v0.1.0 launch readiness is complete and validated end to end.**

Implemented and validated:

- Rust workspace and CLI
- TXT / Markdown ingestion
- DOCX ingestion
- EPUB ingestion
- text-based PDF ingestion with explicit OCR limitation for scanned/image-only files
- chapter segmentation
- unified serializable `Manuscript -> Book -> Chapter -> Scene -> Paragraph` model
- source provenance on every extracted chapter, scene, and paragraph (including PDF page, EPUB resource, and DOCX paragraph)
- deterministic manuscript intelligence with evidence-backed character, relationship, chapter, and terminology seeds
- observed literary-profile metrics and proposal-only pre-translation context that never promotes inferred state to canon
- extensible format-parser registry with explicit corrupted, empty, unsupported, parsing, and structure errors
- provider-neutral literary translation pipeline
- deterministic `EchoProvider` for credential-free end-to-end testing
- production `OpenAIProvider` using the Responses API with credentials supplied only through environment variables
- automatic provider selection (`OpenAI` when `OPENAI_API_KEY` is set, otherwise `Echo`), with explicit override support
- durable local JSON persistence for Translation Memory, Glossary, Character Bible, and relationship context
- passage-relevant runtime retrieval from persisted project memory
- deterministic end-to-end quality gate for empty output, prompt leakage, structure/truncation warnings, and terminology drift
- native publication-oriented Persian RTL DOCX export
- automatic final `manuscript.docx` generation from the `run` workflow
- bounded provider passage processing for oversized chapters
- 120-chapter credential-free end-to-end regression coverage
- hardened Rust CI, dependency audit, Dependabot, and CLI smoke tests
- tag-driven cross-platform GitHub Release publishing for Linux x86_64, macOS arm64, and Windows x86_64
- SHA-256 release checksums
- successful first versioned release validation as `v0.1.0`

The original v0.1.0 launch blockers are complete. Post-v0.1 development now focuses on project orchestration, human review workflows, literary decision traceability, large-book efficiency/cost control, offline resilience, and a stable v1 compatibility contract. See `docs/NEXT_DEVELOPMENT_CYCLES.md`.

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

Analyze literary context before translation (offline and deterministic):

```bash
cargo run -p literary-engine -- analyze ../input/original_files/story.epub
cargo run -p literary-engine -- analyze ../input/original_files/story.epub --format json
```

Analysis retains bounded references to stable chapter, scene, paragraph, and source-location IDs, not
manuscript excerpts. Character, relationship, and terminology results are inferred seeds. Existing
Character Bible and Glossary entries remain approved canon, take precedence during context assembly,
and are never overwritten by analysis. See `docs/MANUSCRIPT_INTELLIGENCE.md`.

Prepare chapter translation requests:

```bash
cargo run -p literary-engine -- prepare ../input/original_files/story.epub fa
```

Run the end-to-end pipeline:

```bash
cargo run -p literary-engine -- run ../input/original_files/story.epub fa ../output/runtime
```

A successful run emits per-chapter text artifacts, a runtime manifest, and the final Persian `manuscript.docx`.

Provider selection for `run`:

- set `OPENAI_API_KEY` to use the production OpenAI provider automatically
- optionally set `OPENAI_MODEL` to override the default model
- set `LITERARY_ENGINE_PROVIDER=openai` to require OpenAI explicitly
- set `LITERARY_ENGINE_PROVIDER=echo` for deterministic local/offline testing
- without an OpenAI key or explicit provider override, the CLI falls back to `EchoProvider`

Optional persisted project memory:

- `LITERARY_ENGINE_MEMORY_FILE=/path/to/translation-memory.json`
- `LITERARY_ENGINE_GLOSSARY_FILE=/path/to/glossary.json`
- `LITERARY_ENGINE_CHARACTER_BIBLE_FILE=/path/to/character-bible.json`

The OpenAI provider disables response storage in its API requests. Secrets are not committed to repository files.

## Runtime Flow

```text
Source manuscript
      ↓
Document ingestion
      ↓
Chapter segmentation
      ↓
Deterministic manuscript intelligence
      ↓
Passage-relevant project memory
      ↓
Translation provider
      ↓
Revision / quality-review passes
      ↓
Deterministic quality gate
      ↓
Chapter artifacts + manifest
      ↓
Persian RTL manuscript.docx
```

## Structured Manuscript Contract

`document_engine::ingest_file` is the canonical ingestion entry point for TXT, Markdown, DOCX,
EPUB, and text-based PDF files. It returns a JSON-serializable `Manuscript`; its ordered chapters
feed the existing translation, project-memory, and quality-gate flow directly. Scene separators
such as `***` are preserved as scene boundaries, while stable IDs and `SourceLocation` values make
each paragraph traceable to its source file, format, page or archive resource, chapter, scene, and
paragraph position.

New formats can be added by implementing `ManuscriptParser` and registering the parser with
`DocumentIngestor`, without changing the manuscript model or downstream translation pipeline.

## Repository Map

- `engine/` — Rust workspace and executable runtime
- `prompts/` — translation and editing instructions
- `glossary/` — names, terms, and fixed translation decisions
- `character_bible/` — voice/personality references
- `translation_memory/` — previous translation decisions
- `docs/` — architecture, milestones, and development plans
- `.github/workflows/` — CI, security audit, and release automation

## Design Principle

The system prioritizes a natural Persian reading experience over word-for-word translation. Its goal is to combine translation accuracy, literary editing, character-voice preservation, and long-range consistency in one reproducible workflow.

## Launch Tracking

The v0.1.0 launch checklist was completed in GitHub issue #10. Post-launch work is tracked through the development-cycle roadmap in `docs/NEXT_DEVELOPMENT_CYCLES.md`.
