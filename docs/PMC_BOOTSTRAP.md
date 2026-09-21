# PMC Bootstrap — Persian Literary Translation Engine

Purpose: durable repository-side seed for Project Memory Core when this repository is connected to a local Obsidian-compatible PMC vault.

Project ID: `persian-literary-translation-engine`
Repository: `hiidkaboutyou-spec/Persian-Literary-Translation-Engine`
Primary branch: `main`

## Project Objective

Build a production-grade English-to-Persian literary translation engine that reads naturally in Persian while preserving authorial intent, narrative structure, character voice, relationships, emotional subtext, continuity, terminology, and literary effect.

## Durable Product Requirements

- Avoid literal word-for-word Persian when it damages naturalness, voice, emotion, humor, intimacy, sarcasm, or narrative intent.
- Naturalization must not become invention, omission, censorship, amplification, or semantic drift.
- Character voice and relationship/register changes must remain consistent across long novels.
- Translation memory, glossary, character knowledge, literary findings, and human decisions must be traceable and passage-relevant.
- Human review remains the final approval/canon boundary.
- Long books must be resumable without silently reusing outputs generated under stale source/context/profile fingerprints.
- Publication output supports high-fidelity Persian DOCX and canonical source-preserving Persian EPUB round trip from EPUB sources.

## Architecture Invariants

- Rust-first core.
- `document-engine` owns document ingestion, structure, source provenance, and publication-format reconstruction boundaries.
- `literary-intelligence-engine` owns deterministic literary understanding and inferred seeds.
- `advanced-literary-analysis` owns optional provider-assisted bounded literary findings.
- `memory-engine` owns translation memory, glossary knowledge, and Context Packet v2 retrieval contracts.
- `character-engine` owns canonical character/relationship knowledge.
- `translation-core` consumes approved/bounded context and executes translation; it does not own literary canon.
- `quality-engine` owns deterministic quality gates; probabilistic/model evidence is never human approval.
- `literary-review-engine` owns post-translation literary review evidence and revision proposals; it does not own canon or auto-apply revisions.
- `human-review-workflow` owns review lifecycle and promotion decisions.
- `project-engine` owns persistence/application orchestration, atomic operations, recovery, and state tracking.
- `ApplicationService` is the application boundary for CLI/UI/product surfaces.
- Heavy models, validators, and developer-memory tools must not silently become runtime requirements.
- Publication reconstruction must use explicit source provenance; do not guess EPUB structure from flattened translated text.

## Memory Ownership — Keep These Separate

### Translation runtime/project memory

Product data owned by the Rust engine: translation memory, glossary, character bible, relationships, literary context, review decisions, checkpoints, fingerprints, project state, translated artifacts, source block provenance, and persisted literary-review artifacts. This data can affect product behavior.

### Project Memory Core (PMC)

Curated durable engineering knowledge: architecture decisions, constraints, roadmap, implementation status, debugging lessons, dependency decisions, publication standards, and handoffs. A local Obsidian-compatible PMC vault remains the long-term target. This file is only its repository-side seed until local PMC is actually connected.

### projectmem

Optional operational coding-history companion for issues, attempts, fixes, decisions, notes, and pre-edit checks. It supplements PMC/repository docs; it does not replace either and never replaces Rust runtime memory.

Never put proprietary manuscript text, generated book translations, credentials, API/model keys, or private reviewer material into PMC/projectmem as a substitute for native runtime memory.

## Current Canonical State

Canonical `main` is verified through Phase 19.

- Phase 13 — Manuscript Intelligence & Literary Context Seeding — merged.
- Phase 14 — Literary Intelligence Review & Canon Promotion — merged.
- Phase 15 — Advanced Model-Assisted Literary Analysis — merged.
- Phase 16 — Application Orchestration Layer — merged.
- Phase 17 — Controlled External Integrations — PR #90, merge `6a4b8807d8c54878f1f10db5cab1f1290fcc60fb`.
- Safe Persian language diagnostics + EPUB validation tooling — PR #91, merge `ee76e6c0a1a01ce85f030e027d6306c7371638b6`.
- Isolated projectmem developer-memory tooling — PR #92, merge `fa84b13313fce3b0b611cd64dc06d4fb088ad422`.
- Projectmem canonical-state docs — PR #93, merge `3b1dfb5a28261d6d2999ac1d530a51f4249e5550`.
- Phase 18 — Context Packet v2 & Selective Long-Novel Retrieval — PR #94, merge `2af408a19b8f69db93aff8e6896eaf189c4d69ae`.
- Phase 19 — Literary Fidelity & Persian Naturalness Review Stack — PR #95, merge `d073dab10c0965197745a6cbc7b8e56c946835e8`; post-merge Rust/Phase 18/Phase 19 workflows verified green.

