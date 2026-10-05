# Literary semantic chunk-boundary repair — 2026-10-05

## Observed

The production provider pipeline bounds oversized passages at 24,000 characters. The current splitter is Unicode-safe and lossless, but when a passage exceeds the limit it chooses the last whitespace before the hard boundary.

That means a nearby paragraph or sentence boundary can be ignored in favor of a later word boundary inside the next paragraph/sentence.

## Evidence

Current `translation-core/src/pipeline.rs` before this change:
- computes a Unicode-safe hard boundary;
- scans backward for any whitespace;
- has no paragraph or sentence preference.

The previous severe-truncation repair explicitly recorded long-passage segmentation as the next quality risk to measure. This refresh is based on current main after PR #158; that EPUB preview change does not touch translation-core passage segmentation.

A deterministic regression fixture demonstrates the gap: when a paragraph break is safely inside the final 40% of chunk capacity and later ordinary spaces also exist, the old algorithm chooses the later ordinary space and splits inside the following paragraph.

## Research

Compared:
- `text-splitter` (Rust): semantic hierarchy from character/word through Unicode sentence and newline levels;
- recursive splitter patterns used by LangChain-style implementations;
- chapter-context literary translation research showing that discourse/context preservation matters for literary translation.

Decision: **ADAPT**, not ADOPT.

No dependency is added. The existing small Unicode-safe splitter is retained, with a bounded hierarchy:
1. nearby paragraph break;
2. nearby sentence boundary;
3. whitespace;
4. Unicode-safe hard boundary.

Natural boundaries are considered only after 60% of capacity is filled so an early short paragraph/sentence does not create pathologically tiny chunks.

## Change

`split_passage` now delegates boundary choice to `preferred_semantic_boundary`.

Supported sentence terminators cover common English/Persian/CJK punctuation: `.`, `!`, `?`, `؟`, `。`, `！`, `？`.

The splitter remains byte-safe, character-bounded and lossless: concatenating chunks reconstructs the exact input.

## Regression / acceptance

Added fixtures proving:
- nearby paragraph boundary beats a later word boundary;
- nearby sentence boundary beats a later word boundary when no paragraph boundary fits;
- early natural boundaries do not force tiny chunks;
- a sentence terminator exactly on the hard limit keeps the complete sentence in the current chunk;
- existing Unicode/lossless and max-size properties remain.

## Before -> after

Before: semantic boundary near capacity -> ignored if later whitespace exists.

After: semantic boundary near capacity -> preferred without exceeding the existing character limit.

## Alternatives rejected

- Add `text-splitter` dependency now: rejected because this bounded fix does not need a new runtime dependency/MSRV/transitive surface.
- Embedding-based semantic chunking: rejected as overengineering for provider passage segmentation and would add latency/failure modes.
- Overlap/retranslate neighboring chunks: deferred until a literary benchmark proves benefit; it complicates exact reassembly and duplicate suppression.

## Risk / rollback

Risk: different provider request boundaries can change model output. The algorithm does not alter text, order, maximum size, provider selection, context payload or persistence schema.

Rollback: revert the focused translation-core commit.

## Validation status

Exact-head Rust/CI validation is required before merge. Human literary evaluation is still needed to claim improved translation quality; this PR proves only that chunk boundaries preserve higher-level textual structure more often.
