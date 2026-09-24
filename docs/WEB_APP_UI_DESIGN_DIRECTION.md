# Web App UI Design Direction

## Product intent

The mobile/web surface must feel like a purpose-built literary translation studio, not a generic SaaS dashboard, admin template, or stock shadcn application.

Working design direction: **Editorial Translation Atelier / Living Manuscript**.

The interface should make the manuscript itself feel central: pages, margins, annotations, chapter flow, source/translation relationships, review state, and provenance should visually drive the product.

## Non-negotiable quality bar

- Mobile-first and genuinely comfortable in iPhone Safari, not merely responsive after desktop design.
- Distinctive visual identity with original composition, spacing, typography, motion, states, and interaction patterns.
- Excellent Persian/RTL typography and bidirectional source/target layouts.
- No generic dashboard look, no copied landing-page aesthetic, no excessive glassmorphism, and no decorative motion that competes with reading.
- Accessibility, reduced-motion support, keyboard navigation, and large touch targets are first-class product requirements.
- Every visual flourish must preserve manuscript readability and review accuracy.
- The browser surface must reuse canonical engine/application APIs; UI convenience must never create a second translation workflow.

## Experience concept

### Library / Home

Projects should appear as editorial folios rather than generic cards. Each project surface should communicate:

- book title and source format;
- current chapter / translation progress;
- review state;
- last activity;
- unresolved findings;
- a clear "Continue" action.

"New manuscript" should be a high-confidence primary action that opens a focused upload flow.

### First-run book test

The default safe path is:

1. Choose or drop a manuscript.
2. Inspect/import it.
3. Show detected chapters and basic manuscript intelligence.
4. Offer **Translate one chapter** as the primary first test.
5. Preview cost/usage before any model-backed work.
6. Show live stage progress without pretending provider work is instantaneous.
7. Land directly in a review surface when the chapter completes.

The UI must make it visually obvious that one-chapter testing is safe and does not commit the user to translating the entire book.

### Translation workspace

On mobile:

- segmented Source / Persian / Compare views;
- fast swipe or tap transitions without losing scroll position;
- sticky chapter/status header;
- persistent review actions within thumb reach;
- inline highlights for terminology, omission, voice, and continuity findings;
- a bottom-sheet detail surface for provenance and review evidence.

On larger screens:

- resizable source ↔ Persian reading desk;
- optional third review/provenance rail;
- synchronized paragraph navigation when evidence can be aligned safely;
- focus mode that hides nonessential chrome.

### Review experience

Review should feel like editorial markup, not issue triage.

- findings attach visually to text, paragraph, or margin;
- accept/edit/reject/defer are clear but not visually noisy;
- source evidence and provenance are one gesture away;
- model suggestions and human-approved canon must remain visibly different;
- destructive or propagation actions require explicit confirmation.

### Export

The export surface should feel like a publication handoff:

- quality status;
- unresolved blockers;
- included chapters;
- DOCX/other available artifacts;
- clear indication of RTL/Persian publication formatting.

## Visual language

### Composition

- editorial page proportions, generous reading measure, intentional negative space;
- thin rules, margin markers, folio/page cues, restrained manuscript texture;
- asymmetry may be used in hero/library compositions, but reading and review surfaces stay stable;
- mobile navigation should minimize chrome and maximize manuscript area.

### Color

Base palette should be derived from paper, ink, graphite, plum, terracotta, sage, and muted amber rather than default blue SaaS tokens.

Themes may include:
- Paper / Ink
- Midnight Ink
- Rose Paper
- Sage Manuscript

Themes must share semantic tokens so status colors and accessibility remain consistent.

### Typography

Persian/Arabic UI and manuscript text should use a locally bundled, license-compatible typeface with strong readability. **Vazirmatn** is an approved candidate for evaluation because it is purpose-built for Persian/Arabic web/application use and published under SIL OFL 1.1.

Typography should distinguish:
- UI labels;
- manuscript reading;
- metadata/provenance;
- numerical/status data.

