# Phase 22 Research — Product Surface & Distribution Hardening

Status: active implementation branch `phase-22-product-surface-distribution`; not canonical until final PR gates pass, merge to `main`, and canonical handoff is recorded.

Research snapshot: 2026-09-18.

## Goal

Expose the stable application layer through a usable desktop product while preserving the project's Rust-first architecture, explicit human review, project persistence contracts, and publication guarantees.

The UI is an adapter. It does not become a second translation engine.

## Existing application boundary

Phase 16+ already created the correct product boundary: `project_engine::application::ApplicationService`.

It already owns project lifecycle, snapshots, analysis, review decisions, canon edits, translation, pause/resume, manual revisions, literary review, export, history, source verification, typed capabilities, and typed recovery errors.

The service documentation explicitly anticipated both the CLI and a future Tauri backend calling this same facade. Phase 22 therefore does not create a new HTTP service, database, Electron process, or UI-owned orchestration layer.

## Desktop framework research

### Tauri 2 — selected

Current reviewed stable line:

- Tauri core `2.11.5` — released 2026-07-01;
- Tauri CLI `2.11.4`;
- `tauri-build 2.6.3`;
- Rust MSRV for `tauri 2.11.5`: 1.77.2;
- license: MIT OR Apache-2.0.

Tauri 2 uses a Rust backend plus the operating system WebView (WKWebView on macOS), has an official application bundler, macOS app/DMG output, signing/notarization support, capabilities/CSP security controls, and an updater system.

Tauri 2.12 was still an open milestone during research and Tauri 3 remained alpha. Phase 22 therefore pins the latest reviewed 2.11 stable line rather than tracking a milestone/pre-release.

Primary references:

- https://v2.tauri.app/release/
- https://crates.io/crates/tauri/2.11.5
- https://github.com/tauri-apps/tauri
- https://v2.tauri.app/security/capabilities/
- https://v2.tauri.app/security/csp/
- https://v2.tauri.app/distribute/dmg/
- https://v2.tauri.app/distribute/sign/macos/

Decision: **selected**.

Why it fits this repository:

1. Rust stays the trusted backend.
2. `ApplicationService` maps naturally to bounded IPC commands.
3. System WebView avoids shipping a Chromium/Electron runtime.
4. Static local frontend assets need no application web server.
5. Official bundling covers the target macOS `.app` / `.dmg` path.
6. Capability/CSP design allows the frontend attack surface to stay narrow.

### Dioxus — deferred

Dioxus is a capable MIT/Apache Rust UI framework with desktop/WebView support and an integrated bundler.

Decision: no dependency in Phase 22. It would introduce a second state/component architecture without solving a gap that Tauri plus the already-existing application facade leaves open.

Reference: https://github.com/DioxusLabs/dioxus

### egui / eframe — deferred

egui is MIT/Apache and pure-Rust/immediate-mode with desktop support.

Decision: no dependency in Phase 22. It is strong for tooling-style interfaces, but the project is content/editor heavy and already has an explicit future-Tauri application boundary. Moving to immediate-mode UI would not improve domain isolation or distribution enough to justify the additional architecture shift.

Reference: https://github.com/emilk/egui

### Slint — deferred

Slint provides a polished native/declarative UI surface and stable 1.x APIs.

Decision: no dependency. Its licensing/distribution choices are more complex for this project than the selected MIT/Apache Tauri path, and it provides no measured requirement that Tauri cannot satisfy here.

Reference: https://github.com/slint-ui/slint

## Frontend decision: static local HTML/CSS/JS

Phase 22 deliberately does **not** adopt React/Vue/Svelte/Vite.

The product surface is not expected to own business state. It renders serialized application DTOs and sends bounded commands. A static frontend therefore:

- removes a Node/npm dependency from build and release;
- reduces dependency/supply-chain surface;
- avoids a development web server in the packaged application;
- keeps migration to another renderer possible because application/domain contracts remain Rust-owned.

Tauri's `withGlobalTauri` bridge is enabled for the bundled local frontend. No remote content is permitted.

## UI surface

Phase 22 desktop exposes:

- project create/open/import and snapshot;
- source verification and recovery warnings;
- deterministic and advanced analysis;
- review queue with explicit human approve/reject/defer/reopen decisions;
- character and glossary canon editing;
- provider configuration;
- translation start/resume/pause and progress;
- paragraph-level manual translation revisions;
- post-translation literary review evidence;
- DOCX/EPUB publication export;
- bounded project history.

