# Persian Literary Translation Engine

A personal professional workflow for fiction and fanfiction translation into high-quality Persian.

## Purpose

This repository is designed to create a consistent literary translation pipeline:

- preserve character voices
- preserve emotional tone and narrative intent
- maintain terminology consistency
- store translation decisions between chapters
- prepare publication-style Persian manuscripts

## Workflow

1. Place source files in `input/original_files`
2. Build project glossary in `glossary/`
3. Create character profiles in `character_bible/`
4. Apply translation prompts from `prompts/`
5. Review with quality control rules
6. Export final DOCX manuscript

## Repository Map

- `prompts/` — master instructions and editing passes
- `glossary/` — names, terms, fixed translations
- `character_bible/` — voice and personality references
- `translation_memory/` — previous translation choices
- `tools/` — future automation utilities
- `config/` — project settings

## Design Principle

The system prioritizes a natural Persian reading experience over word-for-word translation. It combines translation accuracy, literary editing, and consistency management.
