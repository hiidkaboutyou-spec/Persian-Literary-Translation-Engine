# Phase 35 — Reproducible Blind Review Dossier Verification

## Product outcome and boundary

Phase 34 checks that a dossier's counters are internally consistent. It cannot detect a plausible but altered preference count, changed reviewer judgment, or substituted reveal assignment. The next safe capability is an offline `literary-engine blind-review verify <blind-bundle.json> <reveal-key.json> <dossier.json> <ledger.json> [ledger2.json ...]` command that recomputes the exact Phase-32 dossier and compares the complete JSON value. The command accepts rights-safe research artifacts, emits only a short verification result, and writes no artifact. It does not call a provider or decide whether a provider may handle private books.

## Design and compatibility

- Validate the blind bundle and reveal key separately, require equal corpus ID and exact case-ID sets, and compare each review ledger's recorded fingerprint with the bytes of the *supplied* blind bundle.
- Reuse the existing Phase-32 `build_dossier` logic so every reviewer, judgment, tie, deferral, disagreement count, identity field, and non-admission flag is reconstructed. Compare complete JSON objects, ignoring harmless key order or whitespace while rejecting extra fields.
- Reject duplicate input paths, including aliases resolved through canonical paths. Require at least one ledger. Verification never changes or recreates the dossier.
- Preserve schema v1 and the existing `fnv1a64` fingerprint in saved review ledgers. This is a compatibility step, not a cryptographic authentication claim. A person who controls all supplied local files can fabricate a mutually consistent set. The Phase-31 reveal key does not bind the bundle's bytes and the legacy fingerprint is collision-prone. Reviewer identity also remains a self-declared label. Verification must never be treated as an independent human-identity attestation or provider authorization.
- Phase-33 admission assessment remains isolated from the normal CLI and is not automatically made eligible by this verification result. Integrating trust policy or owner authorization requires its own decision and evidence.

## GitHub research and tool decision — 2026-09-23

| Candidate | Observed capability | Decision for this stage |
| --- | --- | --- |
| [in-toto Attestation](https://github.com/in-toto/attestation) | Specifies subject digests and independently authenticated metadata. Its digest-set guidance recommends a cryptographically secure digest for immutable references. | Adapt the input-binding concept; no installer or signature/attestation framework until reviewer identity and trust roots are deliberately designed. A local checksum is not a signature. |
| [RustCrypto `sha2`](https://github.com/RustCrypto/hashes) | Pure Rust SHA-256, MIT OR Apache-2.0; versions already occur transitively in the existing lockfile. | Defer a schema-v2 cryptographic binding migration. Phase-31's exact-head guard requires no CLI dependency-manifest change, and changing only new ledger fingerprints would leave the reveal-key binding and old-artifact trust question unresolved. Research a coherent v2 migration before adoption. |
| [cargo-audit / RustSec](https://github.com/rustsec/rustsec) | Audits Rust dependency locks against advisories. | Existing Security CI is the authority; another local installation adds no measured gap to dossier verification. |
| [cargo-auditable](https://github.com/rust-secure-code/cargo-auditable) | Embeds a build's dependency list in binaries. | Useful for release inventory, but unrelated to human review evidence; no installation in this stage. |

The seven developer-oriented repositories requested earlier are separately reviewed in `docs/DEVELOPER_TOOL_CANDIDATES_2026-09-23.md`. No new GitHub package is needed for this native Rust change.

## Exit criteria and validation

1. A dossier created through `init`, `record`, and `dossier` verifies using the original four kinds of local input.
2. Changing a preference count, completed decision, reveal assignment, or even whitespace of the exact blind-bundle bytes fails verification; duplicate/aliased inputs and wrong case sets fail.
3. Existing Phase-31/32/33 behavior, no-production-selector guard, and macOS arm64 check remain green.
4. Rust format, Clippy, workspace tests/build, Security and all triggered exact-head workflows pass before merge. The local execution environment lacks Cargo, so CI is the executable evidence for these gates.

## Next frontier

If project owners need stronger authenticity, design a versioned Phase-31/32 evidence chain binding a cryptographic bundle digest into the reveal key, migration for existing ledgers, and an explicit reviewer-identity/trust mechanism. Obtain authoritative hosted-provider retention/training/rights terms and real human literary review before any separate owner authorization or provider activation.
