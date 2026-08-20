# Storage Layer Design

## Goal

Create a usable Rust-first translation engine where the core workflow is independent from storage providers.

## Architecture

Translation Runtime

-> Storage Traits

-> Backends

- SQLite (local/offline)
- Supabase/Postgres (sync/cloud)

## Required Stores

### TranslationMemoryStore

Responsible for:

- saving approved translations
- retrieving previous decisions
- future similarity search

### GlossaryStore

Responsible for:

- terminology consistency
- preferred translations
- project-specific rules

## Design Rules

- Domain logic must not depend on database implementation.
- Private manuscripts remain local unless explicitly configured.
- Every backend must preserve project isolation.

## Next Implementation Steps

1. SQLite backend
2. Supabase adapter
3. Retrieval ranking
4. Integration with translation context builder
