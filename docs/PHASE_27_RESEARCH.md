# Phase 27 Research — Real-Book Pilot Readiness & Resume Integrity

Date: 2026-09-20

## Goal

Prepare the existing production pipeline for the first user-supplied full-book pilot without pretending that synthetic tests are equivalent to a real literary evaluation.

Phase 27 is deliberately operational rather than feature-heavy. It hardens the behavior that matters when a translation takes many chapters, many provider calls, multiple sessions, and repeated resume cycles:

1. semantic translation-plan identity;
2. exact checkpoint reuse;
3. monotonic and exact progress reconstruction;
4. bounded repeated resume that actually advances;
5. a rights-safe multi-chapter end-to-end pilot rehearsal;
6. a human-centered protocol for the later real-book evaluation.

No user manuscript or generated book translation is committed to Git, PMC, projectmem, Linear, or CI.

## Why this is the next measured gap

Phase 26 closes the long-span coreference evidence boundary, but the project still had two operational defects visible from the current application code:

### Bounded resume could stall

`max_chapters` is documented as “translate at most this many chapters this run.”

Before Phase 27, reusing an already-valid checkpoint incremented the same counter as a newly translated chapter. A project translated one chapter at a time could therefore repeatedly consume its one-chapter budget by merely re-reading chapter 1 and never reach chapter 2.

Phase 27 changes the budget to count **new provider translations only**. Reused valid checkpoints rebuild progress but do not consume the current run's translation budget.

### Persisted paragraph progress could double-count

Resume loaded the old persisted totals and then added every reused chapter again. Chapter progress was mostly protected by `max(index + 1)`, but paragraph progress was additive and could exceed the actual book total.

Phase 27 reconstructs completed chapter/paragraph progress from currently valid checkpoints on every run. Exact final progress is therefore:

- completed chapters == total chapters;
- completed paragraphs == total paragraphs;
- percent == 1.0.

## Translation-plan fingerprint

Source and Context Packet fingerprints are necessary but not sufficient for safe reuse.

Two runs may have identical source and context while differing in:

- provider;
- resolved model;
- target language;
- translation style profile;
- production pipeline contract.

Phase 27 introduces a deterministic translation-plan fingerprint over:

- protocol/version tag;
- resolved provider name;
- effective model identity;
- target language;
- style profile;
- the current default literary pipeline contract identifier.

`max_chapters` is intentionally excluded because it is an execution budget, not a semantic translation choice.

Each completed chapter stores a `.plan-fingerprint` checkpoint and the structured chapter artifact records the same identity.

A resume may reuse a chapter only when all three match:

1. source fingerprint;
2. context fingerprint;
3. translation-plan fingerprint.

Legacy checkpoints with no plan fingerprint are safe to open but are regenerated instead of being silently trusted.

## Research findings

### Document-level translation + smaller refinement is currently the strongest general recipe

Tan et al., ACL 2026, *What Does LLM Refinement Actually Improve? A Systematic Study on Document-Level Literary Translation*:

https://aclanthology.org/2026.acl-long.268/

Across six LLMs and seven language pairs, the study reports the strongest and most stable improvements from document-level MT followed by segment-level refinement. Fully document-level refinement was less reliable, and most gains were in fluency rather than adequacy.

Decision:

- do not rewrite our translation/refinement architecture solely from this paper;
- preserve the current bounded pipeline for Phase 27;
- make segment-boundary and early/middle/late-book quality explicit pilot observations;
- only change refinement granularity after EN -> FA evidence from our own pilot/benchmark shows a measured gain.

### Book-length evaluation must account for segmentation and omissions/additions

Wang et al., EMNLP 2025, *Extending Automatic Machine Translation Evaluation to Book-Length Documents*:

https://aclanthology.org/2025.emnlp-main.1645/

SEGALE treats long translations as continuous text and uses segmentation/alignment so evaluation can handle arbitrary-length output and sentence-boundary differences, including under-/over-translation.

Decision:

- reuse our existing monotonic alignment and omission/addition review architecture rather than adding a competing evaluator;
- pilot checks must include missing/extra-content evidence, not only lexical overlap.

### No current automatic metric is trustworthy as a single document-coherence oracle

*MetaDocEval: A Contrastive Framework for Evaluating Machine Translation Metrics at the Document-Level*, EAMT 2026:

https://aclanthology.org/2026.eamt-1.19/

