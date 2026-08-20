# Observability Plan

## Goal

Track failures in the Rust translation engine without collecting private manuscript content.

## Signals

### Errors

Capture:
- parser failures
- database failures
- export failures
- provider connection failures

### Metrics

Track:
- chapters processed
- translation duration
- glossary hits
- memory lookups
- failed pipeline stages

### Privacy Rules

Never send:
- source manuscripts
- private user documents
- full translated chapters
- API keys

Prefer:
- error categories
- stack traces without sensitive data
- anonymized project identifiers

## Future Integration

A Sentry-compatible error reporting layer may be added behind a Rust interface so monitoring does not affect core logic.
