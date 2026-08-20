# Threat Model

## System

Persian Literary Translation Engine

## Assets

- source manuscripts
- translation memory database
- glossary data
- character profiles
- API credentials
- generated manuscripts

## Entry Points

- imported documents
- configuration files
- external model responses
- user-created glossary entries

## Risks

### Document Injection

Imported text may contain instructions that should not affect the application.

Mitigation:
- treat documents as content only
- separate prompts from imported text

### Secret Leakage

Mitigation:
- environment variables
- ignored local files
- secret scanning in CI

### Database Integrity

Mitigation:
- validated schemas
- backups
- controlled writes

### Dependency Risk

Mitigation:
- cargo audit checks
- dependency review

## Rust Security Principles

- minimize unsafe code
- validate inputs
- prefer typed structures
- use explicit error handling
