# Rust-First Architecture

## Core Decision

The entire application layer will be written in Rust.

Goal: create a reliable, fast, cross-platform literary translation engine focused on Persian fiction workflows.

## Planned Stack

- Rust: core engine, CLI, orchestration, file processing
- Serde: configuration and data models
- Tokio: asynchronous tasks
- SQLite: translation memory storage
- Tauri (future): optional desktop interface

## Modules

- parser: PDF/EPUB/DOCX/TXT extraction
- glossary: terminology management
- character_bible: character continuity data
- memory: translation memory database
- pipeline: translation workflow orchestration
- formatter: final manuscript generation
- quality: consistency checks

## User Experience Goal

The user should not need programming knowledge. The final workflow should be:

Input file -> Analyze -> Translate -> Edit -> Quality Check -> DOCX output
