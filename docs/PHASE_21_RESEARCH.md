# Phase 21 Research — Literary Evaluation Corpus & Benchmarking

Status: implementation branch `phase-21-literary-evaluation-benchmarking`; not canonical until PR merge and post-merge verification.

Research snapshot: 2026-09-18.

## Goal

Measure whether changes improve actual English-to-Persian literary translation quality rather than merely satisfying unit tests or one automatic metric.

Phase 21 must detect regressions in:

- semantic fidelity and omission/addition;
- Persian naturalness/readability;
- character voice;
- relationship/register;
- dialogue and emotional subtext;
- terminology and named-entity continuity;
- short-window long-context continuity.

Human judgment remains the approval authority. Automatic metrics and deterministic challenge assertions are evidence channels only.

## Why one metric is not enough

### Literary evaluation evidence

Zhang, Zhao, and Eger's LITEVAL work (NAACL 2025) compared multiple human and automatic evaluation schemes on literary translation. Their results show that evaluation complexity and evaluator expertise materially affect adequacy: professional-translator scalar judgments and simpler preference-style evaluation were more discriminating than complex MQM annotation for distinguishing human literary translations from strong LLM output.

Reference:

- https://aclanthology.org/2025.naacl-long.548/

Decision: Phase 21 does **not** collapse quality into one MQM-style score. Human scorecards are deliberately focused and bounded.

### MQM remains diagnostic, not literary authority

The WMT expert-evaluation work remains useful because its error hierarchy makes accuracy, fluency, terminology, style, and locale failures inspectable, and it demonstrated that professional translators with document context produce more reliable evaluations than crowd ratings.

References:

- https://github.com/google/wmt-mqm-human-evaluation
- https://aclanthology.org/2021.tacl-1.87/

Decision: reuse the idea of typed error/dimension evidence, but do not import WMT datasets or make MQM the literary acceptance criterion.

### Document-level metrics remain weak

MetaDocEval (EAMT 2026) reports that current metrics do not genuinely capture document-level coherence; reference-based metrics can overfit lexical overlap, and short context windows of roughly three sentences are more useful than indiscriminately feeding long documents.

Reference:

- https://aclanthology.org/2026.eamt-1.19/

The 2025 survey of document-level MT evaluation likewise identifies reference diversity, sentence-alignment dependence, and LLM-judge bias/interpretability as unresolved problems:

- https://arxiv.org/abs/2504.14804

Decision: Phase 21 evaluates continuity through explicit challenge cases and bounded nearby context rather than claiming that a system-level metric measures whole-novel coherence.

## Phase 21 benchmark architecture

### 1. Project-owned rights-safe challenge corpus

Committed fixture:

`benchmarks/phase21/corpus-v1.json`

All English passages, Persian references, and deliberately degraded translations are synthetic and authored for this repository. No third-party novel, subtitle, tweet, fanfiction, or proprietary manuscript text is committed.

Every corpus declares:

- schema version;
- source/target language;
- ownership/license note;
- explicit `rights_safe=true`;
- focused literary dimensions;
- deterministic anchors;
- contrastive degraded variants.

The Rust loader refuses committed benchmark corpora that are not explicitly rights-safe.

### 2. Contrastive regression, not absolute literary scoring

Each case includes one or more deliberately degraded variants. CI proves:

- the project-owned reference satisfies its deterministic anchors;
- each declared degraded variant actually triggers its intended failure class;
- a reference sanity submission scores above a degraded sanity submission.

Anchors are narrow factual/regression assertions. They may check polarity, terminology, concrete details, address/register forms, entity continuity, or known calques. They do **not** claim to measure overall literary quality.

### 3. Focused human scorecards

`literary-evaluation-engine` defines human scorecards with explicit reviewer expertise and 1–5 ratings.

One pass may rate at most four dimensions. This is deliberate: research indicates overly complex annotation schemes can reduce evaluation adequacy. Cases themselves also focus on at most four dimensions.

Suggested reviewer modes:

- professional translator;
- native target-language reader;
- trained reviewer.

Human notes and pairwise preference remain first-class. An overall literary-quality rating is recorded separately from deterministic anchors and automatic metrics.

### 4. Native Rust evaluation boundary

New crate:

`engine/crates/literary-evaluation-engine`

Responsibilities:

