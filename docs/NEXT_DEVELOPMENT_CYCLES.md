# Next Development Cycles

These cycles extend the existing four milestones while preserving the product goal: a usable Rust-based Persian literary translation system that can ingest long-form fiction, preserve context and character voice across chapters, generate high-quality Persian, evaluate consistency, and export a publication-ready manuscript.

## Delivery note — Phase 15

Phase 15 adds optional model-assisted literary analysis on top of the deterministic Phase 13
intelligence: bounded analysis units, a provider-neutral `LiteraryAnalysisProvider` (mock + OpenAI),
structured evidence-backed findings for voice/tone/POV/relationships/subtext and related
categories, deterministic evidence/schema/confidence validation, fingerprint-keyed cache/resume,
and review-only `Literary` proposals that flow through the Phase 14 human review ledger — never
into canon. Deterministic analysis remains fully offline and is unchanged. See
`docs/ADVANCED_LITERARY_ANALYSIS.md`.

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

## Cycle 13 — Mobile-First Web App & No-Terminal Book Testing

### Goal
Make the engine usable from an iPhone or other mobile browser without requiring Terminal, Rust tooling, or direct filesystem access, while reusing the existing application/service layer instead of duplicating translation logic.

### Work
- Build a responsive mobile-first web app that works well in iPhone Safari and modern desktop browsers. The surface must meet the distinctive editorial UI quality bar in `docs/WEB_APP_UI_DESIGN_DIRECTION.md`, not ship as a generic dashboard/template.
- Let the user upload supported manuscript formats (EPUB, DOCX, TXT, Markdown, and text-based PDF) and create a project from the browser.
- Make the safest first-run path a one-chapter test by default, with an explicit choice before translating more chapters.
- Show import/analysis/translation progress, chapter state, quality findings, warnings, and resumable failures in the UI.
- Expose review actions for terminology, character facts, literary findings, corrections, approvals, and selective retranslation through the same canonical application layer used by the CLI/desktop surface.
- Allow users to preview translated chapter text in the browser and download/export the final Persian RTL manuscript and supported intermediate artifacts.
- Keep provider credentials and other secrets server-side; never embed API keys in browser code, logs, downloadable artifacts, or project files.
- Add upload-size limits, content-type/format validation, rate limiting, bounded job execution, cost/usage previews, and explicit confirmation before expensive multi-chapter runs.
- Preserve project isolation, source provenance, checkpoints, memory, review ledgers, backup/recovery semantics, and all existing safety gates.
- Add mobile end-to-end tests covering upload -> inspect/analyze -> one-chapter translation -> review -> resume -> export.
- Add visual-regression, RTL/bidirectional, reduced-motion, accessibility, touch-target and originality/design-review gates for the core web surfaces.
- Maintain a license/adoption ledger for any GitHub-derived UI components or patterns; use permissive building blocks selectively while preserving a project-specific visual system.
- Document a simple user journey: open the site, choose a book, test one chapter, review the result, then continue the book if desired.

### Exit criteria
- A user can open the web app on an iPhone, upload a supported book, and run a one-chapter translation test without using Terminal.
- The web app uses the canonical engine/application APIs rather than a second translation implementation.
- Interrupted browser sessions can reconnect to a persisted project/job without losing completed work.
- Secrets and manuscript contents do not leak into client bundles, logs, URLs, or public artifacts.
- A user can review the first translated chapter and export/download a valid Persian result from the browser.
- The mobile workflow is covered by automated tests and documented as a supported product path.
- The production UI passes the project-specific design gate: strong editorial identity, excellent Persian/RTL reading, non-generic composition, accessible interaction, restrained motion, and documented provenance for adopted open-source UI code.

## Cycle 14 — Persian Publishing Studio & Editable Multi-Format Export

### Goal
Turn the canonical translated manuscript into a professionally designed Persian book and make publication controls usable from the web app. DOCX/Word, EPUB, and PDF must be generated from one editable Book IR and share the same approved Persian typography and book-design decisions.

See `docs/PERSIAN_PUBLISHING_ENGINE.md` for the normative publishing contract and open-source adoption map.

