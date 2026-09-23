# Phase 32 Research — Blind Human Provider Review

## Purpose

Phase 31 can generate rights-safe provider submissions and a counterbalanced blind A/B bundle, but it deliberately stops before interpreting human preference. Phase 32 adds the missing evidence layer: a blind, auditable human review ledger and a post-review dossier. It remains research-only and does not change the production provider selector.

## Evidence behind the design

1. Zhang, Zhao, and Eger, **“How Good Are LLMs for Literary Translation, Really?”**, NAACL 2025, found that literary-translation evaluation is sensitive to evaluator expertise and scheme complexity. In their setup, the simpler Best-Worst Scaling approach was substantially more adequate than complex MQM with student evaluators, while automatic metrics performed poorly. DOI: `10.18653/v1/2025.naacl-long.548`; https://aclanthology.org/2025.naacl-long.548/
2. Song, Riley, Deutsch, and Freitag, **“Enhancing Human Evaluation in Machine Translation with Comparative Judgement”**, ACL 2025, reported higher inter-annotator agreement for side-by-side settings than pointwise MQM and described simplified side-by-side relative ranking as an efficient alternative. DOI: `10.18653/v1/2025.acl-long.1002`; https://aclanthology.org/2025.acl-long.1002/
3. Hiebl and Gromann, **“Comparative Quality Assessment of Human and Machine Translation with Best-Worst Scaling”**, EAMT 2024, provides additional evidence that comparative preference judgments are a practical way to collect subjective translation-quality evidence. https://aclanthology.org/2024.eamt-1.42/

These papers do **not** prove that a provider is suitable for this project or for English→Persian literary translation. They motivate the evaluation *shape*: comparative, blinded, simple, and explicitly human-reviewed.

## Phase 32 design

Phase 32 is integrated as `literary-engine blind-review ...` rather than as a second Cargo binary. This preserves the established `cargo run -p literary-engine -- ...` contract used by Phase 31 and leaves Cargo dependency manifests unchanged.

### Blind review ledger

`literary-engine blind-review init` consumes only the Phase 31 blind bundle. It does not accept the reveal key. The resulting ledger contains:

- corpus id;
- fingerprint of the exact blind-bundle bytes;
- reviewer id;
- one pending record per case;
- no provider/model/system identity.

### Judgment recording

`literary-engine blind-review record` records one of:

- Candidate A;
- Candidate B;
- tie;
- defer / cannot judge.

Every judgment requires a non-empty reason. Optional notes may be recorded for adequacy, voice/style, cultural/pragmatic fit, and continuity. Tie and defer are first-class outcomes; the tool never forces a preference.

### Reveal and dossier

`literary-engine blind-review dossier` is the only command that consumes the Phase 31 reveal key. It requires complete ledgers, rejects duplicate reviewer ledgers, requires all ledgers to share the same blind-bundle fingerprint, verifies exact case-id agreement with the reveal key, and then maps A/B preferences to the underlying system ids.

The dossier reports preference counts, ties, deferrals, and multi-reviewer agreement/disagreement. It intentionally emits:

- `automatic_winner: null`;
- `human_comparative_evidence_only: true`;
- `production_admission: "not_granted"`;
- `requires_explicit_admission_decision: true`.

A preference count is evidence, not authority to change production configuration.

## Safety boundaries

- Rights-safe qualification material only. No private manuscripts.
- The reveal key is structurally absent from review initialization and recording.
- Phase 32 adds no Atria/OpenAI provider execution and no production-provider selector wiring.
- New outputs fail closed on collisions with protected input files.
- Dossier generation rejects duplicate input paths so the same ledger cannot be counted twice under different CLI positions.
- Writes use a same-directory temporary file and rename, reducing partial-ledger corruption risk.
- No new runtime dependency or Cargo manifest change is introduced.

## Known binding limitation inherited from Phase 31

The Phase 31 reveal-key schema v1 identifies the corpus and exact case-id assignments but does not include the blind-bundle byte fingerprint. Phase 32 therefore binds all reviewer ledgers to the *same* blind bundle by fingerprint and separately checks the reveal key’s corpus/case set. The dossier records this binding strength explicitly instead of pretending it is cryptographic provenance. A future schema may place the bundle fingerprint in the reveal key itself, but Phase 32 does not silently rewrite Phase 31 artifacts.

## Production admission remains separate

Even strong human preference evidence is insufficient for production admission. Any future provider-admission stage must separately address privacy/retention terms, data handling, rights, reliability/failure behavior, cost/limits, and explicit project-owner authorization. Phase 32 cannot grant that admission.