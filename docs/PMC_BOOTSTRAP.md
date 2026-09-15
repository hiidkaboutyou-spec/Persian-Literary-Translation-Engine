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
- `memory-engine` owns translation memory, glossary knowledge, and Context Packet v2 retrieval contracts.
- `character-engine` owns canonical character/relationship knowledge.
- `translation-core` consumes approved/bounded context and executes translation; it does not own literary canon.
- `quality-engine` owns deterministic quality gates; probabilistic/model evidence is never human approval.
- `literary-review-engine` owns Phase 19 post-translation literary review evidence and revision proposals; it does not own canon or auto-apply revisions.
- `human-review-workflow` owns review lifecycle and promotion decisions.
- `project-engine` owns persistence/application orchestration, atomic operations, recovery, and state tracking.
- `ApplicationService` is the application boundary for CLI/UI/product surfaces.
- Heavy models, validators, and developer-memory tools must not silently become runtime requirements.

## Memory Ownership — Keep These Separate

### Translation runtime/project memory

Product data owned by the Rust engine: translation memory, glossary, character bible, relationships, literary context, review decisions, checkpoints, fingerprints, project state, and persisted Phase 19 review artifacts. This data can affect product behavior.

### Project Memory Core (PMC)

Curated durable engineering knowledge: architecture decisions, constraints, roadmap, implementation status, debugging lessons, dependency decisions, and handoffs. A local Obsidian-compatible PMC vault remains the long-term target. This file is only its repository-side seed until local PMC is actually connected.

### projectmem

Optional operational coding-history companion for issues, attempts, fixes, decisions, notes, and pre-edit checks. It supplements PMC/repository docs; it does not replace either and never replaces Rust runtime memory.

Never put proprietary manuscript text, generated book translations, credentials, API/model keys, or private reviewer material into PMC/projectmem as a substitute for native runtime memory.

## Current Canonical State

Canonical `main` is verified through Phase 18.

- Phase 13 — Manuscript Intelligence & Literary Context Seeding — merged.
- Phase 14 — Literary Intelligence Review & Canon Promotion — merged.
- Phase 15 — Advanced Model-Assisted Literary Analysis — merged.
- Phase 16 — Application Orchestration Layer — merged.
- Phase 17 — Controlled External Integrations — PR #90, merge commit `6a4b8807d8c54878f1f10db5cab1f1290fcc60fb`.
- Safe Persian language diagnostics + EPUB validation tooling — PR #91, merge commit `ee76e6c0a1a01ce85f030e027d6306c7371638b6`.
- Isolated projectmem developer-memory tooling — PR #92, merge commit `fa84b13313fce3b0b611cd64dc06d4fb088ad422`.
- Projectmem canonical-state docs — PR #93, merge commit `3b1dfb5a28261d6d2999ac1d530a51f4249e5550`.
- Phase 18 — Context Packet v2 & Selective Long-Novel Retrieval — PR #94, merge commit `2af408a19b8f69db93aff8e6896eaf189c4d69ae`.

## Phase 18 Durable Decisions

- Context Packet v2 is the shared context policy for CLI and `ApplicationService`.
- Packet items carry typed provenance/authority/reasoning metadata and SHA-256 fingerprints.
- Approved canon/reviewed evidence outranks unresolved inference.
- Deterministic lexical/polarity/diversity retrieval remains the safety floor.
- Optional BGE-M3/FastEmbed semantic retrieval/reranking augments candidate selection only.
- Missing/failing semantic tooling falls back to deterministic retrieval and cannot block translation.
- Semantic tools cannot invent IDs/canon/project memory.
- BGE model weights are not downloaded during normal build/default CI.
- Linux and Apple Silicon arm64 are required compatibility gates for the optional model adapter.

## Phase 19 Branch State — Not Canonical Until Merged

Branch: `phase-19-literary-review-stack`.

Phase 19 implementation currently contains:

- `literary-review-engine` with independent omission/addition, semantic fidelity, character voice, relationship/register, Persian naturalness, dialogue/subtext, and terminology/continuity dimensions;
- explicit unevaluated state when evidence was not actually produced;
- native deterministic review evidence;
- provider-neutral bounded critic with paragraph-index validation, requested-dimension enforcement, and revision proposals that never auto-apply;
- optional OpenAI critic adapter while native/mock review remains credential-free;
- native bounded monotonic Rust alignment supporting 1:1, 1:N, N:1, N:M, source-only and target-only gaps;
- optional `tools/literary-alignment` BGE adapter reusing the Phase 18 model boundary;
- strict sidecar response validation for unit identity, coverage, index bounds, monotonicity, finite values, and schema;
- post-translation `ApplicationService::review_translation` and `get_literary_review` APIs;
- per-chapter persisted review artifacts with source/translation/context fingerprints and staleness detection after manual edits;
- CLI `project review-translation` as a thin adapter over `ApplicationService`;
- permanent read-only Phase 19 CI for native review, application artifacts/staleness, alignment protocol, BGE compile, Apple Silicon, lockfiles, and security audits.

Do not call Phase 19 canonical until the PR is merged and `main` is post-merge verified.

## Phase 19 Dependency Decisions

1. **BGE-M3 / FastEmbed** — approved only as the existing optional Rust model boundary reused from Phase 18. No second embedding stack.
2. **Hazm 0.12.1** — blocked. Hazm itself is MIT and useful for Persian NLP, but it mandates NLTK and the compatible NLTK line is currently affected by unpatched High-severity `GHSA-8mgp-746c-j5xp` / `CVE-2026-81726`. Do not add an advisory waiver merely to enable Hazm. Re-evaluate after a patched compatible NLTK release and fresh audit.
3. **Vecalign** — Apache-2.0 core and useful algorithmic reference. Do not install while native Rust alignment meets the requirement. Vecalign requires Cython/C compilation and bundled Bleualign test/dev data has separate GPL licensing; never copy those datasets blindly.
4. **SentWeave 0.3.3** — audited reference only. Release provenance: `amajdalawi/sentweave@61e9af7086a4339448ab4b2c25b8a2071961d123`, Apache-2.0, PyPI sdist SHA-256 `ef6414bdd1d7fa4064f31fdf1b446c7f2601955777fcdf64988fc09bca9d2940`. One-off Linux/Apple-Silicon install/smoke/audit succeeded, but adding its Python/Cython surface is unnecessary while native Rust alignment works.
5. **DadmaTools** — evaluate only if a benchmark proves a concrete Persian NLP gap after native Phase 19 review.
6. **TransAgents / Armenian literary-agent projects** — role/critic design references only; do not import their orchestration or memory ownership.

See `docs/PHASE_19_RESEARCH.md` for detailed research.

## Canonical External / Supporting Tool Decisions

- **BookForge** — validated EPUB ingestion boundary, pinned revision; native document types remain canonical and validation failures never silently fall back.
- **COMET/XCOMET/DocCOMET** — optional Python quality-evidence sidecar; no automatic approval.
- **Lingua 1.8.0** — optional English/Persian-only language diagnostic, disabled by default; no rewriting/approval.
- **EPUBCheck 5.3.0** — optional checksum-pinned publication validator; distribution not vendored.
- **projectmem 0.3.3** — optional developer-side coding memory only; safe profile disables hooks, watcher, history backfill, global inheritance, and automatic bridge/MCP edits.
- **PMC vs projectmem vs runtime memory** — keep ownership domains separate.
- **ContextWeaver / TranslateBooksWithLLMs / TransAgents** — architecture references unless a concrete native gap is demonstrated.
- External upgrades require license/privacy/security/resource/failure-mode review plus appropriate tests.

## Supporting-Tool Shortlist After Phase 19

These are not current runtime dependencies and must not interrupt the numbered roadmap.

### OpenDataLoader PDF

Potentially useful for difficult PDF ingestion: structured Markdown/JSON/HTML, bounding boxes, reading order, tables, and OCR/hybrid handling. Before adoption, benchmark deterministic local mode against the current `document-engine` PDF path using project-owned fixtures. Keep Java/Python/AI-hybrid requirements optional and outside the Rust domain model.

