# Application Orchestration Layer (Phase 16)

Phase 16 adds one **coherent application boundary** on top of the domain engines from Phases 13–15.
A future desktop app — and the CLI today — can open a project, import a book, analyze it, review
intelligence, translate it, watch progress, resume work, edit results, and export a book **without
understanding which crate owns which file**.

```text
Desktop UI / CLI
        ↓
ApplicationService   (project-engine::application)
        ↓
Domain Engines       (document · literary-intelligence · advanced-literary-analysis ·
                      human-review-workflow · character · memory · translation-core · quality)
```

The boundary lives in the existing `project-engine` crate (`project-engine/src/application/`),
which already owned the project manifest and review-store persistence. No new crate was added.

## Project layout

One predictable root per project. Everything the engine produces lives here; nothing is guessed.

```text
<project-dir>/
├── project.json                      application manifest (schema 1)
├── source/<original>                 immutable imported source copy
├── intelligence/deterministic.json  Phase 13 artifact
├── intelligence/advanced.json        Phase 15 artifact
├── review/review-ledger.json         Phase 14 ledger
├── canon/characters.json             canonical Character Bible
├── canon/glossary.json               canonical Glossary
├── translation/progress.json         durable translation progress
├── translation/chapters/             per-chapter artifacts + checkpoints
├── history/history.json              bounded audit history
├── exports/                          DOCX exports
└── .lock                             project lock
```

The original source file is **copied** into `source/` at import and never modified afterwards;
all later work reads the immutable copy, so a replaced or edited original cannot silently poison
analysis or checkpoints.

## Application API

`project_engine::application::ApplicationService` exposes the operations a UI needs. All of them
are synchronous and file-based; the service holds no mutable shared state.

```text
create_project(root, name, source?)     open_project(root)
import_book(project, source)            snapshot(project)
analyze_book(project)                   run_advanced_analysis(project, settings)
list_review_items / get_review_item     decide_review_item(project, id, action, replacement)
preview_promotion(project, ids, res)    apply_promotion(project, plan, reviewer, reason)
list_characters / get_character         upsert_character(project, profile, replace)
list_glossary_entries / get_entry       upsert_glossary_entry(project, entry, replace)
start_translation(project, config)      resume_translation(project, config)
request_pause(project)                  get_progress(project)
get_translated_chapter / get_text       apply_manual_translation_edit(project, ...)
export_project(project)                 history(project)
verify_source(project)                  capabilities() · test_provider_configuration(...)
recommended_next_action(snapshot)       error_payload(error)
```

Callers never orchestrate engine crates and never mutate engine JSON files directly. Engine
writes happen only through the atomic persistence helpers owned by the application layer
(temp-file + rename, same machinery the Phase 14 review store uses).

## Workflow

```text
Import → Analyze → Review → Translate → Edit → Export
```

```text
create project
  ↓
import book            (format detected; source copied; fingerprint stored)
  ↓
snapshot               (UI home screen: statuses, counts, next action — never re-parses the book)
  ↓
analyze                (Phase 13 deterministic + Phase 14 review reconcile, one call)
  ↓
analyze-advanced       (optional Phase 15, mock or openai; bounded units; cache)
  ↓
review                 (approve / edit / reject / defer / reopen — Phase 14 rules)
  ↓
promote                (preview → explicit apply; Literary findings never selected)
  ↓
translate / resume     (echo, auto, or openai; pause at chapter boundary; checkpoints)
  ↓
read / edit            (per-chapter artifacts; manual edits keep revision history)
  ↓
export                 (DOCX; EPUB reported as unavailable capability)
```

## State model

A project may simultaneously hold several independent sub-states, so the snapshot carries both a
high-level `status` and structured sub-counts rather than one mutually exclusive string.

- **`ProjectStatus`** — `Empty → Imported → Analyzed/NeedsReview → ReadyToTranslate →
  Translating/Paused → TranslationComplete → ReadyToExport → Exported`; `Error` when the source
  fingerprint no longer matches the imported copy.
- **`ArtifactState`** — `Fresh / Stale / Missing / InProgress / Failed` for source, analysis, and
  advanced-analysis records.
- **`NextAction`** — deterministic recommendation (`ImportBook`, `RunAnalysis`,
  `ReviewIntelligence`, `ConfigureProvider`, `StartTranslation`, `ResumeTranslation`,
  `ReimportSource`, `Export`, `None`). Reading it never mutates anything.

Staleness rules are surfaced, not silently repaired:

- source file changed → `status: Error`, `next_action: ReimportSource`, warning in snapshot
- canon promoted/edited after a translation run started → `translation.context_stale: true`
- repeated snapshots read only persisted metadata, so they stay cheap even for 100+ chapter books

## Review and canon

The application layer reuses the Phase 14 lifecycle exactly — it never re-implements review
validation and never offers an `approve-and-force-apply` shortcut.

- decisions go through `human-review-workflow` transition/conflict rules
- terminology approvals require an explicit preferred translation (`Edit` with the replacement);
  relationship approvals require reviewed dynamics/address/boundary notes
