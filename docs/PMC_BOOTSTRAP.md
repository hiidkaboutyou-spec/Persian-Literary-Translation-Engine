# PMC Bootstrap — Persian Literary Translation Engine

Purpose: durable repository-side seed for Project Memory Core when this repository is connected to a local Obsidian-compatible PMC vault.

Project ID: `persian-literary-translation-engine`
Repository: `hiidkaboutyou-spec/Persian-Literary-Translation-Engine`
Primary branch: `main`

## Project Objective

Build a production-grade English-to-Persian literary translation engine that reads naturally in Persian while preserving authorial intent, narrative structure, character voice, relationships, emotional subtext, continuity, terminology, and literary effect.

## Durable Product Requirements

- Avoid literal word-for-word Persian when it damages naturalness, voice, emotion, humor, intimacy, sarcasm, or narrative intent.
- Naturalization must not become invention, omission, censorship, or semantic drift.
- Character voice and relationship/register changes must remain consistent across long novels.
- Translation memory, glossary, character knowledge, literary findings, and human decisions must be traceable and passage-relevant.
- Human review remains the final approval/canon boundary.
- Long books must be resumable without silently reusing outputs generated under stale source or context.
- Publication output should support high-fidelity Persian DOCX and, through the roadmap, EPUB.

## Architecture Invariants

- Rust-first core.
- `document-engine` owns document ingestion/structure/provenance.
- `literary-intelligence-engine` owns deterministic literary understanding and inferred seeds.
- `advanced-literary-analysis` owns optional provider-assisted bounded literary findings.
- `memory-engine` owns translation memory and glossary knowledge.
- `character-engine` owns canonical character/relationship knowledge.
- `translation-core` consumes approved/bounded context and executes translation; it does not own literary canon.
- `quality-engine` evaluates output; probabilistic/model quality evidence is not human approval.
- `human-review-workflow` owns review lifecycle and promotion decisions.
- `project-engine` owns persistence/application orchestration, atomic operations, recovery, and state tracking.
- `ApplicationService` is the application boundary for UI/product surfaces.
- Heavy models, validators, and developer-memory tools must not silently become runtime requirements.

## Memory Ownership — Keep These Separate

### Translation runtime/project memory

Product data owned by the Rust engine: translation memory, glossary, character bible, relationships, literary context, review decisions, checkpoints, fingerprints, and project state. This data can affect translated output.

### Project Memory Core (PMC)

The intended curated durable engineering knowledge store: architecture decisions, constraints, roadmap, implementation status, debugging lessons, dependency decisions, and handoffs. A local Obsidian-compatible PMC vault remains the long-term target. This file is its repository-side seed until local PMC is connected.

### projectmem

An optional operational coding-history companion for issues, attempts, fixes, decisions, notes, and pre-edit checks. It supplements PMC and repository docs; it does not replace either and never replaces Rust runtime memory.

Never put proprietary manuscript text, generated book translations, credentials, API/model keys, or private reviewer material into PMC/projectmem as a substitute for native runtime memory.

See `docs/PROJECT_MEMORY_INTEGRATIONS.md` for research and detailed boundaries.

## Current Canonical State

Canonical `main` includes Phases 13–17 and the supporting tooling below.

- Phase 13 — Manuscript Intelligence & Literary Context Seeding — merged.
- Phase 14 — Literary Intelligence Review & Canon Promotion — merged.
- Phase 15 — Advanced Model-Assisted Literary Analysis — merged.
- Phase 16 — Application Orchestration Layer — merged.
- Phase 17 — Controlled External Integrations — merged via PR #90, merge commit `6a4b8807d8c54878f1f10db5cab1f1290fcc60fb`.
- Safe Persian language diagnostics + EPUB validation tooling — merged via PR #91, merge commit `ee76e6c0a1a01ce85f030e027d6306c7371638b6`.
- Isolated projectmem developer-memory tooling — merged via PR #92, merge commit `fa84b13313fce3b0b611cd64dc06d4fb088ad422`.

## Canonical External / Supporting Tool Decisions

1. **BookForge** — validated EPUB ingestion boundary, pinned to `23f8c9d3c97a06f48e13424698441bfb4b037844`; native `document-engine` remains the domain contract. No silent fallback after a BookForge failure.
2. **COMET/XCOMET/DocCOMET** — optional Python sidecar behind JSON/process boundary; advisory evidence only, never approval/canon.
3. **Lingua 1.8.0** — optional English/Persian-only language diagnostic under `language-diagnostics`, disabled by default; cannot rewrite or independently reject literary output.
4. **EPUBCheck 5.3.0** — optional checksum-pinned publication validator; binary not vendored; missing Java/tool cannot break current runtime paths.
5. **projectmem 0.3.3** — optional developer-side coding memory, release commit `e8d73137acde6f091ef6196f88ba5eccf6eb0e8a`, MIT. Installed only into isolated `.venv/projectmem` and not referenced by Rust runtime.
6. **projectmem safe profile** — disable Git hooks, watcher, Git-history backfill, global-memory inheritance, automatic `AGENTS.md`/`CLAUDE.md` edits, and automatic MCP-config output. Raw/runtime files are gitignored. Codex MCP config is printed by helper, not silently written to a developer home directory.
7. **Memory hierarchy** — PMC = curated durable engineering truth; projectmem = operational coding history; native Rust stores = translation/book runtime memory. Do not merge these ownership domains.
8. **ContextWeaver / TranslateBooksWithLLMs / TransAgents** — architecture/research references unless a concrete native gap is demonstrated.
9. External dependency/tool upgrades require license/privacy/security review and tests appropriate to that surface.

