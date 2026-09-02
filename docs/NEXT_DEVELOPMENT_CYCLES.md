# Next Development Cycles

These cycles extend the existing four milestones while preserving the product goal: a usable Rust-based Persian literary translation system that can ingest long-form fiction, preserve context and character voice across chapters, generate high-quality Persian, evaluate consistency, and export a publication-ready manuscript.

## Delivery note — Phase 14

Phase 14 now implements the literary-intelligence portion of Cycles 9 and 10: persistent proposal
review, human decisions, reconciliation, typed canon conflicts, promotion preview/apply, and audit
lineage. The historical cycle numbering below is retained for roadmap continuity. Remaining Cycle 9
work is broader project orchestration and chapter-level review; remaining Cycle 10 work includes
deeper literary-rule inference and output-span traceability.

## Cycle 5 — Provider Integration & End-to-End Translation Runtime

### Goal
Turn the existing architecture into a real translation run that can process one chapter from input to reviewed output.

### Work
- Define a provider-neutral `TranslationProvider` trait in Rust.
- Implement request/response models for translation, revision, and quality-review passes.
- Build deterministic context assembly from glossary, character profiles, relationship context, and prior translation memory.
- Add retry, timeout, structured error, and resumable-job behavior.
- Wire document ingestion -> chapter extraction -> context builder -> provider -> quality engine -> output.
- Add a CLI command for a complete single-document run.
- Persist per-chapter run metadata and checkpoints.

### Exit criteria
- A local input document can be translated end to end through the Rust CLI.
- Interrupted runs can resume without retranslating completed chapters.
- Provider code is replaceable without changing the core pipeline.
- A run produces translated chapter output plus a machine-readable quality report.

## Cycle 6 — Durable Literary Memory & Retrieval

### Goal
Make long-form translation coherent across chapters instead of treating each chapter as an isolated prompt.

### Work
- Implement persistent storage for translation memory, glossary entries, character voice notes, names, honorifics, relationship state, and unresolved decisions.
- Add retrieval APIs scoped by project, chapter, character, and term.
- Implement exact glossary precedence before semantic retrieval.
- Add context-budget selection so only the most relevant memory is injected.
- Track provenance for every retrieved memory item.
- Add conflict detection for inconsistent names, terminology, POV, tense, and character voice.
- Add memory update rules after each approved chapter.

### Exit criteria
- Decisions made in earlier chapters are automatically reused later.
- Conflicting glossary/character decisions are surfaced before export.
- Memory survives process restarts.
- Retrieval behavior is covered by deterministic tests.

## Cycle 7 — Editorial Quality, Persian Publishing Export & User Workflow

### Goal
Move from raw translated text to a professional Persian literary manuscript workflow.

### Work
- Add multi-pass review stages for semantic fidelity, natural Persian prose, dialogue, character voice, continuity, and repetition.
- Introduce configurable quality thresholds and blocking/non-blocking findings.
- Build a Persian typography normalization layer for punctuation, ZWNJ, quotation marks, spacing, numerals, and paragraph structure.
- Implement DOCX export with RTL layout, heading hierarchy, chapter breaks, metadata, and stable book styling.
- Add plain-text/Markdown export for debugging and interoperability.
- Add CLI commands for `analyze`, `translate`, `review`, `resume`, and `export`.
- Produce a clear per-run summary of chapters completed, warnings, unresolved terms, and quality status.

### Exit criteria
- The engine produces a readable, consistently formatted Persian DOCX manuscript.
- A user can run the full workflow without editing internal project files manually.
- Quality failures are visible and actionable before final export.

## Cycle 8 — Evaluation, Reliability, Security & Release

### Goal
Make the engine dependable enough for repeated real-world use and a versioned release.

### Work
- Build a representative evaluation corpus covering dialogue-heavy fiction, emotional prose, long chapters, recurring terminology, and character-heavy scenes.
- Add regression tests for ingestion, chapter boundaries, memory retrieval, context assembly, provider failures, resume behavior, and export.
- Add quality benchmarks for consistency, glossary adherence, omission detection, and voice preservation.
- Add CI for formatting, linting, tests, release builds, dependency auditing, and security checks.
- Add structured logging without leaking source text or secrets.
- Harden secret/config handling and document local/private data guarantees.
- Add versioned configuration migrations.
- Produce release binaries and installation documentation.
- Run an end-to-end acceptance test on a complete multi-chapter project.

### Exit criteria
- CI is green on the supported platforms.
- The full acceptance corpus completes without data loss or unrecoverable pipeline failure.
- Release artifacts are reproducible.
- A new user can install the CLI, configure a provider, translate a project, resume it, review results, and export a final manuscript from documented instructions.

## Cycle 9 — Project Orchestration & Human-in-the-Loop Workflow

### Goal
Turn the released engine into a practical daily translation workspace where the user can start, inspect, correct, approve, and continue a book without touching internal files.

### Work
- Add a first-class project manifest describing source files, target language, provider configuration, style profile, memory store, checkpoints, and export settings.
- Add `project init`, `project status`, `project doctor`, and `project clean` CLI commands.
- Add explicit chapter states: pending, translating, review-needed, approved, blocked, exported.
- Add commands to approve/reject suggested glossary entries, character facts, and unresolved translation decisions.
- Add a safe retranslation flow for one paragraph/chapter without invalidating approved downstream memory unless requested.
- Add diff-oriented review output so users can inspect revision changes before accepting them.
- Make every destructive operation dry-run capable and require explicit confirmation flags in non-interactive mode.
- Add machine-readable JSON output for CLI commands so future desktop/UI integrations do not need to parse terminal prose.

