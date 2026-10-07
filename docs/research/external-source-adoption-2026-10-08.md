# External source adoption audit — 2026-10-08

## Scope

Reviewed for the Persian Literary Translation Engine:

- awesome-selfhosted/awesome-selfhosted
- trimstray/the-book-of-secret-knowledge
- avelino/awesome-go
- Solido/awesome-flutter
- OpenMinis/OpenMinis release 0-beta-build32
- Context7, Firecrawl and PostHog developer tooling

The goal is not to make the dependency graph larger. The goal is to reuse only ideas or components that improve the real Rust/Tauri literary product while preserving private-manuscript, human-review and publication boundaries.

## Decisions

### Awesome Selfhosted — discovery catalog, no service added

Useful categories include monitoring/status, backups, analytics, search and document/e-book systems. The repository already has project-owned CI, artifact gates and local product boundaries; adding another hosted service has no measured benefit in this slice. Future candidates must be evaluated individually rather than installing a catalog wholesale.

PostHog appears in the catalog, but this project deliberately remains local/private by default. Remote product analytics is therefore deferred unless an explicit opt-in, content-free telemetry design proves a concrete product need.

### The Book of Secret Knowledge — operational reference only

Useful as a source of Linux/network/debugging techniques and incident-response vocabulary. Only non-destructive, understood diagnostics may be adapted. Commands that inspect process memory, traffic, credentials or private files are not suitable as automatic CI/desktop behavior and must never be copied blindly.

### Awesome Go — defer direct adoption

The list is useful for discovering CLI, queue, testing and document tooling, but the production engine is Rust and the desktop adapter is Tauri. No Go sidecar solves a measured gap today. Introducing one would add build, packaging, IPC and security surface without evidence of better literary or publishing quality.

### Awesome Flutter — UX reference only; no framework migration

Flutter resources around RTL, localization, file pickers, state management and document viewers are useful for feature discovery. Phase 22 already owns those responsibilities through Rust/Tauri and locally bundled frontend assets. A Flutter migration would duplicate the desktop stack and violate the current architecture without solving a demonstrated gap.

### OpenMinis 0-beta-build32 — adapt architecture patterns, do not copy code

The 2026-10-07 beta release adds incremental trimming of old reasoning/tool output for long runs, steer/stop controls for sub-agents, bounded parallelism, run-now scheduled tasks, MCP/skills controls, per-provider response timeouts, provider-disable fallback, tool-call/result ordering after compaction, orphan-session cleanup and crash recovery.

Adopted clean-room rules for this repository:

1. long engineering runs discard stale/large context incrementally but never compact away a failing check, unresolved review, rollback note or acceptance blocker;
2. one mutation authority owns a branch/integration path at a time; parallel read-only research is allowed;
3. resume from durable GitHub/repository truth after interruption instead of replaying uncertain mutations;
4. a removed/disabled provider cannot silently remain active and an unavailable provider cannot promote an unqualified fallback;
5. provider-specific timeout behavior stays explicit and bounded. The existing Ollama qualification adapter already has this boundary, so no duplicate implementation is added.

OpenMinis is GPL-3.0. No implementation code was copied; only independently expressed architecture rules were adapted.

### Context7 — adopted as developer research, not runtime

Use it to resolve current library/framework API behavior before changes where outdated API knowledge could cause bad code. It receives no manuscript or reviewer content and is not a production dependency.

### Firecrawl — adopted as developer/upstream research, not runtime

Use its developer/repository research surface to inspect current upstream README, releases, issues and implementation evidence when a dependency or external architecture choice is material. It is not part of document ingestion or manuscript translation.

### PostHog — used for current analytics guidance; runtime integration deferred

Current PostHog guidance confirms server SDKs can be disabled/no-op and data collection can be controlled before transmission. That is not enough to justify a network analytics dependency for a private local manuscript application. Default runtime instrumentation remains absent. A future proposal must be opt-in and prove a strict content/PII exclusion boundary before code is added.

## Permanent acceptance checklist

For every external candidate: define the concrete gap; inspect current upstream docs/source/release; verify exact license; measure maintenance; check Rust/Tauri and Apple Silicon fit; model privacy/network/secret impact; benchmark against the native path; add failure/rollback tests; and merge only after exact-head CI. Popularity or inclusion in an awesome-list is never sufficient evidence.

## Rollback

This slice changes repository engineering policy/documentation only. Revert the AGENTS.md section and this note; there is no runtime, provider, manuscript schema, UI, dependency, secret or publication-format change.
