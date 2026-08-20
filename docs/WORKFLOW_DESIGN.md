# Translation Workflow Design

## Goal

Build a Rust-first literary translation engine for long fiction.

## Pipeline

1. Import source document
2. Split chapters and scenes
3. Build character context
4. Apply glossary rules
5. Use translation memory
6. Run literary editing checks
7. Export manuscript

## Non negotiable principles

- Rust remains the core implementation language
- Character consistency matters
- Previous translation decisions must be reusable
- Output quality is prioritized over literal translation
