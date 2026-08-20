# Next Implementation Plan

## Goal

Convert the architecture into a usable Rust application.

## Milestone 1: Storage Layer

Create Rust traits:

- TranslationMemoryStore
- GlossaryStore
- CharacterMemoryStore

Implement:

- SQLite backend first
- Supabase adapter second

## Milestone 2: Retrieval Runtime

Build:

- exact glossary lookup
- previous translation lookup
- semantic retrieval interface
- context ranking

## Milestone 3: Document Pipeline

Support:

- TXT
- EPUB
- DOCX
- PDF

Create:

- chapters
- scenes
- dialogue segments

## Milestone 4: User Workflow

Provide simple commands:

translation-engine create
translation-engine import
translation-engine translate
translation-engine export

## Rule

Prioritize a working product over adding more architecture documents.
