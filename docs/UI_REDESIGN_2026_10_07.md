# Living Manuscript UI redesign — 2026-10-07

Base: `main@caa9f2a04836da1b9f5080c4b81830b42857185f`

## User problem

The product engine has substantially outgrown its interface. Current main already contains real manuscript import, translation, canon, literary review, pilot review, local EPUB preview, provider qualification and publishing foundations, but the desktop surface still presents those capabilities as a dense form/dashboard.

The product's own UI direction requires an Editorial Translation Atelier / Living Manuscript experience, strong Persian/RTL treatment, manuscript-first composition, reduced-motion support and a safe one-chapter first test.

## Research used

- Current repository state, open PRs, Issue #138 and the canonical UI/publishing design documents were reviewed before implementation.
- Tauri 2 current documentation was checked through Context7. The existing static local HTML/CSS/JS frontend remains a supported architecture; Rust commands stay behind the existing Tauri invoke boundary.
- Firecrawl developer research confirmed that modern Tauri products can retain a local static frontend while still using responsive, accessible interface patterns.
- The historical Phase-25 UI PR was treated as a design/reference branch only. It was not merged because it predates current-main Pilot Review and other newer product behavior.
- PostHog guidance was reviewed only to decide whether analytics should be added. It was deliberately not adopted in this change because manuscript text, review notes and private reading behavior must remain local-first. Any future analytics should be explicit opt-in and limited to non-textual product events.

## Implementation

- Rebuilt application chrome around the manuscript instead of a generic dashboard.
- Grouped navigation into Manuscript / Editorial / System without removing any current-main view.
- Added editorial paper/ink visual tokens, restrained motion, dark-mode tokens and visible focus states.
- Added a manuscript hero and contextual view introductions.
- Preserved all current command bindings and DOM IDs required by the Rust/Tauri application surface.
- Kept Pilot Review and current human-review flows intact.
- Turned the Translation Editor into a source ↔ Persian reading desk.
- Added a safe `Preset · 1 chapter` control. It only sets the bounded chapter count and literary style; it does not trigger provider work.
- Added narrow-window/mobile-sized responsive behavior instead of the historical UI branch's desktop-only minimum width.
- Added a repository-owned UI Contract workflow that checks:
  - no remote frontend content;
  - no `innerHTML` or browser storage;
  - JavaScript syntax;
  - unique HTML IDs and all JS-referenced IDs;
  - preservation of current Pilot Review bindings;
  - keyboard focus, reduced motion, RTL and narrow-window contracts.

## Explicit non-adoptions

- No React, Tailwind, shadcn, Tiptap, Framer Motion, Vite or frontend package manager.
- No remote fonts, scripts, images or CDNs.
- No analytics SDK, session replay or manuscript telemetry.
- No merge of the old Phase-25 UI branch.
- No change to Rust domain/application authority.

## Risk and rollback

The change is frontend-only plus a CI contract and this record. It does not modify translation algorithms, Book IR, provider admission, persistence or exports.

Rollback is a revert of the UI PR. Current main remains the recovery point.