- corpus/submission schema validation;
- rights-safe provenance enforcement;
- deterministic anchor checks;
- contrastive fixture sanity checks;
- per-dimension coverage reporting;
- human scorecard schema/validation;
- optional reference-metric process boundary.

The crate does **not** own translation, canon, review approval, or model inference.

CLI:

`literary-engine benchmark <corpus.json> <submission.json> [--format json]`

The CLI reports deterministic challenge evidence and explicitly states that anchors are not a complete literary-quality score.

## SacreBLEU / chrF++ decision

Upstream:

`mjpost/sacrebleu`

Selected package:

`sacrebleu==2.6.0`

Release: 12 January 2026.
License: Apache-2.0.
Python: >=3.9.
Upstream release commit: `2277caccfc7b956671a6a09f1646f62250034157`.

References:

- https://github.com/mjpost/sacrebleu
- https://pypi.org/project/sacrebleu/

Phase 21 uses an isolated optional sidecar:

`tools/sacrebleu-evaluator`

It exposes **chrF2++** only. The tool receives project-provided hypotheses/references over stdin and does not download WMT or other external corpora.

Why chrF++ rather than BLEU as the default reference-overlap signal:

- character n-grams are less dependent on a language-specific tokenizer;
- chrF++ also includes word n-grams;
- Persian morphology/orthography and legitimate literary rephrasing make token-overlap-only BLEU especially brittle.

This does **not** make chrF++ an acceptance threshold. A faithful literary translation can legitimately diverge from one reference.

The sidecar package/version is pinned. Normal build, translation, review, and publication do not require Python/SacreBLEU.

## COMET/XCOMET decision

The existing optional COMET boundary remains sufficient. Do not add a second COMET runtime.

COMET/XCOMET may be run as optional evidence against reference cases or reference-free outputs, but:

- checkpoint/model licenses are reviewed separately;
- model weights are not normal-CI dependencies;
- scores/error spans do not equal approval;
- Phase 21 does not set a universal COMET pass threshold.

## Persian corpus candidate due diligence

### iPerUDT

- CC0 1.0 repository license;
- 3,000 informal Persian sentences / 54,904 tokens;
- useful for colloquial syntax diagnostics;
- not an EN->FA literary reference corpus.

Decision: possible future diagnostic evidence if a measured benchmark gap justifies it. Not installed in Phase 21 runtime.

### Mizan

- repository declares CC BY 4.0;
- about one million Persian-English sentence pairs described as collected from literary works.

Decision: do not vendor or use as committed benchmark gold until the rights/provenance of the underlying third-party literary texts and the suitability of alignments/references are separately verified.

### Degarbayan-SC

- repository license GPL-3.0;
- corpus described as aligned colloquial movie subtitles.

Decision: research only. Repository GPL does not by itself resolve rights in underlying movie-subtitle text.

### FarSSiM

- informal Persian semantic-similarity pairs derived from tweets;
- no explicit LICENSE/COPYING file found in the reviewed repository.

Decision: blocked from project ingestion/bundling absent explicit permission/license.

## Why no generic Persian NLP toolkit was installed

Phase 21 has not demonstrated a gap requiring Hazm, DadmaTools, Parsivar, ParsBERT, or another generic stack.

- Hazm remains blocked on the recorded dependency-security path until a patched compatible NLTK route is verified.
- DadmaTools remains conditional on a measured linguistic-diagnostic gap.
- aggressive informal-to-formal preprocessing would damage literary voice and is not acceptable default behavior.

The benchmark is intentionally capable of proving whether such a gap exists later.

## Validation requirements

Before Phase 21 can merge:

1. Rust lockfile validation, rustfmt, Clippy `-D warnings`, and tests.
2. Corpus schema/provenance validation.
3. Contrastive fixture sanity: every declared degradation must be observable by its intended deterministic challenge.
4. CLI reference/degraded benchmark smoke.
5. Optional SacreBLEU sidecar syntax/install/version/protocol smoke with `sacrebleu==2.6.0`.
6. Dependency audit for the Rust workspace and SacreBLEU environment.
7. No external corpora or model weights downloaded by normal Rust build/test.
8. Apple Silicon arm64 compile/test for benchmark surfaces.
9. Phase 18/19/20 and normal Rust/security gates remain green.
10. No benchmark score may be documented as human approval.

## Completion rule

Do not call Phase 21 canonical until its final PR head passes the permanent Phase 21 workflow plus normal repository gates, is merged to `main`, and the canonical status/PMC handoff is recorded.
