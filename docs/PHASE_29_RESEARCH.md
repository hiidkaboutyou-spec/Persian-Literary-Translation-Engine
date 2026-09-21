# Phase 29 Research — Human Pilot Review Ledger & Resolution Loop

Date: 2026-09-20

## Measured gap after Phase 28

Phase 28 can tell the reviewer **where** to inspect the current whole-book translation, but it does not persist whether a sampled target was actually reviewed, whether the reviewer found a problem, whether an apparent issue was intentionally accepted, or whether a later edit made the earlier human decision stale.

That means the product can currently generate a good review sample but cannot answer, durably and locally:

- Which current targets have been reviewed by a human?
- Which targets still need revision?
- Which automated/review concerns were deliberately accepted as literary choices?
- Did a later translation edit invalidate an earlier human sign-off?
- Is the current selected sample completely resolved?

Phase 29 closes that workflow gap. It does **not** add another automatic evaluator.

## Research findings

### Error Span Annotation supports focused human review without full MQM overhead

Kocmi et al., WMT 2024, *Error Span Annotation: A Balanced Approach for Human Evaluation of Machine Translation*:

https://aclanthology.org/2024.wmt-1.131/

ESA combines span-level error marking and severity with lighter-weight judgment than full MQM. The authors report annotation quality comparable to MQM with lower annotation burden.

WMT25 subsequently used professional annotators with ESA for most evaluated language pairs:

https://aclanthology.org/2025.wmt-1.22/

Decision:

- Phase 29 stores optional source/translation **character spans**, review dimension and severity;
- do not require a huge MQM taxonomy;
- keep one focused record per sampled target;
- human outcome remains explicit.

These studies are not Persian-specific proof of literary quality. They are workflow evidence only.

### Automatic highlights can be accurate without being useful to editors

Sarti et al., TACL 2025, *QE4PE: Word-level Quality Estimation for Human Post-Editing*:

https://aclanthology.org/2025.tacl-1.64/

In a realistic study with professional post-editors, the usefulness of error highlights depended on domain, language and editor behavior, with only modest differences among several highlight sources. Accuracy and workflow usability were not the same problem.

Decision:

- Phase 29 does not add automatic word-level QE;
- it does not pre-fill human findings from a model;
- automated Phase-19/28 evidence may guide attention, but the reviewer explicitly records the human decision.

### Document context is pervasive but does not always change the judgment

Kim, WMT 2025, *Context Is Ubiquitous, but Rarely Changes Judgments: Revisiting Document-Level MT Evaluation*:

https://aclanthology.org/2025.wmt-1.5/

The study reports that context is broadly present while only part of it changes human judgments.

Decision:

- keep target identity local and focused;
- the desktop reviewer can inspect neighboring context from the normal project workspace;
- do not copy large context windows into the ledger;
- do not turn "more context" into an automatic scoring rule.

### Literary creativity remains poorly captured by automated judges

Gerrits, van Noord & Guerberof Arenas, EAMT 2026, *Creativity Bias: How Machine Evaluation Struggles with Creativity in Literary Translations*:

https://aclanthology.org/2026.eamt-1.43/

Automatic metrics and LLM-as-a-judge correlated poorly with professional judgments of literary creativity and could penalize creative/culturally appropriate solutions.

Zhang et al., Findings ACL 2026, *Beyond Reproduction: A Paired-Task Framework for Assessing LLM Comprehension and Creativity in Literary Translation*:

https://aclanthology.org/2026.findings-acl.2030/

Strong source comprehension did not imply human-level translational creativity.

Decision:

- provide an explicit `accepted_as_is` human outcome for intentional literary choices;
- automatic review evidence can never create this outcome;
- no overall literary score is introduced;
- a human may resolve a sampled target even when automatic evidence remains advisory, but contradictory human records are rejected (for example `critical` + `accepted_as_is`).

## Architecture

### Ownership

Phase 29 lives in `project-engine::application::pilot_review`.

It is project/editorial workflow state. It does not belong in:

- Character Bible canon;
- translation memory;
- literary-intelligence canon;
- the Phase-14 intelligence-review ledger;
- the Phase-21 benchmark corpus.

### Persistence

Local project path:

`review/pilot-review-ledger.json`

The ledger is append-only.

A later review of the same target appends a new record. Earlier records are preserved as history.

### Target identity

A target ID is a stable opaque SHA-256-derived identity over:

- project ID;
- chapter ID;
- paragraph ID (or chapter-level sentinel).

The ID does **not** contain source/translation prose.

Target identity deliberately excludes translation fingerprints so the same location remains identifiable after an edit.

### Currentness / staleness

Every human record is bound conservatively to the reviewed chapter state:

- SHA-256 fingerprint of the full chapter source;
- SHA-256 fingerprint of the full chapter translation;
- translation-context fingerprint;
- translation-plan fingerprint.

