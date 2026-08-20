# Persian Literary Translation Engine - Product Goals

## Core Mission

Build a practical personal translation engine for long-form fiction and fanfiction.

The system must prioritize:

- natural Persian literary output
- character voice preservation
- emotional continuity
- terminology consistency
- translation memory across chapters
- easy workflow for non-programmers

## Rust-First Rule

All core application logic must be written in Rust.

Rust handles:

- project management
- parsing pipeline
- memory system
- glossary engine
- quality checks
- export orchestration

External AI providers are integration points only and must not replace the Rust core.

## User Experience Goal

The final workflow should be:

1. Create project
2. Import story files
3. Analyze characters and terminology
4. Translate chapters
5. Review quality
6. Export final manuscript

The user should not need programming knowledge.
