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

## Phase 20 — Publication-Grade EPUB Round Trip

- The production publication target is EPUB 3.3, the current stable W3C Recommendation. EPUB 3.4 remains a future migration target while it is a Candidate Recommendation.
- EPUBCheck 5.3.0 is the authoritative Phase 20 conformance gate because it validates EPUB 3.3. EPUBCheck 5.4.x is a future-compatibility signal, not an implicit standards migration while it validates EPUB 3 files against EPUB 3.4 rules.
- Reuse revision-pinned BookForge for source-aware EPUB reconstruction rather than adding another EPUB framework.
- Preserve exact source block provenance from ingestion through translated artifacts. Publication reconstruction uses an explicit `BookForge block ID -> translated text` map and fails closed for unknown, duplicate, missing, empty, or mismatched mappings.
- Never regenerate publication EPUB from flattened chapter text and never guess structure by paragraph order/count.
- BookForge owns marker-aware XHTML reconstruction plus target `dc:language`, `lang`, and `xml:lang` rewriting. The native publishing layer adds only the RTL metadata BookForge does not own: XHTML root `dir="rtl"` and OPF spine `page-progression-direction="rtl"`.
- Images, CSS, navigation, hyperlinks, notes, and other non-translated resources remain source-derived. Repeated exports from identical source/artifacts must be deterministic, and export must not mutate the imported source.
- The permanent Phase 20 gate uses a generated rights-safe EPUB fixture, EPUBCheck 5.3.0 before/after export, source checksum verification, byte-identical repeated export, representative asset/link/inline-markup preservation checks, and Linux plus Apple Silicon validation.
- `adult-intimacy` is an explicit literary-fidelity style profile, not automatic classification. It requires caller confirmation that all participants in sexual content are adults; confirmation must never be inferred. It preserves source explicitness/markedness, consent/refusal/coercion/power cues, agency, sensory information, POV, intensity, and pacing while detecting both sanitization and amplification. Its automated findings remain review evidence, never canon or human approval.

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
- guessed EPUB block alignment or whole-book regeneration from flattened translated text
- silently changing the publication standard merely because a newer validator exists

## Phase 25 — Narrative Speaker Intelligence

- Quotation speaker attribution is owned by `literary-intelligence-engine` and reuses `CharacterBible`; it must not become a parallel character/canon store.
- The first production baseline is deterministic and high-precision: explicit canonical-name/approved-alias plus nearby speech-verb patterns only.
- Prefer a wrong-speaker rate of zero on the committed regression corpus over artificially high coverage. Unsupported pronouns, implicit conversational alternation and unclear cases remain unresolved.
- Treat names inside quoted speech as content/vocatives, not speaker identity.
- Before a quote, reject `verb + name` as a generic speaker pattern because the name may be a speech verb's object/addressee.
- Bound explicit attribution cues to the target quote. Do not reuse pre-quote tags across a hard sentence boundary or cross another quotation; conflicting local character cues fail closed.
- Speaker-map context is Deterministic evidence. It may point to a Canonical character profile but does not itself gain Canonical authority.
- Permanent Phase-25 CI uses a project-owned synthetic benchmark and requires no model/corpus download.
- LitBank CC BY 4.0 is an optional future external/reference benchmark with attribution; PDNC/BookCoref/Maverick remain research-only under non-commercial terms.
- ModernBookNLP, BookNLP and FastCoref are not installed in Phase 25. Any future sidecar requires separate code/checkpoint/data licensing, resource/privacy review and benchmark-proven gain.
- No new external runtime dependency is justified for the native Phase-25 baseline.


## Phase 27 — Real-Book Pilot Readiness & Resume Integrity

- After Phase 26, the measured gap is operational full-book reliability rather than another model integration.
- Resume reuse is valid only when source, Context Packet, and translation-plan fingerprints all match.
- Translation-plan identity covers resolved provider/model, target language, style profile, a version tag, and the production pipeline contract. `max_chapters` remains an execution budget and is excluded.
- Resume reconstructs completed chapter/paragraph counts from currently valid checkpoints. Persisted counters are not blindly incremented.
- Reused checkpoints do not consume the current run's `max_chapters`; the budget limits newly translated chapters.
- Structured chapter artifacts are part of checkpoint validity; plain text plus fingerprint sidecars are not enough for safe reuse.
- Operational paragraph completion counts source paragraphs, not output alignment records.
- Export validates current-plan completeness and per-chapter plan/source identity before generating DOCX/EPUB, preventing mixed-plan books after partial retranslation.
- Missing legacy plan identity or a changed plan invalidates reuse and triggers regeneration instead of mixing outputs across configurations.
- The permanent Phase-27 rehearsal is a project-owned synthetic multi-chapter workflow. It proves restart/resume/export integrity, not literary quality.
- A real book remains private project data. GitHub, PMC/projectmem, Linear and CI may store only non-text operational metadata.
- Research through 2026 is treated conservatively: document-level metrics are not trusted as a single coherence oracle, and reported refinement recipes do not change the EN->FA production default until our own pilot shows a measured gain.
- No workflow engine, cloud backend, new provider/model runtime or second persistence owner is introduced by this baseline.

## Phase 28 — Private Whole-Book Audit & Human Review Sampling

- Whole-book audit is an application-layer read model over existing artifacts; it does not become a new persistence/canon owner.
- Mechanical artifact safety and human literary-review workflow state are separate dimensions.
- Audit serialization is intentionally text-free: stable IDs/fingerprints/counts/status codes only.
- Bounded human-review sampling covers book position plus deterministic risk/evidence signals instead of asking one evaluator to judge the whole novel at once.
- Existing Phase-19 literary review/alignment evidence is reused; no competing evaluator is introduced.
- No network/model call, metric package, analytics SDK, database, cloud backend or new runtime dependency is justified for the Phase-28 baseline.
