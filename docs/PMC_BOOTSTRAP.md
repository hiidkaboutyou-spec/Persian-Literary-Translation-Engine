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
- `quality-engine` evaluates output; quality evidence is not human approval.
- `human-review-workflow` owns review lifecycle and promotion decisions.
- `project-engine` owns persistence/application orchestration, atomic project operations, recovery, and state tracking.
- `ApplicationService` is the product/application boundary for future UI surfaces.

## Memory Model — Important Distinction

The translation engine already has runtime/project memory for the book itself: translation memory, glossary, character bible, relationships, literary context, decisions, checkpoints, and project state.

PMC is separate. PMC stores durable engineering/project knowledge for future development sessions: architecture decisions, constraints, roadmap, implementation status, debugging lessons, and handoff context. PMC must not replace the engine's runtime literary memory.

## Current Canonical State

Canonical merged state on `main` is through Phase 16.

- Phase 13: Manuscript Intelligence & Literary Context Seeding — merged.
- Phase 14: Literary Intelligence Review & Canon Promotion — merged.
- Phase 15: Advanced Model-Assisted Literary Analysis — merged.
- Phase 16: Application Orchestration Layer — merged.

Use `docs/IMPLEMENTATION_STATUS.md` as the detailed implementation inventory and `docs/IMPLEMENTATION_ROADMAP.md` as the delivery map.

## Branch-Scoped Current Work

Phase 17 is currently branch-only and must not be described as canonical until merged.

Branch: `phase-17-external-integrations`
PR: #90 — `Phase 17: BookForge EPUB + optional COMET quality integration`

Phase 17 decisions:

1. BookForge is integrated selectively for deterministic validated EPUB ingestion, pinned to a known upstream revision. Native `document-engine` remains the domain contract exposed to the rest of the system.
2. The legacy EPUB parser remains available as an explicit compatibility build path. A BookForge validation failure must not silently fall back to the permissive parser.
3. COMET/XCOMET/DocCOMET is an optional Python sidecar behind a JSON/process boundary; Python does not become part of the Rust domain core.
4. External quality metrics are evidence only and can never set human-approved/canon state.
5. ContextWeaver, TranslateBooksWithLLMs, TransAgents, and similar repositories are architecture/research references unless a specific capability is missing from the native engine. Do not import whole external architectures into the project.
6. External dependency upgrades require license/privacy review, reproducible lockfile updates, CI, audit, release build, and smoke-test validation.
7. No manuscript text, API credentials, model credentials, or secrets belong in Git.

## Current Roadmap

Follow `docs/IMPLEMENTATION_ROADMAP.md`.

After Phase 17 is merged and post-merge verified, the intended sequence is:

- Phase 18 — Context Packet v2 & Selective Long-Novel Retrieval
- Phase 19 — Literary Fidelity & Persian Naturalness Review Stack
- Phase 20 — Publication-Grade EPUB Round Trip
- Phase 21 — Literary Evaluation Corpus & Benchmarking
- Phase 22 — Product Surface & Distribution Hardening

Do not start a later numbered phase by silently abandoning validation or documentation of the active phase.

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
- optional COMET-family evidence
- final human judgment

## External Integration Policy

Prefer extracting one well-defined capability over adopting another repository's orchestration model.

Direct dependency is acceptable only when:

- the capability is materially better than the native implementation,
- licensing is compatible,
- the dependency is pinned/reproducible,
- failure semantics are explicit,
- native persistence and review contracts remain intact,
- tests cover both the integration and its compatibility/fallback boundary.

Otherwise use the external project only as a design reference and implement the needed behavior natively.

## Development Safety Rules

Before merge:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- full workspace tests
- compatibility-path checks when an optional feature is introduced
- Python sidecar syntax/protocol checks when applicable
- `cargo audit`
- release CLI build
- CLI smoke tests

Never mark a phase merged/released based only on branch implementation. Verify target-branch evidence after merge.

## Source-of-Truth Files

- `AGENTS.md` — engineering/agent constraints
- `docs/IMPLEMENTATION_STATUS.md` — implemented capabilities
- `docs/IMPLEMENTATION_ROADMAP.md` — current roadmap
- `docs/ENGINEERING_DECISIONS.md` — durable engineering decisions
- `docs/AI_MEMORY_PIPELINE.md` and `docs/HYBRID_MEMORY_SEARCH_ARCHITECTURE.md` — runtime memory design
- `docs/EXTERNAL_INTEGRATIONS.md` — Phase 17 external dependency boundaries (branch-scoped until merged)

## PMC Promotion Guidance

When a local PMC vault is available, promote the stable sections above into normal PMC notes rather than copying this file as one giant note:

- Project / Project Home — objective and architecture overview
- Current State — canonical main state + separately labelled active branch
- Decisions — Rust-first boundary, human approval boundary, external-integration policy, COMET advisory-only policy
- Constraints — no silent fallback, no secret/manuscript commits, preserve source/context fingerprints
- Plans — roadmap phases 17–22
- Handoff — current PR/CI position and immediate next actions

Branch-only claims from Phase 17 should retain `applies_to_branch: phase-17-external-integrations` and an implementation state of `implemented`, not `merged`, until verified on `main`.