## Phase 18 Durable Decisions

- Context Packet v2 is the shared context policy for CLI and `ApplicationService`.
- Packet items carry typed provenance/authority/selection metadata and SHA-256 fingerprints.
- Approved canon/reviewed evidence outranks unresolved inference.
- Deterministic lexical/polarity/diversity retrieval remains the safety floor.
- Optional BGE-M3/FastEmbed semantic retrieval/reranking augments candidate selection only.
- Missing/failing semantic tooling falls back to deterministic retrieval and cannot block translation.
- Semantic tools cannot invent IDs/canon/project memory.
- BGE model weights are not downloaded during normal build/default CI.
- Linux and Apple Silicon arm64 are required compatibility gates for the optional model adapter.

## Phase 19 Durable Decisions

- Post-translation literary review is separate from deterministic blocking quality and human canon review.
- Automated review produces evidence/proposals only; it never auto-applies or human-approves a revision.
- Unevaluated dimensions remain explicitly unevaluated; unavailable tooling is not a pass.
- Review artifacts are fingerprinted against source, translated text, and translation context; manual edits make prior evidence stale.
- Native bounded monotonic Rust alignment owns 1:1, 1:N, N:1, N:M and source/target-gap alignment.
- Reuse the existing optional BGE-M3 model boundary rather than adding a second embedding stack.
- Hazm remains blocked until its mandatory compatible NLTK path is patched and freshly audited.
- Vecalign/SentWeave remain research references while native Rust meets the measured need; DadmaTools remains conditional on a demonstrated gap.

## Phase 20 Canonical State

PR #98 merged to `main` at `1611cd731160c122baa68c9e80c1d4faeb7dfcfc`.

Canonical Phase 20 includes:

- BookForge source block IDs propagated into native `SourceLocation` and translated artifacts;
- marker-aware source-preserving EPUB reconstruction with explicit `block_id -> translated text` mapping;
- fail-closed rejection of missing, unknown, duplicate, empty, or mismatched block provenance instead of positional guessing;
- BookForge source-aware language rewrite plus native RTL `dir="rtl"` and OPF `page-progression-direction="rtl"` publication metadata;
- preservation of source-derived CSS, images, navigation, links, inline markup, notes/resources through reconstruction rather than plain-text EPUB regeneration;
- deterministic publication output and source non-mutation contract;
- ApplicationService/CLI selection between DOCX and EPUB publication;
- permanent read-only Phase 20 CI with generated rights-safe EPUB fixture, EPUBCheck 5.3.0 input/output checks, repeated-export byte comparison, source checksum, representative resource/markup assertions, no-default-features compatibility, and Apple Silicon arm64 validation;
- supporting explicit `adult-intimacy` fidelity style profile for confirmed-adult source material, while `literary` remains the default and automated intimacy evidence remains non-canonical.

The final PR head passed the dedicated Phase 20 publication gate, Rust CI, Security/cargo-audit, Phase 18, Phase 19, Project Memory Tooling, and Apple Silicon arm64 checks. The permanent Phase 20 publication workflow is retained on `main` pushes.

## Phase 20 Durable Decisions

1. **Publication standard** — target EPUB 3.3, the current W3C Recommendation. EPUB 3.4 remains deferred while it is a Candidate Recommendation.
2. **Conformance validator** — keep checksum-pinned EPUBCheck 5.3.0 as the Phase 20 authoritative gate because it validates EPUB 3.3. EPUBCheck 5.4.x is a future-compatibility signal while it applies EPUB 3.4 rules to EPUB 3 content; do not silently migrate standards.
3. **EPUB ownership** — continue using revision-pinned BookForge (`23f8c9d3c97a06f48e13424698441bfb4b037844`) instead of adding another EPUB framework. Persist native provenance, not BookForge IR.
4. **Reconstruction contract** — explicit block identity only. Publication export fails closed if provenance is incomplete or structural markers are damaged; never guess from paragraph position/count.
5. **RTL ownership split** — BookForge owns source-aware XHTML rebuild and target `dc:language` / `lang` / `xml:lang`; the native layer adds only `dir="rtl"` and OPF `page-progression-direction="rtl"` for RTL targets.
6. **Determinism/privacy** — repeated export of the same source/artifacts must be byte-stable; export must not mutate source; validation fixtures must be rights-safe/project-owned.
7. **Adult intimacy fidelity** — `adult-intimacy` is explicit opt-in only and requires caller confirmation that every participant in sexual content is an adult. Never infer confirmation. Preserve source explicitness/markedness, consent/hesitation/refusal/coercion/power cues, agency/referents, sensory channels, POV, emotional intensity, and pacing; detect both sanitization and amplification; do not sexualize nonsexual text. Automated findings remain evidence only.

