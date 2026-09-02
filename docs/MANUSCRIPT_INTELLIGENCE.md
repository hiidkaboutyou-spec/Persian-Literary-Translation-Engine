# Manuscript Intelligence

`literary-intelligence-engine` turns `document_engine::Manuscript` into a versioned,
machine-readable `ManuscriptIntelligence` aggregate before translation starts. The baseline analyzer
is deterministic, credential-free, provider-neutral, and stable for unchanged manuscript IDs,
content, configuration, and engine version.

## Ownership

- `document-engine` owns parsing, hierarchy, stable IDs, and source locations.
- `literary-intelligence-engine` owns evidence-derived seeds, chapter maps, observed profile metrics,
  and initialization proposals.
- `character-engine` and `memory-engine` continue to own approved Character Bible, relationship, and
  Glossary state.
- `translation-core` consumes bounded chapter-relevant proposal context; it does not analyze books.
- Human review owns approval, rejection, and promotion to canon.

Analysis never writes durable memory. Every generated seed has `status: inferred`, explicit
confidence based on supporting evidence count, and bounded evidence references to existing chapter,
scene, paragraph, and `SourceLocation` identifiers. No source excerpts are persisted in the
aggregate. Higher-level fields that cannot be established defensibly by deterministic rules remain
unset in the separate inferred literary-profile section.

Phase 14 consumes eligible seeds through the separate Human Review boundary. Re-running analysis
does not inspect or mutate the review ledger by itself; `literary-engine review sync` is the explicit
reconciliation operation. Approval remains a review decision, not a canonical mutation. See
`INTELLIGENCE_REVIEW_AND_CANON_PROMOTION.md` for the promotion workflow.

## Canon precedence and conflicts

Approved Character Bible identities and aliases are resolved before unknown character candidates.
Approved relationships are marked as canonical matches. Exact Glossary entries attach the approved
translation to a terminology seed, including Persian/Arabic-normalized matches. Ambiguous inferred
names that overlap an approved identity are reported as deterministic conflicts and excluded from
translation context. The initialization proposal declares `mutates_canon: false`.

## CLI

```bash
cd engine
cargo run -p literary-engine -- analyze ../input/story.epub
cargo run -p literary-engine -- analyze ../input/story.epub --format json
```

Text output is a compact count/metric summary. JSON output exposes schema version 1 and the complete
structured aggregate. The existing `run` and `resume` workflows analyze once before translation and
append only bounded, chapter-relevant inferred context after approved Character Bible, Glossary, and
Translation Memory context.

## Bounds

The default analyzer retains at most eight evidence locations per seed, 512 inferred character
candidates, 256 terminology seeds, and a bounded recurring-phrase working set. It uses ordered
maps/sets for stable output and does not create random identifiers. Evidence counts may exceed
retained evidence length so confidence reflects total support without unbounded provenance growth.
