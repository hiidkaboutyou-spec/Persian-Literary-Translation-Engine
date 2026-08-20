# Implementation Roadmap

## Phase 1 - Memory Foundation

- Translation memory CRUD
- Glossary enforcement
- Character voice retrieval
- Style guide retrieval

## Phase 2 - Document Intelligence

- Streaming PDF importer
- EPUB importer
- DOCX importer
- Chapter and dialogue segmentation

## Phase 3 - Translation Runtime

- Rust AI provider abstraction
- Prompt isolation
- Context assembly
- Quality checks

## Phase 4 - Publishing Output

- Persian RTL DOCX export
- EPUB export
- Book formatting

## Design Rule

The project remains Rust-first. External services such as Supabase are storage adapters, not the core engine.
