# Phase 30 Research — Desktop Pilot Review Workspace

Date: 2026-09-22

## Measured gap after Phase 29

Phase 28 can deterministically select a privacy-safe, bounded whole-book review sample.
Phase 29 can persist append-only, fingerprint-bound human decisions and expose them
through Rust `ApplicationService` plus Tauri commands.

The remaining product gap is usability: the desktop frontend does not expose
`get_pilot_audit`, `get_pilot_review_summary`, or `record_pilot_review`.
A reviewer therefore cannot complete the real-book pilot workflow from the app
without an external caller.

Phase 30 closes that product-surface gap. It does not add a new evaluator.

## Research findings

### Quality-estimation assistance can help, but inaccurate signals can hinder editing

Liu, Dai & Li, MT Summit 2025, *Introducing Quality Estimation to Machine
Translation Post-editing Workflow: An Empirical Study on Its Usefulness*:

https://aclanthology.org/2025.mtsummit-1.38/

The study reports faster post-editing with sentence-level QE, while interviews
also found that inaccurate QE can hinder the process.

Decision:

- do not add a new sentence/word QE model to Phase 30;
- surface existing Phase-28 sampling and Phase-29 human state;
- keep automated evidence advisory;
- let the reviewer inspect the actual local source/translation before deciding.

### Downstream usability matters separately from model accuracy

Sarti et al., TACL 2025, *QE4PE: Word-level Quality Estimation for Human
Post-Editing*:

https://aclanthology.org/2025.tacl-1.64/

The work studies how error highlights affect editing speed, quality and choices,
not only detector accuracy.

Decision:

- Phase 30 does not auto-highlight model-predicted word errors;
- optional human spans remain available through the existing Phase-29 schema;
- the first UI surface prioritizes a bounded queue, local context, explicit
  human outcome and direct navigation to the normal editor.

### Human post-editors can over-edit acceptable alternatives

Mihalache & Varela Salinas, EAMT 2026, *Meaning-Making Process and Error
Dynamics in ChatGPT-Mediated Translation*:

https://aclanthology.org/2026.eamt-1.40/

The study distinguishes model errors, editor-introduced errors and acceptable
alternative reformulations. It is not Persian/literary proof, but it supports a
workflow that separates inspection and judgment from mutation.

Decision:

- `accepted_as_is` stays first-class in the UI;
- selecting a target never mutates the translation;
- opening the editor is a separate explicit action;
- edits continue through the existing revision ledger and stale earlier review
  evidence as designed.

### Diagnose and repair should remain separate responsibilities

Wang & Wu, ACL Industry 2026, *Diagnose, Then Repair: A Two-Stage MQM-Guided
Post-Editing Framework for Domain-Specific Machine Translation*:

https://aclanthology.org/2026.acl-industry.115/

The paper reports improved controllability from separating diagnosis and repair.
Its automated LLM evaluator is not adopted here: Phase 30 preserves the
project's human-authority rule and adds no LLM judge.

Decision:

- queue/decision and editor/mutation remain separate surfaces;
- no automated review result can press the equivalent of `clear` or
  `accepted_as_is`;
- no repair is applied from the pilot view.

### Tauri security favors a narrow IPC boundary and restrictive local assets

Tauri 2 security / IPC references:

https://tauri.app/security/csp/
https://v2.tauri.app/concept/inter-process-communication/
https://v2.tauri.app/reference/config/

Decision:

- keep the existing bundled local frontend and restrictive CSP;
- communicate only through exposed Tauri commands;
- keep domain validation/currentness/staleness in Rust;
- render project text with `textContent` / DOM text nodes, never `innerHTML`;
- introduce no browser storage, remote frontend URL, analytics, or `fetch`;
- reviewer notes and book text stay local to the project/app process.

## Phase 30 product contract

The desktop Pilot Review workspace must provide:

1. bounded `max_review_targets` including the valid value zero;
2. mechanical export state and sampled-review state shown separately;
3. audit issue codes and whether they are mechanically blocking;
4. a queue of current Phase-28 targets with reason, chapter and current/stale
   human-review state;
5. local source/Persian context for the selected target, including neighboring
   paragraph context when available;
6. explicit human outcomes: `clear`, `accepted_as_is`,
   `needs_revision`;
7. optional existing Phase-19 dimension/severity plus Unicode character spans;
8. direct navigation to the normal translation editor without auto-editing;
9. refresh after a recorded decision so Rust recomputes currentness and sample
   completion;
10. no frontend persistence of manuscript text, translation text, reviewer
    notes or review state.

## Ownership

Rust remains authoritative for:

- target selection;
- target currentness;
- source/translation/plan/context fingerprints;
- outcome coherence;
- span bounds;
- append-only persistence;
- stale-record detection;
- sample completion.

JavaScript owns only:

- requesting the Rust state;
- rendering it;
- collecting a user submission;
- requesting the existing mutation;
- navigating to the normal editor.

## Validation

Phase 30 adds a static contract checker that fails if the pilot UI:

- loses required DOM controls;
- introduces duplicate IDs or inline event handlers;
- stops invoking the canonical Rust commands;
- introduces `innerHTML` / `outerHTML`;
- introduces browser persistence APIs;
- introduces frontend network `fetch` or remote URLs.

The dedicated workflow also runs JavaScript syntax checking, the existing
Phase-29 human-review tests, dependency-diff protection, Tauri backend
compile/tests, and Apple Silicon validation.

## Explicit non-adoptions

Phase 30 adds no:

- new QE or LLM judge;
- automatic word/error highlighter;
- automatic rewrite/post-editor;
- database or cloud review service;
- analytics/telemetry SDK;
- browser persistence;
- remote frontend content;
- new Rust/Python/Node runtime dependency;
- automatic export approval;
- automatic canon promotion.

## Exit criteria

Phase 30 becomes canonical only when:

1. Phase 29 is canonical and its exact completion evidence is recorded;
2. the desktop exposes the Phase-28/29 review loop end to end;
3. project text is rendered locally with safe text nodes;
4. target inspection never mutates text;
5. edit navigation uses the existing revision path;
6. Rust remains the only owner of review validation/currentness/staleness;
7. zero-target selection remains valid;
8. no browser persistence/network/remote frontend content is introduced;
9. the static UI privacy/IPC contract passes;
10. the dedicated Linux and Apple Silicon Phase-30 jobs pass;
11. Rust/Security/Phases 18–29/Desktop/Trusted Release/Project Memory remain
    green on the exact final head;
12. no runtime dependency is introduced;
13. final head/merge/run IDs are recorded after landing.
