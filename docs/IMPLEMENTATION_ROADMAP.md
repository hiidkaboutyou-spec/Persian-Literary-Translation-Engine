# Implementation Roadmap

This file is the current high-level delivery map for the Persian Literary Translation Engine. The older bootstrap roadmap is superseded by the production architecture that now exists on `main`.

## Product Goal

Build a production-grade English-to-Persian literary translation system that preserves author intent, narrative structure, character voice, relationships, emotional subtext, terminology, and long-novel continuity while keeping human review as the final authority.

## Non-Negotiable Design Rules

- Rust remains the core language and domain/runtime boundary.
- Python is allowed only behind narrow sidecar/tool boundaries when a mature capability is impractical to reproduce in Rust.
- Literary understanding, translation execution, memory, quality evaluation, review, persistence, and publishing remain separated by explicit contracts.
- Model inference never becomes canon automatically.
- External quality scores never equal human approval.
- Approved canon outranks unresolved inference in translation context.
- Long-running work is resumable and fingerprint-safe.
- External repositories are integrated selectively; do not vendor or replace native architecture wholesale.
- Secrets, credentials, manuscripts, generated translations, and private reviewer material are not committed to Git/developer-memory systems.
- A numbered phase is not canonical until its PR is merged and post-merge checks on `main` are verified.

## Delivered Foundation

The canonical `main` branch now includes:

- structured TXT/Markdown/DOCX/EPUB/text-PDF ingestion and source provenance;
- translation memory, glossary, character bible, relationship context, and Persian-aware normalization/retrieval;
- provider-neutral multi-pass translation runtime with bounded requests and resumable checkpoints;
- deterministic quality gates and cross-chapter consistency checks;
- manuscript intelligence and evidence-backed context seeding;
- human review ledger, conflict handling, preview/apply canon promotion, audit lineage, and atomic recovery;
- optional bounded model-assisted literary analysis whose findings remain review-controlled;
- Context Packet v2 and optional bounded BGE-M3 retrieval;
- Phase 19 literary fidelity/Persian-naturalness review and native monotonic alignment;
- project-oriented `ApplicationService` orchestration for Import -> Analyze -> Review -> Translate -> Review Translation -> Edit -> Export;
- CLI/JSON interfaces, release/security CI, large-book regressions, and Persian RTL DOCX output;
- controlled external integrations with pinned provenance and explicit failure boundaries.

## Verified Recent Phases

### Phase 17 — Controlled External Integrations — canonical

PR #90; merge `6a4b8807d8c54878f1f10db5cab1f1290fcc60fb`.

Delivered revision-pinned BookForge EPUB ingestion, explicit no-silent-fallback compatibility behavior, optional isolated COMET quality evidence, and documented licensing/privacy/failure boundaries.

### Phase 18 — Context Packet v2 & Selective Long-Novel Retrieval — canonical

PR #94; merge `2af408a19b8f69db93aff8e6896eaf189c4d69ae`.

Delivered typed Context Packet v2 with authority/provenance/budgets/fingerprints, one shared context assembly policy, deterministic lexical fallback, optional bounded BGE-M3/FastEmbed semantic ranking, timeout/failure fallback, resume invalidation, and Linux/Apple Silicon validation without default model downloads.

### Phase 19 — Literary Fidelity & Persian Naturalness Review Stack — canonical

PR #95; merge `d073dab10c0965197745a6cbc7b8e56c946835e8`.

Delivered:

- independent omission/addition, semantic fidelity, character voice, relationship/register, Persian naturalness, dialogue/subtext, and terminology/continuity review dimensions;
- explicit unevaluated state when evidence was not produced;
- provider-neutral bounded critic with validated evidence and non-auto-applied revision proposals;
- persisted, fingerprinted per-chapter post-translation review artifacts and staleness detection;
- native bounded monotonic Rust alignment supporting 1:1, 1:N, N:1, N:M and source/target gaps;
- optional BGE-M3 alignment adapter reusing the Phase 18 model boundary;
- permanent Linux/Apple Silicon review/alignment validation;
- post-merge verification on `main`.

Phase 19 dependency decisions remain in force: Hazm is blocked pending a patched compatible NLTK path; Vecalign/SentWeave remain design references while native Rust meets the requirement; DadmaTools remains conditional on a measured gap.

