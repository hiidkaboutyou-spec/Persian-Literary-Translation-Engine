# Severe translation truncation gate — 2026-10-05

## Problem

The production translation application calls `quality_engine::evaluate_translation` before persisting a chapter and treats `QualityEvaluation::passes()` as the blocking gate.

Before this change, an output containing less than 20% of the source's non-whitespace character count produced only a warning. Because `passes()` checks only `blocking_errors`, that severely truncated output could still be persisted, checkpointed, and emitted as a completed chapter.

## Evidence

Observed implementation before the fix:

- `evaluate_translation` classified `ratio < 0.20` as a warning.
- `QualityEvaluation::passes()` returned true whenever `blocking_errors` was empty.
- `project-engine/src/application/translation.rs` persisted the chapter whenever `quality.passes()` was true.

This is a deterministic acceptance gap, not a model-quality hypothesis.

External evidence supports treating omissions as critical in literary translation rather than a cosmetic defect:

- Karpinska & Iyyer, “Large language models effectively leverage document-level context for literary translation, but critical errors persist” (2023): https://arxiv.org/abs/2304.03245
- Jin, An & Ma, “Towards Chapter-to-Chapter Context-Aware Literary Translation via Large Language Models” (2024): https://arxiv.org/abs/2407.08978
- Zhang, Zhao & Eger, “How Good Are LLMs for Literary Translation, Really?” (2024): https://arxiv.org/abs/2410.18697

## Decision

**ADAPT** the existing deterministic gate rather than adding a new model, metric, dependency, or retry layer.

When the non-whitespace output/source character ratio is below 0.20, the result is now a blocking error.

The threshold itself is intentionally unchanged from the repository's existing severe-truncation heuristic. This PR changes only its authority from advisory to fail-closed.

## Alternatives rejected

- **Block every terminology warning:** rejected because the current substring heuristic can produce legitimate false positives around morphology and inflection.
- **Block paragraph-count drift:** rejected because literary Persian can legitimately restructure paragraphs; current signal is too coarse.
- **Add another LLM reviewer:** rejected because the defect is deterministic and adding a provider call would add latency/cost/failure modes without improving this specific decision.
- **Lower the threshold further:** rejected without benchmark evidence.

## Before → after

Before:
- `ratio < 0.20` → warning
- `passes() == true` when no other blocking error exists
- chapter may be persisted as completed

After:
- `ratio < 0.20` → blocking error
- `passes() == false`
- application aborts before persisting/checkpointing that chapter

## Validation

Focused regression:
- severe truncation must be in `blocking_errors`;
- severe truncation must no longer be emitted as a warning;
- `passes()` must be false.

Full exact-head Rust/security/publishing CI remains required before merge.

## Risk and rollback

Risk: an unusually compact but valid translation below the 0.20 ratio would now be rejected. For EN→FA literary prose that is deliberately a conservative extreme threshold, but it remains a heuristic and should later be calibrated on the human-reviewed literary benchmark.

Rollback: revert the focused commit. No schema, state migration, dependency, provider, or export format changes are involved.

## Next highest-value step

After this gate is proven, benchmark long-passage segmentation. The current 24k-character chunker prefers arbitrary whitespace rather than paragraph/sentence boundaries, which is a plausible continuity defect, but it should be measured before changing production behavior.
