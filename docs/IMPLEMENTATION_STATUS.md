# Implementation Status

## Vision

A production-grade English-to-Persian literary translation engine with a Rust core that preserves author intent, character voice, relationship/register, terminology, emotional subtext, continuity, and natural Persian while keeping human review as the final authority.

## Canonical Main State

`main` is canonical through Phase 33 (PR #124, merge `08d18a0ee639339e2625de442cc4c3aa86605e1d`).

Phase 26 PR #111 landed from final validated head `8418fdf4eb06252dec8a014c13e789efa7fe800e` at merge commit `5c0d4c8514b999faf786971da8541f7ea5a1b773`.

Phase 27 PR #113 landed from final validated head `f75212a496d5073d249e47cb920abf1c4303eae9` at merge commit `e06182d0f6ea487d47d56ad76672e595b3b8e25a`. The final head passed the dedicated Phase 27 gate, Rust CI, Security, Phases 18–26, Desktop Product, Trusted Release, Project Memory, and Apple Silicon checks.

Phase 28 PR #114 landed from final validated head `940ee71a1da8a3a8e9d35616f913ca97f43a9c17` at merge commit `12467a8052ee9a3dcaf4eb350ae705fe0086e510`.

Phase 29 PR #116 landed from final validated head `7eb15fc9ce16f5436b7f1339b8ce62c4a0218f9a` at merge commit `8415f559a6bf295d39504108e40864b2c40e65e5`. The exact final head passed Phase 29, Rust CI, Security, Phases 18–28, Desktop Product, Trusted Release, Project Memory Tooling, and Apple Silicon validation.

Phase 21 implementation merged through PR #100 at:

```text
0e4b8ebf1bdb4dd7c931e3ba44cc64f23358d4b1
```

The final reviewed Phase 21 head `d543b0448bdb8c886e58c69b719183daf07a66bc` passed the dedicated Phase 21 benchmark gate, Rust CI, Security/cargo-audit, Phase 18 context/retrieval, Phase 19 literary review, Phase 20 publication regression, Project Memory Tooling, and Apple Silicon arm64 benchmark checks before merge. The permanent Phase 21 workflow also runs on pushes to `main`.

### Phase 27 — Real-Book Pilot Readiness & Resume Integrity — canonical

Measured operational gaps closed:

- bounded resume previously counted reused checkpoints against `max_chapters`, allowing a one-chapter resume budget to stall on chapter 1 forever;
- paragraph progress was additive across resumes and could exceed the actual book total;
- checkpoint reuse was bound to source/context but not the semantic translation plan, so provider/model/target/style changes could reuse stale output.

Phase-27 contract:

- checkpoint reuse requires source + context + translation-plan fingerprints;
- translation-plan identity includes resolved provider/model, target language, style profile, protocol version, and pipeline contract;
- `max_chapters` counts newly translated chapters, not reused checkpoints;
- progress counters are rebuilt from valid current-plan checkpoints on every resume;
- legacy/mismatched checkpoints are regenerated rather than silently trusted;
- structured artifacts must match source/context/plan before checkpoint reuse;
- operational progress counts completed source paragraphs and can reconstruct later valid checkpoints around a repaired hole;
- export refuses partial or mixed-plan books even if stale artifacts still exist on disk;
- permanent CI rehearses a 12-chapter repeated-resume flow through exact completion, export, and reopen;
- the later real-book pilot keeps manuscript/translation text out of GitHub, PMC/projectmem, Linear, CI, logs, and external benchmark services.

No new runtime dependency, provider, model stack, cloud backend, or second persistence owner is introduced.

Detailed research: `docs/PHASE_27_RESEARCH.md`.

### Phase 28 — Private Whole-Book Audit & Human Review Sampling — canonical

Phase 28 is canonical via PR #114; final validated head `940ee71a1da8a3a8e9d35616f913ca97f43a9c17`; merge `12467a8052ee9a3dcaf4eb350ae705fe0086e510`.

Measured gap after Phase 27: the engine can safely complete/resume/export a full book, but the first real-book pilot still needs a deterministic local way to identify review coverage gaps and bounded human-review targets without uploading prose or trusting whole-book automatic scores.

Phase-28 contract:

- local/read-only audit from current project artifacts;
- no network/model calls;
- no prose in audit serialization;
- separate mechanical readiness from human-review workflow state;
- mixed-plan/source/stale artifacts remain mechanically blocking;
- missing/stale/attention literary reviews and post-edit stale quality remain human-review evidence, not automatic literary verdicts;
- deterministic bounded sampling covers book position plus risk/evidence-driven chapters.
- partial-run sampling excludes untranslated and stale-plan chapters;
- the desktop backend exposes the same audit through a read-only Tauri command; no audit logic moves into JavaScript.

Detailed research: `docs/PHASE_28_RESEARCH.md`.

### Phase 29 — Human Pilot Review Ledger & Resolution Loop — canonical

Phase 29 is canonical via PR #116; final validated head `7eb15fc9ce16f5436b7f1339b8ce62c4a0218f9a`; merge `8415f559a6bf295d39504108e40864b2c40e65e5`.

Measured gap: Phase 28 selects useful whole-book review targets but does not persist whether a human inspected a current target, found a problem, deliberately accepted a literary choice, or must re-review after a later edit.

Phase-29 contract:

- append-only local human decision records;
- currentness requires exact source + translation + plan fingerprints;
- manual edits make old records stale without deleting history;
- no automatic evaluator can create `clear` or `accepted_as_is`;
- span/severity findings are optional and Unicode-safe;
- sampled-review completion is separate from export safety and from objective quality claims;
- notes stay local project data and are never copied automatically to developer memory/CI/telemetry;
- no new dependency or external review service.

Detailed research: `docs/PHASE_29_RESEARCH.md`.

### Active Phase 30 — Desktop Pilot Review Workspace

Branch: `phase-30-pilot-review-workspace`.

Measured gap after Phase 29: the Rust/Tauri backend can compute the whole-book pilot audit and persist human decisions, but the desktop frontend does not expose that workflow. A reviewer cannot complete the pilot from the product UI.

Phase-30 contract:

- local Pilot Review navigation/view over the existing Rust-owned Phase-28/29 APIs;
- mechanical export state and sampled-review state shown separately;
- bounded current-target queue with reason, current/stale human record state and zero-target support;
- source/Persian target context loaded from the local translated chapter only after explicit target selection;
- explicit `clear`, `accepted_as_is`, and `needs_revision` submissions through `record_pilot_review`;
- optional existing Phase-19 dimension/severity and Unicode spans, with Rust remaining authoritative for validation;
- direct navigation to the normal translation editor; selecting/reviewing a target never auto-edits text;
- no browser persistence, `innerHTML`, remote frontend content, network `fetch`, telemetry, model, metric, provider or new runtime dependency;
- dedicated Linux + Apple Silicon UI/IPC/privacy validation.

Research and exit criteria: `docs/PHASE_30_RESEARCH.md`.

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

## Phase 23 — Trusted Release & Supply-Chain Hardening — canonical

PR #103; final reviewed head `bdf3e563380f5f169a9ecf48847c100baf904c67`; merge commit `adc2ab2294feec6ff190b4e4d11c3fa6407ca7f2`.

Delivered locked release builds, per-target CycloneDX SBOMs, provenance/SBOM attestations for tagged CLI artifacts, multi-platform release-contract validation, Apple Silicon desktop integrity evidence, fail-closed updater state, and exact-revision optional `ip-as-logo` design skill provenance. Real Developer ID/notarization/updater signing credentials remain external and were not fabricated.

## Phase 24 — Literary Precision & Persian Polish — canonical

PR #104; final reviewed head `e8a0420a57aa4a6663fda8d32bd4336069237421`; merge commit `2fa48dfd39437f79e6ac9db949f54e6595f2cc0a`.

Delivered native Persian typography evidence, explicit bounded human acceptance of review-proposed paragraph revisions, desktop support for that action, and independent strict EPUB reopen validation through dev-only `rbook 0.7.10`. Existing BookForge/runtime, human-approval and review-staleness boundaries remain intact.

Every required final-head gate was green: Phase 18–24, Rust CI, Security, Phase 22 Desktop, Phase 23 Trusted Release and Project Memory Tooling.

Detailed research and completion criteria: `docs/PHASE_24_RESEARCH.md`.

## Phase 25 — Narrative Speaker & Coreference Intelligence — canonical

PR #106; final reviewed head `9037f569cffb2618b915248db2c60015764d034c`; merge commit `e39a46dd652aaea6fd8d990d70a32fba2d96b0d4`.

Delivered:

- native high-precision quotation speaker attribution inside `literary-intelligence-engine`;
- canonical name/approved-alias resolution through the existing `CharacterBible`, with no second canon owner;
- quote-local subject/object and vocative guards, ambiguous/conflicting-cue fail-closed behavior, and no gender inference;
- deterministic bounded `SPEAKER MAP` evidence in Context Packet v2 with Deterministic rather than Canonical authority;
- a project-owned synthetic speaker benchmark requiring zero incorrect resolved labels while preserving intentionally unresolved pronoun/no-cue cases;
- permanent Linux + Apple Silicon Phase-25 validation with zero new model/runtime dependencies.

Final-head successful runs: Phase 25 `35394490546`, Rust CI `35394490505`, Security `35394490514`, Phase 18 `35394490532`, Phase 19 `35394490605`, Phase 20 `35394490515`, Phase 21 `35394490502`, Phase 22 `35394490517`, Phase 23 `35394490508`, Phase 24 `35394490548`, Project Memory Tooling `35394490562`.

Detailed research and completion record: `docs/PHASE_25_RESEARCH.md`.

## Current Branch — Phase 26 Long-Span Literary Coreference Evidence

Branch: `phase-26-long-span-coreference-evidence`; draft PR #111.

Implemented:

- provider/model-neutral coreference request/response schema in `literary-intelligence-engine`;
- exact character-offset/text validation, unique IDs and duplicate-span rejection before any evidence can be used;
- bounded optional subprocess execution with timeout;
- canonical cluster anchoring only through existing `CharacterBible` names/approved aliases;
- fail-closed omission for unanchored or conflicting-canonical clusters;
- opt-in `COREFERENCE MAP` Context Packet evidence with `Inferred` authority;
- a project-owned synthetic Phase-26 regression corpus with successful and fail-closed long-span cases;
- dedicated Linux/Apple Silicon Phase-26 validation;
- zero new external model/runtime dependencies.

Phase-26 external-model admission remains gated by source/checkpoint/data licensing, rights-safe measured gain, false-merge analysis, privacy/resource/failure review and Linux/Apple Silicon validation. LitBank may be used as attributed CC BY 4.0 reference evidence; BookCoref/xCoRe/Maverick remain non-commercial research references and are not product dependencies.

Detailed research and exit criteria: `docs/PHASE_26_RESEARCH.md`.

## Phase 30 — Desktop Pilot Review Workspace — canonical

PR #119; final reviewed head `1608b1d49a8f2c5fb3d475db64c32901ec013657`; merge commit `8929e6befe0b2b932fbf0d3aceaf466e99a88955`.

Delivered the end-to-end local desktop pilot-review surface on top of Phase 28/29 without moving review authority into JavaScript. The final audit also fixed an async selection race: review/editor actions remain disabled until the exact selected target chapter has loaded, and stale responses from an earlier target are discarded.

Final-head successful runs: Phase 30 `35788768638`, Phase 22 Desktop `35788768662`, Phase 23 Trusted Release `35788768764`, Phase 18 `35788768599`, Phase 19 `35788768729`, Phase 21 `35788768640`, Phase 24 `35788768743`, Phase 25 `35788768722`, Phase 26 `35788768644`, Phase 27 `35788768746`, Phase 28 `35788768802`, Phase 29 `35788768735`, and Project Memory Tooling `35788768566`.

No engine/Cargo/publication dependency surface changed in Phase 30, so the previously documented inherited Rust/Security/Phase-20 publication gates remained valid under the Phase-30 exit contract.

Detailed research and completion record: `docs/PHASE_30_RESEARCH.md`.

## Phase 31 — Provider Qualification & Experimental Atria Adapter — canonical

PR #122 merged at `10f34055e7a4c14dcc0e2185ca8f1a3540c67e88`. Atria remains restricted to rights-safe qualification; no production provider selector or desktop wiring was added. The CLI records deterministic evidence and supports blind comparison without an automatic literary winner. Research: `docs/PHASE_31_RESEARCH.md`.

## Phase 32 — Blind Human Provider Review — canonical

PR #123 merged at `6764b195d35364a1a9323244b09c18167d23a7c8`. The existing CLI now records blind human judgments and generates a separate post-review dossier. The reveal key is kept out of the reviewer ledger; the dossier grants no production admission. Research: `docs/PHASE_32_RESEARCH.md`.

## Phase 33 — Provider Admission Governance — canonical

PR #124 merged at `08d18a0ee639339e2625de442cc4c3aa86605e1d`, from final head `fb06a2f94e1a7b8bc7d79b5a93a7fea514922da4`. Exact-head Rust CI, Security, Phase 31 and Phase 33 gates succeeded. The separate admission assessment accepts a Phase-32 dossier and governance profile; unknown or unacceptable evidence blocks eligibility. Assessment does not authorize a provider, change runtime selection, or send a manuscript. Hosted Atria data-handling terms remain unknown. Research: `docs/PHASE_33_RESEARCH.md`.

## Phase 34 — Admission Artifact Integrity — in progress

Branch: `phase-34-admission-artifact-integrity`. Close the Phase-33 output-install race and require a structurally complete Phase-32 dossier before governance eligibility can be assessed. Verify collision preservation and reject missing/inconsistent reviewer counts and reveal binding. No provider activation or manuscript transfer is in scope.

The seven proposed developer-tool repositories were reviewed in `docs/DEVELOPER_TOOL_CANDIDATES_2026-09-23.md`; none has an installation case against a measured gap in this phase.

## CLI / Desktop

The canonical CLI and desktop application remain operationally independent. Phase 23 adds release evidence around them; it does not move distribution tooling into runtime code.

## Current Handoff

Phase 33 is canonical; Phase 34 hardens assessment output creation against concurrent destination creation. Private-book provider admission still requires verified terms, human literary evidence, and a separate explicit owner decision. The Atria adapter remains research-only.
