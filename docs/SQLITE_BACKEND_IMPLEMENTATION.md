# SQLite Backend Implementation

## Goal

Provide a local-first storage backend for the Rust translation engine.

## Responsibilities

SQLite stores:

- translation memory
- glossary entries
- future character memory
- project metadata

## Design Rule

The Rust domain layer does not depend on SQLite.

Storage adapters can be replaced by:

- Supabase
- other databases
- local files

## Next Steps

- implement CRUD through storage traits
- add migrations
- add tests
- add search indexes
