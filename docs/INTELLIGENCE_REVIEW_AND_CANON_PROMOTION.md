# Literary Intelligence Review and Canon Promotion

Phase 13 produces bounded, evidence-backed inferences. Phase 14 adds the human-controlled path from
those inferences to durable project knowledge.

```text
Manuscript analysis -> review sync -> human decision -> promotion preview -> explicit apply -> canon
```

Two invariants define the boundary:

```text
inference != canon
review decision != persisted canon mutation
```

## Ownership

- `literary-intelligence-engine` owns inferred seeds, evidence, confidence, and Phase 13 IDs.
- `human-review-workflow` owns review identity, lifecycle, reconciliation, conflicts, plans, and audit.
- `character-engine` owns canonical profiles, aliases, and relationship state.
- `memory-engine` owns the canonical Glossary.
- `project-engine` owns atomic ledger writes and recoverable multi-file promotion transactions.
- `literary-engine` only orchestrates these boundaries.

No Chapter Map or observed analysis metric is promoted. Current Phase 13 leaves inferred literary
profile fields unset, and the runtime has no durable literary-rule owner, so Phase 14 deliberately
does not invent one. A future literary-rule proposal receives an unsupported-promotion error until
that owner and its runtime precedence contract exist.

## Ledger and stable identity

The review ledger is local JSON with schema version 1. It stores bounded evidence references, never
chapter excerpts. A review ID is a SHA-256 digest over the ledger/analysis schema, manuscript ID,
proposal kind, and Phase 13 subject/seed identity. It is not random.

A separate semantic fingerprint detects meaningful changes:

- characters include canonical candidate, aliases, and promotable observations;
- relationships include the normalized unordered pair, candidate meaning, and forms of address;
- terminology includes the normalized source expression and category.

Evidence count, confidence, and evidence locations are deliberately excluded. More evidence can
therefore enrich a deferred or rejected item without erasing the human decision. A changed semantic
value archives the prior proposal, replacement, state, and decisions, increments the revision, and
returns the current revision to `pending`. Missing proposals become `obsolete` and cannot be
promoted. A proposal that is now satisfied by externally approved canon also becomes obsolete rather
than returning to the pending queue. Unchanged re-analysis preserves rejected, deferred, approved,
and applied state.

The ledger is bound to one manuscript ID. Reusing it for another manuscript fails instead of mixing
private project knowledge.

## Lifecycle

Supported transitions are:

```text
Pending  -> Approved | Edited | Rejected | Deferred
Deferred -> Approved | Edited | Rejected
Approved -> Applied (promotion only)
Edited   -> Applied (promotion only)
Approved | Edited | Rejected -> Pending (explicit reopen only)
Applied  -> terminal
```

`approve` accepts the inferred value. `edit` accepts a typed replacement while retaining the original
inference and evidence. `reject` remains traceable and survives unchanged analysis. `defer` remains
unresolved and can accumulate evidence. Every operation requires a reviewer and reason.

Terminology cannot be approved without a non-empty preferred translation. A relationship
co-occurrence cannot be approved until a human supplies dynamics, address, or boundary meaning.

Structured edit JSON uses a tagged value. For example:

```json
{
  "kind": "terminology",
  "value": {
    "source_term": "Duke",
    "preferred_translation": "دوک اعظم",
    "context": "formal title"
  }
}
```

Character replacements contain `canonical_name`, `aliases`, `voice_notes`, and
`personality_notes`. Relationship replacements contain `character_a`, `character_b`,
`dynamic_notes`, `address_notes`, and `boundaries_notes`.

## Conflicts and resolutions

Promotion plans use a typed severity field; current behavior emits informational or blocking
conflicts. Implemented conflict paths are:

- canonical character identity already used as another character's alias;
- character alias owned by another canonical character;
- existing character profile differs;
- approved Glossary translation differs;
- a legacy Glossary contains multiple normalized translations for one term;
- normalized relationship pair has contradictory state;
- proposal is stale/obsolete;
- proposal is already applied.