The first review UI intentionally does not synthesize complex `ReviewedValue` edits for proposals. Character/glossary corrections are edited through canonical application methods instead; unsupported review edits fail rather than invent data.

## Provider configuration gap found

Research found an application-layer product bug: `TranslationConfig.model` existed but the OpenAI translation provider path ignored it and resolved the model only through `OPENAI_MODEL`.

Phase 22 fixes this at the application boundary:

- explicit `TranslationConfig.model` wins;
- `OPENAI_MODEL` remains a fallback;
- `gpt-5.6` remains the existing final default;
- tests prove explicit model selection wins.

The desktop provider credential is session-only. The UI sends it once to the Rust process; it is not persisted in project JSON, localStorage, JavaScript state after submission, or repository configuration.

## Dialog / filesystem boundary

Selected: official `tauri-plugin-dialog 2.7.2`, MIT/Apache.

The plugin is used **from Rust** to pick source files/project folders. The frontend is not granted a general filesystem API.

Reference: https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/dialog

## Security model

1. Bundled local assets only; no remote pages/CDNs.
2. Restrictive CSP: self scripts/styles/assets plus Tauri IPC.
3. No `innerHTML` for manuscript/provider content.
4. No JavaScript filesystem plugin.
5. Project locking/persistence stays in `ApplicationService`.
6. Typed `ApplicationErrorPayload` is the UI error contract.
7. Long-running application operations execute through Tauri's blocking task pool so the UI event loop remains responsive.
8. Translation progress is read through the persisted application progress API; pause uses the existing safe checkpoint contract.
9. Human review/canon rules remain application-owned.
10. Secrets, manuscripts, private translations, and reviewer material are never developer-memory/projectmem content.

Tauri 2.11.1 also tightened ACL behavior for remote origins. Phase 22 still avoids remote origins entirely instead of depending on that fix as the primary defense.

## Cargo/workspace isolation

The desktop crate lives under `desktop/src-tauri` and is **not** added to the `engine/` workspace.

Reason: Tauri's platform/WebView dependencies must never become a hidden requirement for core translation, CLI, review, or publication builds.

The desktop crate uses path dependencies on application/domain DTO crates only where needed.

## Distribution

Phase 22 initially validates an Apple Silicon macOS application bundle.

Configuration:

- application bundle identifier: `io.github.hiidkaboutyou.persianliteraryengine`;
- minimum macOS version: 12.0;
- bundle targets: `.app` and `.dmg`;
- Apple Silicon CI: `macos-15`.

Public direct distribution is not declared production-ready without Apple code signing and notarization. Those require Apple Developer credentials/signing identity and are intentionally external secrets.

The updater is **deferred**, not omitted accidentally. Tauri update artifacts require signing plus a trusted update endpoint. Phase 22 will not create an unsigned or placeholder update channel.

## Desktop lockfile bootstrap

The desktop crate needs its own committed `Cargo.lock`. Because it is intentionally outside the engine workspace, its dependency graph is independently reproducible.

The Phase 22 CI bootstrap generates the lockfile once, audits/compiles it, and publishes it as a workflow artifact. The exact validated lockfile is then committed to the branch and subsequent validation switches to `--locked`. Canonical Phase 22 must not merge without that committed lockfile.

## Exit criteria

Phase 22 is not canonical until:

1. desktop Cargo lockfile is committed and subsequent checks use `--locked`;
2. rustfmt/Clippy/tests pass for desktop and affected engine application surfaces;
3. static UI security checks prove no remote scripts/fonts/CDNs and no `innerHTML`/localStorage secret persistence;
4. project create/open/import/snapshot, analysis, review, canon, translation/pause/resume, manual edit, literary review, history and export IPC commands compile against `ApplicationService`;
5. explicit translation model selection test passes;
6. Apple Silicon app bundle builds successfully;
7. desktop and engine dependency graphs pass audit;
8. existing Phase 18–21/core Rust/security gates remain green;
9. signing/notarization requirements are documented without committing credentials;
10. roadmap/status/PMC/external-integration records are updated and PR merges to `main`.

Only then should Phase 22 be called canonical.