The reported result is especially relevant to this project: current metric families do not reliably capture document-level coherence, and short windows around roughly three sentences can be more useful than feeding ever-longer contexts into a scorer.

Decision:

- no “overall book quality score” becomes a release gate;
- use whole-book deterministic invariants plus bounded local windows and human review;
- automatic evidence remains advisory.

### Literary evaluation quality depends strongly on human expertise and rubric complexity

Zhang, Zhao & Eger, NAACL 2025, *How Good Are LLMs for Literary Translation, Really?*:

https://aclanthology.org/2025.naacl-long.548/

Their LitEval study found evaluator expertise and evaluation-scheme complexity materially affect judgments; simpler comparative schemes were more reliable for distinguishing literary quality than a large all-at-once error taxonomy.

Decision:

- keep the Phase-21 human scorecard focused;
- for the first real-book pilot, use small sampled passages and focused comparisons instead of asking a reviewer to assign one giant book-wide score;
- preserve human judgment as the final authority.

### Input length and position remain risk factors

Peng, Bawden & Yvon, MT Summit 2025, *Investigating Length Issues in Document-level Machine Translation*:

https://aclanthology.org/2025.mtsummit-1.3/

They report degradation as document length grows and position effects where later material can be weaker.

Decision:

The real-book sampling protocol must deliberately cover:

- early chapters;
- middle chapters;
- late chapters;
- long chapters;
- chunk/refinement boundaries;
- dialogue-heavy passages;
- coreference-heavy passages;
- recurring terminology/relationship-register passages.

## Rights-safe Phase-27 rehearsal

Permanent CI does **not** download or commit a novel.

Instead, the existing project-owned synthetic multi-chapter manuscript generator is used as an operational rehearsal. The Phase-27 regression performs a 12-chapter flow in two-chapter translation slices:

1. import;
2. deterministic analysis;
3. review/canon promotion;
4. translate first two chapters;
5. repeatedly resume with the same bounded budget;
6. verify exact progress after every resume;
7. reach completion;
8. export DOCX;
9. reopen the project and verify durable exact progress.

This proves orchestration/resume integrity, not literary quality.

## First real-book pilot protocol

When the user supplies a book, keep the source and generated translation inside the normal project workspace only.

Do not copy manuscript text into:

- GitHub issues/PRs;
- repository fixtures;
- PMC/projectmem;
- Linear;
- external benchmark services;
- logs or telemetry.

The pilot should record only non-text operational metadata in developer tooling, for example:

- chapter number / stable ID;
- failure class;
- quality dimension;
- whether the issue is reproducible;
- fix commit / status.

### Required observations

For a full pilot, record:

1. import/reopen integrity;
2. chapter count and source fingerprint stability;
3. translation-plan identity;
4. pause/resume and process-restart recovery;
5. no progress inflation;
6. no stale checkpoint reuse after provider/model/target/style changes;
7. deterministic quality-gate failures;
8. early/middle/late literary samples;
9. dialogue/speaker/coreference samples;
10. terminology and relationship/register recurrence;
11. long-chapter/chunk-boundary samples;
12. omission/addition evidence;
13. human revisions and staleness behavior;
14. DOCX and, when source is EPUB, EPUB round-trip validation;
15. reopened final project state.

## Explicit non-adoptions

Phase 27 does not add:

- a workflow engine such as Temporal;
- a second persistence owner;
- a new translation provider;
- a new automatic literary-quality oracle;
- a new external corpus;
- a new model runtime;
- Supabase/cloud sync;
- automatic upload of manuscripts to issue trackers or research services.

Existing native project persistence and chapter checkpoints are sufficient for the measured operational gap.

## Exit criteria

Phase 27 becomes canonical only when:

1. bounded resume advances past reused checkpoints;
2. resumed paragraph/chapter progress is exact, never inflated;
3. provider/model/target/style plan changes invalidate old chapter reuse;
4. legacy no-plan checkpoints fail safe by regeneration;
5. structured artifacts persist plan identity;
6. a project-owned 12-chapter repeated-resume rehearsal reaches exact completion and export;
7. Linux and Apple Silicon Phase-27 gates pass;
8. Rust CI, Security, Phases 18–26, Desktop, Trusted Release and Project Memory remain green;
9. Phase 26 is first merged/canonical, then the Phase-27 branch is rebased/retargeted or verified against canonical main;
10. no manuscript/private translation is committed or copied into developer-memory/tracking systems;
11. exact final head/merge/run IDs are recorded after landing.
