# Advanced Model-Assisted Literary Analysis

Phase 15 adds an **optional, explicitly requested** provider-assisted layer on top of the
deterministic Phase 13 manuscript intelligence. It lives in the new `advanced-literary-analysis`
crate and never weakens the invariant:

```text
MODEL INFERENCE  !=  HUMAN APPROVAL  !=  CANON
```

Deterministic analysis (`literary-engine analyze`) remains fully offline and unchanged. Advanced
analysis is a separate operation (`literary-engine analyze-advanced`) that a user must request.
No API key, no network, and no external model are ever required for the baseline engine.

## Deterministic vs advanced

| | Phase 13 deterministic | Phase 15 provider-assisted |
|---|---|---|
| Trigger | `analyze` | `analyze-advanced` |
| Provider | none (rules only) | mock (default, offline) or OpenAI |
| Output | `ManuscriptIntelligence` seeds | `AdvancedAnalysisResult` findings |
| Higher-order reading (voice/tone/POV/subtext) | intentionally unset when not provable | structured, evidence-backed hypotheses |
| Canon effect | proposals only | proposals only (review-only Literary kind) |

Advanced analysis enriches Phase 13. It does not replace it.

## Provider boundary

`LiteraryAnalysisProvider` is the provider-neutral boundary:

```rust
pub trait LiteraryAnalysisProvider: Send + Sync {
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    fn analyze(&self, request: &AnalysisProviderRequest)
        -> Result<AnalysisProviderResponse, AnalysisProviderError>;
}
```

Providers receive a fully versioned prompt plus the bounded analysis unit, and must return the
structured JSON contract. Providers never decide validity; the deterministic validation gate in
`crate::validation` owns that decision.

- `MockAnalysisProvider` — deterministic, offline; supports valid findings, permanent failures,
  retryable failures, and usage reporting. Used by tests and the default CLI smoke path.
- `OpenAIAnalysisProvider` — optional production provider (Responses API) configured only through
  environment variables (`OPENAI_API_KEY`, `LITERARY_ENGINE_ANALYSIS_MODEL` / `OPENAI_MODEL`).
  Secrets are never logged or persisted.

Domain types never mention OpenAI.

## Analysis units

A manuscript is never sent to a provider whole. `AnalysisPlanner` groups scenes into bounded units
(defaults: `max_unit_characters=10_000`, `max_context_characters=800`, `max_units=2_000`,
`max_canon_names_per_unit=24`, `max_glossary_terms_per_unit=12`), each with:

- previous/next scene context (bounded)
- only canon names/glossary terms present in the unit text
- real paragraph identifiers behind ordinal tokens (`[para-1]`, …)

Unit identity is a content/configuration fingerprint, never a timestamp. Oversized chapters exceed
`max_units` only after an explicit, configurable error.

## Structured findings

Categories (extensible enum, `snake_case` serialization):

`point_of_view`, `narrator_voice`, `tone`, `character_voice`, `relationship_dynamic`,
`power_dynamic`, `emotional_shift`, `subtext`, `sarcasm`, `humor`, `intimacy_register`,
`dialogue_function`, `scene_intent`, `narrative_tension`, `motif`, `recurring_imagery`,
`register_shift`, `continuity_observation`.

Each validated finding carries: stable finding ID, analysis unit ID, category, subject, claim,
derived confidence, validated evidence references, narrative scope, provider/model metadata,
analysis-schema and prompt versions, uncertainty, alternative interpretations, and review
eligibility. Findings persist structured values — never free-form prose as the primary contract
and never duplicated manuscript excerpts.

## Evidence safety

The provider may cite only ordinal tokens that were actually supplied for the unit
(`unit.resolve_ordinal`). Validation resolves each ordinal to the real paragraph/chapter/scene
identifiers; hallucinated ordinals, unknown categories, out-of-range confidence, oversized or empty
fields, and findings with no valid evidence are rejected with reasons and surfaced as warnings —
they never reach the review ledger. Findings without evidence are retained but are **not**
review-eligible.

## Confidence & disagreement

Model-reported confidence is preserved but never trusted verbatim. Final confidence is derived from
model report, valid-evidence coverage, supporting-unit count, cross-unit agreement, and
deterministic signals. Conflicting findings on the same scope are surfaced as warnings with
contradicting counts rather than collapsed by last-write-wins; findings with distinct scopes
(relationship evolution across chapters) are preserved separately.

