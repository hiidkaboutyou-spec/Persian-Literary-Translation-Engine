# Phase 26 Research — Editorial Workspace UX & Accessibility

Date: 2026-09-19

## Goal

Turn the canonical Phase-22 desktop product surface into a calm, keyboard-first English→Persian literary workspace that is comfortable for long-form editing without moving translation, review, canon, persistence, provider, filesystem, or publishing authority into JavaScript.

Phase 26 is a product-surface phase. It does **not** change literary inference, translation semantics, persisted schemas, provider defaults, or publication formats.

## Canonical predecessor

Phase 25 Narrative Speaker & Coreference Intelligence merged through PR #106 at:

`e39a46dd652aaea6fd8d990d70a32fba2d96b0d4`

Final reviewed Phase-25 head:

`9037f569cffb2618b915248db2c60015764d034c`

Final-head validation was green for Phase 25, Rust CI, Security, Phase 18, Phase 19, Phase 20, Phase 21, Phase 22 Desktop, Phase 23 Trusted Release, Phase 24 Literary Precision, and Project Memory Tooling.

The Phase-25 speaker/coreference ownership and fail-closed rules remain unchanged.

## Why this phase now

Phase 22 proved that the stable Rust `ApplicationService` can support a real desktop product without moving domain orchestration into the frontend. Phase 24 added an explicit human acceptance surface for literary-review proposals. Phase 25 improved deterministic speaker evidence.

The remaining product gap is not another model or memory stack. It is sustained day-to-day usability:

- faster navigation across project/review/editor surfaces;
- clearer English/Persian editing focus;
- better keyboard access;
- explicit reduced-motion behavior;
- readable theme choices for long sessions;
- stronger focus/live-region semantics;
- a distinctive editorial identity without remote assets or a frontend framework.

## Superseded parallel UI experiments

Three pre-Phase-25 branches explored overlapping desktop redesigns from base `1deb09cbdff9c29e0e3218a1cd27bfd0affd4fd0`:

- PR #107 — Product UI: editorial motion workspace;
- PR #108 — Phase 25: visual experience and theme system;
- PR #109 — Phase 25: editorial motion desktop UX.

They are **design inputs, not canonical phase history**.

Decision:

- use PR #108 as the implementation baseline because it contains the strongest bounded product set: command palette, four in-session themes, Focus Persian, keyboard navigation, progressive View Transitions and reduced-motion handling;
- carry forward accessibility strengths from #107, especially explicit live-region and focus-return behavior;
- do not carry forward #109's pointer-relative decorative motion/glow system because it adds motion/complexity without a measured editing benefit;
- do not reuse Phase-25 numbering from the visual branches. Phase 25 is canonically speaker/coreference intelligence.

Phase 26 is rebuilt from current `main`, not merged from any stale pre-Phase-25 branch.

## Current platform research

### Tauri

Official Tauri release records show `tauri 2.11.5` as the current stable core release and `tauri-cli 2.11.4` as the current CLI release at the time of this review.

Decision: **no Tauri upgrade is needed for Phase 26**.

The existing desktop pins remain:

- `tauri = 2.11.5`;
- `tauri-build = 2.6.3`;
- `tauri-plugin-dialog = 2.7.2`;
- distribution CLI `2.11.4`.

This keeps the previously audited Phase-22 dependency graph and avoids mixing product UX work with dependency migration.

References:
- https://tauri.app/release/
- https://tauri.app/release/tauri/all-versions/

### Apple macOS interaction guidance

Apple's current macOS guidance emphasizes:

- keyboard shortcuts for frequent actions;
- large-display information density with fewer nested levels;
- personalization of workspace appearance where useful;
- preserving expected system shortcuts rather than repurposing them.

Phase-26 interpretation:

- `⌘K`/Ctrl+K opens a local command palette;
- `⌘1`…`⌘8` navigate the eight existing workspace surfaces;
- no standard destructive/system shortcut is repurposed;
- the desktop remains resizable and uses the existing native window shell;
- themes are presentation preferences only and do not affect project state.

References:
- https://developer.apple.com/design/human-interface-guidelines/keyboards
- https://developer.apple.com/design/human-interface-guidelines/designing-for-macos/

### WCAG 2.2

WCAG 2.2 reinforces visible keyboard focus and focus-not-obscured behavior; animation triggered by interaction should be reducible when it is not essential.

Phase-26 interpretation:

- visible `:focus-visible` treatment on interactive controls;
- navigation exposes `aria-current`;
- notices use a polite atomic live region;
- command palette uses dialog semantics and returns focus to the prior control;
- reduced-motion preference disables/shortens nonessential motion;
- theme controls expose `aria-pressed`;
- no visual state is the only representation of critical workflow authority.