- promotion is preview-then-apply bound to a plan ID with fingerprint checks
- **Literary findings are reviewable but never promotable** — approving them records the human
  decision; they are excluded from canon promotion and only influence translation context via
  chapter-scoped evidence
- after promotion, canon counts and the canon fingerprint refresh so translation staleness
  tracking stays accurate

## Translation lifecycle

- `start_translation` requires deterministic analysis (the hard gate); canon is optional context.
- runs are chunked per chapter with durable `translation/progress.json`; `--max-chapters` on the
  CLI/`max_chapters` in the config stops a run at a safe chapter boundary in a **Paused** state.
- `resume_translation` discovers the checkpoint automatically and verifies the source and context
  fingerprints before continuing; incompatible progress is rejected, not silently reused.
- `request_pause` is honored at the next chapter boundary (never mid-write).
- `get_progress` returns run id, state, chapter/paragraph counts, `percent` (`0.0..=1.0`
  fraction), last checkpoint, and timestamps.
- `apply_manual_translation_edit` preserves the source paragraph, stores the previous translation,
  records origin `manual` + reviewer + timestamp as a revision, and marks the chapter's quality
  result stale.
- DOCX export reuses the existing publication exporter; unsupported formats are reported through
  `ApplicationCapabilities`, never faked.

## Provider configuration

- translation: `echo` (deterministic, offline) · `auto` (openai when `OPENAI_API_KEY` is set,
  else echo) · `openai`
- advanced analysis: `mock` (deterministic, offline) · `openai`
- `test_provider_configuration` validates a configuration **without exposing secrets**; API keys
  are read from the environment only and are never persisted in project.json, ledgers, history,
  logs, or exports.

## Persistence and recovery

- every critical write (manifest, ledger, canon, progress, history) is atomic: write temp file,
  rename into place — a crash never leaves a half-written JSON file.
- the `.lock` project lock is exclusive per project (create-new lock file, 30-minute stale
  timeout so a crash cannot deadlock the project; stale locks are removed and retried). Read-only
  operations do not need the lock.
- corrupt project.json fails open with a typed `InvalidProject` error and a recovery hint, and is
  never silently treated as an empty project.
- audit history (`history/history.json`) records `project_created`, `source_imported`,
  `analysis_completed`, `advanced_analysis_completed`, `canon_promoted`,
  `translation_*`, `export_completed`, and manual edits, bounded to 500 entries.

## Events

`ProjectEvent` (pure Rust, no UI framework) lets a future desktop app stream progress:
`ImportCompleted`, `AnalysisStarted/Completed`, `AdvancedAnalysisStarted/Completed`,
`ReviewStateChanged`, `CanonPromoted`, `TranslationStarted`, `ChapterStarted/Completed`,
`TranslationPaused/Completed`, `ManualEdit`, `ExportCompleted`. Every emitted event is also
written to project history.

## Error model

UI-facing errors are typed (`ProjectNotFound`, `InvalidProject`, `UnsupportedProjectVersion`,
`ImportFailed`, `AnalysisFailed`, `ProviderNotConfigured`, `ReviewConflict`,
`PromotionBlocked`, `NoCheckpoint`, `ResumeIncompatible`, `ProjectLocked`,
`ExportUnavailable`, `PersistenceFailure`, …) and serialize to a payload with a machine-readable
`recovery_hint` — the UI never parses internal error strings.

## CLI

`literary-engine project <command> <dir> …` is a thin adapter over `ApplicationService`:

```bash
literary-engine project create <dir> [--name N] [--source book]
literary-engine project import <dir> <book>
literary-engine project status <dir> [--format json]
literary-engine project analyze <dir>
literary-engine project analyze-advanced <dir> [--provider mock|openai]
literary-engine project review <dir> list|approve-all|promote
literary-engine project translate <dir> [--provider echo|auto|openai] [--max-chapters n]
literary-engine project resume <dir> [--provider ...]
literary-engine project progress <dir> [--format json]
literary-engine project export <dir>
literary-engine project history <dir> [--format json]
```

The flat pre-Phase-16 commands (`inspect`, `analyze`, `analyze-advanced`, `review`, `run`,
`resume`) keep working unchanged. Both surfaces share the same domain engines and — for project
workflows — the same application code path, so CLI behavior cannot diverge from the future
desktop app.

## Tests

- `crates/project-engine/tests/application_workflow.rs` — 18 application tests through
  `ApplicationService` only: lifecycle, the full offline workflow (import → deterministic →
  advanced mock → review → promote → echo translation → export), snapshot accuracy per stage,
  deterministic next actions, reopen-after-every-stage, review lifecycle (reject/defer/reopen/
  approve with Phase 14 rules), character/glossary APIs, pause/resume with monotonic progress,
  manual edits with revision history, translation gating, source-mismatch detection, canon-revision
  staleness, project locking, corrupt-manifest crash recovery, event emission, provider config,
  advanced-analysis cache reuse, JSON round-trips of UI-facing models, Unicode, and a
  120-chapter regression.
- `cli/tests/project_orchestration.rs` — 2 end-to-end tests driving the real binary:
  full project workflow + pause/resume via `--max-chapters`.
