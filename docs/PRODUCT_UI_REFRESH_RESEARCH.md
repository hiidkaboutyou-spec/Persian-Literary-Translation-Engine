# Product UI Refresh Research — Editorial Motion Studio

Date: 2026-09-18

## Purpose

Refresh the desktop product surface into a distinctive, book-centered English-to-Persian literary translation workspace without changing translation/runtime ownership, introducing a frontend framework, adding remote assets, or weakening macOS behavior.

This is a **supporting product-surface iteration**, not Phase 25. The canonical project memory reserves any future Phase 25 work for a rights-safe speaker/coreference evidence benchmark. UI polish must not silently consume that phase number.

## Design research

### MotionSites

Reference: https://motionsites.ai

Useful principles observed:

- strong hero hierarchy with one memorable visual idea rather than many equally loud elements;
- depth created with layered translucent surfaces and ambient background movement;
- motion used as a presentation rhythm rather than as navigation logic;
- visual variety produced through composition, scale and atmosphere rather than a large component library.

Decision:

- borrow the principle of restrained ambient motion and layered depth;
- do **not** copy MotionSites branding, templates, imagery, prompts, source code or commercial visual assets;
- translate the idea into a desktop literary workspace: paper/page motifs, ink-like darks, Persian/editorial typography and quiet motion.

### Linear

References:

- https://linear.app/changelog/2024-03-20-new-linear-ui
- https://linear.app/now/how-we-redesigned-the-linear-ui

Useful principles:

- reduce visual noise while increasing information density;
- align labels, controls and sidebar rhythm precisely;
- preserve strong hierarchy between navigation, metadata and primary work;
- use platform conventions to make a web-rendered desktop UI feel native.

Decision:

- compact sidebar;
- stronger current-view hierarchy;
- editorial title hierarchy without oversized marketing UI;
- cards/panels remain operational surfaces rather than decorative landing-page blocks.

### Raycast-style keyboard navigation

Research references describe Raycast as a keyboard-first macOS interface with a compact command surface.

Decision:

- add a local command palette using only existing static JavaScript;
- `Cmd+K` on macOS / `Ctrl+K` elsewhere opens the switcher;
- `Cmd/Ctrl+1…8` moves directly among product workspaces;
- no command is allowed to bypass existing ApplicationService/human-review boundaries.

### Writing/editorial apps

Ulysses/Craft/editorial references reinforced:

- text should be the quietest and most readable part of the product;
- navigation chrome should recede while manuscript and translation surfaces remain visually primary;
- the bilingual editor benefits from a side-by-side source/target reading rhythm.

Decision:

- source text uses a system editorial serif stack;
- Persian translation uses a local Persian/system serif fallback stack;
- no external webfonts are downloaded;
- the editor becomes a true two-column bilingual desk on wide windows, collapsing safely on narrower supported sizes.

## macOS / Tauri research

Relevant current sources:

- Tauri 2.11.5 `TitleBarStyle`: https://docs.rs/tauri/2.11.5/tauri/enum.TitleBarStyle.html
- Tauri window customization: https://v2.tauri.app/learn/window-customization
- Apple motion guidance: https://developer.apple.com/design/human-interface-guidelines/motion

Findings:

- Tauri renders the current desktop frontend in WKWebView on macOS.
- Full custom/overlay title bars require custom drag behavior and carry macOS-specific caveats.
- Tauri's own customization documentation notes that custom titlebars can lose native window behaviors.
- Apple guidance recommends avoiding unnecessary repeated motion and respecting reduced-motion preferences.

Decision:

- **retain native macOS titlebar/window behavior**; do not add an overlay/custom titlebar merely for appearance;
- keep all visual innovation inside the content surface;
- all nonessential UI animation obeys `prefers-reduced-motion: reduce`;
- keep current Tauri/Rust/static-frontend architecture and existing CSP;
- validate the finished product through the existing `macos-15` Apple Silicon desktop workflow and real `.app` bundle build.

## GitHub research

### GitButler

Upstream: https://github.com/gitbutlerapp/gitbutler

GitButler is an actively maintained Tauri/Rust desktop product with a sophisticated multi-surface workspace.

Useful lesson:

- Tauri can support dense professional desktop workflows without moving domain ownership into frontend code.

Decision:

- architecture inspiration only;
- do not import Svelte/design-core or GitButler components;
- the existing static frontend remains intentionally smaller and easier to audit.

### Tauri UI/starter repositories

Reviewed search results included modern Tauri v2 starters using React, TypeScript, Tailwind and shadcn.

Decision:

- do not adopt those stacks merely for visual polish;
- the project already has a secure, local, framework-free frontend and does not need Node/Vite runtime/build ownership for this refresh.

## Implemented visual system

### Visual identity

- warm paper/ink light theme and deep editorial dark theme;
- emerald primary accent, muted plum secondary accent and restrained gold tertiary accent;
- system sans for product controls;
- system editorial serif for English literary text;
- local Persian/system serif fallback for target text;
- page/book motif on the project surface;
- translucent panels and ambient gradients without remote images.

### Motion

- short view-entry transitions;
- slow ambient color drift;
- quiet book/page breathing motif;
- progress shimmer;
- command-palette entry motion;
- complete reduced-motion override for users/system settings that request it.

Motion never carries critical state; every state remains understandable with animations disabled.

### Interaction

- `Cmd/Ctrl+K` command palette;
- arrow-key command selection, Enter activation and Escape close;
- `Cmd/Ctrl+1…8` workspace shortcuts;
- visible focus treatments;
- `aria-current` navigation state;
- no `innerHTML`, remote scripts, remote images, webfonts, localStorage or sessionStorage.

### Editor

- bilingual side-by-side source/target layout at normal desktop width;
- Persian target remains explicit RTL;
- existing manual-revision and stale-quality semantics are unchanged;
- UI styling never changes stored manuscript/translation text.

## Security and architecture boundary

No dependency was added.

The refresh does not change:

- Rust/ApplicationService ownership;
- project persistence;
- translation provider behavior;
- human review/canon authority;
- provider secret handling;
- EPUB/DOCX publishing;
- updater/signing state;
- CSP or remote-content policy.

The design references are research only.

## Validation

The permanent Phase 22 desktop workflow is extended to run on:

- `main`;
- the original Phase 22 branch;
- `product-ui-editorial-motion-refresh`.

Static UI checks now additionally require:

- `prefers-reduced-motion`;
- command-palette markup;
- platform-aware keyboard shortcut handling.

The existing workflow still verifies:

- Apple Silicon runner;
- committed desktop Cargo.lock with `--locked`;
- Rust formatting/Clippy/backend tests;
- project-engine regression tests;
- no remote UI content;
- no `innerHTML`;
- no localStorage/sessionStorage;
- restrictive CSP;
- Rust cargo audit;
- pinned Tauri CLI;
- a real unsigned/ad-hoc macOS `.app` bundle.

## Completion rule

This UI refresh is complete only when the PR exact head passes the macOS desktop product gate plus Security/Project Memory and any path-triggered canonical regression gates. Record final head/PR/merge/run IDs in Roadmap, Status and PMC after merge.