## Safe Supporting Tooling

Small supporting integrations may land between numbered phases only when they do not change runtime defaults, remain isolated/advisory where appropriate, and pass the same security/CI rules.

Current supporting tooling includes:

- optional `lingua-rs` English/Persian language diagnostics;
- checksum-pinned EPUBCheck tooling;
- optional projectmem developer-side coding memory;
- optional COMET and BGE sidecars behind explicit process boundaries.

OpenDataLoader PDF, ripwire, and Headroom remain future benchmark/developer candidates rather than runtime dependencies.

## Verified Current Capability

### Phase 20 — Publication-Grade EPUB Round Trip — canonical

PR #98; merge commit `1611cd731160c122baa68c9e80c1d4faeb7dfcfc`.

Goal: produce a translated Persian EPUB while preserving the source book's structure/assets deterministically and refusing unsafe structural guesses.

Delivered:

- stable BookForge block identity propagated through native source provenance and translated artifacts;
- explicit `BookForge block ID -> translated text` reconstruction mapping;
- fail-closed rejection of missing/unknown/duplicate/empty/mismatched publication block mappings;
- structural marker preservation requirements through translation/revision/review;
- source-aware BookForge rebuild instead of reconstructing an EPUB from flattened chapter text;
- preservation of source-derived XHTML structure, links, navigation, images, styles, and non-translated resources;
- BookForge target-language rewrite plus native Persian/RTL `dir="rtl"` and OPF `page-progression-direction="rtl"` metadata;
- deterministic publication ZIP behavior and source non-mutation contract;
- application/CLI `--export-format docx|epub` while keeping DOCX publishing intact;
- permanent read-only Phase 20 workflow with rights-safe generated EPUB fixture, EPUBCheck validation, repeated-export byte comparison, source checksum, preservation assertions, no-default-features compatibility, and Apple Silicon arm64 validation;
- supporting explicit `adult-intimacy` fidelity profile for confirmed-adult source material, with `literary` remaining the default and automated intimacy findings remaining non-canonical.

Standards decision:

- target **EPUB 3.3**, the current W3C Recommendation;
- use **EPUBCheck 5.3.0** as Phase 20's authoritative conformance gate because it checks EPUB 3.3;
- defer EPUB 3.4 / EPUBCheck 5.4.x as the authoritative target while EPUB 3.4 remains a Candidate Recommendation; treat 5.4.x as future-compatibility evidence, not an automatic migration.

Completion evidence:

- final PR head passed the dedicated Phase 20 publication workflow, including EPUBCheck 5.3.0 input/output validation and Apple Silicon arm64 coverage;
- representative CSS/image/link/inline markup preservation, Persian language/RTL metadata, repeated-export byte identity, and source checksum non-mutation were asserted;
- Rust CI and Security/cargo-audit were green on the final PR head;
- PR #98 merged to `main` at `1611cd731160c122baa68c9e80c1d4faeb7dfcfc`;
- the permanent Phase 20 workflow is retained and configured to run on `main` pushes.

Detailed research and completion record: `docs/PHASE_20_RESEARCH.md`.

## Forward Roadmap

### Phase 21 — Literary Evaluation Corpus & Benchmarking — canonical

PR #100; implementation merge commit `0e4b8ebf1bdb4dd7c931e3ba44cc64f23358d4b1`.

Goal: measure whether changes improve actual Persian literary translation quality rather than only passing unit tests.

Delivered:

- project-owned/rights-safe EN -> FA literary challenge corpus with eight focused cases;
- strict provenance schema that refuses committed corpora not explicitly marked rights-safe;
- deterministic terminology/voice/register/fidelity/subtext/continuity challenge anchors;
- contrastive degraded variants that CI must prove are detected in their declared failure dimension;
- native Rust `literary-evaluation-engine` and CLI benchmark runner;
- bounded human-review scorecards with evaluator expertise and 1–5 focused dimension ratings;
- reuse of optional COMET/XCOMET evidence;
- isolated pinned SacreBLEU 2.6.0 chrF2++ sidecar as reference-overlap evidence only;
- no Mizan/iPerUDT/Degarbayan/FarSSiM data vendored into the project.

