# Phase 38 Canonical Handoff

Date: 2026-09-26

## Canonical state

Phase 38 reveal-authority provenance merged via PR #134. Final validated PR head: `8e8fd081520ce004db6687020d5e7f40688af4b6`. Squash merge on `main`: `914781a103679dbb9964583668bbeabb0104f879`. All 32 observed exact-head checks completed successfully, including Phase 38 Linux and macOS arm64, Phase 36/37, Rust CI, Security, publication/release regressions and Desktop integrity on Apple Silicon. Post-merge main workflows were still running when this handoff was written; their result is separate evidence.

## Product and security change

The canonical `literary-engine blind-review sign-reveal-authority` and `verify-reveal-authority` commands authenticate the exact schema-v2 hidden reveal key against the exact blind bundle, project/review context and expected authority principal. Validation checks corpus/case identity and bundle SHA-256 before signing or verifying. The distinct OpenSSH SSHSIG namespace prevents reviewer-ledger signatures from being treated as reveal-authority signatures. Local allowed-signers and revocation material remain external to project files. No secret key, manuscript or real reviewer content was committed.

The standalone shell signer and separate Rust binary were removed. The canonical CLI reuses the existing OpenSSH process/trust helper. Synthetic tests cover altered mapping or bytes, cross-context replay, wrong principal/trust namespace, revoked key, malformed signature, symlink/path confusion, legacy reveal evidence and pre-reveal bundle leakage. Signatures establish provenance only; they grant no literary approval or production provider admission.

## Research and dependency decision

OpenSSH's upstream SSHSIG format and the repository's existing offline trust boundary supply the required primitive. Sigstore/Cosign, DSSE signing runtimes and extra cryptography crates were not installed. The open major dependency upgrades and Cycle 14 Persian publishing tools are separate compatibility and RTL work with independent gates.

## Next frontier

1. Check any post-merge `main` workflow failures against merge commit `914781a` before starting implementation.
2. Resume authoritative hosted-provider terms/data-handling evidence and representative real human English→Persian literary evaluation on rights-safe or locally held manuscripts; do not upload private book text to engineering records or CI.
3. Keep production provider activation blocked until governance evidence and explicit owner authorization are complete.
4. Cycle 14's editable Persian DOCX/EPUB/PDF publishing studio remains a separate roadmap track; evaluate its pinned tooling and RTL/export QA in focused PRs.
