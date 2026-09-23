# Phase 37 — Authenticated Reviewer Evidence with Offline SSHSIG

Date: 2026-09-23

## Product outcome

Phase 36 cryptographically binds the exact blind-comparison bundle bytes to schema-v2 reveal keys, review ledgers, dossiers and offline reconstruction, but SHA-256 alone cannot prove who authored a ledger. Phase 37 adds an **optional, local, offline reviewer-authentication layer** for completed schema-v2 blind-review ledgers.

The outcome is deliberately narrow:

- prove that the exact completed ledger bytes were signed by a key that the verifier's independent trust file authorizes for the ledger's reviewer principal;
- preserve the existing Phase-36 exact-byte evidence chain and dossier reconstruction;
- support explicit revocation;
- keep private signing keys, reviewer trust roots and revocation state outside project persistence;
- never turn a reviewer signature into provider admission, literary approval, or permission to send a private manuscript.

## Threat model

### Properties Phase 37 is intended to add

Phase 37 addresses the Phase-36 coordinated-rewrite gap **when the verifier independently protects the trust root**.

An attacker who changes any signed ledger byte, including JSON whitespace, reviewer notes or a recorded decision, cannot keep the existing signature valid. An attacker who substitutes another signing key cannot pass verification unless that key is independently authorized for the same principal in the verifier-controlled allowed-signers file.

The authenticated end-to-end command also retains Phase 36 checks for:

- exact blind-bundle SHA-256;
- schema-v2 key/ledger binding;
- bundle/key case identity;
- exact dossier reconstruction;
- completed ledgers;
- duplicate reviewer rejection;
- no automatic winner and no production admission.

### Properties Phase 37 does not claim

- Real-world identity proof. A reviewer principal means only “the verifier enrolled this key for this principal.”
- Trusted timestamping or non-repudiation.
- Protection if an attacker can also replace the verifier's allowed-signers trust root and revocation file.
- Hosted identity recovery.
- Provider privacy/retention safety.
- Production provider authorization.
- Human literary approval beyond the judgments explicitly present in the signed ledger.

## Selected authenticity boundary: OpenSSH SSHSIG

Primary upstream references:

- OpenSSH portable: https://github.com/openssh/openssh-portable
- ssh-keygen manual: https://man.openbsd.org/ssh-keygen
- SSHSIG protocol: https://github.com/openssh/openssh-portable/blob/master/PROTOCOL.sshsig

OpenSSH provides the exact primitives required by the threat model without adding a Rust cryptography dependency:

- `ssh-keygen -Y sign` signs data from standard input and emits a detached SSH signature;
- `ssh-keygen -Y verify` verifies data from standard input against a verifier-controlled allowed-signers file;
- `-I` supplies the reviewer principal;
- `-n` supplies a mandatory signature namespace for domain separation;
- `-r` accepts a Key Revocation List or one-public-key-per-line revocation file;
- the signing key may be a private key file or a public key whose private half is available through `ssh-agent`.

Phase 37 uses the fixed project namespace:

```text
blind-review@persian-literary-translation-engine
```

The engine invokes `ssh-keygen` directly without a shell. Ledger bytes are streamed locally through stdin. No reviewer ledger or manuscript text is sent to a network service.

## Reviewer enrollment and trust ownership

Reviewer enrollment is explicitly a **human/operator trust decision**, not a project-generated identity.

The verifier maintains an external OpenSSH allowed-signers file whose principal maps to an approved public key. The authenticated CLI takes that file as an explicit input. The engine does not:

- generate reviewer private keys;
- copy private keys into the project;
- write allowed-signers entries;
- infer trust from GitHub usernames, email addresses or provider accounts;
- commit trust files or signatures;
- contact a certificate authority, OIDC issuer or transparency log.

Authenticated reviewer principals use a deliberately narrow token grammar: 1–128 ASCII bytes containing letters, digits, `.`, `_`, `@`, `+`, `-` or `:`. This keeps the principal unambiguous at the local trust boundary. Existing unsigned review ledgers keep their historical reviewer strings; the restriction applies only when claiming authenticated evidence.

## Key custody, rotation, recovery and revocation

Private key custody is outside the engine. A reviewer may use an existing OpenSSH private key or an agent-backed key. The engine receives only the key path/reference supplied to `ssh-keygen`; it never serializes private key material into project state.

Rotation is performed by changing the verifier-owned allowed-signers trust policy. Revocation is enforced through an optional explicit `--revocations` file accepted by both signature-only and full authenticated verification.

Phase 37 intentionally does not invent key recovery. Loss of a private key is handled by enrolling a replacement key through the human-controlled trust process. Historical trust semantics depend on the verifier's retained trust/revocation policy; this phase does not claim trusted timestamps or automatic archival PKI.

## Exact-byte signing contract

A reviewer ledger may be signed only when:

1. it is schema v2;
2. its `bundle_sha256` is structurally valid;
3. every case is completed;
4. every completed judgment has its required reason;
5. the reviewer principal satisfies the authenticated-principal grammar.

The signature covers the **exact bytes read from the completed ledger file**. Phase 37 never reserializes the ledger before signing.

