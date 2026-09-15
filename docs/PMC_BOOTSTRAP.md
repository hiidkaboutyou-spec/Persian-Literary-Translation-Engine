# PMC Bootstrap — Persian Literary Translation Engine

Purpose: provide a concise, durable seed for Project Memory Core when this repository is connected to a local Obsidian-compatible PMC vault.

Project ID: `persian-literary-translation-engine`
Repository: `hiidkaboutyou-spec/Persian-Literary-Translation-Engine`
Primary branch: `main`

## Project Objective

Build a production-grade English-to-Persian literary translation engine that produces natural Persian which reads as if originally written in Persian while preserving the source author's intent, narrative structure, character voice, relationships, emotional subtext, continuity, terminology, and literary effect.

## Durable Product Requirements

- Avoid literal, word-for-word Persian when it damages naturalness, voice, emotion, humor, intimacy, sarcasm, or narrative intent.
- Preserve meaning and authorial intent; naturalization must not become invention, omission, censorship, or semantic drift.
- Character voice and relationship/register changes must remain consistent across long novels.
- Translation memory, glossary, character knowledge, literary findings, and human decisions must be traceable and chapter/passage relevant.
- Human review remains the final approval/canon boundary.
- Long books must be resumable without silently reusing outputs generated under stale source or context.
- Publication output should ultimately support high-fidelity Persian DOCX and EPUB workflows.

## Architecture Invariants

- Rust-first core.
- `document-engine` owns document ingestion/structure/provenance.
- `literary-intelligence-engine` owns deterministic literary understanding and inferred seeds.
- `advanced-literary-analysis` owns optional provider-assisted bounded literary findings.
- `memory-engine` owns translation memory and glossary knowledge.
- `character-engine` owns canonical character/relationship knowledge.
- `translation-core` consumes approved/bounded context and executes translation; it must not become the owner of literary canon.
- `quality-engine` evaluates output; probabilistic/model quality evidence is not human approval.
- `human-review-workflow` owns review lifecycle and promotion decisions.
- `project-engine` owns persistence/application orchestration, atomic project operations, recovery, and state tracking.
- `ApplicationService` is the product/application boundary for future UI surfaces.
- Heavy models and optional tools must not silently become runtime requirements.

## Memory Model — Important Distinction

The translation engine already has runtime/project memory for the book itself: translation memory, glossary, character bible, relationships, literary context, decisions, checkpoints, and project state.

PMC is separate. PMC stores durable engineering/project knowledge for future development sessions: architecture decisions, constraints, roadmap, implementation status, debugging lessons, dependency decisions, and handoff context. PMC must not replace the engine's runtime literary memory.

## Current Canonical State

Canonical merged state on `main` is through **Phase 17**.

- Phase 13: Manuscript Intelligence & Literary Context Seeding — merged.
- Phase 14: Literary Intelligence Review & Canon Promotion — merged.
- Phase 15: Advanced Model-Assisted Literary Analysis — merged.
- Phase 16: Application Orchestration Layer — merged.
- Phase 17: Controlled External Integrations — merged via PR #90, merge commit `6a4b8807d8c54878f1f10db5cab1f1290fcc60fb`.

Phase 17 canonical decisions:

1. BookForge is integrated selectively for deterministic validated EPUB ingestion, pinned to `23f8c9d3c97a06f48e13424698441bfb4b037844`. Native `document-engine` remains the domain contract exposed to the rest of the system.
2. The legacy EPUB parser remains available as an explicit compatibility build path. A BookForge validation failure must not silently fall back to the permissive parser.
3. Hostile/malformed archive preflight failures remain strict BookForge failures but are normalized to the public `CorruptedFile` error contract; invalid EPUB structure remains `InvalidStructure`.
4. COMET/XCOMET/DocCOMET is an optional Python sidecar behind a JSON/process boundary; Python does not become part of the Rust domain core.
5. External quality metrics are evidence only and can never set human-approved/canon state.
6. ContextWeaver, TranslateBooksWithLLMs, TransAgents, and similar repositories are architecture/research references unless a specific capability is missing from the native engine. Do not import whole external architectures into the project.
7. External dependency upgrades require license/privacy review, reproducible lockfile updates, CI, audit, release build, and smoke-test validation.
8. No manuscript text, API credentials, model credentials, or secrets belong in Git.

## Current Branch-Scoped Supporting Work

Branch: `chore/safe-quality-tooling`
Implementation state: implemented on branch, not canonical until merged.

Purpose: add low-risk supporting tools without changing current runtime defaults or pretending to complete a future numbered phase.

Branch decisions:

- `lingua-rs` is pinned to `1.8.0`, Apache-2.0, optional under `quality-engine` feature `language-diagnostics`.
- Lingua default features are disabled; only English and Persian models are enabled.
- Lingua output is advisory language/leakage evidence only. It does not modify the existing deterministic quality result, rewrite text, approve text, or reject literary multilingual content by itself.
- EPUBCheck is pinned to production release `5.3.0`, BSD-3-Clause, official release SHA-256 `6c07e68584b2e2ce2f89fe06e1246dfead3eb36b46b340e7d93524f29dcff6c5`.
- EPUBCheck binaries are not vendored. The optional installer downloads the official archive, verifies its checksum, and stores it under ignored local `.tools/` storage.
- Missing Java/EPUBCheck must never break ingestion, translation, DOCX export, or the existing runtime. EPUBCheck becomes a publication conformance gate only when the EPUB publishing path reaches Phase 20.
- CI must separately compile/test `quality-engine --features language-diagnostics` and syntax-check EPUBCheck wrapper scripts.