## Project-Memory Research Decisions

- **projectmem** — selected companion because it is local-first, MIT, Codex/MCP-oriented, event-sourced, and can be safely isolated with upstream opt-outs. Dedicated CI installs the exact selected version, audits its Python dependency graph, and verifies safe-init does not create/alter hooks, `AGENTS.md`/`CLAUDE.md`, or watcher state.
- **Serena** — useful Markdown/progressive-disclosure memory design, but current full application from v2 onward is GPL-3.0-or-later and includes broader semantic editing/language-server ownership; reference only.
- **Global Agent Memory** — MIT/local-first reference but overlaps PMC and adds a broader/newer storage/dashboard surface; no dependency now.
- **MemoryWiki** — relevant Markdown/local-first ideas, but current maturity and unresolved repository SPDX metadata do not justify a dependency.
- **automatic transcript/session memory systems** — not selected; raw chat capture is the wrong abstraction and increases privacy/secret-retention risk.

## Deferred Translation Dependencies

Only introduce these when their owning roadmap phase and benchmark justify them:

- **FlagEmbedding / BGE-M3** — Phase 18 semantic retrieval/reranking candidate; model downloads optional.
- **Hazm** — Phase 19 Persian linguistic diagnostics candidate; isolated Python, no auto-rewrite.
- **DadmaTools** — conditional Phase 19 candidate only if a measured gap remains.
- **Vecalign** — Phase 19/21 alignment/omission evidence candidate; dataset/license review required.
- **SacreBLEU / chrF++** — Phase 21 benchmark candidate after a rights-safe EN→FA reference corpus exists.
- `booktrans`, Tolmach/KazKozDev book translators, `epublate`, ArmenianLitTranslator — research/design references under their documented licensing/provenance boundaries.

## Current Roadmap

Follow `docs/IMPLEMENTATION_ROADMAP.md`.

- Phase 18 — Context Packet v2 & Selective Long-Novel Retrieval
- Phase 19 — Literary Fidelity & Persian Naturalness Review Stack
- Phase 20 — Publication-Grade EPUB Round Trip
- Phase 21 — Literary Evaluation Corpus & Benchmarking
- Phase 22 — Product Surface & Distribution Hardening

Supporting tooling may land between phases only when it leaves runtime defaults intact, has an explicit owner/failure boundary, and passes validation. It does not count as completion of a future numbered phase.

## Quality Philosophy

No single automatic metric is authoritative for literary translation. Combine deterministic structure/safety checks, omission/addition evidence, semantic faithfulness, character voice, relationship/register consistency, Persian naturalness/readability, subtext/emotional-effect preservation, terminology/continuity, optional language/COMET evidence, and final human judgment.

## Development Safety Rules

Use checks appropriate to the changed surface: reproducible dependency state, rustfmt/Clippy/workspace tests for Rust changes, compatibility and optional-feature tests, sidecar/wrapper tests, vulnerability audits, release build/CLI smoke where relevant, and dedicated isolated CI for developer-only tooling. Do not make projectmem a hidden prerequisite for normal Rust CI.

Never mark branch-only work merged based only on implementation; verify `main` after merge and update this seed when canonical state changes.

## Source-of-Truth Files

- `AGENTS.md` — engineering/agent constraints
- `docs/IMPLEMENTATION_STATUS.md` — implemented product capabilities
- `docs/IMPLEMENTATION_ROADMAP.md` — roadmap
- `docs/ENGINEERING_DECISIONS.md` — engineering decisions
- `docs/AI_MEMORY_PIPELINE.md` and `docs/HYBRID_MEMORY_SEARCH_ARCHITECTURE.md` — runtime memory design
- `docs/EXTERNAL_INTEGRATIONS.md` — translation dependency/research boundaries
- `docs/PROJECT_MEMORY_INTEGRATIONS.md` — developer/agent memory research and ownership boundaries
- this file — bootstrap seed for eventual local PMC promotion

## PMC Promotion Guidance

When a local PMC vault is available, promote stable material into focused notes:

- Project / Project Home — objective and architecture overview
- Current State — canonical `main` state
- Decisions — Rust-first boundary, human-approval boundary, external-integration policy, project-memory ownership, selected/deferred dependencies
- Constraints — no silent fallback, no secrets/manuscripts in developer memory, preserve source/context fingerprints, optional tools stay optional
- Plans — phases 18–22
- Handoff — latest canonical commit/PR state and immediate next action