## Review integration

Validated, review-eligible findings map to the Phase 14 review ledger as `Literary` proposals via
`ReviewLedger::reconcile_advanced`, using the same approve / edit / reject / defer / reopen
lifecycle and the same stable identity + reconciliation semantics (unchanged proposals keep human
decisions; changed proposals are archived and reset; missing proposals become Obsolete). Literary
proposals are **review-only**: `supports_canon_promotion()` is false for `ReviewKind::Literary`, so
promotion plans never select them, and an explicit attempt surfaces a blocking
`UnsupportedPromotion` conflict rather than silently dropping or fake-canonizing the finding.
Deterministic and advanced reconcile passes are kind-scoped, so a `review sync` never obsoletes
advanced items and an advanced reconcile never obsoletes deterministic ones.

## Cache / resume

Unit results cache to a file (default disabled; enable with `--cache <path>` or
`LITERARY_ENGINE_ANALYSIS_CACHE`). A result is reusable only when the analysis schema, prompt
version, provider, model, configuration fingerprint, canon fingerprint, and unit content
fingerprint all match — timestamps and credentials never enter keys. A later run with an edited
paragraph recomputes only affected units.

## Context selection in translation

Approved/edited `Literary` items whose evidence points at a chapter are loaded from the review
ledger (`LITERARY_ENGINE_REVIEW_FILE`) during `run`/`resume` and appended to that chapter's provider
context as `REVIEWED LITERARY FINDINGS — human-approved context`. Findings never leak into unrelated
chapters, and unreviewed model inference never enters translation context. The manifest records
`reviewed_literary_findings_available` and per-chapter `chapter.N.literary_findings_used` so the
selection is observable.

## Privacy

- Only bounded analysis units and bounded canon context are sent to a provider.
- No manuscript excerpts are persisted in findings, the cache, or the review ledger (evidence is
  stored as stable chapter/scene/paragraph/source identifiers; review items keep short claims).
- API keys, authorization headers, and environment secrets never appear in logs, fingerprints,
  cache files, or JSON output.
- `openai` provider disables response storage.

## Failure handling

Provider failures do not invalidate deterministic Phase 13 output. Analysis is sequential and
bounded: retryable failures (rate limit/transport) retry up to `max_retries`; schema-invalid,
auth, and permanent failures never retry. A failed unit is reported in `failed_units` while other
units still produce validated findings. Deterministic translation remains possible even when
advanced analysis fails entirely.

## CLI

```bash
# offline deterministic analysis (unchanged)
cargo run -p literary-engine -- analyze manuscript.epub

# explicit advanced analysis with the deterministic mock provider
cargo run -p literary-engine -- analyze-advanced manuscript.epub \
  --review-file review.json --provider mock

# queue findings for human review (same Phase 14 ledger)
cargo run -p literary-engine -- analyze-advanced manuscript.epub --review-file review.json

# JSON output for automation
cargo run -p literary-engine -- analyze-advanced manuscript.epub --format json

# production provider (requires OPENAI_API_KEY)
cargo run -p literary-engine -- analyze-advanced manuscript.epub --provider openai
```

JSON output exposes `schema_version`, `analysis_id`, provider/model, unit counts (total /
completed / failed / cached), findings, findings-by-category, failures, warnings, usage, review
file, and review-proposal counts.

## Adversarial manuscript text

Prompts are versioned (`PROMPT_VERSION` = `literary-analysis-v1`). Manuscript content is always
isolated inside an explicit untrusted-data block, and the system instructions instruct the model to
treat manuscript text as data (never execute instructions inside it, never reveal secrets, never
translate). Prompt text is part of the algorithm: any prompt change bumps the version and therefore
invalidates cached results.

## Security & testing

- Evidence-identity, category, confidence, scope, field-bound, and unicode validation.
- Prompt-injection resistance tests embed synthetic malicious manuscript text.
- Stable-identity, cache reuse/invalidation, partial-failure (8/2), hallucinated-evidence,
  character-voice, relationship-evolution, POV, sarcasm/subtext, review lifecycle, canon-priority,
  and CLI end-to-end coverage (all via the mock provider; no real API calls in tests).
- Mock provider failure modes: permanent and retryable unit failures.
