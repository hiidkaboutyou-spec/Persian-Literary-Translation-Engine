# Engineering Decisions

## Core

The engine is built around a Rust core. Python/external tooling is allowed only behind narrow optional process/tool boundaries when reproducing the capability natively is unjustified.

## Memory

The system keeps separate concepts:

- translation memory
- glossary memory
- character voice memory
- relationship context
- literary review evidence
- curated developer/project memory

Runtime literary/project memory, human canon, PMC, and developer-side projectmem are different ownership domains and must not be collapsed into one store.

## Product Direction

The target is a real user workflow:

Upload a story -> analyze -> review intelligence -> translate -> review translation -> edit -> publish/export a professional Persian manuscript.

## Phase 18 — Context Retrieval

- Context Packet v2 is native and shared by CLI/ApplicationService.
- Deterministic lexical/polarity/diversity retrieval remains the fallback floor.
- BGE-M3/FastEmbed may augment retrieval only through optional bounded tooling.
- Semantic tooling cannot invent canon IDs, own project memory, or break translation when unavailable.
- Context packet fingerprints participate in resume validity.

## Phase 19 — Literary Review

- Post-translation literary review is a separate domain from the deterministic blocking quality gate and from human intelligence/canon review.
- Automated review produces evidence and revision proposals; it never auto-applies a rewrite or marks text human-approved.
- Unevaluated dimensions stay explicitly unevaluated; provider/tool failure is not a pass.
- Persisted review artifacts are fingerprinted against source, translated text, and translation context; manual edits make old evidence stale.
- Alignment is native bounded monotonic Rust dynamic programming. Reuse the existing optional BGE-M3 embedding boundary instead of adding a second embedding stack.
- Vecalign and SentWeave remain algorithm/research references while native alignment satisfies the measured need.
- Hazm remains blocked until its mandatory NLTK dependency has a patched compatible version and a fresh security audit passes.
- DadmaTools is conditional on a measured Persian NLP gap, not feature accumulation.

## External-Dependency Rule

Before adopting a GitHub repository/package/model/tool:

1. prove a concrete capability gap;
2. review license, provenance, maintenance, dependencies, model/data licenses, privacy, and failure modes;
3. prefer a reasonably small native Rust capability when safer;
4. keep heavy/model-backed tools optional with deterministic fallback;
5. benchmark on project-owned/rights-safe fixtures;
6. validate relevant Linux/macOS platforms and security audits;
7. keep automated evidence separate from canon/human approval.

## Supporting Tooling Direction

- OpenDataLoader PDF is a future ingestion benchmark candidate, not a replacement for `document-engine`.
- ripwire is a future developer-only code-intelligence/MCP candidate, never a production runtime dependency.
- Headroom is conditional developer/research context compression; do not place lossy compression in literary runtime/provider context without a fidelity benchmark.

## Avoid

- simple word replacement
- stateless translation
- losing character voices between chapters
- architecture that prevents future model providers
- duplicate memory/canon owners
- silent fallbacks that hide external-tool failure
- importing large frameworks when a narrow capability is all the project needs
