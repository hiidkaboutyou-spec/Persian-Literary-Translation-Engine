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

- Phases 1–37 are canonical on `main`.
- Phase 36 cryptographically binds exact blind-review bundle bytes to schema-v2 reveal keys and reviewer ledgers with SHA-256 while preserving legacy evidence compatibility.
- Phase 37 merged via PR #131 at canonical merge commit `396fb780fa225eedd0c3eb393ec6fd755add7b01` after the final exact head `c43ecc6fdd9f1aa4aaef187515a06274e1b0a8ad` passed all 20 observed pull-request workflows, including Phase 18, Phase 37, Rust CI, Security, Project Memory Tooling, and the affected Phase regression gates.
- Phase 37 adds optional offline OpenSSH SSHSIG authentication for exact completed schema-v2 reviewer-ledger bytes and deliberately authenticates reviewer ledgers only.
- The Phase-37 validation repair keeps sidecar stdout/stderr memory hard-bounded while draining oversized pipes to EOF, preserving fail-closed `OutputTooLarge` semantics without increasing the configured limit.
- The hidden reveal key's Candidate A/B → system mapping remains a separate, unauthenticated authority artifact. Phase 37 does not claim end-to-end authenticity for that mapping.
- No new application package dependency was required for Phase 37; OpenSSH remains an explicit local executable boundary.
- Private signing keys, reviewer trust roots, revocation files, manuscripts and real reviewer prose must remain outside Git/project memory.

## Next Priority

1. Phase 38: design reveal-authority provenance that authenticates the hidden Candidate A/B → system mapping without weakening blind review or exposing the mapping before reveal.
2. Require explicit domain separation, exact-byte/digest binding to the Phase-36 bundle/reveal evidence, replay resistance, signer authorization/revocation, and fail-closed verification; prefer the existing OpenSSH SSHSIG trust boundary unless research demonstrates a concrete need for another dependency.
3. Add rights-safe synthetic negative tests for tampered mapping, wrong authority, wrong bundle/review identity, revoked authority, legacy evidence, and pre-reveal information leakage before any production path can consume authenticated reveal evidence.
4. After the blind-review authority chain is trustworthy, return to authoritative hosted-provider data-handling evidence and representative real human English→Persian literary evaluation.
5. Keep production provider activation blocked until the separate governance evidence and explicit owner authorization requirements are satisfied.

## Constraints

Do not turn the project into a simple API wrapper. The value is the literary memory and workflow system.
