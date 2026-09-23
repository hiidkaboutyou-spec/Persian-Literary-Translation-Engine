# Phase 37 — Reviewer Identity & Evidence Authenticity Threat Model

Date: 2026-09-23
Status: research/design gate; no signing implementation yet

## Why this phase exists

Phase 36 binds the exact blind-review bundle bytes to evidence with SHA-256. That proves integrity against accidental or uncoordinated mutation, but an attacker able to rewrite the bundle and all unsigned evidence can recompute every digest. Phase 37 defines the missing authenticity boundary before any signing library is admitted.

## Security property

For new signed review evidence, offline verification must establish all of the following without sending manuscript or review text to a network service:

1. the exact bundle bytes are the bytes that were reviewed;
2. the evidence was signed by a reviewer key that was enrolled for this project at signing time;
3. the signature covers an explicit payload type plus the evidence payload, not an ambiguous raw JSON serialization;
4. a different reviewer key cannot satisfy the claimed reviewer identity;
5. revoked or superseded keys are rejected according to an explicit historical policy;
6. legacy v1/v2 unsigned artifacts remain readable but are never presented as signed/authenticated;
7. verification is fail-closed for malformed envelopes, unknown schema versions, unknown keys, signature failure, bundle digest mismatch, or identity mismatch.

A valid signature does not prove that a review is correct, unbiased, or high quality. Human-review policy remains separate from cryptographic authenticity.

## Threat model

### In scope

- coordinated rewrite of blind bundle plus unsigned SHA-256 evidence;
- evidence copied from another project/case/reviewer;
- claimed reviewer identity changed after signing;
- signature substitution or key-id spoofing;
- key rotation and revocation;
- malformed/canonicalization-confused payloads;
- downgrade from signed evidence to legacy unsigned evidence;
- verification on an offline machine.

### Out of scope for this phase

- a fully compromised reviewer endpoint while its private key is unlocked;
- coercion or malicious reviewer judgment;
- compromise of the operating system trust boundary;
- public non-repudiation through an external transparency service;
- automatic winner selection or production admission.

## Identity and enrollment model

Phase 37 should use a **project-local reviewer identity**, not GitHub/OIDC identity, as the authority for private literary review.

Each project owns an append-only enrollment/trust record containing at minimum:

- stable opaque `reviewer_id` (not email/name);
- public verification key;
- deterministic key fingerprint derived from the public key;
- algorithm identifier fixed by schema, not chosen by untrusted evidence;
- enrollment sequence/time metadata already available locally;
- status: active, revoked, superseded;
- optional replacement-key fingerprint;
- revocation sequence/time and local reason code.

Human-readable names are UI metadata only and must never be the cryptographic identity key.

Enrollment/revocation records are project authority. Importing a public key does not silently enroll it.

## Key custody boundary

Private signing keys must not be committed to Git, stored in project manifests, embedded in review bundles, logged, uploaded to providers, or copied into publication packages.

Implementation work must define an OS-local secret-storage adapter before persistent private keys are enabled. On macOS the preferred production boundary is Keychain-backed custody. Test code may use deterministic in-memory keys that never ship as production credentials.

There is no automatic private-key recovery. Loss means enroll a replacement key and supersede/revoke the old key. Rotation must preserve historical verification metadata without making the old key active for new signatures.

## Envelope semantics

Do not sign ad-hoc serialized JSON bytes whose meaning depends on canonicalization. Use a project-owned, versioned envelope with DSSE-style pre-authentication encoding semantics:

- fixed `payload_type` identifying the exact Phase 37 evidence schema;
- payload bytes encoded without semantic reserialization during verification;
- signature over a domain-separated encoding of payload type + payload bytes;
- `key_id` treated only as a lookup hint; authority comes from the enrolled public key that actually verifies;
- reviewer/project/case identifiers and Phase-36 bundle SHA-256 included inside the signed payload.

This follows the useful DSSE property that verifiers can authenticate payload bytes before interpreting them and avoids making JSON canonicalization part of the security boundary.

## Revocation policy