### Exit criteria
- A complete book project can be managed using documented CLI commands only.
- The user can correct a decision and selectively propagate it without manually editing storage files.
- Project state is inspectable and recoverable after interruption or partial failure.
- All state-changing commands have deterministic tests and safe failure behavior.

## Cycle 10 — Literary Intelligence & Decision Traceability

### Goal
Improve translation quality without making the system opaque: every important literary decision should be explainable, reviewable, and grounded in project context.

### Work
- Add structured literary profiles for narrator voice, dialogue register, recurring imagery, character idiolect, intimacy level, humor/sarcasm, and relationship dynamics.
- Add scene-aware context selection so dialogue-heavy, action-heavy, reflective, and emotionally intense passages receive different relevant memory.
- Add omission/expansion detection using source-target structural comparison.
- Add contradiction detection between generated text and approved character/glossary facts.
- Track why each memory item was retrieved and which output spans it influenced where technically practical.
- Add confidence/severity scoring for findings while keeping deterministic rules separate from model-based judgments.
- Add an approval ledger for human decisions so later runs can distinguish model suggestions from user-approved canon.
- Build regression fixtures for difficult Persian literary phenomena such as colloquial dialogue, honorific shifts, code-switching, punctuation, and ZWNJ-sensitive forms.

### Exit criteria
- Quality reports identify omissions, terminology conflicts, voice drift, and continuity problems with actionable locations.
- Approved user decisions always override model suggestions.
- A user can inspect the provenance of important glossary, character, and continuity decisions.
- Literary-quality regression fixtures remain stable across provider/model upgrades.

## Cycle 11 — Large-Book Performance, Cost Control & Offline Resilience

### Goal
Make the engine efficient and dependable for full novels and very long fanfiction projects rather than only small test documents.

### Work
- Add streaming/chunked ingestion paths that avoid loading entire large documents into memory when unnecessary.
- Add bounded concurrency for provider calls with deterministic chapter ordering.
- Add configurable token/context budgets and per-run cost estimation before execution.
- Cache reusable analysis and retrieval results using content-addressed keys.
- Add incremental reprocessing so unchanged chapters are not re-analyzed or retranslated.
- Add provider rate-limit handling, exponential backoff, circuit breaking, and graceful pause/resume behavior.
- Add local/offline operation for analysis, memory inspection, review, and export when no provider is available.
- Benchmark memory use, throughput, checkpoint overhead, and export time on representative long-form projects.

### Exit criteria
- Large projects complete without unbounded memory growth.
- Re-running an unchanged project avoids redundant provider work.
- Rate limits or network loss do not corrupt project state.
- The CLI can estimate expected provider usage before a translation run and report actual usage afterward.

## Cycle 12 — Stable v1 Product, Compatibility & Distribution

### Goal
Freeze a dependable v1 contract and make the Rust engine straightforward to install, upgrade, automate, and extend without destabilizing core translation behavior.

### Work
- Define and version the public Rust interfaces for providers, storage adapters, document importers, quality checks, and exporters.
- Stabilize project/config schemas and provide tested migrations from all supported pre-v1 formats.
- Add compatibility tests for supported operating systems and document formats.
- Produce signed/versioned release artifacts where the distribution environment supports signing.
- Add installation paths appropriate for the project, including downloadable binaries and Cargo installation if feasible.
- Add a `doctor` command that verifies provider credentials, filesystem permissions, storage health, document support, and export prerequisites without exposing secrets.
- Publish a concise end-to-end user guide built around one real book workflow rather than architecture internals.
- Add release notes, upgrade/rollback instructions, backup/restore instructions, and a support/debug bundle that excludes source text and credentials by default.
- Run a final acceptance matrix covering new project creation, import, translation, interruption/resume, memory reuse, review, correction propagation, export, backup, restore, and upgrade.

### Exit criteria
- The v1 CLI and project format have explicit compatibility guarantees.
- A clean machine can install the engine and complete the documented end-to-end workflow.
- Upgrades preserve existing translation projects through tested migrations.
- The final acceptance matrix passes without data loss, silent omission, or secret leakage.

## Definition of Product Completion

The project is considered functionally complete when a user can:

1. Create a translation project.
2. Import a supported fiction document.
3. Automatically identify chapters and build project context.
4. Translate chapter-by-chapter through a replaceable provider.
5. Preserve terminology, character voice, relationship context, and prior decisions throughout the book.
6. Resume interrupted work safely.
7. Run automated editorial and consistency checks.
8. Resolve surfaced conflicts.
9. Export a polished RTL Persian DOCX manuscript.
10. Repeat the workflow reliably under automated tests and CI.
11. Manage project state, approvals, corrections, and selective retranslation entirely through supported user-facing commands.
12. Inspect the provenance of important literary decisions and trust user-approved decisions over model suggestions.
13. Process large long-form projects without redundant work or fragile state.
14. Install, upgrade, back up, restore, and use a stable v1 release on a clean supported system.

## Priority Rule

Until the end-to-end runtime in Cycle 5 works, implementation work should take priority over additional architecture-only documentation. Every subsequent cycle must leave the repository in a more runnable, testable, user-facing state.

Cycles 9-12 must not start by adding new product surfaces unless Cycles 5-8 exit criteria are materially satisfied. Their purpose is to turn a working release into a dependable daily-use product, not to postpone the core runtime behind additional architecture work.
