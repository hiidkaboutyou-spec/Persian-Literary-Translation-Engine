# Codex Security Plan

## Purpose

Security review strategy for the Rust-first Persian Literary Translation Engine.

## Security Priorities

1. Protect private manuscripts.
2. Prevent secrets from entering source control.
3. Treat imported text as untrusted data.
4. Validate file formats and paths.
5. Keep unsafe Rust usage minimized.
6. Review third-party dependencies.

## Review Areas

### Input Handling

- PDF/EPUB/DOCX parsing
- malformed documents
- oversized files
- path traversal risks

### Data Storage

- SQLite translation memory
- glossary files
- character profiles
- backups

### AI Boundary

- isolate prompts from imported content
- prevent instruction injection
- redact sensitive logs

### Build Security

- cargo audit
- dependency updates
- CI checks
- secret scanning

## Definition of Done

The engine should safely process user manuscripts while preserving privacy and translation quality.