Existing approved canon is never overwritten implicitly. A blocking conflict prevents apply until a
supported resolution is supplied in a JSON array:

```json
[
  {
    "conflict_id": "conflict-...",
    "resolution": "use_reviewed_proposal"
  }
]
```

Supported resolution names are `keep_canon`, `use_reviewed_proposal`, `merge`, and
`cancel_promotion`. Each conflict lists its allowed subset. A colliding alias cannot be stolen from
another character; edit the proposal, keep canon, or cancel that promotion. Glossary values do not
support an ambiguous merge. A legacy duplicate Glossary term must be replaced with the reviewed
value or cancelled; explicit replacement collapses it to one entry. Relationship/profile merge is
deterministic and preserves existing non-empty canon while adding compatible reviewed information.

## Preview and apply

`review promote --dry-run` loads current canon, validates all selected approved/edited revisions,
computes exact operations and before/after values, reports conflicts, and writes nothing. Omit
`--item` to preview all approved-not-applied items.

The returned plan ID binds:

- review item IDs and revisions;
- Character Bible and Glossary fingerprints;
- conflict resolutions;
- exact operations and outcomes.

Apply requires that ID:

```bash
literary-engine review promote --apply --plan-id promotion-... \
  --review-file review.json \
  --character-bible character-bible.json \
  --glossary glossary.json \
  --reviewer editor --reason "Apply approved literary canon"
```

If review or canon changed after preview, the regenerated plan ID differs and apply fails as stale.
Applying a recorded plan again returns `already_applied` without writing duplicates.

## Atomicity and recovery

Decisions use same-directory staged writes and atomic replacement. Promotion builds and serializes
the complete resulting Character Bible, Glossary, and ledger before canon changes. It then writes a
recovery journal containing before/after fingerprints and same-directory backups. Any ordinary
failure restores all prior files. After process interruption, the next ledger/promotion operation
detects the journal: a fully written result is finalized; a mixed result is rolled back. The review
item is never marked applied separately from its canonical mutation.

The durable promotion record embeds the plan, including before/after values, evidence references,
review item/revision, resolutions, and reviewer decision history. It can answer what changed, why,
which proposal caused it, whether a human edited it, and which canon values existed before and after.

## CLI reference

```text
review sync <manuscript> --review-file <path>
review list --review-file <path> [--status <filter>] [--kind <filter>]
review show <id> --review-file <path>
review approve <id> --review-file <path> --reviewer <name> --reason <text>
review edit <id> --review-file <path> --replacement <file|-> --reviewer <name> --reason <text>
review reject <id> --review-file <path> --reviewer <name> --reason <text>
review defer <id> --review-file <path> --reviewer <name> --reason <text>
review reopen <id> --review-file <path> --reviewer <name> --reason <text>
review promote --dry-run --review-file <path> [--item <id> ...] [--resolutions <file|->]
review promote --apply --plan-id <id> --review-file <path> [--item <id> ...]
```

Path flags override environment variables. The review ledger uses
`LITERARY_ENGINE_REVIEW_FILE`; Character Bible and Glossary use their existing variables. Apply
requires durable canonical paths. All operations accept `--format json`, and JSON outputs include a
schema version. Queue filters include `all`, `pending`, `deferred`, `approved-not-applied`,
`conflicted`, `rejected`, `applied`, and `obsolete`; kind filters include `character`, `relationship`, and
`terminology`.

## Translation and resume

Promotion writes the same canon files that `run` and `resume` already load. Approved Character
Bible/relationship and Glossary/Translation Memory context remains ahead of unresolved Phase 13
inference in provider context.

Chapter checkpoints now contain schema version, source fingerprint, and the fingerprint of the
actual assembled chapter context. A canon change invalidates reuse instead of silently continuing
under stale knowledge. Legacy source-only checkpoints are recognized but conservatively translated
once because they cannot prove context compatibility.