The local source/target spans still refer only to the selected target. This deliberately makes a prior sign-off stale when another paragraph or the translation context in the same chapter changes, because literary judgments can depend on surrounding context.

A record is current only when all three still match the present target.

After a manual edit or plan change:

- old human records are preserved;
- they become stale evidence;
- they do not satisfy the current review sample;
- re-review creates a new append-only record.

### Human outcomes

`clear`

- no unresolved findings;
- human reviewed the current target and found no issue requiring a recorded exception.

`accepted_as_is`

- reviewer deliberately accepts the current translation despite an observation/advisory concern;
- requires a note or explicit finding;
- cannot retain a human `critical` finding.

`needs_revision`

- must contain at least one `warning` or `critical` human finding;
- target remains unresolved until the text changes and/or a later human record resolves it.

### Findings

A finding contains:

- existing Phase-19 `ReviewDimension`;
- existing `ReviewSeverity`;
- optional source Unicode-character span;
- optional target Unicode-character span;
- bounded reviewer-authored note.

Spans are Unicode scalar-value offsets, never byte offsets.

The engine validates every span against the exact current text before persistence.

### Privacy

The engine never automatically copies source or translated prose into the pilot-review ledger.

The ledger stores:

- opaque IDs;
- fingerprints;
- plan identity;
- spans;
- human outcome;
- reviewer name/label;
- optional reviewer-authored local notes;
- timestamp.

Reviewer notes are local project data and may themselves contain sensitive information entered by the user. They must never be copied automatically to GitHub, PMC/projectmem, CI logs, telemetry or external services.

### Bounded state

- maximum 5,000 records per project;
- maximum 16 findings per record;
- bounded reviewer/note sizes;
- capacity overflow fails closed rather than deleting review history silently.

## Application and desktop boundary

Phase 29 adds:

- `ApplicationService::pilot_review_summary`;
- `ApplicationService::record_pilot_review`;
- read-only Tauri summary command;
- explicit Tauri mutation command for human review records.

Domain/workflow rules remain in Rust.

JavaScript must not reimplement:

- target currentness;
- span validation;
- outcome validation;
- staleness;
- sample completion.

## Completion semantics

`sample_review_complete` means:

- Phase-28 mechanical artifacts are current;
- at least one current review target exists;
- every selected target has a current human record;
- every current human record resolves the target through `clear` or `accepted_as_is`;
- no current target remains `needs_revision`.

It does **not** mean:

- the whole book is objectively high quality;
- every chapter was exhaustively reviewed;
- export is automatically approved;
- canon may be mutated;
- automated evaluation passed.

Export behavior remains unchanged in Phase 29.

## Explicit non-adoptions

Phase 29 adds no:

- LLM-as-a-judge;
- quality-estimation model;
- automatic word/error highlighting model;
- external database;
- cloud review service;
- telemetry SDK;
- new Rust/Python runtime dependency;
- automatic export gate;
- automatic canon promotion.

## Exit criteria

Phase 29 becomes canonical only when:

1. Phase 28 is canonical first;
2. ledger persistence is local and append-only;
3. current records are bound to exact full-chapter source/translation, context and plan fingerprints;
4. a manual edit makes the previous record stale without deleting it;
5. only a current Phase-28 target may receive a record;
6. Unicode character spans are validated fail-closed;
7. `clear`, `accepted_as_is`, and `needs_revision` enforce coherent rules;
8. sample completion is a workflow state, never a quality score;
9. engine-written ledger data never copies source/translated prose automatically;
10. history events contain target ID/outcome only, not book prose or reviewer note;
11. Tauri exposes the Rust-owned workflow without moving rules to frontend code;
12. permanent Linux and Apple Silicon gates pass;
13. Rust/Security/Phases 18–28/Desktop/Trusted Release/Project Memory remain green on the exact final head;
14. no new runtime dependency is introduced;
15. exact final head/merge/run IDs are recorded after landing.


## Canonical completion record

Phase 29 became canonical through PR #116.

- final validated head: `7eb15fc9ce16f5436b7f1339b8ce62c4a0218f9a`;
- merge commit: `8415f559a6bf295d39504108e40864b2c40e65e5`;
- Phase 29: `35657134822`;
- Rust CI: `35657134550`;
- Security: `35657134515`;
- Phase 18: `35657134553`;
- Phase 19: `35657134523`;
- Phase 20: `35657134737`;
- Phase 21: `35657134563`;
- Phase 22 Desktop Product: `35657134561`;
- Phase 23 Trusted Release: `35657134500`;
- Phase 24: `35657134591`;
- Phase 25: `35657134537`;
- Phase 26: `35657134536`;
- Phase 27: `35657134556`;
- Phase 28: `35657134527`;
- Project Memory Tooling: `35657134535`.

All required exact-head gates completed successfully before merge.
