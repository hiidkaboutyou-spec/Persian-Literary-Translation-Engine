# Implementation Status

## Vision

A production-grade English-to-Persian literary translation engine with a Rust core that preserves author intent, character voice, relationship/register, terminology, emotional subtext, continuity, and natural Persian while keeping human review as the final authority.

## Canonical Main State

`main` is verified through Phase 21.

Phase 21 implementation merged through PR #100 at:

```text
0e4b8ebf1bdb4dd7c931e3ba44cc64f23358d4b1
```

The final reviewed Phase 21 head `d543b0448bdb8c886e58c69b719183daf07a66bc` passed the dedicated Phase 21 benchmark gate, Rust CI, Security/cargo-audit, Phase 18 context/retrieval, Phase 19 literary review, Phase 20 publication regression, Project Memory Tooling, and Apple Silicon arm64 benchmark checks before merge. The permanent Phase 21 workflow also runs on pushes to `main`.

### Core crates and application boundaries

- **translation-core** — provider-neutral translate -> revise -> quality pipeline with deterministic EchoProvider and production OpenAIProvider, bounded oversized-passage handling, immutable structural-marker guidance, and explicit translation style profiles.
- **document-engine** — TXT/Markdown/DOCX/EPUB/text-PDF ingestion, structured `Manuscript -> Book -> Chapter -> Scene -> Paragraph`, provenance, parser registry, Unicode-safe segmentation, Persian RTL DOCX export, and strict revision-pinned BookForge EPUB ingestion.
- **memory-engine** — durable translation memory/glossary, deterministic lexical/polarity/diversity retrieval, Context Packet v2 types/provenance/fingerprints, and optional semantic candidate fusion boundaries.
- **character-engine** — character profiles, aliases, relationships, word-boundary-aware relevance, and canonical JSON persistence.
- **quality-engine** — deterministic blocking checks plus optional advisory COMET and English/Persian Lingua diagnostics; probabilistic evidence never equals approval.
- **literary-intelligence-engine** — deterministic manuscript analysis, chapter maps, continuity hooks, entity/terminology seeds, observed literary metrics, evidence-backed initialization proposals, and shared Context Packet v2 assembly.
- **advanced-literary-analysis** — optional bounded provider-assisted literary findings with cache/resume, structured validation, stable evidence, and review-only output.
- **literary-review-engine** — Phase 19 post-translation review evidence for omission/addition, semantic fidelity, character voice, relationship/register, Persian naturalness, dialogue/subtext, terminology/continuity, plus profile-specific dimensions such as intimacy fidelity; automated review never auto-applies or human-approves revisions.
- **literary-evaluation-engine** — Phase 21 rights-safe literary benchmark schemas, deterministic challenge anchors, contrastive regression evidence, bounded human scorecards, and optional reference-metric sidecar contracts; benchmark evidence never becomes literary approval.
- **human-review-workflow** — versioned review ledger, stable IDs, lifecycle validation, typed conflicts, promotion plans, audit lineage, and explicit human decisions.
- **project-engine** — manifests, atomic persistence/recovery, application orchestration, translation lifecycle, literary-review/canon integration, checkpoint/fingerprint safety, manual revisions, export, history, and UI-ready snapshots.
- **literary-reference-knowledge** — editorial/reference sources and validation rules.
- **text-normalization** — shared Persian/Arabic normalization, matching, negation, and similarity helpers.

### Phase 17 — Controlled External Integrations — canonical

PR #90; merge commit `6a4b8807d8c54878f1f10db5cab1f1290fcc60fb`.

Revision-pinned BookForge EPUB parsing and optional isolated COMET evidence became approved boundaries. Safe supporting tooling added afterward includes optional Lingua diagnostics, checksum-pinned EPUBCheck tooling, and isolated projectmem developer memory.

### Phase 18 — Context Packet v2 & Selective Long-Novel Retrieval — canonical

PR #94; merge commit `2af408a19b8f69db93aff8e6896eaf189c4d69ae`.

Delivered typed/budgeted Context Packet v2, explicit authority/provenance/fingerprints, deterministic fallback retrieval, optional bounded BGE-M3/FastEmbed semantic ranking, timeout/failure fallback, context-aware resume invalidation, and Linux/Apple Silicon validation without model downloads in normal CI.

### Phase 19 — Literary Fidelity & Persian Naturalness Review — canonical

PR #95; merge commit `d073dab10c0965197745a6cbc7b8e56c946835e8`.

Delivered:

