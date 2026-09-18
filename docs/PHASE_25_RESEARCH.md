# Phase 25 Research — Visual Experience & Theme System

Date: 2026-09-18

## Goal

Turn the canonical Phase-22/24 desktop product surface into a distinctive, calm, keyboard-first literary workspace without moving any translation, review, persistence, filesystem, provider, or publication authority into JavaScript.

This phase was explicitly reprioritized by the user after Phase 24 canonicalization because theme quality, visual identity, motion and day-to-day usability are product requirements, not decoration. The previously recorded character/coreference benchmark remains a future research target, but it is deferred until this visual-product phase is complete.

## Product direction

The visual concept is **editorial glass + living manuscript**:

- macOS-native spatial rhythm and keyboard behavior;
- translucent navigation and floating surfaces, but opaque/readable text regions;
- literary display typography for titles paired with the system UI font for controls;
- subtle manuscript/book motifs rather than generic SaaS illustrations;
- motion as state feedback, never as decoration that blocks work;
- explicit theme identities rather than a naïve light/dark inversion;
- Persian editing optimized for long reading sessions.

The implementation remains local static HTML/CSS/JavaScript and keeps the existing Tauri/Rust application boundary unchanged.

## Research references

### MotionSites AI

Reference: https://motionsites.ai

Observed useful visual patterns:

- large expressive hero typography;
- layered depth instead of flat card grids;
- ambient gradients/orbits that provide motion without requiring user attention;
- clear focal hierarchy and generous negative space;
- micro-motion used to make transitions feel intentional.

Decision: visual inspiration only. No code, remote asset, font, script or branding is copied. The desktop product uses a quieter productivity interpretation suitable for hours of manuscript work.

### macOS 26 / Apple design direction

Apple's 2025–2026 design direction emphasizes Liquid Glass navigation/materials, content-first sidebars and fluid controls across macOS 26.

Project interpretation:

- system font stack for UI;
- translucent sidebar/top-level floating controls;
- strong content readability in editor/text surfaces;
- compact control sizing and keyboard-first navigation;
- separate light and dark palette design;
- motion bounded to roughly 150–300ms for ordinary state changes;
- accessibility through explicit focus states and reduced-motion handling.

### ceorkm/macos-design-skill

Upstream: `ceorkm/macos-design-skill`.

Repository README declares MIT licensing.

Useful references:

- sidebar/topbar/content composition;
- 8px-oriented spacing system;
- keyboard shortcuts as first-class UI;
- command palette for multi-surface productivity apps;
- independent light/dark color treatment;
- backdrop blur/saturation for navigation/floating layers;
- layered rather than heavy single shadows;
- progressive disclosure.

Decision: reference only. Nothing is installed or vendored.

### julianmateu/light-cmd-palette

Upstream: `julianmateu/light-cmd-palette`.

License: MIT.

Useful idea: a dependency-light command palette can be built with vanilla JS/CSS and keyboard navigation.

Decision: reference only. Phase 25 implements its own bounded command palette in existing local `app.js`/CSS so there is no npm package, bundle step, extra runtime dependency or upstream synchronization burden.

### tauri-apps/window-vibrancy

Upstream: `tauri-apps/window-vibrancy`.

Reviewed version: 0.8.0.
License: `Apache-2.0 OR MIT`.

The current crate supports classic macOS vibrancy and a macOS 26+ `apply_liquid_glass` path. Its documented Tauri WebView integration also requires a transparent window and `macOSPrivateApi: true`.

Decision: **not adopted in Phase 25**.

Reason:

- real native glass is visually attractive but is not necessary to satisfy the UI requirement;
- enabling private macOS APIs changes the release/distribution boundary established in Phases 22–23;
- CSS `backdrop-filter` over a local visual background supplies a strong glass effect without introducing Rust/platform dependencies or private API use;
- the current public-release path should not be complicated merely for decoration.

Any future adoption requires a separate macOS distribution/signing/App Store review and an explicit fallback for non-macOS platforms.

### Split-view libraries

Lightweight pane libraries such as Split.js validate the value of adjustable side-by-side workspaces.

Decision: no dependency. The current translation editor only needs a stable source/Persian two-column layout plus a one-click Persian focus mode. CSS Grid owns this smaller requirement. A draggable splitter can be reconsidered only if user testing demonstrates a real need.

## Implemented Phase 25 design system

### 1. Four distinct themes

Phase 25 provides four in-session themes:

- **System** — follows macOS light/dark appearance;
- **Midnight Ink** — dark plum editorial workspace;
- **Rose Paper** — warm ivory/dusty-rose reading palette;
- **Sage Manuscript** — calm green/cream long-session palette.