### Work
- Add a canonical editable Book IR for chapter/paragraph/run/scene-break/note/image/metadata/publication-style state.
- Add protected Persian normalization for Yeh/Kaf, digits, punctuation, ZWNJ, whitespace, and mixed-script direction without corrupting URLs, identifiers, code, or explicit Latin runs.
- Adopt/evaluate the approved Persian GitHub tooling and font families under pinned versions/licenses, including `rust-persian-tools`, Persian Tools/Virastar differential fixtures, Vazirmatn, Sahel, Samim, and Parastoo.
- Add a tokenized Persian book design system with Literary Classic, Modern Novel, Compact Paperback, Large Reading, and Digital EPUB profiles.
- Upgrade DOCX export to a genuinely editable publisher-facing Word document with styles, semantic RTL, protected LTR runs, sections, page numbering, headers/footers, notes, links/bookmarks, and TOC support where source semantics allow it.
- Extend the existing Phase 20 EPUB round-trip path with publication profiles, Persian typography CSS, font policy, and re-import/edit/regenerate guarantees.
- Add Paged.js-based professional print preview/PDF rendering using sanitized semantic HTML and the same publication tokens shown in the web app.
- Add a Book Design surface in the mobile-first web app for font, trim, margins, body type, line height, paragraph behavior, chapter openings, scene breaks, headers/footers, and export presets.
- Add deterministic QA for DOCX structure/round-trip, EPUBCheck/re-import, and PDF font embedding/text extraction/BiDi/visual-regression fixtures.
- Preserve a single source of truth: browser edits regenerate every format; do not implement three independent manuscript states.

### Exit criteria
- A user can edit one canonical manuscript in the web app and export matching DOCX, EPUB, and PDF versions.
- DOCX is directly editable in Microsoft Word and retains professional Persian book structure instead of flattened formatting.
- EPUB passes EPUBCheck and preserves valid Persian RTL/navigation/assets after round trip.
- PDF is a professional final render regenerated from the editable project and contains selectable/searchable Persian text with embedded fonts.
- A user does not need to manually repair RTL, نیم‌فاصله, Persian characters/punctuation, chapter openings, ordinary page numbering, or baseline book layout after export.
- Persian/mixed-script fixtures and each output-format publication gate pass on supported CI environments.

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
15. Open the product from a mobile browser, upload a supported book, run and review a one-chapter translation test, resume the project, and export the result without using Terminal.\n16. Edit one canonical Persian manuscript, choose a professional book-design profile, and generate matching editable DOCX/Word and EPUB outputs plus a professional regenerable PDF without manual RTL/typography repair.

## Priority Rule

Until the end-to-end runtime in Cycle 5 works, implementation work should take priority over additional architecture-only documentation. Every subsequent cycle must leave the repository in a more runnable, testable, user-facing state.

Cycles 9-14 must not start by adding new product surfaces unless Cycles 5-8 exit criteria are materially satisfied. Their purpose is to turn a working release into a dependable daily-use product, not to postpone the core runtime behind additional architecture work.

## Cross-cutting developer tooling & external-project adoption track — 2026-09-25

This track records external GitHub projects that may improve future development or inspire later product phases. It is **not** permission to bulk-install dependencies. The Rust-first architecture, provider neutrality, literary/project-memory boundaries, human review, privacy, publishing guarantees, and existing external-integration policy remain authoritative.

### Priority A — developer-side context and research tooling

1. **codebase-memory-mcp — evaluate, then adopt developer-side if gates pass**
   - Purpose: structural codebase knowledge, dependency/call-graph navigation, impact analysis, and faster agent context recovery during large next-stage work.
   - This is coding memory only. It must never become literary translation memory, Character Bible, glossary, relationship context, review state, or runtime retrieval.
   - Keep this repository's index/config isolated from every other project.
   - Before adoption:
     - pin and review the exact upstream revision/license;
     - audit install scripts, background services/watchers, telemetry, config mutations, and data locations;
     - prefer manual MCP configuration and prohibit automatic modification of `AGENTS.md`/agent policy files;
     - exclude manuscripts, credentials, provider secrets, review ledgers, generated books, and private project data from any unintended external transmission;
     - generated indexes/caches must be ignored, disposable, and non-canonical;
     - normal Rust build/test/export must work identically without the tool.
   - Benchmark on a representative cross-crate change and record measurable effects on correctness, navigation effort, regression discovery, and context/tool usage.

2. **Graft — evaluate only after the codebase-memory-mcp benchmark**
   - Test whether it contributes complementary repository context rather than duplicating another context engine.
   - Keep optional and developer-side.
   - Do not permit unreviewed edits to `AGENTS.md`, project-memory policy, provider configuration, or core architecture docs.
   - Adopt only when an A/B task demonstrates material benefit with no new privacy or instruction-conflict risk.

3. **Hyperresearch — high-value research workflow candidate**
   - Evaluate for deep research phases involving EPUB/DOCX/PDF standards, Persian NLP, typography, provider/model evidence, datasets, licensing, security, web architecture, and open-source adoption.
   - Store conclusions as normal reviewed research notes with sources/provenance; the research tool itself is not canonical evidence.
   - It must remain outside translation/runtime execution and must not gain manuscript/provider secrets by default.
   - Prefer it when a future phase explicitly calls for broad comparative research before implementation.

