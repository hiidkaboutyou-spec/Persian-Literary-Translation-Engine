# Research Notes

## Goal

Build a practical Rust-first literary translation engine for long-form fiction and fanfiction.

## Design principles gathered from existing translation workflows

- Translation quality depends on context, not only sentence conversion.
- Character voice and terminology consistency require persistent memory.
- Long documents need chapter/scene level processing.
- Glossaries and translation memory are core features.
- Import/export should be separated from translation logic.

## Rust Decision

The core application logic remains Rust. External AI providers are integration layers only and are not the application language.