This matters because a signature over a reconstructed object could accidentally authenticate a representation different from the evidence file. Exact-byte signing ensures that any byte-level mutation invalidates the signature.

The full authenticated verification path reads each ledger once, uses those same in-memory bytes for Phase-36 evidence checks and SSHSIG verification, and therefore avoids a sign/verify time-of-check/time-of-use gap between separate ledger reads.

## CLI surface

```text
literary-engine blind-review sign-ledger <ledger.json> <signature.sig> --key <ssh-key>

literary-engine blind-review verify-ledger-signature \
  <ledger.json> <signature.sig> \
  --allowed-signers <allowed_signers> \
  [--revocations <krl-or-revoked-keys>]

literary-engine blind-review verify-authenticated \
  <blind-bundle.json> <reveal-key.json> <dossier.json> \
  <ledger.json> <signature.sig> [ledger2.json signature2.sig ...] \
  --allowed-signers <allowed_signers> \
  [--revocations <krl-or-revoked-keys>]
```

The existing `blind-review verify` remains unchanged in meaning and continues to support legacy evidence. `verify-authenticated` intentionally requires a schema-v2 reveal key and schema-v2 ledgers; legacy v1 evidence is never silently upgraded to authenticated status.

Signature creation refuses to overwrite an existing signature path so a previous evidence artifact is not silently replaced.

## Alternatives reviewed

### ed25519-dalek 3.x — deferred

`ed25519-dalek` is a mature Rust Ed25519 implementation, but adding a signature primitive would also add a new cryptographic dependency surface while leaving enrollment, private-key storage, agent/hardware-key support and revocation for this project to design and maintain.

The measured requirement is local reviewer evidence, not a custom project PKI. OpenSSH already supplies the needed signing, external custody, verifier trust-file and revocation boundaries.

Upstream: https://github.com/dalek-cryptography/curve25519-dalek/tree/main/ed25519-dalek

### Sigstore/Cosign — deferred for reviewer evidence

Sigstore is appropriate for software-artifact provenance and keyless OIDC identities, and this repository already uses GitHub artifact attestations for release provenance. For private human review, Fulcio/OIDC/Rekor would introduce hosted identity, network and public-transparency-log semantics that are not required by the current local/offline threat model.

No Cosign, Fulcio, Rekor or OIDC dependency is introduced by Phase 37.

Upstream: https://github.com/sigstore/cosign

### DSSE / in-toto — design reference only

DSSE's explicit payload type/domain-separation model and in-toto's digest-bound subjects remain useful design references. They deliberately do not solve key ownership or PKI by themselves. OpenSSH SSHSIG already supplies a namespaced detached-signature envelope for this local use case, so another envelope dependency would duplicate surface area without closing an additional measured gap.

Upstreams:

- https://github.com/secure-systems-lab/dsse
- https://github.com/in-toto/attestation

## Dependency and privacy decision

**No Cargo/Python/npm dependency is added.**

OpenSSH is an optional local executable boundary used only by the explicit authentication commands and the Phase-37 dedicated CI gate. Normal translation, review recording, dossier creation, publishing, desktop use and legacy verification do not require it.

If `ssh-keygen` is unavailable, authentication commands fail with an actionable error. They never silently downgrade to unsigned verification.

No private manuscript, translation, reviewer prose, key or trust record is sent to an external service by this implementation.

## Validation matrix

Permanent Phase-37 CI must prove on Linux and Apple Silicon macOS:

1. OpenSSH `ssh-keygen` is available.
2. Rust formatting succeeds.
3. All provider-review unit/regression tests succeed.
4. A real temporary Ed25519 test key can sign exact schema-v2 ledger bytes.
5. The matching allowed-signers principal verifies.
6. Full authenticated dossier verification succeeds using the signed bytes.
7. A one-byte/whitespace ledger change invalidates the signature.
8. A key enrolled under the wrong principal is rejected.
9. A KRL-revoked key is rejected.
10. Legacy schema-v1 ledgers are rejected by authenticated verification.
11. Unsafe reviewer-principal strings are rejected.
12. Clippy succeeds for the CLI target.
13. Existing non-admission guards remain present.
14. Normal project CI/security and Phase-31/32/33/35/36 regressions remain green on the exact PR head before merge.

## Rollback and compatibility

The feature is additive:

- no persisted schema is rewritten;
- no provider selector changes;
- no production admission changes;
- no default runtime dependency changes;
- existing unsigned v1/v2 artifacts remain readable by their existing commands;
- removing the three Phase-37 authentication commands would leave all Phase-36 evidence intact.

## Exit criteria

Phase 37 is canonical only when:

- implementation, threat model and CLI semantics agree;
- real SSHSIG sign/verify/revocation tests pass on Linux and Apple Silicon;
- required exact-head repository CI/security/regression gates are green;
- no private signing key or allowed-signers trust file is committed;
- the PR records the exact final head and validation evidence;
- merge occurs without bypassing a required failure.

## Next frontier after Phase 37

After reviewer authenticity is canonical, the next provider-admission work must return to the unresolved product evidence rather than adding more cryptography: obtain authoritative hosted-provider data-handling terms, collect representative real human English→Persian literary evidence under the now-authenticated local review process, and require a separate explicit owner authorization before any private-book production activation.