- `literary-review-engine` with independent evidence dimensions for omission/addition, semantic fidelity, character voice, relationship/register, Persian naturalness, dialogue/subtext, and terminology/continuity;
- explicit unevaluated state instead of treating unavailable evidence as a pass;
- provider-neutral bounded critic contracts with paragraph-index/evidence validation and revision proposals that never auto-apply;
- native bounded monotonic Rust alignment covering 1:1, 1:N, N:1, N:M and source/target gaps;
- optional BGE-M3 alignment adapter reusing the approved Phase 18 model boundary;
- strict alignment response validation and deterministic behavior when optional tools are absent/failing;
- persisted per-chapter literary-review artifacts with source/translation/context fingerprints and manual-edit staleness detection;
- `ApplicationService` plus CLI `project review-translation` boundary;
- permanent Linux/Apple Silicon Phase 19 validation.

Dependency decisions remain: Hazm blocked while its mandatory NLTK dependency is affected by the recorded unpatched advisory; Vecalign/SentWeave remain references while native Rust satisfies the measured need; DadmaTools remains conditional on a demonstrated Persian NLP gap.

### Phase 20 — Publication-Grade EPUB Round Trip — canonical

PR #98; merge commit `1611cd731160c122baa68c9e80c1d4faeb7dfcfc`.

Delivered:

- native `SourceLocation.block_id` provenance for structured source-format block identity;
- EPUB paragraph/heading provenance carried into translated chapter artifacts;
- explicit `BookForge block ID -> translated text` reconstruction instead of positional guessing;
- fail-closed rejection of unknown, duplicate, missing, empty, or mismatched EPUB block translations;
- BookForge marker-aware structural reconstruction and validation using pinned revision `23f8c9d3c97a06f48e13424698441bfb4b037844`;
- source-aware preservation of XHTML structure and non-translated resources instead of regenerating the book from plain translated chapter text;
- target language rewriting through BookForge plus native `dir="rtl"` and OPF `page-progression-direction="rtl"` for Persian/other RTL targets;
- deterministic publication ZIP output and source non-mutation contract;
- application/CLI `project export ... --export-format docx|epub` while the existing DOCX path remains intact;
- application capability declaration for DOCX and EPUB exports;
- `literary` default translation style plus explicit opt-in `adult-intimacy` fidelity profile with mandatory caller confirmation that every participant in sexual content is an adult; automated intimacy-fidelity evidence remains non-canonical;
- permanent read-only `Phase 20 EPUB Round Trip` workflow with a generated rights-safe EPUB 3.3 fixture, EPUBCheck 5.3.0 before/after validation, source checksum verification, repeated-export byte comparison, representative CSS/image/link/inline-markup preservation checks, Linux validation, no-default-features document compatibility, and Apple Silicon arm64 compile/tests.

### Phase 20 standards/dependency decisions

- **EPUB 3.3** is the publication target because it is the current W3C Recommendation.
- **EPUB 3.4** remains deferred while it is a Candidate Recommendation; do not silently redefine the product standard.
- **EPUBCheck 5.3.0** remains the authoritative Phase 20 conformance gate because it validates EPUB 3.3.
- **EPUBCheck 5.4.x** is a future-compatibility signal while it validates EPUB 3 content against EPUB 3.4 rules; adoption requires a deliberate standards migration.
- No second EPUB framework, model, vector store, or publication database is added; Phase 20 reuses the approved BookForge boundary and the existing checksum-pinned validator tooling.

Detailed research and completion gates are in `docs/PHASE_20_RESEARCH.md`.

### Phase 21 — Literary Evaluation Corpus & Benchmarking — canonical

PR #100; implementation merge commit `0e4b8ebf1bdb4dd7c931e3ba44cc64f23358d4b1`.

Delivered:

- native `literary-evaluation-engine` with strict rights-safe corpus/submission schemas;
- eight project-owned synthetic EN→FA literary challenge cases covering semantic fidelity, Persian naturalness, character voice, relationship/register, dialogue/subtext, terminology, concrete detail/agency, and bounded continuity;
- deterministic anchors plus declared contrastive degradations so CI verifies the benchmark catches the failure class it claims to measure;
- bounded human scorecards with explicit reviewer expertise, 1–5 ratings, optional pairwise preference, and no more than four focused dimensions per evaluation pass;
- CLI `literary-engine benchmark <corpus.json> <submission.json>`;
- optional isolated SacreBLEU 2.6.0 chrF2++ evidence with no external test-set download;
- reuse of the existing optional COMET/XCOMET evidence boundary rather than a second neural-metric stack;
- external Persian corpora remain research candidates only; none are vendored into the committed benchmark.

Research and validation policy: `docs/PHASE_21_RESEARCH.md`.

## CLI

Canonical commands now include `benchmark <corpus.json> <submission.json>` in addition to the existing translation/project/review commands.

