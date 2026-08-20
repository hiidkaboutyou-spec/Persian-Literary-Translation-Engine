# V1 Runtime Plan

## Goal
Build a usable Persian literary translation engine, not only a framework.

## Runtime Flow

Input file
-> document ingestion
-> chapter extraction
-> character/context memory
-> glossary retrieval
-> translation pipeline
-> quality review
-> manuscript export

## First Usable Release

The first release must support:

- CLI execution
- local document loading
- chapter based processing
- persistent translation decisions
- deterministic glossary reuse
- quality report generation

## Engineering Rules

- Rust remains the core language.
- Providers must be replaceable.
- Memory must survive between chapters.
- Translation quality is evaluated by consistency, voice preservation, and readability.
