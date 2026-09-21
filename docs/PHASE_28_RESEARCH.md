# Phase 28 Research — Private Whole-Book Audit & Human Review Sampling

Date: 2026-09-20

## Goal

Turn Phase 27's mechanically safe full-book execution into a practical first-real-book review workflow without inventing a single automatic "book quality score".

Phase 28 adds a local, deterministic, text-free audit over existing project artifacts. It answers operational questions such as:

- Is the current translation complete under one translation plan?
- Are any chapter artifacts stale/mixed-plan/missing?
- Which chapters have no literary review, stale review, or review findings requiring attention?
- Which chapters contain human edits whose prior quality evidence is now stale?
- Which bounded set of chapters should a human deliberately inspect across the whole book?

The audit makes no network/model calls and contains no manuscript or translated prose.

## Research basis

### Literary automatic evaluation is not a substitute for professional review

Zhang, Zhao & Eger, NAACL 2025, *How Good Are LLMs for Literary Translation, Really?*

https://aclanthology.org/2025.naacl-long.548/

Their literary-translation study reports that evaluation adequacy depends strongly on evaluator expertise and evaluation-scheme complexity. Simpler comparison schemes were substantially more reliable than complex MQM-style literary judgments, while automatic metrics performed poorly.

Decision:

- do not create a Phase-28 overall literary score;
- expose concrete workflow/evidence states;
- make human review targets bounded and understandable.

### Creativity is systematically difficult for machine judges

Gerrits, van Noord & Guerberof Arenas, EAMT 2026, *Creativity Bias: How Machine Evaluation Struggles with Creativity in Literary Translations*.

https://aclanthology.org/2026.eamt-1.43/

Professional literary-translator annotations showed poor correlation for automatic metrics and LLM-as-a-judge on creativity, including systematic machine-translation preference and penalties for creative/culturally appropriate solutions.

Decision:

- no LLM-as-a-judge result may automatically mark a chapter "clear";
- Phase 28 samples passages for humans instead of replacing them.

### Source comprehension and translational creativity are different capabilities

Zhang et al., Findings ACL 2026, *Beyond Reproduction: A Paired-Task Framework for Assessing LLM Comprehension and Creativity in Literary Translation*.

https://aclanthology.org/2026.findings-acl.2030/

Across literary excerpts, strong source comprehension did not imply human-level translational creativity; models often remained literal or contextually inappropriate.

Decision:

- operational integrity and literary review remain separate dimensions;
- "mechanically export-ready" never means "literarily approved".

### Long evaluator inputs hide errors

Domhan & Zhu, EMNLP 2025, *Same evaluation, more tokens: On the effect of input length for machine translation evaluation using Large Language Models*.

https://aclanthology.org/2025.emnlp-main.402/

Longer evaluation inputs led to fewer detected error spans and worse ranking accuracy. Focused/local evaluation strategies mitigated this length bias.

Decision:

- Phase 28 selects bounded local review targets instead of sending a whole book to one evaluator;
- selected targets are distributed across the book and around known workflow risks.

### Book-length evaluation must handle segmentation and under/over-translation

Wang et al., EMNLP 2025, *Extending Automatic Machine Translation Evaluation to Book-Length Documents*.

https://aclanthology.org/2025.emnlp-main.1645/

SEGALE evaluates continuous long translations through segmentation/alignment and explicitly handles arbitrary boundaries plus under-/over-translation.

Decision:

- reuse the project's existing Phase-19 structural/alignment evidence;
- Phase 28 reports missing/stale/attention states rather than adding a competing evaluator.

## Phase-28 audit contract

The report contains only identifiers and metadata:

- project ID;
- source fingerprint;
- translation-plan fingerprint;
- chapter/paragraph counts;
- chapter IDs/indexes;
- stable paragraph IDs for suggested review locations;
- issue codes;
- review-sampling reasons.

It deliberately excludes:

- source prose;
- translated prose;
- revision text;
- Character Bible content;
- glossary content;
- provider secrets;
- prompts/responses.

## Two separate readiness dimensions

### Mechanically export-ready

This is true only when Phase-27 mechanical invariants hold:

- translation is complete;
- progress counts equal current source totals;
- a current translation-plan identity exists;
- progress belongs to the imported source;
- every current chapter has a matching translated artifact;
- every chapter artifact matches current source and translation plan.

This is a publication-safety property, not a literary-quality judgment.

### Human-review clear

This is true only when:

- the translation is complete;
- every translated chapter has a current literary-review artifact;
- no review artifact is stale;
- no current literary review requests attention;
- no chapter has stale post-edit quality evidence.

It is workflow state, not proof that the translation is artistically perfect.

## Deterministic sampling

The audit recommends at most 12 review targets by default.

Selection includes only chapters whose translated artifact matches the current source and translation plan. During a partial run, untranslated/stale-plan chapters are never suggested as translation-review targets.

Selection includes:

- early-book coverage among current translated chapters;
- middle-book coverage among current translated chapters;
- late-book coverage among current translated chapters;
- the longest chapter;
- the most dialogue-heavy local paragraph when dialogue evidence exists;
- chapters with manual revisions;
- chapters with stale literary review;
- chapters whose current literary review requires attention;
- chapters with stale quality evidence after human edits.

Risk/evidence-driven targets are prioritized over structural samples when the cap is reached. Results are then returned in chapter order.

No source/translated text is embedded in the target. The UI/reviewer may resolve the local paragraph ID inside the user's project workspace.

## Privacy boundary

A user-supplied book stays in native project storage.

The audit is exposed through `ApplicationService` and a read-only Tauri command so the desktop product can consume the same local Rust authority without moving orchestration into JavaScript. The audit report may be shown in the desktop UI or used locally, but manuscript/translation text must not be copied into:

- GitHub;
- PMC/project memory;
- Linear;
- CI logs/artifacts;
- external evaluation services;
- telemetry.

Developer tracking may retain only non-text metadata such as chapter ID, issue code, reproducibility, and fix commit.

## Explicit non-adoptions

Phase 28 does not add:

- a new metric package;
- LLM-as-a-judge;
- a cloud observability platform;
- an analytics SDK;
- a database;
- a new provider/model;
- an external corpus;
- a workflow engine.

The measured need is deterministic local audit/orchestration, so native Rust is sufficient.

## Exit criteria

Phase 28 becomes canonical only when:

1. the audit is computed entirely locally with no model/network dependency;
2. serialized audit output contains no manuscript/translation prose;
3. mechanical readiness detects mixed-plan/stale/missing chapter artifacts;
4. literary-review coverage/staleness/attention are reported separately from mechanical readiness;
5. manual edits and stale quality evidence become explicit human-review targets;
6. review sampling includes early/middle/late and content-risk coverage while remaining bounded;
7. the audit exposes stable opaque paragraph IDs rather than text excerpts;
8. partial-run sampling never points to untranslated or stale-plan chapters;
9. the same audit is exposed through the Rust ApplicationService and read-only Tauri command without frontend orchestration;
10. project-owned tests prove text-free output, manual-edit/stale-review behavior, mixed-plan fail-closed behavior, zero-target behavior, and partial-run actionability;
11. Linux and Apple Silicon Phase-28 gates pass;
12. Rust/Security/Phases 18–27/Desktop/Trusted Release/Project Memory remain green;
13. Phase 26 and Phase 27 are canonical before Phase 28 is landed;
14. exact final head/merge/run IDs are recorded after landing.
