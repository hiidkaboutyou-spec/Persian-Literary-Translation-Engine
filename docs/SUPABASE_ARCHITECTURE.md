# Supabase Architecture

## Purpose

Supabase is used as the optional cloud data layer for the Rust translation engine.

The Rust core remains the source of truth for processing.

## Stored Data

- Projects
- Translation Memory
- Glossary entries
- Character profiles

## Privacy Rules

Never store:

- API keys
- raw private manuscripts in telemetry
- unnecessary user content

## Security

All exposed tables must use Row Level Security.

Future authentication layer will add project ownership policies.

## Rust Boundary

Rust modules communicate with storage through an abstraction layer. Supabase is not hardcoded into domain logic.
