# Phase 23 Research — Trusted Release & Supply-Chain Hardening

Date: 2026-09-18

## Goal

Turn the now-buildable desktop/product surface and existing CLI release process into a release chain whose dependency graph, software bill of materials, provenance, and signing boundaries are explicit and verifiable.

Phase 23 does **not** redefine literary-translation behavior. It hardens distribution and adds an optional, isolated product-identity design skill.

## Phase 22 canonical handoff

Phase 22 implementation PR #102 merged to `main` at:

- final reviewed PR head: `8229e5ce2ce0126ca567b4074b84434f4a260a2d`
- merge commit: `dc2bf1eee2f5d2dedc7c97d0164c3c26b8ac979b`

Final-head validation succeeded:

- Phase 22 Desktop Product: run `35335168206`
- Rust CI: run `35335167950`
- Security: run `35335168513`
- Phase 18: run `35335168705`
- Phase 19: run `35335168191`
- Phase 20: run `35335168289`
- Phase 21: run `35335168962`
- Project Memory Tooling: run `35335168359`

The desktop dependency graph is now committed at `desktop/src-tauri/Cargo.lock`, validated with `--locked`, audited, and used to build a real unsigned/ad-hoc macOS arm64 `.app`.

## Findings

### 1. Existing CLI release workflow had a lockfile integrity defect

The pre-Phase-23 `.github/workflows/release.yml` ran `cargo generate-lockfile` immediately before a `--locked` release build.

That defeats the intended release invariant: the release job may refresh compatible transitive dependencies and then build a dependency graph that differs from the one reviewed on `main`.

Phase 23 removes generation from release and instead verifies the committed `engine/Cargo.lock` using non-mutating `cargo metadata --locked`.

### 2. GitHub artifact attestations fit the repository's release model

GitHub artifact attestations use Sigstore-backed signed claims that connect an artifact to the repository, workflow, commit SHA, event, and build environment.

Official references:

- https://docs.github.com/en/actions/concepts/security/artifact-attestations
- https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations

For public repositories, attestations use the public Sigstore transparency infrastructure. GitHub's current documented workflow uses `actions/attest@v4` with `id-token: write` and `attestations: write`.

Decision:

- generate provenance attestations only for actual tagged release artifacts;
- do not attest ordinary CI test artifacts merely because they exist;
- keep SHA-256 checksums as a simple independent integrity surface;
- publish SBOMs alongside the release artifacts.

Attestation is evidence of provenance, not proof that an artifact is safe.

### 3. CycloneDX is the selected Rust SBOM path

Reviewed integration:

- repository: https://github.com/CycloneDX/cyclonedx-rust-cargo
- tool: `cargo-cyclonedx`
- selected version: `0.5.9`
- license: Apache-2.0
- release date: 2026-03-19

Why it is useful here:

- reads Cargo metadata plus Cargo.lock rather than only a lockfile;
- can target a particular Rust target triple;
- can describe binaries specifically;
- supports JSON CycloneDX output;
- version 0.5.9 supports `SOURCE_DATE_EPOCH`, allowing stable build timestamps and omission of a random serial number.

Official project reference:

- https://github.com/CycloneDX/cyclonedx-rust-cargo

Phase-23 use is CI/release tooling only. It is not a runtime dependency and is installed at the exact reviewed version with `--locked`.

### 4. macOS public distribution still needs real Apple identity

Apple documents Developer ID signing plus notarization as the direct-distribution path for Mac software downloaded outside the Mac App Store.

Official references:

- https://developer.apple.com/help/account/certificates/create-developer-id-certificates
- https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution
- https://developer.apple.com/documentation/notaryapi

Notarization scans a Developer-ID-signed submission and, when accepted, produces a ticket that Gatekeeper can use. Current notarization automation should use `notarytool` or Apple's Notary API; old `altool` submission is obsolete.

Decision:

- do not fabricate Apple signing identity;
- do not store Developer ID certificates/private keys/App Store Connect keys in Git, PMC, projectmem, frontend state, or project files;
- current CI may build an unsigned/ad-hoc validation app only;
- a public desktop release remains blocked until real external credentials are explicitly configured.

### 5. Tauri updater must remain disabled until its signing trust root exists

Official Tauri v2 updater documentation:

- https://v2.tauri.app/plugin/updater/

Tauri requires update signatures and does not permit disabling signature verification. Its public key belongs in application configuration; its private signing key must remain secret and is supplied at build time.