Each theme defines its own surfaces, borders, text levels, accent, state colors, editor/source backgrounds, ambient glows and shadow behavior.

Theme preference is intentionally **not** persisted in `localStorage` or `sessionStorage`. Phase-22 security CI forbids frontend storage, and provider/session secrets must never be normalized into frontend persistence. The selected theme is therefore an in-memory presentation preference for the current app session; System is the launch default.

### 2. Literary identity

The previous generic `PL` badge is replaced by a local CSS/HTML book-page mark with a small sparkle. No external logo asset or generated image becomes canonical.

The Project view includes a restrained manuscript hero:

- layered paper/folio shape;
- Persian chapter detail;
- literary display font drawn from local/system fonts only;
- ambient gradients with no remote image asset.

### 3. macOS-style navigation

The sidebar now groups screens by task:

- Workspace — Project, Workflow, Translation Editor;
- Literary intelligence — Intelligence Review, Canon, Literary Review;
- System — History, Provider.

Primary navigation has visible `⌘1`…`⌘8` shortcuts, clear active state, keyboard focus state, and compact section labels.

### 4. Command palette

A local `⌘K` palette provides:

- view navigation;
- open/create/import actions;
- analysis/translation/review/export actions when the current application state allows them;
- theme switching;
- Translation Editor focus toggle.

The palette is keyboard navigable with arrows/Return, is rendered through DOM/textContent APIs, and never becomes an alternate domain-logic owner. Commands route into existing UI/application actions.

No npm dependency is added.

### 5. Translation Editor

The paragraph editor is now source/Persian side-by-side by default.

Each paragraph has:

- explicit source and Persian roles;
- stable paragraph number;
- serif source-reading surface;
- Persian-optimized font stack, direction and line height;
- dirty-state-aware Save button;
- one-click **Focus Persian** mode that hides the source pane without mutating the chapter.

All manual edits still use the existing `apply_manual_edit` Tauri command and preserve Phase-24 revision/staleness semantics.

### 6. Motion and accessibility

Motion is progressive:

- ordinary hover/state feedback stays short;
- view changes use `document.startViewTransition` when the current WKWebView supports it;
- unsupported WebViews use the existing immediate state update plus CSS entry animation;
- command palette and toast transitions are bounded;
- `prefers-reduced-motion: reduce` disables ambient and transition-heavy behavior.

Keyboard focus uses explicit focus-visible rings.

### 7. No remote visual surface

Phase 25 adds no:

- remote font;
- CDN;
- remote image;
- remote script;
- frontend package manager;
- React/Vue/Svelte/Vite runtime;
- frontend filesystem API;
- `innerHTML`;
- `localStorage` or `sessionStorage`.

The existing restrictive Tauri CSP remains unchanged.

## macOS compatibility

The user's target Mac is Apple Silicon on macOS 26.

Phase 25 deliberately uses APIs that work well in the current WKWebView while retaining fallbacks:

- system font stack;
- CSS backdrop blur;
- CSS Grid;
- native `<dialog>` with an `open`-attribute fallback path;
- optional View Transitions with a direct-update fallback;
- reduced-motion media query.

No native-private-API dependency is required for the visual system.

Actual app-bundle validation remains owned by the canonical Phase-22 Apple Silicon desktop workflow.

## Phase 25 validation contract

Phase 25 is canonical only when:

1. UI files contain no remote URLs/assets/scripts/fonts;
2. UI files contain no `innerHTML`, `localStorage` or `sessionStorage`;
3. every static `$("id")` JavaScript reference resolves to an HTML ID;
4. HTML has no duplicate IDs;
5. the command palette, four-theme selector, keyboard navigation and Persian focus control are present;
6. CSS explicitly implements `prefers-reduced-motion`;
7. JavaScript syntax validation passes;
8. the existing Tauri CSP is unchanged/restrictive;
9. Phase-22 Desktop Product builds a real Apple Silicon app bundle from the exact final head;
10. Security and all affected canonical regression workflows stay green;
11. PR is merged only after the exact final head is green and documentation/PMC records completion evidence.

## Deferred product work

- Native `window-vibrancy` / macOS 26 Liquid Glass through private APIs;
- persisted UI preferences through a Rust-owned app-config store;
- draggable source/Persian pane splitter;
- custom app icon/mascot selection from the vendored `ip-as-logo` workflow;
- the previously researched speaker/coreference benchmark.

These remain separate decisions rather than hidden additions to the visual phase.