## CI/CD

Canonical CI includes the permanent Phase 21 Linux + Apple Silicon benchmark workflow covering Rust fmt/Clippy/tests, contrastive corpus sanity, CLI good-vs-degraded regression, SacreBLEU protocol/install/audit checks, and workspace/security regression gates.

## Non-Negotiable Constraints

- Rust remains the core language.
- No simple machine-translation wrapper replaces literary translation logic.
- Preserve author intent and narrative structure.
- Human review remains the approval/canon boundary.
- External/model metrics remain evidence, not authority.
- Benchmark anchors are narrow regression evidence, not an overall literary-quality score.
- Committed benchmark text must be project-owned/rights-safe or have explicit reviewed redistribution rights.
- Optional tools/models must not become hidden runtime requirements.
- Review/provider/alignment failures remain explicit; absence is never interpreted as a clean review.
- EPUB publication provenance must be explicit; incomplete/mismatched block identity fails closed rather than guessing.
- No secrets, proprietary manuscripts, generated translations, or private reviewer material are committed to Git or developer-memory systems.

## Test Coverage

Coverage now also includes Phase 21 rights/provenance schema enforcement, deterministic anchor evaluation, contrastive degradation sanity, human-scorecard validation, CLI benchmark JSON/text output, optional chrF2++ protocol validation, and Apple Silicon benchmark compatibility.

## Phase 22 — Product Surface & Distribution Hardening — canonical

PR #102; final reviewed head `8229e5ce2ce0126ca567b4074b84434f4a260a2d`; merge commit `dc2bf1eee2f5d2dedc7c97d0164c3c26b8ac979b`.

Delivered:

- isolated Tauri 2 desktop product surface outside the core Rust workspace;
- local static frontend with bounded Rust IPC through `ApplicationService`;
- project/source workflow, intelligence review, canon editing, translation lifecycle, manual revision, literary-review evidence, history, provider configuration, and DOCX/EPUB export;
- session-only provider credentials, restrictive CSP, Rust-owned native file dialogs, and no remote frontend content;
- explicit application-layer fix so `TranslationConfig.model` is honored;
- committed independent `desktop/src-tauri/Cargo.lock` and final `--locked` validation;
- real macOS arm64 `.app` bundle build, desktop/core audit, static UI security checks, and regression validation.

Final-head successful runs:

- Phase 22 Desktop Product `35335168206`;
- Rust CI `35335167950`;
- Security `35335168513`;
- Phase 18 `35335168705`;
- Phase 19 `35335168191`;
- Phase 20 `35335168289`;
- Phase 21 `35335168962`;
- Project Memory Tooling `35335168359`.

Signing/notarization remains a real external-credential release concern; the canonical Phase 22 bundle proves buildability, not notarized public distribution. Tauri updater remains intentionally disabled.

Detailed record: `docs/PHASE_22_RESEARCH.md`.

## Current Branch — Phase 23 Trusted Release & Supply-Chain Hardening

Branch: `phase-23-trusted-release-supply-chain`.

Phase 23 hardens the release path without changing literary translation behavior.

Implemented on the branch so far:

- fixed the CLI release workflow so it verifies the committed `engine/Cargo.lock` instead of regenerating it during release;
- pinned `cargo-cyclonedx 0.5.9` for per-target JSON SBOM generation;
- added GitHub `actions/attest@v4` provenance and per-binary SBOM attestation configuration for tagged releases;
- retained SHA-256 release checksums;
- added permanent Phase-23 CLI release-contract/SBOM validation;
- added permanent Apple Silicon desktop SBOM + locked app-build + integrity-manifest validation;
- explicitly checks that Tauri updater remains disabled until a real signing trust root and trusted endpoint exist;
- vendored the user-requested `ip-as-logo` Agent Skill from exact upstream commit `acb834c717bcd0a487c49732d08397ba280d690b`, preserving its MIT notice and verifying exact upstream Git blobs in CI;
- the visual skill is developer/design-only and cannot affect translation/runtime/publishing.

Public desktop release remains credential-blocked for Developer ID signing and Apple notarization; those credentials are not fabricated or stored in the repository.

Detailed research and exit criteria: `docs/PHASE_23_RESEARCH.md`.

## CLI / Desktop

The canonical CLI and desktop application remain operationally independent. Phase 23 adds release evidence around them; it does not move distribution tooling into runtime code.

## Current Handoff

Finish Phase-23 branch validation, review any CI failures, confirm release/SBOM workflow syntax across supported targets, then open/merge the Phase-23 PR only when the exact final head is green. Do not enable updater or claim a notarized desktop release without real external credentials.