Detailed rationale: `docs/PHASE_20_RESEARCH.md`.

## Canonical External / Supporting Tool Decisions

- **BookForge** — validated EPUB ingestion/reconstruction boundary at the pinned revision; native document types remain canonical and validation failures never silently fall back in the default path.
- **COMET/XCOMET/DocCOMET** — optional Python quality-evidence sidecar; no automatic approval.
- **Lingua 1.8.0** — optional English/Persian-only language diagnostic, disabled by default; no rewriting/approval.
- **EPUBCheck 5.3.0** — checksum-pinned EPUB 3.3 publication validator; distribution not vendored; Phase 20 publication CI may install it explicitly.
- **projectmem 0.3.3** — optional developer-side coding memory only; safe profile disables hooks, watcher, history backfill, global inheritance, and automatic bridge/MCP edits.
- **PMC vs projectmem vs runtime memory** — keep ownership domains separate.
- **ContextWeaver / TranslateBooksWithLLMs / TransAgents** — architecture references unless a concrete native gap is demonstrated.
- External upgrades require license/privacy/security/resource/failure-mode review plus appropriate tests.

## Supporting-Tool Shortlist

These are not current runtime dependencies and must not interrupt the numbered roadmap.

- **OpenDataLoader PDF** — benchmark candidate for difficult PDF ingestion using project-owned fixtures; any Java/Python/AI-hybrid path remains optional and maps to native document types.
- **ripwire** — potential developer-only code intelligence/MCP; never a build/runtime prerequisite or project-memory owner.
- **Headroom** — potential developer/research context compression only; do not place lossy compression in literary runtime/provider context without a fidelity benchmark.

## Project-Memory Research Decisions

- **projectmem** selected as the operational companion because it is local-first, MIT, Codex/MCP-oriented, and safely isolated with upstream opt-outs.
- **Serena** useful reference but full application is GPL-3.0-or-later and broader than needed; reference only.
- **Global Agent Memory** overlaps PMC/projectmem and adds broader storage/dashboard surface; no dependency.
- **MemoryWiki** relevant ideas but current maturity/provenance do not justify dependency.
- **automatic transcript/session memory systems** not selected; raw chat capture is the wrong abstraction and increases privacy/secret-retention risk.

## Phase 21 Canonical State

PR #100 merged to `main` at `0e4b8ebf1bdb4dd7c931e3ba44cc64f23358d4b1`. Final reviewed head: `d543b0448bdb8c886e58c69b719183daf07a66bc`.

Final Phase 21 gates were green: dedicated Phase 21 benchmark workflow, Rust CI, Security/cargo-audit, Phase 18, Phase 19, Phase 20 regression, Project Memory Tooling, and Apple Silicon arm64.

Durable Phase 21 decisions:

1. **Benchmark ownership** — committed gold/challenge fixtures must be project-owned/rights-safe or have explicit reviewed redistribution rights. Public availability is not a license.
2. **Evaluation shape** — use multiple independent evidence channels. Deterministic anchors, chrF2++, COMET/XCOMET, provider critics, and human ratings must not be collapsed into one approval score.
3. **Contrastive validity** — every declared benchmark failure type needs a deliberately degraded variant that actually triggers the intended challenge; otherwise the benchmark claim is unproven.
4. **Human-first literary judgment** — human scorecards record reviewer expertise, focused 1–5 dimensions, notes, and optional pairwise preference. One pass/case is capped at four dimensions to keep evaluation cognitively bounded.
5. **Reference metrics** — SacreBLEU 2.6.0/chrF2++ is optional isolated evidence only. It must not download external test sets through the project workflow and never acts as an acceptance threshold.
6. **Neural metrics** — reuse the existing COMET/XCOMET process boundary; do not add another neural metric stack or default model download.
7. **Persian corpora** — iPerUDT may later be useful as CC0 colloquial-syntax evidence; Mizan, Degarbayan-SC, and FarSSiM remain blocked from committed benchmark use until their dataset-level rights/provenance questions are resolved. None are installed merely because they are available.
8. **Dependency proof** — Phase 21 is the mechanism that must prove a real gap before DadmaTools/other Persian NLP packages are considered. Hazm remains blocked on the recorded security path.

Detailed rationale: `docs/PHASE_21_RESEARCH.md`.

## Phase 22 Canonical Decisions — Desktop Product

Canonical via PR #102; merge `dc2bf1eee2f5d2dedc7c97d0164c3c26b8ac979b`.

Durable decisions:

1. **Tauri 2 selected** — use the reviewed stable 2.11 line because it matches the existing Rust `ApplicationService` facade, uses system WebViews, and has official macOS bundling/security support. Do not jump to an open 2.12 milestone or Tauri 3 alpha without a new review.
2. **UI never owns domain orchestration** — translation, analysis, review, canon, persistence, locking, progress, recovery and export stay in `ApplicationService`.
3. **Desktop dependency isolation** — `desktop/src-tauri` stays outside the `engine/` Cargo workspace. Tauri/WebView dependencies must never become hidden requirements for the core CLI/runtime.
4. **Static local frontend** — no React/Vue/Svelte/Vite dependency is justified yet. Local HTML/CSS/JS is sufficient for rendering application DTOs and sending bounded commands.
5. **No remote content** — no CDN, remote script/font/page or manuscript `innerHTML`. CSP remains restrictive and manuscript text is assigned as text/textarea content.
6. **Filesystem authority stays Rust-side** — official dialog plugin selects files/folders; do not grant a general JavaScript filesystem API merely for convenience.
7. **Secrets are session-only** — OpenAI key may be installed in the desktop process environment for the active session, but is never stored in project JSON, frontend storage, Git, PMC or projectmem.
8. **Explicit model selection must be truthful** — `TranslationConfig.model` now has precedence over `OPENAI_MODEL`; UI configuration must never claim a selection the provider path ignores.
9. **Updater deferred** — do not enable Tauri updater until update artifacts are signed and a trusted endpoint is configured/tested.
10. **macOS release gate** — unsigned/ad-hoc bundle CI proves buildability; public direct distribution still requires Apple code signing/notarization credentials external to the repository.
11. **Independent lockfile required** — Phase 22 cannot become canonical until the validated `desktop/src-tauri/Cargo.lock` is committed and subsequent builds are locked.

Detailed rationale: `docs/PHASE_22_RESEARCH.md`.

## Phase 23 Canonical Decisions — Trusted Release & Supply Chain

Durable decisions:

1. **Release lockfiles are immutable inputs** — release workflows may verify committed Cargo.lock files with `cargo metadata --locked` but must not run `cargo generate-lockfile` immediately before publishing.
2. **SBOM is release evidence, not runtime** — use pinned `cargo-cyclonedx 0.5.9` only in CI/release tooling. Normal translation/desktop runtime must not depend on it.
3. **Three independent release evidence layers** — SHA-256 checksums, CycloneDX SBOMs, and GitHub/Sigstore artifact attestations answer different questions and must not be collapsed into a single "safe" claim.
4. **Attest actual releases, not routine test artifacts** — tagged CLI releases receive provenance/SBOM attestations. Ordinary Phase-23 validation artifacts are not promoted as trusted public releases.
5. **Apple credentials stay external** — Developer ID certificates, App Store Connect/Notary keys, keychain material and notarization credentials are never committed to Git, PMC, projectmem, project files, or frontend state.
6. **Updater stays fail-closed** — do not add `tauri-plugin-updater`, update endpoints, or updater artifacts until a real Tauri signing key/public trust root and HTTPS distribution path are configured and tested.
7. **ip-as-logo is design-only** — the vendored MIT Agent Skill is pinned to upstream commit `acb834c717bcd0a487c49732d08397ba280d690b`. It has no translation/runtime authority.
8. **No installer for a text-only skill** — vendor the reviewed `SKILL.md` + license directly rather than running a moving `npx skills@latest` supply-chain path.
9. **Visual candidates require human selection** — mascot/app-icon outputs never replace canonical product identity automatically.
10. **Upstream skill updates are review events** — no auto-sync; any revision requires new provenance/license/security review and recorded blob IDs.
11. **Release actions are immutable references** — release-sensitive GitHub Actions use reviewed full-length commit SHAs, not movable major tags.
12. **Attestation permissions are publish-only** — `id-token: write`, `attestations: write`, and `artifact-metadata: write` exist only on the tagged-release publish job, never as workflow-wide defaults.
13. **Legacy SPDX separators are preserved as evidence** — do not rewrite transitive crate metadata such as `MIT/Apache-2.0` just to satisfy CycloneDX `--license-strict`; generate the complete SBOM with warnings visible instead.

Detailed rationale: `docs/PHASE_23_RESEARCH.md`.

## Current Roadmap