Completion evidence:

- final PR head `d543b0448bdb8c886e58c69b719183daf07a66bc`;
- Phase 21 workflow run `35325302381`: success across native benchmark, optional reference metric, and Apple Silicon arm64 jobs;
- Rust CI `35325302426`, Security `35325302301`, Phase 18 `35325302478`, Phase 19 `35325302325`, Phase 20 `35325302400`, and Project Memory Tooling `35325302257`: success;
- committed corpus passed rights/provenance validation and contrastive-sanity checks;
- reference submission passed every deterministic anchor; deliberately degraded submission scored lower;
- SacreBLEU 2.6.0 install/protocol/pip-audit checks passed without project-driven external corpus downloads;
- PR #100 merged to `main` at `0e4b8ebf1bdb4dd7c931e3ba44cc64f23358d4b1`;
- permanent Phase 21 workflow is configured for future `main` pushes.

Detailed research and completion record: `docs/PHASE_21_RESEARCH.md`.

## Current Phase

### Phase 22 — Product Surface & Distribution Hardening — canonical

PR #102; final reviewed head `8229e5ce2ce0126ca567b4074b84434f4a260a2d`; merge `dc2bf1eee2f5d2dedc7c97d0164c3c26b8ac979b`.

Goal: expose the stable application layer through a usable product without moving domain logic into the UI.

Delivered:

- isolated Tauri 2.11 desktop shell outside the `engine/` workspace;
- locally bundled static frontend with no Node runtime/remote content;
- bounded `ApplicationService` IPC surface;
- project/import/analysis/review/canon/translation/manual-edit/literary-review/history/export/provider workflows;
- session-only provider credentials and Rust-side native dialogs;
- explicit provider-model configuration fix;
- committed independent desktop Cargo.lock and strict `--locked` validation;
- Apple Silicon app-bundle build, Rust audits, static UI security assertions and core regression validation.

Final Phase-22 run: `35335168206`; all Rust/Security/Phase 18–21/Project Memory companion gates were green on the same head.

### Phase 23 — Trusted Release & Supply-Chain Hardening — active branch/validation

Branch: `phase-23-trusted-release-supply-chain`.

Goal: make release artifacts traceable to an exact reviewed dependency graph and build workflow while preserving a fail-closed boundary around credentials.

Implemented:

- release no longer runs `cargo generate-lockfile`; committed engine graph is verified non-mutating with `cargo metadata --locked`;
- per-target CycloneDX JSON SBOMs generated by pinned `cargo-cyclonedx 0.5.9`;
- tagged CLI releases configured for GitHub provenance attestations and per-binary SBOM attestations;
- SHA-256 checksums remain published;
- permanent Phase-23 release-contract and Apple Silicon desktop-integrity workflow;
- desktop updater activation explicitly forbidden until signing key/trusted endpoint exist;
- exact-revision vendored `ip-as-logo` MIT Agent Skill for optional mascot/product-identity work only, with Git-blob provenance checks and no runtime dependency.

Exit criteria:

- release matrix generates its target SBOMs successfully;
- release workflow stays lockfile-nonmutating;
- Phase-23 CLI SBOM smoke test succeeds;
- Apple Silicon desktop locked build + SBOM + integrity manifest succeeds;
- vendored skill exact provenance checks succeed;
- existing Rust/Security/Phase 18–22/Project Memory gates remain green;
- no Apple Developer or Tauri updater private key enters Git/PMC/projectmem;
- updater remains disabled until separately credentialed/tested;
- final PR head is reviewed and merged, then exact completion evidence is recorded.

Developer ID signing, Apple notarization/stapling, and production updater activation are intentionally deferred until real human-controlled credentials exist. Do not simulate them.

Detailed research: `docs/PHASE_23_RESEARCH.md`.

## Next Action Rule

Always finish and verify the current numbered phase before starting the next numbered phase. Supporting tooling may be added only when it leaves runtime defaults intact and has an explicit owner/failure boundary. Do not treat branch-only work as merged. When a phase changes architecture, persistence, quality, or publishing contracts, update `IMPLEMENTATION_STATUS.md`, this roadmap, engineering decisions, external-integration notes, and project-memory notes together.
