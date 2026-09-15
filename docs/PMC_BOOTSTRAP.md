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
- Heavy models, validators, and developer-memory tools must not silently become runtime requirements.

## Memory Model — Important Distinction

There are three deliberately separate memory domains.

### Translation runtime/project memory

The Rust engine owns book data: translation memory, glossary, character bible, relationships, literary context, decisions, checkpoints, fingerprints, review state, and project state. This is product data and can affect translated output.

### Project Memory Core (PMC)

PMC is the intended curated durable engineering knowledge store for future development sessions: architecture decisions, constraints, roadmap, implementation status, debugging lessons, dependency decisions, and handoff context. A local Obsidian-compatible PMC vault is the long-term target. This file is the repository-side bootstrap seed until that vault is connected.

### projectmem

`projectmem` is an optional operational coding-history companion: issues, attempts, fixes, decisions, notes, and pre-edit checks. It supplements PMC but does not replace PMC, repository documentation, or Rust runtime memory. It must never contain proprietary manuscript text, generated book translations, credentials, or private reviewer material.

See `docs/PROJECT_MEMORY_INTEGRATIONS.md` for the research and boundaries.

## Current Canonical State

Canonical `main` includes Phases 13–17 plus the safe supporting-quality tooling merged after Phase 17.

- Phase 13: Manuscript Intelligence & Literary Context Seeding — merged.
- Phase 14: Literary Intelligence Review & Canon Promotion — merged.
- Phase 15: Advanced Model-Assisted Literary Analysis — merged.
- Phase 16: Application Orchestration Layer — merged.
- Phase 17: Controlled External Integrations — merged via PR #90, merge commit `6a4b8807d8c54878f1f10db5cab1f1290fcc60fb`.
- Safe Persian language diagnostics + EPUB validation tooling — merged via PR #91, merge commit `ee76e6c0a1a01ce85f030e027d6306c7371638b6`.

### Canonical external-tool decisions

1. BookForge is integrated selectively for deterministic validated EPUB ingestion, pinned to `23f8c9d3c97a06f48e13424698441bfb4b037844`; native `document-engine` remains the domain contract.
2. The legacy EPUB parser is an explicit compatibility build path, never a silent fallback after BookForge failure.
3. Malformed archive/decompression failures remain strict but are normalized to the public `CorruptedFile` contract; invalid EPUB structure remains `InvalidStructure`.
4. COMET/XCOMET/DocCOMET is an optional Python sidecar behind a JSON/process boundary and can provide evidence only, never approval/canon.
5. Lingua 1.8.0 is optional English/Persian-only diagnostic evidence under feature `language-diagnostics`, disabled by default. It cannot rewrite or independently reject literary output.
6. EPUBCheck 5.3.0 is an optional checksum-pinned external publication validator; its binary is not vendored and missing Java/EPUBCheck cannot break current runtime paths.
7. ContextWeaver, TranslateBooksWithLLMs, TransAgents, and similar repositories remain references where native capabilities already own the problem.
8. External dependency changes require license/privacy review, reproducible dependency state, tests, security checks, release build, and smoke validation.
9. No manuscript text, API/model credentials, secrets, or private review content belongs in Git or developer-memory tools.

## Current Branch-Scoped Supporting Work

Branch: `chore-project-memory-tooling`
Implementation state: branch-only until its PR is merged and `main` is re-verified.

Purpose: add a safe optional developer-memory companion without coupling it to the translation product.

Branch decisions:

- selected upstream: `riponcm/projectmem` release `0.3.3`, release commit `e8d73137acde6f091ef6196f88ba5eccf6eb0e8a`, MIT;
- install only into isolated `.venv/projectmem` using a supported Python 3.10–3.12 interpreter;
- safe initialization disables Git hooks, watcher, Git-history backfill, global-memory inheritance, automatic `AGENTS.md`/`CLAUDE.md` modification, and automatic MCP-config output;
- projectmem may maintain local `.projectmem/` operational memory and a machine-local project registry, but it is not application/project persistence;
- raw runtime projectmem state is gitignored;
- Codex MCP configuration is printed by a helper, never silently written into a developer's home directory;
- dedicated CI must prove safe initialization does not create/alter hooks, bridge files, or watcher state;
- projectmem failure/absence must never block Rust build, translation, review, or export.

