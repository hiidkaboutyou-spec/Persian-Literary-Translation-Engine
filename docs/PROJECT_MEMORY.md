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

## Current Canonical Frontier

- Phases 1–36 are canonical on `main`.
- Phase 36 cryptographically binds exact blind-review bundle bytes to schema-v2 reveal keys and reviewer ledgers with SHA-256 while preserving legacy evidence compatibility.
- Phase 37 is in progress on PR #131 and adds optional offline OpenSSH SSHSIG authentication for exact completed reviewer-ledger bytes.
- Phase 37 deliberately authenticates reviewer ledgers only. The hidden reveal key's Candidate A/B → system mapping remains a separate, unauthenticated authority artifact.
- No new application package dependency is required for Phase 37; OpenSSH is an explicit local executable boundary.
- Private signing keys, reviewer trust roots, revocation files, manuscripts and real reviewer prose must remain outside Git/project memory.

## Next Priority

1. Finish exact-head Linux/macOS/security/regression validation for Phase 37 and merge only if every required gate is green.
2. Phase 38: design reveal-authority provenance that authenticates the hidden mapping without weakening blind review.
3. After the blind-review authority chain is trustworthy, return to authoritative hosted-provider data-handling evidence and representative real human English→Persian literary evaluation.
4. Keep production provider activation blocked until the separate governance evidence and explicit owner authorization requirements are satisfied.

## Constraints

Do not turn the project into a simple API wrapper. The value is the literary memory and workflow system.
