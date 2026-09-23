# Phase 36 — Versioned Cryptographic Blind-Review Evidence Binding

## Goal

Replace Phase 31/32's non-cryptographic FNV-only review binding with a versioned SHA-256 chain that binds the exact blind-bundle bytes to the reveal key, each new review ledger, and the reconstructed dossier, while preserving read compatibility for legacy schema-v1 artifacts.

This phase does **not** authenticate reviewer identity, authorize a provider, send a manuscript, or change production provider selection.

## Research

### in-toto Attestation Framework

The in-toto Attestation Framework models artifact claims around a subject digest and recommends cryptographic digests when immutability is required. Phase 36 adopts that narrow design principle: the exact blind-bundle bytes become the subject-like artifact and SHA-256 becomes the stable binding carried through the evidence chain.

Upstream: https://github.com/in-toto/attestation

### RustCrypto hashes / sha2

RustCrypto recommends SHA-2, SHA-3 or BLAKE3 for new hash applications. The repository already has `sha2 = "0.10"` as a direct `project-engine` dependency and `sha2 0.10.9` in the lockfile, so no new dependency or package source is introduced. Phase 36 exposes a tiny project-owned helper with a known-vector test and canonical lowercase validation.

Upstream: https://github.com/RustCrypto/hashes

### BLAKE3

BLAKE3 is fast and cryptographically appropriate, but this evidence format benefits more from SHA-256 interoperability and familiarity in attestation ecosystems than from hashing throughput; review artifacts are tiny. No BLAKE3 dependency is added.

Upstream: https://github.com/BLAKE3-team/BLAKE3

### Sigstore / Cosign

Cosign can sign and verify blobs and can bind short-lived signing certificates to OIDC identities, with transparency-log evidence. That is relevant to a future reviewer-identity/authentication phase, but adding Fulcio/Rekor/OIDC/network trust now would mix identity infrastructure with the narrower artifact-integrity migration. No Sigstore runtime or CI dependency is added in Phase 36.

Upstream: https://github.com/sigstore/cosign

### ed25519-dalek

Ed25519 signatures could support local reviewer keys, but a signature algorithm alone does not define reviewer enrollment, key custody, revocation, recovery, or trust roots. Adding it before those policies are designed would create an appearance of identity proof without a trustworthy identity lifecycle. Deferred.

Upstream: https://github.com/dalek-cryptography/curve25519-dalek/tree/main/ed25519-dalek

### cargo-crev

`cargo-crev` provides cryptographically verifiable dependency-review proofs. It is useful developer tooling, but adopting it requires an explicit trust graph and review workflow; it does not solve blind-review artifact binding. Existing Security/cargo-audit remains the enforced dependency-security gate for this phase.

Upstream: https://github.com/crev-dev/cargo-crev

## Versioned design

- Blind comparison bundle remains schema v1; it is the byte artifact being bound.
- Newly generated reveal keys are schema v2 and contain `bundle_sha256` over the exact pretty-serialized bundle bytes written to disk.
- Newly initialized blind-review ledgers are schema v2 and independently compute `bundle_sha256` from the exact bundle bytes supplied to the reviewer.
- A schema-v2 reveal key requires schema-v2 ledgers whose SHA-256 exactly matches the key.
- The dossier becomes schema v2 only when the reveal key is schema v2; it carries the bound SHA-256 and an explicit v2 reveal-binding description.
- Legacy schema-v1 reveal keys and ledgers remain readable. Legacy dossiers remain schema v1 and keep the prior FNV/corpus/case trust semantics.
- A schema-v2 key may never downgrade to a legacy ledger.
- Phase 35 verification recomputes SHA-256 from the supplied bundle bytes and checks the key and every schema-v2 ledger before reconstructing and comparing the dossier.

The existing FNV fingerprint is retained in v2 ledgers/dossiers only for backward compatibility and continuity diagnostics; it is not treated as cryptographic evidence.

### Security limit

SHA-256 here is an unkeyed integrity binding, not an authenticity proof. It detects drift or substitution when at least one bound artifact/digest is independently trusted, but an actor able to rewrite the bundle, reveal key, all ledgers, and dossier together could recompute every digest. Phase 36 therefore must not claim reviewer identity, non-repudiation, trusted timestamping, or resistance to coordinated artifact replacement. Those properties require a separately designed signature/trust-root or append-only evidence model.

## Dependency decision

No new package is installed. The best-fit GitHub library, RustCrypto `sha2`, is already directly present in `project-engine` and revision-resolved in `Cargo.lock`. Reusing it avoids a new supply-chain edge while still replacing the measured integrity gap.

## Exit criteria

1. SHA-256 helper matches a published known vector and rejects non-canonical digest strings.
2. `blind-compare` writes a schema-v2 reveal key bound to the exact bytes of its bundle.
3. `blind-review init` writes a schema-v2 ledger with the independently computed SHA-256.
4. Dossier construction rejects schema-v2 key/ledger digest mismatch and rejects v2-to-v1 downgrade.
5. `blind-review verify` detects bundle-byte changes through SHA-256 before accepting a v2 evidence chain.
6. Legacy v1 review artifacts remain readable with their old, explicitly weaker trust semantics.
7. Rustfmt, Clippy, workspace tests, Security, Phase 31/32 regression gates, and Apple Silicon validation pass at the exact PR head before merge.
8. Production provider selection, private-manuscript handling, automatic winner selection, and provider authorization remain unchanged.

## Next frontier

Reviewer identity/authentication should be designed separately. Before any signature system is adopted, define reviewer enrollment, trust roots, key custody/recovery/revocation, offline vs hosted verification, privacy, and migration semantics. Sigstore/Cosign or a local Ed25519 design can then be evaluated against that explicit threat model instead of being installed speculatively.
