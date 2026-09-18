# Phase 25 Research — Editorial Motion Desktop UX

Date: 2026-09-18

## Goal

Turn the functional Tauri desktop shell into a distinctive, long-session literary workspace that feels designed for books rather than a generic admin dashboard, while preserving the local/static frontend, Rust application boundary, security posture, and macOS build guarantees established in Phases 22–24.

The user explicitly selected MotionSites-style motion/design inspiration as the creative direction and required the result to work on their Mac. Phase 25 translates that direction into an original product system rather than copying a site or importing a motion framework.

## Visual research

### MotionSites AI

Reference: https://motionsites.ai

Observed useful patterns:

- large, confident display typography;
- layered hero compositions with floating/overlapping surfaces;
- motion used as a sense of depth rather than only button feedback;
- high-contrast focal areas surrounded by calmer support content;
- short labels and visual grouping instead of dense dashboard chrome.

Decision: borrow **depth, choreography and composition**, not the landing-page density or promotional behavior. A book-translation workstation needs to remain comfortable for hours of reading/editing.

### Editorial and motion references

Research included current Awwwards motion/editorial galleries, Pangram Pangram's editorial redesign coverage, current book/reading interface references, and Material's editorial typography guidance.

Useful principles:

- typography can carry product identity without custom illustration;
- display/editorial type should be reserved for hierarchy and landmark moments, while controls stay utilitarian;
- reading surfaces need warm, quiet contrast rather than saturated SaaS color everywhere;
- motion should communicate layer/state changes and spatial continuity, not continuously demand attention.

### GitHub: Motion Primitives

Upstream: https://github.com/ibelick/motion-primitives
License: MIT.

Useful reference: carefully choreographed component transitions and microinteraction patterns.

Decision: **reference only**. It is built around Motion/React/Tailwind and is a framework mismatch for the deliberately static Tauri frontend. No package, copied component runtime, React, Tailwind, Vite or Node build dependency is introduced.

### GitHub: micro-interactions-library

Upstream: https://github.com/silvvrodriguez/micro-interactions-library
README license statement: MIT.

Useful reference: dependency-free vanilla/CSS glow and hover microinteraction patterns and the principle that subtle motion should remain isolated and optional.

Decision: **reference only**. Phase 25 implements its own interaction system directly in the existing local CSS/JS instead of vendoring code.

## macOS/Tauri research

Tauri 2 uses WRY/WKWebView on macOS.

Official window-customization guidance notes that fully custom titlebars on macOS can lose native window behavior. Tauri's `TitleBarStyle::Overlay` also has OS-version-dependent titlebar heights and custom drag-region caveats.

Decision:

- retain the native macOS titlebar and traffic lights;
- do not add titlebar/traffic-light plugins;
- do not require private AppKit APIs;
- create the distinctive identity entirely inside the webview content;
- preserve the current `minimumSystemVersion: 12.0` bundle contract;
- validate the production app on the existing Apple Silicon/macOS workflow.

## Accessibility and motion contract

WebKit supports `prefers-reduced-motion`, mapped to macOS Accessibility > Motion/Display preferences.

Phase 25 therefore requires:

- all decorative continuous movement to collapse under `prefers-reduced-motion: reduce`;
- focus-visible treatment for keyboard users;
- no information that exists only in animation;
- no hover-only required action;
- animation limited primarily to opacity/transform/background effects;
- editor text and human-review actions remain stable while decorative motion happens around them.

## Design system: Editorial Translation Atelier

### Identity

The product becomes an **editorial translation atelier**:

- warm paper/canvas surfaces;
- ink-like foregrounds;
- muted plum, terracotta, sage and amber accents;
- native serif display stack (`Iowan Old Style` / Palatino / Georgia) for literary hierarchy;
- native system sans for controls;
- native macOS/Persian serif fallbacks (`Geeza Pro` etc.) for Persian editing.

No remote font, image, CDN, CSS import or script is used.

### Hero language

The home workspace uses a layered manuscript composition with an original page-stack illustration made entirely from local HTML/CSS.

The key product line is:

> Translate the book. Keep its pulse.

This framing reflects the actual engine: voice, context, fidelity and natural Persian with human judgment at the boundary.

### Navigation

The sidebar is reorganized visually into three semantic groups while preserving the existing Tauri commands and view IDs:

1. Manuscript — Project, Workflow, Intelligence
2. Editorial — Canon, Translation, Literary Review
3. System — History, Provider

No backend contract changes are required.

### Translation editor

The paragraph editor becomes a visible EN ↔ FA desk:

- source and translation sit side by side at normal desktop widths;
- source uses a reading-oriented serif treatment;
- Persian uses an RTL Persian-native font stack and larger line-height;
- save actions remain explicit and human-controlled;
- narrow layouts stack the two sides instead of shrinking text.

### Motion language

Motion uses named intent rather than random timing:

- fast feedback: ~150 ms;
- standard workspace transitions: ~260 ms;
- expressive view entrance: ~520 ms;
- slow ambient page/orb motion: several seconds, decorative only.

Cards receive a small pointer-relative glow in normal-motion mode. The effect is disabled by the system reduced-motion preference.

## Security and architecture

Phase 25 intentionally adds **zero frontend dependencies**.

The existing rules remain:

- no remote assets/content;
- no `innerHTML`;
- no localStorage/sessionStorage secrets or state;
- no frontend filesystem authority;
- Tauri `ApplicationService` remains the product behavior owner;
- provider keys remain process-session-only;
- native dialogs remain Rust-owned;
- CSP remains unchanged.

## Implemented files

- `desktop/ui/index.html` — editorial semantic structure, grouped navigation, manuscript hero and section hierarchy;
- `desktop/ui/styles.css` — full local design-token/motion system, dual light/dark palettes, responsive editor and reduced-motion contract;
- `desktop/ui/app.js` — view-entry choreography, `aria-current`, new status text binding and nonessential pointer-glow behavior.

No Tauri command, Rust domain model, persistence format or provider contract is changed by the UI redesign.

## Exit criteria

Phase 25 is canonical only when:

1. all existing UI functionality/IDs required by `app.js` remain present;
2. static UI security checks pass with no remote assets, `innerHTML`, localStorage or sessionStorage;
3. JavaScript syntax and HTML ID uniqueness checks pass;
4. `prefers-reduced-motion` and `:focus-visible` are enforced by the committed stylesheet;
5. no Node/React/Tailwind/Framer/GSAP runtime/build dependency is added;
6. the Phase 22 Apple Silicon workflow builds a real locked macOS `.app` from the Phase 25 head;
7. Rust/Security/Phase 18–24/Project Memory regressions remain green as applicable;
8. final exact-head evidence is recorded and the PR is merged.
