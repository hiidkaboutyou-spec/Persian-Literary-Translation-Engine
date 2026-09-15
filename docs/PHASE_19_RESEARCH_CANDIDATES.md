# Phase 19 Research Candidates — Literary Fidelity & Persian Naturalness

Status: research shortlist only. Phase 19 implementation must not begin until Phase 18 is merged and post-merge verified.

## Selection rule

External projects are adopted only when they fill a measured capability gap and pass maintenance, license, platform, privacy, dependency, and failure-mode review. Heavy tooling remains optional and isolated. No automated score or linguistic tool may silently rewrite literary prose, mutate canon, or equal human approval.

## Shortlist

### roshan-research/hazm — preferred Persian linguistic diagnostic candidate

Current research snapshot:

- project: `roshan-research/hazm`
- observed package version: `0.12.1`
- source license: MIT
- Python requirement: `>=3.12,<3.14`
- useful capabilities: Persian normalization, tokenization, stemming/lemmatization, POS/chunking/dependency tooling and related Persian NLP primitives

Planned Phase 19 role: advisory Persian diagnostics only. Use it to surface suspicious tokenization/normalization/morphology/syntax patterns or untranslated/malformed spans where native checks are insufficient. Do not automatically normalize or rewrite literary output, because deliberate punctuation, colloquial spelling, code-switching, dialect, voice, and stylistic deviations can be intentional.

Integration constraint: Hazm must use its own isolated Python 3.12+ environment. Do not force the existing COMET environment (which is intentionally Python 3.11 oriented) to upgrade.

### thompsonb/vecalign — preferred document/sentence alignment candidate

Current research snapshot:

- project: `thompsonb/vecalign`
- observed package version: `2.0.0`
- latest inspected upstream revision: `f37262758955133d0c9ef1fdff45eba25842a62c` (2026-07)
- source license: Apache-2.0
- Python requirement: `>=3.11`
- supports Linux, macOS Intel, macOS arm64 and Windows through its documented Pixi setup
- requires Cython/Numpy plus a C compiler/Python development headers for builds

Planned Phase 19 role: source↔translation sentence/document alignment evidence for omission, addition, merge/split, or suspicious reorder detection. Alignment is evidence only and must not decide literary correctness by itself.

Do not vendor the upstream test/dev datasets automatically: the Vecalign repository notes that bundled Bleualign-derived dev/test data has GPL licensing separate from Vecalign's Apache-2.0 source. Only the Apache-licensed tool code is a candidate dependency.

### cisnlp/simalign — reference / fallback only

SimAlign can produce multilingual word alignments using transformer embeddings and is MIT-licensed, but its runtime pulls in PyTorch, Transformers, SciPy, NetworkX and scikit-learn. That dependency surface overlaps existing BGE semantic infrastructure and is too heavy for the first Phase 19 alignment layer.

Decision: do not install by default. Revisit only if sentence-level Vecalign evidence proves insufficient and word-level alignment delivers measurable omission/terminology benefits on the Phase 21 literary corpus.

### Dadmatech/DadmaTools — conditional only

DadmaTools offers Persian NER, POS, dependency/constituency parsing, chunking, ezafe detection, spellchecking and informal/formal tooling. It overlaps substantially with Hazm and has a larger model/dependency surface.

Decision: do not install alongside Hazm by default. Evaluate only if Phase 19 tests demonstrate a concrete NER/ezafe/syntax gap that Hazm plus native logic cannot cover.

## Rejected pattern

Do not install generic grammar/style rewriters that transform the Persian output automatically. Literary naturalness is not equivalent to formal-language normalization. Phase 19 reviewers should return evidence, dimensions, spans, and revision proposals; translation mutation remains an explicit controlled step.

## Intended implementation order after Phase 18

1. Native review contract with independent dimensions: omission/addition, semantic fidelity, voice, relationship/register, Persian naturalness, dialogue/subtext, terminology/continuity.
2. Vecalign sidecar for bounded source↔target alignment evidence.
3. Hazm sidecar for Persian-only linguistic diagnostics.
4. Existing Lingua + COMET evidence folded into the same review report without becoming approval gates.
5. Add stronger word-level/NER tooling only if benchmark failures prove a remaining gap.
