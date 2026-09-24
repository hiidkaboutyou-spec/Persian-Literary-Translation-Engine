# Phase 38 — Reveal-Authority Provenance Security Contract

Status: design gate; no production behavior change.
Canonical base: `a6c9b69e9344d748e96a4c81dca9a889f7f7a562` (Phase 37 handoff merged).

## Problem

Phase 36 binds exact blind-review bundle/reveal/ledger evidence with SHA-256. Phase 37 authenticates exact completed reviewer-ledger bytes with offline OpenSSH SSHSIG. The remaining authenticity gap is the hidden Candidate A/B → system mapping in the reveal artifact: a valid reviewer signature does not prove who authorized that mapping.

Phase 38 must authenticate the reveal authority without exposing the mapping before the existing reveal point or weakening blind review.

## Security properties

1. **Blindness preservation.** No new public/pre-reveal artifact may contain Candidate A/B → provider/system identity, a reversible encoding of it, or metadata that materially reveals it.
2. **Exact evidence binding.** The signed authority statement must bind the exact reveal artifact bytes (or their already-defined SHA-256 digest) and the exact blind-review bundle identity/digest. A signature over a free-standing mapping is insufficient.
3. **Domain separation.** Reveal-authority signatures use a dedicated SSHSIG namespace distinct from Phase 37 reviewer-ledger signatures. Proposed namespace: `literary-reveal-authority-v1@persian-literary-translation-engine`.
4. **Context binding.** The signed statement includes schema/version, project/review identifier, bundle digest, reveal-artifact digest, and authority principal. Verification rejects missing, duplicate, mismatched, or unsupported context.
5. **Replay resistance.** Evidence from another project, review, bundle, reveal artifact, schema, or signature namespace must not validate.
6. **Authorization.** Verification uses an explicit project-local allowed-signers trust root and expected authority principal. A cryptographically valid but unauthorized key is failure.
7. **Revocation.** Verification accepts a project-local KRL/public-key revocation source and fails closed for revoked authority keys.
8. **Offline verification.** Verification requires no network, OIDC, transparency log, hosted key service, or public identity publication.
9. **No private-key custody in Git.** Private signing keys, real trust roots, revocation files, manuscripts, reviewer prose, and real hidden mappings remain outside the repository and project memory.
10. **No authenticity overclaim.** A valid authority signature authenticates provenance of the reveal evidence only; it does not imply translation quality, reviewer approval, provider governance approval, or production admission.

## Envelope and signing boundary

Use the existing OpenSSH SSHSIG executable boundary rather than adding a new cryptography runtime dependency.

The bytes passed to `ssh-keygen -Y sign` MUST be a deterministic, versioned authority statement. The statement MUST be assembled from already-validated evidence and MUST NOT depend on JSON canonicalization for signature validity. Prefer an explicit length-delimited or fixed-field textual/binary framing whose parser rejects duplicate fields and ambiguous encodings.

The SSHSIG namespace is part of verification and MUST be constrained in the allowed-signers policy. OpenSSH supports namespace restrictions, principal matching, validity windows, and a revocation file during `-Y verify`; Phase 38 should use these mechanisms rather than recreate key authorization logic.

The authority statement should contain, at minimum:

- protocol marker/version (`PLTE-REVEAL-AUTHORITY-V1`)
- project/review identifier
- blind-review bundle SHA-256
- reveal-artifact SHA-256
- reveal schema version
- authority principal

The hidden mapping itself need not be duplicated into the statement: binding the exact reveal-artifact digest authenticates its bytes while avoiding a second serialization of secret material.

## Why not install Sigstore/Cosign, DSSE, or another crypto crate now

OpenSSH SSHSIG already provides the required offline signing primitive, domain namespace, signer authorization and revocation boundary used by Phase 37. Sigstore's normal model introduces OIDC/Fulcio/Rekor/TUF/network/public-identity surfaces that are unnecessary for a private local reveal authority. DSSE/in-toto provides useful envelope semantics—authenticated payload type, domain separation, verification before parsing—but adding a second signing runtime would increase dependency and migration surface without solving a missing primitive.

Phase 38 therefore adopts the security semantics, not a new runtime dependency. Revisit this decision only if implementation proves SSHSIG cannot express a required property.

## Required negative/acceptance tests before merge

1. valid authorized authority + exact reveal/bundle context succeeds;
2. one-byte reveal mutation fails;
3. one-byte bundle mutation/digest substitution fails;
4. Candidate A/B mapping swap after signing fails;
5. signature copied from another project/review fails;
6. signature copied from another bundle fails;
7. reviewer-ledger SSHSIG namespace substituted for reveal-authority namespace fails;
8. valid signature from an unlisted/wrong authority fails;
9. revoked authority key fails;
10. wrong expected authority principal fails;
11. unsupported statement/schema version fails closed;
12. missing or duplicate context field fails closed;
13. malformed/truncated signature fails;
14. legacy unsigned reveal evidence remains explicitly distinguishable and must not be silently upgraded to authenticated;
15. pre-reveal bundle/export contains no authority statement field that discloses or enables inference of the hidden mapping;
16. verifier works with network unavailable;
17. private key path/material is never serialized into evidence, logs, project memory, or Git fixtures;
18. existing Phase 36/37 evidence remains byte-compatible unless an explicit version migration says otherwise.

## Implementation sequence

1. Inspect the canonical reveal-key/reviewer-ledger structs and Phase 37 SSHSIG helpers; reuse only stable boundaries.
2. Introduce a small versioned reveal-authority statement model and deterministic byte framing.
3. Add sign/verify commands using the existing bounded OpenSSH subprocess pattern and a distinct namespace.
4. Add expected-principal, allowed-signers and revocation inputs; keep them external to project state.
5. Verify exact bundle/reveal digests before accepting authority provenance.
6. Add all negative tests above with synthetic/right-safe fixtures.
7. Add a dedicated Phase 38 Linux + macOS arm64 gate and run Rust/Security/Phase 18/32/36/37 regression gates on the exact final head.
8. Merge only when all required exact-head gates are green; then record the canonical handoff.

## Dependency/install decision

No new dependency is approved at this design gate. Candidate GitHub packages may be researched, but installation requires a concrete missing capability, maintained upstream, license/security review, and proof that the existing OpenSSH boundary cannot meet the requirement.
