# Sentry Integration Plan

## Goal

Add production-grade error monitoring without exposing private manuscripts, translations, glossary data, or API credentials.

## Principles

- Rust-only core remains unchanged
- Observability is separate from business logic
- No source text is sent by default
- No generated manuscript content is sent by default
- Secrets are never logged

## Events Allowed

Allowed examples:

- module name
- operation name
- error category
- duration metrics
- anonymous project identifier

Forbidden:

- manuscript text
- translated paragraphs
- user files
- API keys
- tokens

## Planned Rust Components

- telemetry abstraction
- Sentry adapter
- local logging fallback
- privacy scrubber

## Environment

SENTRY_DSN must be loaded from environment variables or local ignored configuration.

Never commit DSN secrets.