- Phase 18 — Context Packet v2 & Selective Long-Novel Retrieval — canonical/merged.
- Phase 19 — Literary Fidelity & Persian Naturalness Review Stack — canonical/merged.
- Phase 20 — Publication-Grade EPUB Round Trip — canonical/merged (PR #98; `1611cd731160c122baa68c9e80c1d4faeb7dfcfc`).
- Phase 21 — Literary Evaluation Corpus & Benchmarking — canonical/merged (PR #100; `0e4b8ebf1bdb4dd7c931e3ba44cc64f23358d4b1`).
- Phase 22 — Product Surface & Distribution Hardening — canonical/merged (PR #102; `dc2bf1eee2f5d2dedc7c97d0164c3c26b8ac979b`).
- Phase 23 — Trusted Release & Supply-Chain Hardening — canonical/merged (PR #103; `adc2ab2294feec6ff190b4e4d11c3fa6407ca7f2`).
- Phase 24 — Literary Precision & Persian Polish — canonical/merged (PR #104; `2fa48dfd39437f79e6ac9db949f54e6595f2cc0a`).
- Phase 25 — Narrative Speaker & Coreference Intelligence — canonical/merged (PR #106; `e39a46dd652aaea6fd8d990d70a32fba2d96b0d4`).
- Phase 26 — Long-Span Literary Coreference Evidence — active branch `phase-26-long-span-coreference-evidence`; draft PR #111.

Always finish/verify the current numbered phase before starting the next numbered phase. Supporting tooling may land between phases only when runtime defaults remain intact, ownership/failure boundaries are explicit, and validation passes.

## Quality Philosophy

No single automatic metric is authoritative for literary translation. Combine deterministic structure/safety checks, omission/addition evidence, semantic faithfulness, character voice, relationship/register consistency, Persian naturalness/readability, subtext/emotional-effect preservation, terminology/continuity, optional language/COMET evidence, profile-specific fidelity where explicitly enabled, and final human judgment.

## Durable Dependency Rule

Before adding any GitHub repository/package/tool:

1. prove a concrete capability gap;
2. inspect license, release provenance, maintenance, dependencies, model/data licenses, privacy, and failure modes;
3. prefer native Rust when the missing capability is reasonably small and safer to own;
4. isolate heavy/model-backed tools behind explicit optional boundaries;
5. preserve deterministic fallback;
6. benchmark on project-owned/rights-safe fixtures;
7. validate Linux/macOS compatibility where relevant;
8. run vulnerability/security audits;
9. never let automated evidence become canon or human approval.

## Source-of-Truth Files

- `AGENTS.md` — engineering/agent constraints
- `docs/IMPLEMENTATION_STATUS.md` — implemented capabilities/current branch state
- `docs/IMPLEMENTATION_ROADMAP.md` — roadmap
- `docs/PHASE_23_RESEARCH.md` — current trusted-release/supply-chain/branding-tool research
- `docs/PHASE_22_RESEARCH.md` — canonical product/distribution research/decisions
- `docs/PHASE_21_RESEARCH.md` — benchmark/evaluation research/decisions
- `docs/PHASE_20_RESEARCH.md` — publication research/decisions
- `docs/PHASE_19_RESEARCH.md` — literary-review dependency/review research
- `docs/ENGINEERING_DECISIONS.md` — durable engineering decisions
- `docs/AI_MEMORY_PIPELINE.md` and `docs/HYBRID_MEMORY_SEARCH_ARCHITECTURE.md` — runtime memory design
- `docs/EXTERNAL_INTEGRATIONS.md` — external integration boundaries
- `docs/PROJECT_MEMORY_INTEGRATIONS.md` — developer/agent memory ownership
- this file — repository-side PMC bootstrap seed

## PMC Promotion Guidance

When a local PMC vault is actually available, promote stable material into focused notes rather than copying this file wholesale: Project Home, Current State, Decisions, Constraints, Plans, and Handoff.

Updating this repository seed does **not** mean the user's local PMC/Obsidian vault or local Mac configuration was modified. Never claim that without direct local access and verification.

## Durable Phase 24 decisions

1. Phase 23 is canonical via PR #103, final reviewed head `bdf3e563380f5f169a9ecf48847c100baf904c67`, merge `adc2ab2294feec6ff190b4e4d11c3fa6407ca7f2`.
2. Phase 24 is canonical via PR #104; final reviewed head `e8a0420a57aa4a6663fda8d32bd4336069237421`; merge `2fa48dfd39437f79e6ac9db949f54e6595f2cc0a`.
3. Do not change the established comparison semantics of `text-normalization::normalize()` merely to improve publication typography. Publication-quality Persian polish is a separate API.
4. Automatic Persian cleanup is restricted to low-risk Unicode surfaces. Literary punctuation, expressive marks, register and prose style remain advisory/human-controlled.
5. Native Persian typography findings are evidence only. Recording `PersianNaturalness` for that channel means the typography/orthography surface was evaluated, not that literary fluency/voice/style was approved.
6. `rbook = 0.7.10` is pinned as a `document-engine` dev/CI dependency only. BookForge remains the runtime EPUB owner and EPUBCheck remains the standards-conformance gate.
7. Literary review proposals remain non-mutating until explicit human acceptance. Acceptance is fail-closed for stale, ambiguous/multi-target or non-concrete proposals and must use the normal revision ledger.
8. Applying an accepted proposal stales prior quality/review evidence. Re-review is required; an accepted model suggestion is never human approval of the resulting chapter.
9. BookNLP and FastCoref are not installed product dependencies. Evaluate them only as isolated rights-safe character/coreference/speaker benchmarks if a measured gap appears.
10. Virastar and `rezkam/persian` are design references only; do not add JS/Python runtimes for deterministic rules already owned natively.
11. DelTA/Loong/Prozetta/bilingual_book_maker do not justify parallel memory/context systems. Reuse Context Packet v2, approved canon and relevant-only glossary selection unless a benchmark proves a concrete gap.
12. Hazm remains blocked under the existing security decision; DadmaTools remains conditional.
13. Phase-24 dependency commands must be `--locked` after the committed lockfile. Do not reintroduce a CI lockfile bootstrap on canonical main.
14. No source manuscript, generated translation, reviewer private data or provider secret may be added to repository/project memory.
15. **Next research target is speaker/coreference evidence** — the current character context is strong for canonical names/aliases but does not itself resolve pronouns or quotation speakers. Any Phase-25 work must begin with a rights-safe benchmark and must not install BookNLP/FastCoref or another model stack until it proves a measurable gain.

## Durable Phase 25 decisions

1. **Speaker attribution belongs to literary intelligence** — it reuses canonical `CharacterBible` identities and must not create a second character/canon owner.
2. **Precision before coverage** — resolve only explicit high-precision name/alias + speech-verb patterns first. Pronoun-only, implicit turn-taking and unsupported dialogue remain unresolved.
3. **No gender inference** — never infer speaker identity from pronoun gender or character-name assumptions.
4. **Vocatives are not speakers** — character names inside a quotation are content unless independent outside-quote evidence identifies a speaker.
5. **Ask-object guard** — before a quote, `verb + name` is not accepted as speaker evidence because `Mina asked Reza, "..."` makes Reza an object/addressee candidate. After a quote, when `name + speech-verb` and `speech-verb + name` candidates share the exact same speech-verb token (for example `"..." Mina asked Reza`), the inverted candidate is treated as that verb’s object/addressee; unrelated local cues still fail closed.
6. **Quote-local evidence, conflict fail-closed** — attribution cues cannot cross another quotation or a hard pre-quote sentence boundary; if multiple explicit local character cues remain, do not pick the nearest one—leave the quote unresolved.
7. **Context authority stays Deterministic** — the canonical character is known, but quote-to-speaker linkage is inferred evidence and is never promoted to Canonical/HumanApproved automatically.
8. **Permanent CI corpus is project-owned synthetic** — LitBank CC BY 4.0 may be used later as an attributed external/reference benchmark, but CI has no network/corpus dependency.
9. **Non-commercial corpora/models are research-only** — PDNC, BookCoref and Maverick terms do not become product dependencies.
10. **ModernBookNLP/BookNLP/FastCoref are not installed** — a future sidecar requires separate source/model/data license review and a measurable rights-safe gain over the native baseline.
11. **Renard is reference-only** — GPL-3.0-only plus heavy Python/Torch dependencies do not justify integration.
12. **No new runtime dependency in Phase 25 baseline** — the implementation is native Rust inside existing `literary-intelligence-engine`.
13. **Model evidence can never create canon** — any future coreference/speaker sidecar remains optional evidence and normal translation must work without it.
14. **Phase 25 canonical completion** — PR #106 final reviewed head `9037f569cffb2618b915248db2c60015764d034c`; merge `e39a46dd652aaea6fd8d990d70a32fba2d96b0d4`; all final Phase 18–25, Rust, Security, Desktop, Trusted Release and Project Memory gates were green.
15. **Phase 26 research target** — long-span literary coreference (pronouns, nominal mentions and cross-context identity) must begin with rights-safe evaluation and remain evidence-only. LitBank CC BY 4.0 is eligible for attributed reference benchmarking; BookCoref/xCoRe/Maverick remain non-commercial research references. No model runtime is approved by this handoff.

## Durable Phase 26 decisions

1. **Coreference evidence is not canon** — external/model clusters may inform translation context but can never create, merge or mutate canonical characters automatically.
2. **Canonical anchoring is mandatory** — a cluster reaches Context Packet only when its explicit canonical-name/approved-alias mentions resolve to exactly one existing CharacterBible identity.
3. **Conflicts fail closed** — clusters with zero canonical anchors or anchors for multiple canonical characters are omitted rather than guessed.
4. **Source provenance is exact** — mention IDs are globally unique, spans cannot be reused across clusters, offsets must be in bounds, and mention text must exactly equal the declared source slice.
5. **Authority stays Inferred** — model-backed coreference evidence ranks below Phase-25 deterministic speaker evidence and below canonical/human-approved context.
6. **Opt-in compatibility** — existing deterministic and semantic Context Packet APIs retain their behavior; coreference requires the explicit Phase-26 API.
7. **Sidecars are bounded** — optional coreference executables have source-size/cluster/mention limits and a subprocess timeout. Failure cannot make normal translation unusable.
8. **Permanent CI is project-owned/network-free** — committed Phase-26 corpus is synthetic and redistribution-allowed. LitBank CC BY 4.0 is optional attributed reference evaluation, not a CI download.
9. **Non-commercial systems remain research-only** — BookCoref, xCoRe and Maverick are not product/runtime dependencies under reviewed CC BY-NC-SA terms.
10. **FastCoref is not pre-approved** — MIT software alone is insufficient; exact checkpoint terms, security/resources and literary long-span benchmark gain still require review.
11. **NovelCR is blocked from committed use until dataset licensing is explicit** — public availability is not enough.
12. **No model stack added in Phase 26** — no Torch, Transformers, spaCy, xCoRe, Maverick, BookCoref, FastCoref or CorPipe dependency is introduced.


## Durable Phase 27 decisions

1. **Phase 27 is canonical** — final validated head `f75212a496d5073d249e47cb920abf1c4303eae9` landed via PR #113 at merge `e06182d0f6ea487d47d56ad76672e595b3b8e25a` after Phase 27, Rust CI, Security, Phases 18–26, Desktop Product, Trusted Release and Project Memory all passed on the exact head.
2. **The next measured gap is operational full-book reliability** — after long-span coreference, do not add another model stack before proving that multi-session whole-book execution, recovery and export are trustworthy.
3. **Checkpoint reuse is semantic-plan aware** — source/context equality is insufficient. Reuse also requires a deterministic translation-plan fingerprint covering resolved provider/model, target language, style profile, protocol version and pipeline contract.
4. **Execution budgets are not semantic identity** — `max_chapters` is deliberately excluded from the plan fingerprint and counts newly translated chapters only. Valid reused checkpoints must not consume the current run's translation budget.
5. **Resume rebuilds progress from evidence** — completed chapter/paragraph counts are reconstructed from valid current-plan checkpoints on each run instead of adding persisted counts again. Final completed counts must exactly equal book totals.
6. **Legacy/mismatched checkpoints fail safe** — a checkpoint with no Phase-27 plan fingerprint, or a different plan fingerprint, is regenerated rather than silently mixed into the new run.
7. **Permanent pilot CI remains rights-safe** — the Phase-27 12-chapter rehearsal uses project-owned synthetic manuscript generation. A real/user book is never committed as a fixture.
8. **Real-book text stays in native project storage** — do not copy manuscript or generated translation text into GitHub, PMC/projectmem, Linear, CI artifacts/logs, or external benchmark/research services. Developer trackers may store non-text metadata such as chapter ID, failure class and fix status.
9. **No single automatic book-quality score becomes authority** — research through 2026 shows weaknesses in document-level metrics. Use deterministic whole-book invariants, bounded local/discourse windows, omission/addition evidence and focused human review.
10. **Do not change refinement granularity from literature alone** — ACL 2026 evidence favors document translation plus smaller refinement in studied settings, but EN→FA behavior must be measured in our pilot before changing the production default.
11. **Pilot sampling is position-aware** — first real-book review must include early, middle and late chapters; long/chunk-boundary passages; dialogue/coreference; recurring terminology; and relationship/register continuity.
12. **No new runtime dependency in the Phase-27 baseline** — no Temporal/workflow engine, cloud persistence, new provider/model stack or second persistence owner is introduced to solve these operational bugs.

13. **Export is current-plan fail-closed** — disk presence is insufficient. Export requires Completed current progress, exact chapter/source-paragraph totals, current source identity, non-empty plan identity, and every structured chapter artifact matching the current plan/source.
14. **Structured artifacts are part of checkpoint validity** — a text/fingerprint checkpoint without a matching structured chapter artifact is regenerated; EPUB additionally requires exact block provenance.
15. **Operational paragraph progress counts source paragraphs** — provider changes to paragraph segmentation must not distort completion percentages or prevent a valid completed DOCX workflow.
16. **Bounded repair may scan later valid checkpoints** — `max_chapters` limits new provider translations, not evidence reconstruction. After repairing one hole, later valid checkpoints may still be counted in the same resume pass.

17. **Sidecar deadlines include host scheduling delay** — optional coreference process timeouts start before spawn and fail closed when the host has not observed completion within the configured budget. Late completion after scheduler delay is not accepted as timely evidence.


## Phase 27 Canonical Evidence

- Final validated head: `f75212a496d5073d249e47cb920abf1c4303eae9`
- Merge: `e06182d0f6ea487d47d56ad76672e595b3b8e25a`
- Phase 27: `35509554696`
- Rust CI: `35509554703`
- Security: `35509554694`
- Desktop Product: `35509554716`
- Trusted Release: `35509554715`
- Project Memory Tooling: `35509554735`

## Durable Phase 28 decisions

1. **No overall book-quality score** — recent literary/document-level evaluation evidence is not reliable enough to let one automatic metric or LLM judge become the authority for a novel.
2. **Mechanical readiness and human-review state are separate** — `mechanically_export_ready` means artifact/source/plan completion is safe; `human_review_clear` only describes current review workflow coverage and staleness.
3. **Audit output is text-free** — no source prose, translated prose, revision text, Character Bible text, glossary text, prompts, responses or secrets may appear in the serialized audit.
4. **Review targets use stable identifiers** — chapter IDs/indexes and paragraph IDs point the local UI to text inside the project workspace without copying that text into audit/tracking artifacts.
5. **Sampling is bounded and position-aware** — cover early/middle/late book positions plus longest/dialogue-heavy material and chapters with manual edits, stale/attention review evidence or stale post-edit quality.
6. **Automatic review remains evidence-only** — stale/missing/attention states route work to humans; they do not decide artistic quality.
7. **Real-book privacy boundary remains local-first** — do not send manuscript/translation text to GitHub, PMC/projectmem, Linear, CI logs/artifacts, telemetry or external evaluators merely to operate the pilot.
8. **No new runtime dependency in Phase 28 baseline** — native Rust over existing artifacts is sufficient; no metric package, LLM judge, cloud observability, analytics SDK, database, provider or model is added.

9. **Partial-run targets must be actionable** — position/content sampling may only reference chapters whose translation artifact matches the current source and translation plan. Never suggest untranslated/stale-plan chapters for translation review.
10. **Desktop exposure stays Rust-owned** — the audit may be invoked through a read-only Tauri command, but computation remains in `ApplicationService`; do not duplicate audit/orchestration rules in JavaScript.
11. **Zero review targets is a valid caller choice** — `max_review_targets = 0` produces no targets rather than silently forcing one.
12. **Human-review clear is scoped to mechanically current artifacts** — mixed-plan/source-invalid books cannot report human-review-clear even if stale historical review files exist.


## Durable Phase 29 decisions

1. **Human review records are append-only** — later decisions append; earlier decisions remain audit history.
2. **Current human decisions are context-conservative and fingerprint-bound** — SHA-256 fingerprints of the full chapter source and full chapter translation, plus translation-context and translation-plan fingerprints, must all match. A neighboring paragraph/context change therefore stales an earlier sign-off even when the selected paragraph text itself did not change.
3. **Target identity and content identity are separate** — opaque target IDs identify a stable location; fingerprints decide whether a specific human record is still current.
4. **Human authority is explicit** — only a human submission can create `clear`, `accepted_as_is` or `needs_revision`. Automated evidence never manufactures these states.
5. **Accepted literary choices are first-class** — `accepted_as_is` exists because automated literary evidence can incorrectly penalize deliberate creative/cultural choices. It requires a note/finding and cannot coexist with a human critical finding.
6. **Needs-revision must be meaningful** — it requires at least one warning or critical human finding.
7. **Span offsets are Unicode character offsets** — validate against the exact current source/translation text before persistence; never store unchecked byte offsets.
8. **The ledger does not automatically copy book prose** — engine-written fields are IDs, fingerprints, spans, outcome, reviewer label and timestamp. Human-authored notes remain local runtime data.
9. **No silent history deletion** — bounded capacity fails closed instead of pruning old human decisions.
10. **Sample completion is not a book score** — `sample_review_complete` only means all current selected targets are resolved by current human records and mechanical artifacts are current.
11. **Export and canon remain separate authorities** — Phase 29 does not auto-block/approve export and does not promote anything to Character Bible/glossary canon.
12. **No new runtime dependency** — reuse Phase-19 dimension/severity types and native Rust persistence; no QE model, LLM judge, database, cloud review tool or telemetry SDK.