Do not depend on third-party font CDNs in production.

### Motion

Motion should communicate hierarchy, continuity, and state:
- manuscript/page transitions;
- progressive reveal of analysis stages;
- subtle panel expansion;
- review annotations entering/leaving context;
- bounded completion transitions.

No perpetual decorative motion. Respect `prefers-reduced-motion`.

## GitHub research and adoption map

Open-source code may be adopted only after checking license, maintenance status, bundle/runtime impact, accessibility, and whether the code improves the product without flattening its visual identity.

### Strong candidates for direct adoption or selective integration

- **radix-ui/primitives** — MIT. Headless accessible primitives for dialogs, menus, popovers, tabs, focus behavior, and related interaction foundations.
- **shadcn-ui/ui** — MIT. Use as an open-code component distribution/source layer only. All visual tokens and compositions must be customized; never ship a stock shadcn look.
- **shadcn-labs/editorcn** — MIT. Candidate source for Tiptap-based editable/review surfaces. Adopt selectively after validating RTL behavior and editor requirements.
- **ueberdosis/tiptap** — headless editor foundation candidate where rich review editing is required; verify exact package licensing and required extensions at implementation time.
- **ibelick/motion-primitives** — MIT, currently beta. Use as motion-pattern/source inspiration or selective copied components after pinning and testing; do not make the product dependent on its visual style.
- **bvaughn/react-resizable-panels** — MIT. Candidate for desktop source/translation/review split panes.
- **lucide-icons/lucide** — ISC. Candidate icon foundation; use a deliberately small custom subset and avoid icon-heavy UI.
- **rastikerdar/vazirmatn** — SIL OFL 1.1. Candidate locally bundled Persian/Arabic typeface.

### Reference-only projects

These can inform UX research but their code/design must not be copied unless their license is explicitly compatible with the project's distribution strategy:

- **orielhaim/Storyteller** — GPL-3.0. Useful reference for book/project organization, multi-pane writing workspaces, reading modes, RTL, and manuscript-centric navigation.
- **DoktorDaveJoos/manuscript** — PolyForm Noncommercial 1.0.0. Reference only for novelist workflow, reviewable AI actions, story-bible organization, and focused editor UX.
- **RKPYI/rantale** — useful reading-first and mobile novel presentation reference; license must be verified before any code adoption.

## Originality rule

GitHub projects are building blocks and research material, not a visual template.

The final interface must have:
- its own token system;
- its own layout grammar;
- original project/library cards;
- original translation/review workspace composition;
- original motion choreography;
- original empty/loading/error states;
- project-specific bilingual/RTL interactions.

A screenshot should not be identifiable as "a shadcn app" or a clone of any referenced writing product.

## Implementation guardrails

- Prefer a small, inspectable dependency surface.
- Pin production dependencies and run security/license checks.
- Self-host required static assets.
- No manuscript text, API keys, or private review data in analytics, URLs, public error reporting, or client logs.
- Avoid UI frameworks that force business logic away from the canonical application layer.
- Preserve resumability across browser refresh/session loss.
- Add visual regression tests for core mobile and desktop states.
- Add automated RTL, keyboard, touch-target, reduced-motion, and contrast checks.
- Test on real iPhone Safari dimensions, not only responsive desktop emulation.

## UI acceptance gate

The web app is not considered product-ready until all of the following are true:

1. A first-time user can upload a supported book and start a one-chapter test from an iPhone without instructions.
2. Source, Persian output, review findings, and progress remain readable on a small screen.
3. The UI has a distinct project-specific visual identity and is not visually equivalent to a stock component library.
4. RTL/Persian typography and bidirectional interaction pass dedicated tests.
5. Core workflows work with reduced motion and keyboard navigation.
6. A browser refresh or temporary disconnect does not lose completed work.
7. Design review explicitly checks hierarchy, readability, originality, accessibility, motion restraint, and mobile ergonomics.
8. Any GitHub-derived component has a recorded source, license, adaptation rationale, and security/maintenance review.