Reference:
- https://www.w3.org/TR/WCAG22/

### View Transitions

The View Transition API is used only as progressive enhancement for view changes.

Decision:

- call `document.startViewTransition` only when present;
- fall back to direct state update otherwise;
- skip it when reduced motion is requested;
- never make navigation correctness depend on the animation API.

Reference:
- https://developer.mozilla.org/en-US/docs/Web/API/View_Transition_API

## Phase-26 product decisions

### 1. Keep the frontend dependency-free

Remain with locally bundled HTML/CSS/JavaScript.

Do not add:

- React/Vue/Svelte;
- Vite;
- Tailwind;
- animation frameworks;
- npm package management;
- remote fonts, scripts or images.

The existing static surface is already sufficient for the product DTOs and bounded Tauri commands.

### 2. Four session-only themes

Provide:

- System;
- Midnight Ink;
- Rose Paper;
- Sage Manuscript.

Themes are intentionally session-only and presentation-only.

Do not use `localStorage` or `sessionStorage`. A future persisted-preference feature must be Rust-owned and separately designed.

### 3. Keyboard-first navigation

Provide:

- local command palette;
- `⌘K` / Ctrl+K palette shortcut;
- `⌘1`…`⌘8` / Ctrl equivalents for workspace navigation;
- `aria-keyshortcuts` metadata;
- command availability derived from existing UI/application state, never a second orchestration layer.

### 4. Accessible command palette

The palette:

- is a native `<dialog>` where supported;
- exposes modal/dialog semantics;
- has a labelled search input controlling the result list;
- supports arrow navigation, Return and Escape;
- uses text/DOM APIs only;
- stores only an in-memory reference to the element that had focus before opening;
- restores focus to that element when the palette closes if the element still exists.

No manuscript/provider/reviewer text is written to storage.

### 5. English/Persian editing desk

The editor remains paragraph-bounded and uses the existing manual-edit application command.

Provide:

- side-by-side source and Persian panes;
- explicit Persian RTL direction and readable line height;
- paragraph-level save/dirty state;
- Focus Persian mode that hides only the source presentation pane;
- no mutation of source text or chapter structure from the focus toggle.

### 6. Motion is subordinate to editing

Allow:

- short hover/focus feedback;
- restrained ambient treatment;
- progressive View Transition support;
- bounded palette/toast transitions.

Require:

- `prefers-reduced-motion: reduce` behavior;
- no pointer-following glow/motion system;
- no motion needed to understand state or complete an action.

### 7. Local-only security remains unchanged

Continue to forbid:

- remote page/script/font/image/CDN dependencies;
- `innerHTML`;
- frontend storage;
- general JavaScript filesystem authority.

CSP remains the Phase-22 restrictive local/IPC policy.

### 8. No native private glass API

Do not adopt `window-vibrancy` or `macOSPrivateApi` for this phase.

CSS backdrop materials are sufficient for the visual requirement without changing App Store/distribution/private-API risk.

## Validation contract

Phase 26 is canonical only when:

1. JavaScript syntax passes.
2. HTML IDs are unique.
3. every static `$("id")` JavaScript reference resolves to an HTML ID.
4. no remote URL/script/font/image/CDN appears in the UI.
5. no `innerHTML`, `localStorage` or `sessionStorage` appears in the UI.
6. the Tauri CSP remains restrictive.
7. all four themes exist and remain in-session only.
8. command palette and `⌘1`…`⌘8` navigation exist.
9. Focus Persian exists without changing application/domain state.
10. `aria-live`, `aria-current`, `aria-keyshortcuts`, palette modal/search semantics and focus-return code are present.
11. `prefers-reduced-motion` and visible focus behavior are present.
12. optional View Transitions have a direct-update fallback.
13. Apple Silicon validation compiles the locked desktop backend.
14. the canonical Phase-22 workflow builds the exact-head macOS application bundle.
15. Security, Rust/core regression, Phase 24/25 and other affected gates remain green.
16. final phase state, merge SHA and next handoff are written back to roadmap/status/PMC documents after merge.

## Deferred

- persisted UI preference store;
- draggable pane splitter;
- native/private Liquid Glass integration;
- app icon/mascot canonicalization;
- updater activation;
- code signing/notarization credentials;
- third-party frontend frameworks;
- any new literary model/runtime dependency.

## Privacy and memory rule

Repository/PMC/project-memory notes may record architecture, decisions, CI evidence and handoff state only.

Never store:

- source manuscript text;
- generated translation text;
- provider keys;
- reviewer private content.
