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
- Publication output supports high-fidelity Persian DOCX and is being extended with source-preserving EPUB round trip in Phase 20.

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

## Phase 20 Branch State — Not Canonical Until Merged

Branch: `phase-20-publication-epub-roundtrip`.

Implemented branch work includes:

- BookForge source block IDs propagated into native `SourceLocation` and translated artifacts;
- marker-aware source-preserving EPUB reconstruction with explicit `block_id -> translated text` mapping;
- fail-closed rejection of missing, unknown, duplicate, empty, or mismatched block provenance instead of positional guessing;
- BookForge source-aware language rewrite plus native RTL `dir="rtl"` and OPF `page-progression-direction="rtl"` publication metadata;
- preservation of source-derived CSS, images, navigation, links, inline markup, notes/resources through reconstruction rather than plain-text EPUB regeneration;
- deterministic publication output and source non-mutation contract;
- ApplicationService/CLI selection between DOCX and EPUB publication;
- permanent read-only Phase 20 CI with generated rights-safe EPUB fixture, EPUBCheck 5.3.0 input/output checks, repeated-export byte comparison, source checksum, representative resource/markup assertions, no-default-features compatibility, and Apple Silicon arm64 validation;
- supporting explicit `adult-intimacy` fidelity style profile for confirmed-adult source material, while `literary` remains the default and automated intimacy evidence remains non-canonical.

Do not call Phase 20 canonical until its PR is merged and post-merge validation is green.

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

## Current Roadmap

- Phase 18 — Context Packet v2 & Selective Long-Novel Retrieval — canonical/merged.
- Phase 19 — Literary Fidelity & Persian Naturalness Review Stack — canonical/merged.
- Phase 20 — Publication-Grade EPUB Round Trip — active branch, not canonical until merge/post-merge verification.
- Phase 21 — Literary Evaluation Corpus & Benchmarking — next only after Phase 20 is canonical.
- Phase 22 — Product Surface & Distribution Hardening.

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
- `docs/PHASE_20_RESEARCH.md` — current publication research/decisions
- `docs/PHASE_19_RESEARCH.md` — literary-review dependency/review research
- `docs/ENGINEERING_DECISIONS.md` — durable engineering decisions
- `docs/AI_MEMORY_PIPELINE.md` and `docs/HYBRID_MEMORY_SEARCH_ARCHITECTURE.md` — runtime memory design
- `docs/EXTERNAL_INTEGRATIONS.md` — external integration boundaries
- `docs/PROJECT_MEMORY_INTEGRATIONS.md` — developer/agent memory ownership
- this file — repository-side PMC bootstrap seed

## PMC Promotion Guidance

When a local PMC vault is actually available, promote stable material into focused notes rather than copying this file wholesale: Project Home, Current State, Decisions, Constraints, Plans, and Handoff.

Updating this repository seed does **not** mean the user's local PMC/Obsidian vault or local Mac configuration was modified. Never claim that without direct local access and verification.
