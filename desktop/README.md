# Desktop product surface

Phase 22 exposes the existing Rust `ApplicationService` through a thin Tauri 2 desktop shell.

## Architecture

```text
desktop/ui (static HTML/CSS/JS)
        |
        | bounded Tauri commands
        v
desktop/src-tauri
        |
        | ApplicationService only
        v
engine/crates/project-engine
        |
        v
existing domain engines
```

The desktop shell must not duplicate translation, review, canon, persistence, or publication orchestration.

## Dependency boundary

The Tauri crate is intentionally **not** a member of the `engine/` Cargo workspace. Desktop GUI dependencies and platform WebView requirements therefore cannot make the credential-free core CLI/runtime build depend on Tauri.

Pinned Phase 22 stack:

- `tauri = 2.11.5`
- `tauri-build = 2.6.3`
- `tauri-plugin-dialog = 2.7.2`
- Tauri CLI `2.11.4` in distribution CI

The frontend is static, locally bundled HTML/CSS/JS. There is no Node/Vite/runtime web server dependency and no remote CDN/font/script.

## Security

- CSP defaults to bundled `self` assets plus Tauri IPC only.
- No remote web content is loaded.
- Source/project file selection uses the official dialog plugin from Rust.
- Frontend JavaScript has no general filesystem plugin.
- Manuscript text is inserted with DOM `textContent` / textarea values, never `innerHTML`.
- OpenAI credentials are session-only process environment state. The key is not written to project files or frontend storage.
- Application errors cross IPC through the existing typed `ApplicationErrorPayload`.
- Human review decisions remain explicit and automated evidence never becomes approval.

## Phase 26 editorial workspace

The desktop presentation layer now provides a dependency-free editorial workspace over the same bounded Rust application surface:

- four in-session themes: System, Midnight Ink, Rose Paper and Sage Manuscript;
- local command palette on Meta/Ctrl+K;
- Meta/Ctrl+1…8 workspace navigation;
- grouped Workspace / Literary intelligence / System navigation;
- side-by-side English/Persian paragraph editing;
- presentation-only Focus Persian mode;
- progressive View Transitions with direct fallback;
- `prefers-reduced-motion` handling and explicit focus-visible states;
- polite atomic live-region notices, current-page semantics, shortcut metadata and palette focus return.

Theme selection is intentionally not persisted. The frontend still uses no remote assets, frontend storage, package manager or general filesystem API. All translation/review/canon/persistence/export authority remains in `ApplicationService`.

Detailed product and accessibility rationale lives in `docs/PHASE_26_RESEARCH.md`.

## Development

From `desktop/src-tauri`:

```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo tauri dev
```

The permanent Phase 22 workflow validates the desktop backend and an Apple Silicon application bundle.

## Distribution status

Phase 22 produces a macOS app bundle on Apple Silicon. Public direct distribution still requires an Apple Developer signing identity and notarization credentials. Those secrets are deliberately not embedded in the repository.

The Tauri updater is deferred until an authenticated release endpoint and updater signing key are explicitly configured and tested. A local/session build must never silently weaken that requirement.
