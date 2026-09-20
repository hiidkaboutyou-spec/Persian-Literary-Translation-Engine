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

### Phase 23 — Trusted Release & Supply-Chain Hardening — canonical

PR #103; final reviewed head `bdf3e563380f5f169a9ecf48847c100baf904c67`; merge `adc2ab2294feec6ff190b4e4d11c3fa6407ca7f2`.

Delivered:

- committed-lockfile release builds with no release-time dependency refresh;
- pinned CycloneDX JSON SBOM generation and validated filename semantics;
- GitHub provenance plus per-binary SBOM attestations for tagged CLI releases;
- SHA-256 release checksums;
- permanent release-contract, multi-platform CLI and Apple Silicon desktop-integrity validation;
- explicit fail-closed desktop updater boundary pending real signing trust roots;
- exact-revision vendored `ip-as-logo` Agent Skill for optional product-identity work only, isolated from translation/runtime.

Developer ID signing/notarization and production updater activation remain external credentialed release work and are not simulated.

### Phase 24 — Literary Precision & Persian Polish — canonical

PR #104; final reviewed head `e8a0420a57aa4a6663fda8d32bd4336069237421`; merge `2fa48dfd39437f79e6ac9db949f54e6595f2cc0a`.

Delivered:

- native Rust low-risk Persian Unicode polish plus conservative typography/orthography diagnostics;
- native Persian evidence integrated into post-translation literary review without claiming full fluency/voice/style evaluation;
- explicit human-gated, single-paragraph acceptance of concrete literary-review revision proposals with freshness/structure/native-verification checks, audit history and review staleness;
- Tauri command/UI support for explicit proposal acceptance;
- exact `rbook 0.7.10` dev-only dependency for independent strict EPUB reopen validation;
- rights-safe differential EPUB regression preserving BookForge as the only runtime publication owner;
- synchronized committed engine, desktop and literary-alignment lockfiles with permanent `--locked` validation.

Final-head successful runs:

- Phase 24 Literary Precision `35354157358`;
- Rust CI `35354157248`;
- Security `35354157239`;
- Phase 18 `35354157288`;
- Phase 19 `35354157340`;
- Phase 20 `35354157189`;
- Phase 21 `35354157322`;
- Phase 22 Desktop Product `35354157247`;
- Phase 23 Trusted Release `35354157250`;
- Project Memory Tooling `35354157261`.

Research/dependency decisions remain: Virastar/`rezkam/persian` are references only; BookNLP/FastCoref are benchmark candidates only; Hazm remains blocked; DadmaTools remains conditional; no second memory/context owner or free-form chapter-rewrite agent was added.

Detailed research: `docs/PHASE_24_RESEARCH.md`.

### Phase 25 — Narrative Speaker & Coreference Intelligence — canonical

PR #106; final reviewed head `9037f569cffb2618b915248db2c60015764d034c`; merge `e39a46dd652aaea6fd8d990d70a32fba2d96b0d4`.

Canonical Phase 25 provides a native high-precision explicit quotation-speaker baseline, project-owned regression coverage, bounded deterministic Context Packet speaker evidence, and fail-closed ambiguity/pronoun handling without adding BookNLP/FastCoref/Torch/Transformers/spaCy or another model runtime.

### Phase 26 — Long-Span Literary Coreference Evidence — canonical

PR #111 final validated head: `8418fdf4eb06252dec8a014c13e789efa7fe800e`.
Merge commit: `5c0d4c8514b999faf786971da8541f7ea5a1b773`.

Final-head gates were green: Phase 26, Rust CI, Security, Phases 18–25, Desktop Product, Trusted Release, and Project Memory. Final hardening also made optional sidecar deadlines fail closed under host scheduler delay.

Goal: add an optional, rights-safe and fail-closed coreference evidence boundary for pronouns/nominal mentions without letting model clusters become character canon.

Implemented:

- model-agnostic `CoreferenceRequest/CoreferenceResponse` protocol with strict schema, source-offset and exact-text validation;
- globally unique cluster/mention IDs and duplicate-span rejection;
- bounded optional subprocess wrapper with timeout and failure isolation;
- canonical anchoring through existing `CharacterBible` names/approved aliases only;
- clusters with no canonical anchor or conflicting canonical anchors are omitted;
- opt-in Context Packet integration as `ContextAuthority::Inferred`, below deterministic speaker evidence and canonical character context;
- project-owned synthetic Phase-26 corpus covering canonical/alias anchors, pronouns, nominal mentions, longer spans, ambiguous clusters and unanchored clusters;
- no xCoRe/Maverick/BookCoref/FastCoref/CorPipe/Torch/Transformers/spaCy runtime dependency;
- permanent Linux + Apple Silicon Phase-26 validation.

Research decisions:

- LitBank is CC BY 4.0 and eligible for future attributed external/reference evaluation;
- BookCoref and xCoRe are technically relevant to book-scale/cross-context resolution but remain research-only under CC BY-NC-SA terms;
- FastCoref software is MIT but checkpoint licensing, book-scale quality, security and resources require a separate measured admission review;
- CorPipe 2026 is MPL-2.0 and useful as a current multilingual reference but is not itself proof of long-fiction suitability;
- NovelCR remains reference-only until exact dataset-level redistribution/licensing is explicitly verified.

Exit criteria: `docs/PHASE_26_RESEARCH.md`.

### Phase 27 — Real-Book Pilot Readiness & Resume Integrity — active validation

Branch: `phase-27-real-book-pilot-hardening`.
PR #113 now targets canonical `main` after Phase 26 merge.

Goal: make the first user-supplied full-book run operationally trustworthy before adding another model stack.

Current Phase-27 work:

- semantic translation-plan fingerprints bind checkpoint reuse to resolved provider/model, target language, style profile, and pipeline contract;
- resume reconstructs progress from valid current-plan checkpoints instead of incrementing previously persisted counts;
- valid reused checkpoints no longer consume `max_chapters`, so bounded resume advances to genuinely untranslated chapters;
- legacy or mismatched plan checkpoints regenerate instead of silently mixing translation configurations;
- structured chapter artifacts are mandatory reuse evidence and operational progress counts completed source paragraphs;
- `max_chapters` limits new provider work while later valid checkpoints remain discoverable during sparse repair;
- mixed-plan/partial exports fail closed instead of combining stale chapters into a publishable DOCX/EPUB;
- project-owned 12-chapter repeated-resume rehearsal proves exact completion, reopen, and DOCX export without committing a real manuscript;
- first real-book evaluation protocol samples early/middle/late text, long/chunk-boundary passages, dialogue/coreference, terminology/register recurrence, and omission/addition evidence;
- no new runtime dependency/model/provider/cloud service is introduced.

Research and exit criteria: `docs/PHASE_27_RESEARCH.md`.

## Next Action Rule

Always finish and verify the current numbered phase before starting the next numbered phase. Supporting tooling may be added only when it leaves runtime defaults intact and has an explicit owner/failure boundary. Do not treat branch-only work as merged. When a phase changes architecture, persistence, quality, or publishing contracts, update `IMPLEMENTATION_STATUS.md`, this roadmap, engineering decisions, external-integration notes, and project-memory notes together.