Research decisions retained:

- **Serena** — strong memory/code-intelligence design reference, but current full application is GPL-3.0-or-later and broader than the required memory role; do not add as a dependency.
- **Global Agent Memory** — MIT/local-first reference, but overlaps PMC and adds a broader/newer storage/dashboard surface; no dependency now.
- **MemoryWiki** — Markdown/local-first ideas are relevant, but current maturity and unresolved repository SPDX metadata do not justify a dependency.
- **automatic chat/session memory systems** — not selected because transcript capture is the wrong abstraction and increases privacy/secret-retention risk.

## Researched Translation Dependencies — Durable Decisions

These remain deliberately deferred until their owning roadmap phase and benchmark exist.

- **FlagEmbedding / BGE-M3** — Phase 18 semantic retrieval/reranking candidate; optional models only.
- **Hazm** — Phase 19 Persian linguistic diagnostics candidate; isolated Python only, no auto-rewrite.
- **DadmaTools** — conditional Phase 19 candidate only if a measured gap remains.
- **Vecalign** — Phase 19/21 alignment/omission evidence candidate; dataset/license review required.
- **SacreBLEU / chrF++** — Phase 21 benchmark candidate after a rights-safe EN→FA reference corpus exists.
- **booktrans** — architecture reference only.
- **Tolmach / KazKozDev/book-translator** — verifier/refinement ideas only; AGPL boundary.
- **madpin/epublate** — EPUB round-trip ideas only until licensing/provenance is independently verified.
- **ArmenianLitTranslator** — critic/evaluation research reference only.

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

No single automatic metric is authoritative for literary translation. Evaluation should combine deterministic structural/safety checks, omission/addition detection, semantic faithfulness, voice and relationship/register consistency, Persian naturalness/readability, emotional/subtext preservation, terminology/continuity, optional language/COMET evidence, and final human judgment.

## External Integration Policy

Prefer extracting one well-defined capability over adopting another repository's orchestration model. Direct dependency is acceptable only when the capability is materially better than the native implementation, licensing is compatible, the dependency is reproducible, failure semantics are explicit, native persistence/review contracts remain intact, runtime cost is justified, and tests cover the integration boundary. Otherwise keep the project as a design reference.

## Development Safety Rules

Before merge, require the checks relevant to the changed surface: lockfile/dependency reproducibility, rustfmt, Clippy, full workspace tests, compatibility/optional-feature tests, sidecar/wrapper tests, `cargo audit`, release build, CLI smoke, and Security workflow. Developer-only tooling gets its own isolated CI and must not become a hidden Rust-CI prerequisite.

Never mark branch-only work merged/released based only on implementation. Verify target-branch evidence after merge.

## Source-of-Truth Files

- `AGENTS.md` — engineering/agent constraints
- `docs/IMPLEMENTATION_STATUS.md` — implemented product capabilities
- `docs/IMPLEMENTATION_ROADMAP.md` — current roadmap
- `docs/ENGINEERING_DECISIONS.md` — durable engineering decisions
- `docs/AI_MEMORY_PIPELINE.md` and `docs/HYBRID_MEMORY_SEARCH_ARCHITECTURE.md` — runtime memory design
- `docs/EXTERNAL_INTEGRATIONS.md` — translation dependency/research boundaries
- `docs/PROJECT_MEMORY_INTEGRATIONS.md` — developer/agent memory research and ownership boundaries
- this file — bootstrap seed for eventual local PMC promotion

## PMC Promotion Guidance

When a local PMC vault is available, promote stable material into focused notes rather than copying this file as one giant note:

- Project / Project Home — objective and architecture overview
- Current State — canonical `main` state + separately labelled active branch
- Decisions — Rust-first boundary, human approval boundary, external-integration policy, project-memory ownership, selected/deferred dependencies
- Constraints — no silent fallback, no secret/manuscript commits, preserve source/context fingerprints, optional tools stay optional
- Plans — roadmap phases 18–22
- Handoff — active branch/PR/CI position and immediate next actions

Branch-only projectmem claims must retain `applies_to_branch: chore-project-memory-tooling` and an implementation state of `implemented`, not `merged`, until verified on `main`.
