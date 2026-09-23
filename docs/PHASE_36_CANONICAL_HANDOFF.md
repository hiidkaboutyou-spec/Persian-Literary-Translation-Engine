# Phase 36 Canonical Handoff

Date: 2026-09-23

## Canonical state

Phase 36 — Versioned Cryptographic Blind-Review Evidence Binding — is canonical.

- PR: #128
- final validated head: `debc894d8d55a143f28fe2e8a24d5438427e3b3a`
- squash merge on `main`: `14ba7b0f82b9844ef99c6f7138c897ec4e2e3067`
- exact-head validation: all 19 pull-request-triggered workflows returned `success`, including Phase 36 Linux/macOS arm64, Rust CI, Security, Phase 31/32 provider-review regressions, Phase 22 Desktop Product, Trusted Release, Project Memory Tooling, and Phases 18–30 regression gates.

## What Phase 36 changed

New blind-review evidence is schema v2 and cryptographically binds the exact blind-bundle bytes with lowercase SHA-256 across the reveal key, independently initialized reviewer ledgers, dossier construction, and offline verification. Verification recomputes the supplied bundle digest and rejects mismatches or v2-to-v1 ledger downgrade. Legacy schema-v1 artifacts remain readable with explicitly weaker FNV/corpus/case semantics.

The implementation reuses the repository's existing RustCrypto `sha2` dependency. No new package source, provider, model, database, network integration, automatic literary winner, production admission, or private-manuscript transfer was added.

## Research/adoption decision

Fresh research reconfirmed that in-toto and GitHub artifact attestations use cryptographic subject digests and that Sigstore/Cosign can add signed identity/provenance layers. Those systems are not installed for reviewer evidence yet: hashing proves exact-byte integrity but not reviewer identity, and identity requires an explicit enrollment/trust-root/key-custody/recovery/revocation/privacy design. GitHub's artifact-attestation model is primarily build provenance and is not a substitute for human reviewer identity.

No new GitHub project was installed in this handoff. Adding a signing stack before defining its trust model would increase complexity without establishing the property we actually need.

## Next phase candidate — Phase 37 Reviewer Identity & Evidence Authenticity Threat Model

Do not begin implementation by installing a signing library. First define and test the trust model.

Required research questions:

1. Who is allowed to become a reviewer and how is enrollment recorded?
2. What identity is being authenticated: a local editor identity, GitHub/OIDC identity, or another explicitly approved identity?
3. Where are private signing keys or credentials stored, and how are loss, recovery, rotation, and revocation handled?
4. Must verification work fully offline for private manuscripts?
5. What metadata may leave the machine, especially with public transparency logs?
6. How are historical v1/v2 unsigned artifacts represented without falsely upgrading their trust?
7. What is the coordinated-rewrite attacker model after Phase 36's unkeyed SHA-256 chain?
8. What exact acceptance tests prove signature-to-artifact binding, wrong-reviewer rejection, revoked-key rejection, tampered-bundle rejection, and legacy compatibility?

Candidate implementations to evaluate only after the threat model exists:

- local Ed25519 signatures with project-owned enrollment/trust records;
- Sigstore/Cosign keyless identity if its OIDC, privacy, network, and transparency-log model is acceptable;
- in-toto/DSSE-compatible envelope semantics if interoperability is worth the additional format surface.

## Safety boundary for the next run

Keep production provider selection and manuscript transfer unchanged. Do not send review/manuscript text to external services. Do not claim cryptographic reviewer identity until enrollment, verification, revocation and migration tests exist. Prefer a small project-owned abstraction and existing dependencies over a new signing stack unless research demonstrates a concrete gap.