Revocation semantics must be explicit rather than pretending an offline verifier knows global wall-clock truth.

For the first implementation:

- active enrolled key: may sign and verify new evidence;
- superseded/revoked key: cannot create acceptable new evidence;
- historical evidence signed before a locally recorded revocation may remain cryptographically valid but must be reported as `historical_revoked_key`, not equivalent to active-key evidence;
- evidence that claims to post-date local revocation is rejected;
- if trustworthy signing-time ordering cannot be established from local project sequence data, verification reports the ambiguity and fails production admission.

No external timestamp or transparency log is required in the private/offline baseline.

## Legacy migration

Schema v1/v2 evidence remains readable under its existing guarantees. Migration must never synthesize a signature over historical unsigned evidence or label it authenticated. A later reviewer may explicitly re-attest a legacy artifact, producing a new signed record that references the legacy digest and clearly records that the signature time is later than the original review.

## External-tool research and adoption decision

### Sigstore/Cosign — defer

Sigstore keyless signing binds ephemeral signing keys to OIDC identities and normally uses Fulcio/Rekor/TUF trust material. This is excellent for public software provenance, but the default identity/transparency/network model is a poor fit for private manuscript review and strict offline verification. Cosign can verify local keys/bundles, but adopting the full stack now would add a large trust/dependency surface without solving project-local reviewer enrollment.

Decision: do not install Sigstore/Cosign in Phase 37 baseline.

### in-toto / DSSE — adopt semantics, not package

The in-toto envelope specification recommends DSSE, authenticated payload types, multiple signatures, and treating key IDs as hints. Those semantics directly address payload-confusion/canonicalization risks.

Decision: adopt the minimal DSSE-style envelope semantics in a small project-owned format; do not add an in-toto runtime dependency solely for serialization.

### Ed25519 implementation — evaluate after interface/tests

Ed25519 is a strong candidate for the local signature primitive. The repository should prefer a mature Rust implementation and avoid custom cryptography. Before adding `ed25519-dalek` or another crate, first land the trust/envelope interfaces and negative acceptance tests; then add exactly one implementation dependency if the existing lockfile does not already provide an adequate signing primitive.

## Required acceptance tests before claiming reviewer authenticity

1. exact evidence verifies with the enrolled reviewer public key;
2. one-byte bundle mutation fails because the signed payload binds the Phase-36 SHA-256;
3. one-byte signed-payload mutation fails;
4. signature mutation fails;
5. wrong enrolled reviewer key fails;
6. claimed `reviewer_id` substitution fails;
7. `key_id` spoofing cannot override the key that actually verifies;
8. cross-project and cross-case replay fail;
9. unknown/un-enrolled key fails;
10. revoked key cannot sign acceptable new evidence;
11. historical revoked-key evidence receives a distinct non-active status;
12. malformed/unknown envelope or payload schema fails closed;
13. v1/v2 legacy evidence remains readable but explicitly unsigned;
14. verification succeeds with networking unavailable;
15. no manuscript/review plaintext, private key, or identifying reviewer metadata is emitted to logs/network by the signing/verification path.

## Implementation sequence

1. Add project-local reviewer enrollment/trust-record model and state transitions.
2. Add versioned signed-evidence payload and DSSE-style envelope abstraction with no crypto implementation hidden inside domain models.
3. Add verifier policy/result types that distinguish integrity, authenticity, revocation, and legacy states.
4. Add the negative tests above using a test signer abstraction.
5. Evaluate the existing dependency graph; if no suitable primitive exists, add one mature Ed25519 implementation with locked dependency review.
6. Add macOS Keychain-backed production custody separately from pure verification logic.
7. Only after Linux + Apple Silicon gates pass may Phase 37 claim cryptographic reviewer authenticity.

## Safety boundary

No manuscript/review text may be sent to GitHub, Sigstore, OIDC, Rekor, a KMS, or any provider as part of this phase. No new signature library is installed merely because it is popular. No production admission rule changes until enrollment, revocation, offline verification, downgrade resistance, and negative tests are implemented and green.