4. **PI-Desktop — optional external developer workspace**
   - Evaluate as a local-first environment for this repository, coding agents, MCP servers, models, and workflows.
   - Do not treat it as the product's desktop/web architecture or as a replacement for the canonical Tauri/application-service boundaries.
   - Preserve project isolation and ensure the engine/CLI/web/desktop paths do not depend on PI-Desktop availability.

### Priority B — selective inspiration and controlled prototypes

5. **Agency-agents — selective specialist inspiration only**
   - Never bulk-install the full catalog.
   - Future stages may inspect narrowly relevant roles such as security reviewer, research specialist, Rust reviewer, publishing/UX reviewer, accessibility reviewer, or QA specialist.
   - Any useful role should be adapted into project-specific guidance only after provenance/license and instruction-conflict review.
   - Project `AGENTS.md`, human-review boundaries, and repository invariants always outrank imported agent prompts.

6. **Needle — deferred local structured-extraction prototype**
   - Potential future use is narrow local structured extraction/tool calling, not literary translation.
   - Consider only when a concrete pipeline gap exists and compare it with native Rust/deterministic extraction and existing model-assisted analysis.
   - Require benchmarks for Persian/English accuracy, schema reliability, resource use, privacy, licensing, and failure behavior.
   - Disable telemetry where supported. No automatic model downloads in default CI/runtime.
   - Do not promote generated findings directly to canon or human approval.

7. **FreeLLMAPI — development/test provider experiment only**
   - May be tested behind the provider-neutral boundary for non-canonical smoke/stress/fallback experiments.
   - Never make free third-party endpoints the quality baseline for final literary translation or advanced analysis.
   - Do not send copyrighted/private manuscripts, credentials, reviewer data, or durable project memory to unknown providers by default.
   - Promotion beyond development requires explicit model/provider provenance, privacy/terms, quota/reliability, output-quality, cost, and failure-mode review.

8. **9Drive — future web/cloud-storage inspiration, not current integration**
   - Revisit after the mobile web app/project workspace has a real requirement for user-owned cloud-file import/export or multi-account storage.
   - Evaluate architectural ideas such as virtual folders, quota-aware routing, reconnect/sync UX, and provider abstraction rather than copying the application wholesale.
   - Any prototype must pass OAuth/credential, manuscript privacy, deletion/retention, sync-conflict, backup/recovery, quota, and lock-in review.
   - The canonical project/application service and local project model remain the source of truth.

### Priority C — currently unrelated to the translation product

9. **AutoShorts — no current integration**
   - Video/audio short-form clip generation is outside the literary translation/publishing goal.
   - Revisit only if the product scope explicitly expands to audiobook/video promotional media.

10. **OpenMontage — inspiration only if a future media-production scope is approved**
    - Do not integrate into the translation engine today.
    - If future publishing expands into trailers/social video, perform a dedicated architecture and license review and keep media production outside the core literary runtime.

11. **AdGuard Home — no roadmap integration**
    - Network-wide ad/tracker blocking is outside the product boundary and should not be added to runtime, CI, web app, desktop app, or developer bootstrap.

### Sequencing rule for future next-stage work

When these tools are relevant, prefer:

`codebase-memory-mcp benchmark -> optional adoption -> Graft complementary benchmark -> Hyperresearch/relevant specialist guidance -> narrowly justified local/provider/storage prototypes`.

External tooling must not displace the actual product frontier. Every adoption needs a concrete problem statement, pinned provenance, security/privacy/license review, benchmark or acceptance evidence, rollback, and documentation. Inspiration-only projects should yield project-native design decisions rather than new runtime dependencies.

### Trendshift discovery & second-batch tooling review — 2026-09-25

Trendshift is a **candidate-discovery source**, not an installation list. Future research stages may scan its daily/weekly/monthly GitHub momentum views, but adoption decisions must always be based on the upstream repository, exact revision, license, maintenance, security/privacy behavior, benchmarks, and a demonstrated Translation Engine need.

12. **Browser Use — research/browser sidecar and exploratory QA only**
    - Useful for interactive standards/library research, browsing complex documentation, and exploratory web-app UX checks.
    - Do not make manuscript ingestion, translation, review, publishing, or CI correctness depend on an agent navigating arbitrary websites.
    - Deterministic product E2E/CI should continue to prefer controlled browser-test tooling and project-owned fixtures.
    - Any Browser Use evaluation must keep credentials/session profiles separate from manuscripts/provider secrets and document hosted-browser/privacy/ToS/resource implications.

13. **AgentMemory (`rohitg00/agentmemory`) — high-priority developer-memory benchmark**
    - Evaluate only as **coding-agent memory**, never literary/project translation memory.
    - Compare directly with the already-approved developer-side `projectmem` profile and the planned codebase-memory-mcp experiment.
    - Do not enable multiple auto-capture memory systems simultaneously by default.
    - Use a dedicated data directory/namespace for this repository and exclude manuscripts, provider keys, review ledgers, Character Bible, glossary, Translation Memory, project artifacts, and human-review data from unintended capture.
    - Adoption requires auditable recall, forgetting/deletion, stale-memory handling, export/rollback, and a measured improvement on representative cross-crate/long-horizon maintenance work.

