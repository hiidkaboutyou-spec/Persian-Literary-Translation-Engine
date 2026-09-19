# Project Memory

## Mission

Build a Rust-first, production-oriented English→Persian literary translation system that preserves authorial intent, character voice and psychology, relationship/register, emotional subtext, terminology, narrative structure and long-novel continuity while keeping human review as final authority.

## Canonical architecture

- Rust owns the core domain/runtime and project orchestration.
- `ApplicationService` is the product/application facade.
- Translation providers are replaceable adapters.
- Literary intelligence, translation runtime, memory, deterministic quality, literary review, human review and publishing remain explicit boundaries.
- Canon/human approval never comes from an automated score or model inference.
- Private manuscripts, translations, provider credentials and reviewer-private content never belong in Git/PMC/projectmem.

## Canonical main state

Canonical through **Phase 25 — Narrative Speaker & Coreference Intelligence**.

Phase 25:
- PR #106
- final reviewed head: `9037f569cffb2618b915248db2c60015764d034c`
- merge: `e39a46dd652aaea6fd8d990d70a32fba2d96b0d4`
- native high-precision quote-speaker attribution;
- CharacterBible remains the only character/canon owner;
- speaker map is Deterministic evidence, not Canonical/HumanApproved;
- pronoun-only/implicit/ambiguous cases fail unresolved rather than guessed;
- no Torch/Transformers/spaCy/BookNLP/FastCoref/Maverick/Renard default dependency.

Authoritative Phase-25 research: `docs/PHASE_25_RESEARCH.md`.

## Active phase

**Phase 26 — Editorial Workspace UX & Accessibility**

Branch: `phase-26-editorial-workspace-accessibility`.

Current implementation:
- dependency-free local HTML/CSS/JS desktop surface;
- System / Midnight Ink / Rose Paper / Sage Manuscript session themes;
- local Meta/Ctrl+K command palette;
- Meta/Ctrl+1…8 workspace navigation;
- grouped workspace/literary/system navigation;
- English/Persian side-by-side editor;
- presentation-only Focus Persian mode;
- progressive View Transitions with direct fallback;
- reduced-motion handling and stronger visible keyboard focus;
- polite atomic live-region notices;
- `aria-current`, `aria-keyshortcuts`, modal/search semantics and command-palette focus return;
- no remote assets, frontend storage, framework, package manager or new Rust dependency;
- dedicated Phase-26 UI/accessibility + Apple Silicon locked compile workflow;
- canonical Phase-22 real app-bundle workflow extended to run on `main` and Phase 26.

Authoritative Phase-26 research: `docs/PHASE_26_RESEARCH.md`.

## Phase-26 non-negotiable decisions

1. UI remains an adapter over Rust/`ApplicationService`; JavaScript never owns translation/review/canon/persistence/export orchestration.
2. Themes are session-only presentation state; no `localStorage`/`sessionStorage` or project-file persistence.
3. Tauri remains on audited 2.11.5 core / 2.11.4 CLI during this phase.
4. No React/Vue/Svelte/Vite/Tailwind/npm runtime.
5. No remote scripts/fonts/images/pages/CDNs and no `innerHTML`.
6. Keyboard access and visible focus are product correctness, not optional polish.
7. View Transitions are progressive enhancement; reduced-motion and direct fallback are mandatory.
8. Focus Persian hides the source presentation pane only; it never mutates manuscript/revision/canon state.
9. No `window-vibrancy`/`macOSPrivateApi` solely for decoration.
10. PRs #107–#109 are pre-Phase-25 design experiments only; useful ideas may be ported but they do not redefine Phase 25 history.

## Validation handoff

Before Phase 26 can become canonical:

1. exact final branch head must pass `Phase 26 Editorial Workspace`;
2. exact final branch head must pass `Phase 22 Desktop Product` including real Apple Silicon `.app` bundle build;
3. Security and Rust/core regression checks must remain green;
4. Phase 24/25 behavior must not regress;
5. diff must contain no manuscript/private/provider-secret material;
6. after merge, repeat Phase-26 and Phase-22 validation on real `main`;
7. then update roadmap/status/PMC/project memory with final head, merge SHA and next handoff.

## Durable external-tool decisions

- BookForge remains the revision-pinned EPUB structured boundary.
- EPUBCheck 5.3.0 remains authoritative for EPUB 3.3.
- BGE-M3, COMET/XCOMET and other model-backed evidence stay optional/bounded.
- Hazm remains blocked under the existing security decision.
- BookNLP/FastCoref/ModernBookNLP remain benchmark/research candidates only until a rights-safe measurable gain justifies a separately reviewed sidecar.
- projectmem is developer-side coding memory only, never translation/runtime memory.

## Source of truth

Read in this order for continuation:

1. `AGENTS.md`
2. `docs/IMPLEMENTATION_STATUS.md`
3. `docs/IMPLEMENTATION_ROADMAP.md`
4. current phase research document
5. `docs/PMC_BOOTSTRAP.md`
6. `docs/ENGINEERING_DECISIONS.md`

This file is a concise handoff. Detailed contracts in the source-of-truth files override it.