## Researched Dependencies — Durable Decisions

These were deliberately **not installed now** because installing a tool is not automatically useful if its owning phase, benchmark, resource limits, or legal inputs do not exist yet.

- **FlagEmbedding / BGE-M3** — strong Phase 18 candidate for multilingual semantic retrieval/reranking. Keep model downloads optional; native deterministic memory/canon remains owner. Do not install before the context-packet/retrieval contract and benchmark exist.
- **Hazm** — Phase 19 candidate for Persian linguistic diagnostics beyond native normalization. Isolated Python environment only; never auto-rewrite literary prose; no model download in default CI.
- **DadmaTools** — conditional Phase 19 candidate only if a measured NER/syntax gap remains after native code/Hazm. Do not install both toolkits by default.
- **Vecalign** — Phase 19/21 candidate for source/translation alignment and omission evidence. Review dataset licenses separately; do not copy bundled GPL test/dev material into the project without a deliberate licensing decision.
- **SacreBLEU / chrF++** — Phase 21 benchmark candidate only after a rights-safe EN→FA literary reference corpus exists. Metric evidence is not literary judgment.
- **booktrans** — architecture reference for whole-book scouting/selective context/editor-verifier patterns; do not replace native orchestration.
- **Tolmach / KazKozDev/book-translator** — patch-refinement/verifier ideas only; AGPL boundary means no code copy/dependency without deliberate relicensing analysis.
- **madpin/epublate** — EPUB round-trip ideas only until licensing/provenance is independently verified.
- **ArmenianLitTranslator** — critic-role/evaluation-dimension research reference only.

## Current Roadmap

Follow `docs/IMPLEMENTATION_ROADMAP.md`.

Next numbered phases:

- Phase 18 — Context Packet v2 & Selective Long-Novel Retrieval
- Phase 19 — Literary Fidelity & Persian Naturalness Review Stack
- Phase 20 — Publication-Grade EPUB Round Trip
- Phase 21 — Literary Evaluation Corpus & Benchmarking
- Phase 22 — Product Surface & Distribution Hardening

Supporting tooling can land between phases only if it leaves runtime defaults intact, has an explicit owner/boundary, and passes full validation. It does not count as completion of a future numbered phase.

## Quality Philosophy

No single automatic metric is authoritative for literary translation. Evaluation should combine:

- deterministic structural and safety checks
- omission/addition detection
- semantic faithfulness
- character voice consistency
- relationship/register consistency
- Persian naturalness and readability
- literary subtext/emotional-effect preservation
- terminology and cross-chapter continuity
- optional language/COMET-family evidence
- final human judgment

## External Integration Policy

Prefer extracting one well-defined capability over adopting another repository's orchestration model.

Direct dependency is acceptable only when:

- the capability is materially better than the native implementation,
- licensing is compatible,
- the dependency is pinned/reproducible,
- failure semantics are explicit,
- native persistence and review contracts remain intact,
- runtime/default dependency cost is justified,
- tests cover both the integration and its compatibility/fallback boundary.

Otherwise use the external project only as a design reference and implement the needed behavior natively.

## Development Safety Rules

Before merge:

- lockfile reproducibility check
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- full workspace tests
- compatibility-path checks when an optional feature is introduced
- optional-feature tests when an optional dependency is introduced
- sidecar/wrapper syntax and protocol checks where applicable
- `cargo audit`
- release CLI build
- CLI smoke tests
- Security workflow

Never mark branch-only work merged/released based only on implementation. Verify target-branch evidence after merge.

## Source-of-Truth Files

- `AGENTS.md` — engineering/agent constraints
- `docs/IMPLEMENTATION_STATUS.md` — implemented capabilities
- `docs/IMPLEMENTATION_ROADMAP.md` — current roadmap
- `docs/ENGINEERING_DECISIONS.md` — durable engineering decisions
- `docs/AI_MEMORY_PIPELINE.md` and `docs/HYBRID_MEMORY_SEARCH_ARCHITECTURE.md` — runtime memory design
- `docs/EXTERNAL_INTEGRATIONS.md` — dependency/research boundaries
- this file — bootstrap seed for eventual local PMC promotion

## PMC Promotion Guidance

When a local PMC vault is available, promote stable sections above into normal PMC notes rather than copying this file as one giant note:

- Project / Project Home — objective and architecture overview
- Current State — canonical `main` state + separately labelled active branch
- Decisions — Rust-first boundary, human approval boundary, external-integration policy, dependency decisions
- Constraints — no silent fallback, no secret/manuscript commits, preserve source/context fingerprints, optional heavy tools stay optional
- Plans — roadmap phases 18–22
- Handoff — active branch/PR/CI position and immediate next actions

Branch-only claims from `chore/safe-quality-tooling` must retain branch scope and implementation state `implemented`, not `merged`, until verified on `main`.
