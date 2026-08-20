# Project Memory

## Mission

Build a Rust-first Persian literary translation engine for fiction workflows.

## Core Principles

- Rust remains the core orchestration layer.
- External AI providers are adapters.
- Translation decisions must be reusable through memory.
- Character voice and style consistency are first-class features.
- Private manuscripts must be protected.

## Current Architecture

- Glossary memory
- Character memory
- Translation memory
- Hybrid retrieval design
- Supabase optional storage layer
- Security and telemetry boundaries

## Next Priority

Move from architecture to executable runtime:

1. Rust storage traits
2. SQLite implementation
3. Real retrieval engine
4. Document import pipeline
5. Translation workflow CLI

## Constraints

Do not turn the project into a simple API wrapper. The value is the literary memory and workflow system.
