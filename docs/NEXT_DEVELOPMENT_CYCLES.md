# Next Development Cycles

These cycles extend the existing four milestones while preserving the product goal: a usable Rust-based Persian literary translation system that can ingest long-form fiction, preserve context and character voice across chapters, generate high-quality Persian, evaluate consistency, and export a publication-ready manuscript.

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

## Priority Rule

Until the end-to-end runtime in Cycle 5 works, implementation work should take priority over additional architecture-only documentation. Every subsequent cycle must leave the repository in a more runnable, testable, user-facing state.