Decision:

- do not add `tauri-plugin-updater` yet;
- do not set `createUpdaterArtifacts: true` yet;
- do not configure an update endpoint yet;
- Phase-23 CI explicitly checks that updater activation has not happened prematurely;
- updater activation is a later credentialed release step after a signing key, recovery/storage policy, HTTPS endpoint, and update verification test exist.

## Optional product-identity skill: ip-as-logo

User-requested upstream:

- https://github.com/s1dashu/ip-as-logo-skill
- pinned commit: `acb834c717bcd0a487c49732d08397ba280d690b`
- reviewed SKILL.md blob: `391a7dc3214a34a4bc2c5e5c77ba40cef5709c19`
- reviewed LICENSE blob: `234cc2396ef9ed37c4ea1d8c6026ed2e3b6001f6`
- license: MIT

The reviewed repository is unusually low-risk for an Agent Skill because its functional payload is an instruction document rather than executable code. Its README states that the repository contains the skill document and showcase asset, with no scripts, style-runtime dependency, or generation dependency.

Decision:

- vendor only `SKILL.md` and its MIT notice under `tools/agent-skills/ip-as-logo/`;
- do not run `npx skills@latest add ...` because an installer adds unnecessary moving supply-chain surface for a text-only integration;
- verify the vendored files by exact Git blob IDs in Phase-23 CI;
- no automatic upstream syncing;
- use only for explicit mascot/product-identity/app-icon exploration;
- never make it a dependency of translation, literary review, persistence, publishing, or normal product runtime;
- generated images are candidates until a human explicitly chooses a final identity.

The skill itself recommends a top-tier image model and a three-direction/six-candidate workflow. That creative policy applies only when an explicit visual-brand task is being performed.

## Implemented Phase-23 controls

### Existing release workflow

`.github/workflows/release.yml` now:

- refuses lockfile mutation;
- verifies the committed engine graph with `cargo metadata --locked`;
- builds each release target with `--locked`;
- generates per-target CycloneDX JSON SBOMs using `cargo-cyclonedx 0.5.9`;
- publishes SBOMs alongside Linux/macOS/Windows CLI binaries;
- retains SHA-256 release checksums;
- creates GitHub provenance attestation for release assets;
- creates a signed SBOM attestation linking each binary to its corresponding SBOM;
- grants attestation/OIDC write permissions only in the tag-only publish job.

### Permanent Phase-23 validation

`.github/workflows/phase23-trusted-release.yml` validates:

- committed engine lockfile without mutation;
- absence of `cargo generate-lockfile` from release;
- presence of pinned CycloneDX and GitHub attestation steps;
- exact vendored ip-as-logo Git blobs and absence of executable files in its directory;
- credential-free CLI SBOM generation;
- committed desktop lockfile on Apple Silicon;
- updater remains disabled/fail-closed;
- desktop CycloneDX SBOM generation;
- real locked Tauri macOS arm64 app build;
- integrity manifest over the built app binary, desktop Cargo.lock, and desktop SBOM.

## Security boundary

Phase 23 explicitly separates three kinds of evidence:

1. **Checksum** — detects changed bytes.
2. **SBOM** — describes known build components/dependencies.
3. **Attestation** — links release artifacts to repository/workflow/build provenance.

None of these are a substitute for vulnerability review, human release approval, Apple notarization, or literary-quality approval.

## Exit criteria

Phase 23 can become canonical when:

- the vendored design skill passes exact provenance checks;
- release workflow never refreshes a committed lockfile;
- CLI SBOM generation works across the supported release target matrix;
- tagged release configuration contains provenance and SBOM attestation steps with narrowly scoped permissions;
- Phase-23 CLI SBOM smoke gate passes;
- Phase-23 Apple Silicon desktop SBOM + locked app build + integrity-manifest gate passes;
- existing Rust/Security/Phase 18–22/Project Memory gates remain green;
- no Apple/Tauri private signing credential is committed;
- updater remains disabled until a separate credentialed activation is tested;
- PR is reviewed/merged and exact final validation evidence is recorded.

## Deferred credentialed work

The following are intentionally **not** faked in this phase branch:

- Developer ID signing;
- Apple notarization/stapling;
- Tauri update private-key generation/storage;
- signed updater artifacts;
- production update endpoint;
- replacement of the existing app icon by an AI-generated candidate.

Those steps require real human-controlled credentials or a deliberate human visual selection.
