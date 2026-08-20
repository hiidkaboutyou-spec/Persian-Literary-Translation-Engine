# Persian Literary Translation Engine

A personal professional workflow for fiction and fanfiction translation into high-quality Persian.

## Purpose

This repository is designed to create a consistent literary translation pipeline:

- preserve character voices
- preserve emotional tone and narrative intent
- maintain terminology consistency
- store translation decisions between chapters
- prepare publication-style Persian manuscripts

## Current V1 Runtime

The Rust workspace now includes a CLI that can ingest UTF-8 text files, inspect chapter segmentation, and prepare chapter-level translation jobs.

From the `engine/` directory:

```bash
cargo run -p literary-engine -- inspect ../input/original_files/story.txt
cargo run -p literary-engine -- prepare ../input/original_files/story.txt fa
```

`inspect` reports document/chapter structure. `prepare` runs the current translation-preparation pipeline for each chapter. A real model/provider integration and publication export are still follow-up work.

## Workflow

1. Place source files in `input/original_files`
2. Build project glossary in `glossary/`
3. Create character profiles in `character_bible/`
4. Inspect and prepare the source through the Rust CLI
5. Apply translation prompts/providers
6. Review with quality-control rules
7. Export the final publication-style manuscript

## Repository Map

- `engine/` — Rust runtime, CLI, and core engines
- `prompts/` — master instructions and editing passes
- `glossary/` — names, terms, fixed translations
- `character_bible/` — voice and personality references
- `translation_memory/` — previous translation choices
- `tools/` — supporting automation utilities
- `config/` — project settings

## Quality and Security Gates

Pull requests that touch the Rust engine run formatting, Clippy, workspace tests, and a CLI build. Dependency security auditing and Dependabot updates are also configured.

## V1 Launch Gaps

The next production-critical capabilities are:

- DOCX and EPUB ingestion (PDF after text-based formats)
- real translation-provider integration
- persistent glossary / translation / character memory retrieval
- end-to-end quality evaluation
- professional DOCX export
- integration and performance tests on book-sized fixtures
- release packaging and deployment workflow

## Design Principle

The system prioritizes a natural Persian reading experience over word-for-word translation. It combines translation accuracy, literary editing, and consistency management.