### ripwire

Potentially useful developer-only code intelligence: C++23/tree-sitter symbol map, call graph, deterministic ranked context, and MCP. If adopted, install only as developer tooling; it must never become a build/runtime prerequisite or own project memory.

### Headroom

Potential developer/research context compression. Do not place it in literary runtime/provider context without a dedicated fidelity benchmark; lossy compression could alter evidence or nuance. It may be evaluated only for coding/research-agent tool output.

## Project-Memory Research Decisions

- **projectmem** selected as the operational companion because it is local-first, MIT, Codex/MCP-oriented, and safely isolated with upstream opt-outs.
- **Serena** useful reference but full application is GPL-3.0-or-later and broader than needed; reference only.
- **Global Agent Memory** overlaps PMC/projectmem and adds broader storage/dashboard surface; no dependency.
- **MemoryWiki** relevant ideas but current maturity/provenance do not justify dependency.
- **automatic transcript/session memory systems** not selected; raw chat capture is the wrong abstraction and increases privacy/secret-retention risk.

## Current Roadmap

- Phase 18 — Context Packet v2 & Selective Long-Novel Retrieval — canonical/merged.
- Phase 19 — Literary Fidelity & Persian Naturalness Review Stack — branch validation; merge required before canonical.
- Phase 20 — Publication-Grade EPUB Round Trip.
- Phase 21 — Literary Evaluation Corpus & Benchmarking.
- Phase 22 — Product Surface & Distribution Hardening.

Always finish/verify the current numbered phase before starting the next numbered phase. Supporting tooling may land between phases only when it leaves runtime defaults intact, has an explicit owner/failure boundary, and passes validation.

## Quality Philosophy

No single automatic metric is authoritative for literary translation. Combine deterministic structure/safety checks, omission/addition evidence, semantic faithfulness, character voice, relationship/register consistency, Persian naturalness/readability, subtext/emotional-effect preservation, terminology/continuity, optional language/COMET evidence, and final human judgment.

## Durable Dependency Rule

Before adding any GitHub repository/package/tool:

1. prove a concrete capability gap;
2. inspect license, release provenance, maintenance, dependencies, model/data licenses, privacy, and failure modes;
3. prefer native Rust when the missing capability is reasonably small and safer to own;
4. isolate heavy/model-backed tools behind explicit optional boundaries;
5. preserve deterministic fallback;
6. benchmark on project-owned fixtures;
7. validate Linux/macOS compatibility where relevant;
8. run vulnerability/security audits;
9. never let automated evidence become canon or human approval.

## Source-of-Truth Files

- `AGENTS.md` — engineering/agent constraints
- `docs/IMPLEMENTATION_STATUS.md` — implemented capabilities/current branch state
- `docs/IMPLEMENTATION_ROADMAP.md` — roadmap
- `docs/PHASE_19_RESEARCH.md` — Phase 19 dependency/review research
- `docs/ENGINEERING_DECISIONS.md` — engineering decisions
- `docs/AI_MEMORY_PIPELINE.md` and `docs/HYBRID_MEMORY_SEARCH_ARCHITECTURE.md` — runtime memory design
- `docs/EXTERNAL_INTEGRATIONS.md` — external integration boundaries
- `docs/PROJECT_MEMORY_INTEGRATIONS.md` — developer/agent memory ownership
- this file — repository-side PMC bootstrap seed

## PMC Promotion Guidance

When a local PMC vault is actually available, promote stable material into focused notes rather than copying this file wholesale:

- Project / Project Home — objective and architecture overview
- Current State — canonical `main` state only
- Decisions — Rust-first, human-approval boundary, dependency research/selection, memory ownership
- Constraints — no silent fallback, no secrets/manuscripts in developer memory, fingerprint safety, optional tools stay optional
- Plans — current/next phases
- Handoff — latest canonical commit/PR and immediate next action

Do not claim the local PMC vault was updated merely because this repository bootstrap changed.