14. **Scientific Agent Skills — selective research adoption candidate**
    - Do not install all 166 skills.
    - High-value candidates for future research phases are general research/evidence workflows: literature retrieval, reproducible evidence gathering, database lookup, statistical analysis, benchmarking, and other skills directly relevant to translation-quality research.
    - Domain-specific biomedical/chemistry skills remain out of scope unless a concrete research question requires them.
    - Selected skills are developer/research guidance only; they never become translation-runtime dependencies or human approval.

15. **Diagram Design — strong developer/docs skill candidate**
    - Evaluate for architecture diagrams, application-service boundaries, translation/review pipelines, memory retrieval, EPUB/DOCX/PDF publishing flows, threat models, deployment, data schemas, and mobile-web user journeys.
    - Self-contained static HTML/SVG is preferred for portable review artifacts.
    - Diagrams are explanatory views, not normative contracts; schemas/tests/code/Markdown remain authoritative.
    - Pin provenance/license if vendored and keep the skill outside normal engine/runtime dependencies.

16. **Anthropic-Cybersecurity-Skills — defensive subset only**
    - The repository is community-created and not an official Anthropic package.
    - Never bulk-install the entire offensive/security catalog.
    - Review only defensive skills applicable to this product: web-app security, OAuth/session/secret handling, dependency and supply-chain security, CI/GitHub Actions hardening, threat modeling, incident response, local data privacy, and secure file upload/download.
    - Offensive credential-access, persistence, exploitation, or unrelated pentest workflows must not become project tooling.

17. **Awesome Harness Engineering — recurring architecture/reference input**
    - Treat as a curated research index for long-horizon agent reliability: context delivery, memory/state, task decomposition, worktree/PR isolation, verification loops, observability, human review, permissions, and safe autonomy.
    - Periodically compare high-signal patterns with the repository's next-stage protocol and project-memory policy.
    - Import project-native patterns only after evidence review; do not replace the Rust-first architecture or repository guidance with a generic harness.

18. **OpenViking — high-value unified-context benchmark, not immediate runtime integration**
    - Evaluate as a potential developer context database combining resources, coding memories, and skills.
    - Compare against `projectmem`, AgentMemory, and codebase-memory-mcp; avoid running every context system concurrently.
    - Initial use should be an isolated developer-side/local service. The main OpenViking project uses AGPLv3, so embedding/distribution needs a separate license and architecture review.
    - Keep a dedicated repository namespace/instance and do not ingest copyrighted/private manuscripts, provider credentials, review ledgers, durable literary memory, or unrelated-project context by default.
    - Measure retrieval quality, token/context savings, observability, stale-memory behavior, deletion/forgetting, latency/resource use, and operational burden before adoption.

19. **KAT-Coder-Pro — coding-model benchmark only; never a translation provider by default**
    - The exact `KAT-Coder-Pro V9.5` label was not verified in the 2026-09-25 review. Verify the exact identifier before configuring anything.
    - The publicly verified current candidate is proprietary KAT-Coder-Pro V2.5.
    - It may be compared on bounded repository coding tasks if a compatible provider is already available.
    - It must not be promoted into the literary translation/analysis provider set merely because it is a strong coding model.
    - Record cost, latency, tool reliability, Rust/TypeScript/Python change quality, test success, and regression rate against the existing coding-agent baseline.

20. **abi/screenshot-to-code — useful prototype accelerator for the planned web UI**
    - Revisit during the mobile-first web-app implementation/design cycles.
    - Use for translating project-owned screenshots, sketches, mockups, or approved references into throwaway/prototype React/Tailwind-style implementations; do not make it a runtime dependency.
    - Generated code must be refactored through the canonical application/API boundaries and reviewed for Persian RTL/BiDi behavior, accessibility, responsive mobile layout, security, dependency quality, and originality.
    - Do not clone third-party product UI exactly; treat visual references as inspiration and preserve the project's distinctive editorial/book identity.
    - Final production UI remains subject to the existing visual-regression, originality/design, and accessibility gates.

### Context/memory competition rule

`projectmem`, AgentMemory, OpenViking, codebase-memory-mcp, and Graft overlap. They are **candidates to benchmark**, not dependencies to accumulate.

Use one representative long-horizon maintenance task plus one architecture/research task, establish a baseline, evaluate candidates sequentially, and retain the smallest combination that improves correctness, context recovery, provenance, and developer efficiency without contaminating literary memory or raising privacy/maintenance risk.

