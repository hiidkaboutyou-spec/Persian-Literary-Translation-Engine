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
- The hidden reveal key's Candidate A/B → system mapping remains a separate, unauthenticated authority artifact on canonical `main`; Phase 37 does not claim end-to-end authenticity for that mapping.
- No new application package dependency was required for Phase 37; OpenSSH remains an explicit local executable boundary.
- Private signing keys, reviewer trust roots, revocation files, manuscripts and real reviewer prose must remain outside Git/project memory.

## Phase 38 In-Flight Checkpoint

- Phase 38 security/design gate #133 is canonical at `4804cb29fa901cb520698905cacc7920a95cd0a6`.
- Draft PR #134 (`phase-38-reveal-authority-provenance-implementation`) is based on current `main@57a2e22981b61080ed1699d4c090ed20bfe2f10d`; its reconciled executable checkpoint was `2119b4b12f4b70b72ddf99755606c8686f0f1a18` before this memory-only update.
- The dependency-free prototype signs a deterministic `PLTE-REVEAL-AUTHORITY-V1` statement under the dedicated `literary-reveal-authority-v1@persian-literary-translation-engine` SSHSIG namespace. The statement binds project id, review id, exact blind-bundle SHA-256, exact reveal-key SHA-256, and authority principal.
- The prototype rejects signature overwrite, symlinked evidence/trust inputs, invalid context atoms, cross-project/review replay, bundle/reveal mutation, wrong or unlisted authorities, namespace substitution, malformed or missing signatures, and revoked keys. Private signing keys, allowed-signers files and revocation material remain external.
- The dedicated Phase 38 workflow passed on Linux and macOS-15/Apple-Silicon for exact reconciled head `2119b4b12f4b70b72ddf99755606c8686f0f1a18`.
- Inspection of `engine/cli/src/provider_review_cmd.rs` confirms the canonical Rust blind-review surface already owns Phase-37 `sign-ledger`, `verify-ledger-signature`, and `verify-reviewer-authenticated`; Phase 38 must extend this authority surface rather than leave the shell prototype as a permanent parallel implementation.
- No Sigstore/Cosign, DSSE runtime, crypto crate, or unrelated Dependabot upgrade has been added to Phase 38. Existing OpenSSH SSHSIG primitives cover the required local signing, principal/namespace authorization, and revocation boundary.

## Next Priority

1. Integrate the proven reveal-authority framing into the canonical Rust `blind-review` command surface, reusing the existing bounded OpenSSH execution/trust boundary instead of creating a second permanent signing implementation.
2. Preserve exact project/review + bundle/reveal digest binding and the dedicated Phase-38 namespace; distinguish legacy unsigned reveal evidence explicitly rather than silently treating it as authenticated.
3. Add Rust-level negative coverage for schema/context substitution, cross-context replay, tampered reveal/bundle bytes, wrong/unlisted/revoked authority, malformed signature, symlink/path confusion, and pre-reveal leakage.
4. Run Phase 36, Phase 37, Phase 38, Rust CI, Security, Project Memory Tooling, and macOS arm64 gates on the exact final PR head. Do not promote or merge #134 unless the affected required gates are green.
5. After the blind-review authority chain is trustworthy, return to authoritative hosted-provider data-handling evidence and representative real human English→Persian literary evaluation.
6. Keep production provider activation blocked until the separate governance evidence and explicit owner authorization requirements are satisfied.

## Dependency / Open-PR Discipline

- Do not mix Phase 38 with the open major upgrades for `lopdf`, `zip`, `sha2`, `thiserror`, or GitHub Actions. Evaluate and land those separately with their own compatibility/security evidence.
- Historical superseded feature PRs must not be used as an alternate implementation path around the current canonical frontier.
- Never force-merge or force-update Phase 38; preserve exact-head CI evidence and traceable review records.

## Constraints

Do not turn the project into a simple API wrapper. The value is the literary memory and workflow system.
