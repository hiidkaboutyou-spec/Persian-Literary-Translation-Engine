# Phase 20 Research — Publication-Grade EPUB Round Trip

Status: branch implementation on `phase-20-publication-epub-roundtrip`; not canonical until PR merge and post-merge verification.

Research snapshot: 2026-09-17.

## Goal

Produce a translated Persian EPUB from an imported EPUB while preserving source publication structure and assets deterministically. Publication export must fail closed when native literary translation provenance is incomplete or structural markers are damaged. DOCX export remains a first-class independent path.

## Standards decision

### Publication target: EPUB 3.3

EPUB 3.3 is the current W3C Recommendation (13 January 2026). EPUB 3.4 is still a Candidate Recommendation Draft as of 3 August 2026. Phase 20 therefore targets the stable EPUB 3.3 publication contract rather than silently moving the product to a draft standard.

For Persian/RTL output:

- package primary `dc:language` must be the target language (`fa` for Persian);
- XHTML root language metadata must be rewritten to the target language (`lang` and `xml:lang` when applicable);
- XHTML content documents receive `dir="rtl"` for an RTL target;
- the OPF spine receives `page-progression-direction="rtl"` so global page progression is explicit;
- local element-level directionality in source markup remains available to override the document base direction when required.

### EPUBCheck gate: 5.3.0 for Phase 20

EPUBCheck 5.3.0 explicitly validates EPUB 3.3 and remains the authoritative Phase 20 conformance gate. The installer is checksum-pinned and the distribution is not vendored.

EPUBCheck 5.4.0 became the latest production-ready EPUBCheck release on 15 September 2026, but EPUB 3 publications are checked against EPUB 3.4 rules. Because EPUB 3.4 is still a Candidate Recommendation, 5.4.0 is treated as a future-compatibility signal rather than a reason to redefine the Phase 20 publication target. Revisit after EPUB 3.4 becomes a W3C Recommendation or a deliberate project migration is approved.

Durable policy: validator versions are pinned to the publication standard the product claims to support. A newer validator must not silently upgrade the product's normative target; validator and standard move together only after an explicit standards-migration decision and regression pass.

References:

- W3C EPUB 3.3 Recommendation: https://www.w3.org/TR/epub-33/
- W3C EPUB 3.4 Candidate Recommendation Draft: https://www.w3.org/TR/epub-34/
- EPUBCheck releases/change log: https://github.com/w3c/epubcheck/releases and https://github.com/w3c/epubcheck/blob/main/CHANGELOG.md

## BookForge boundary

Approved upstream remains revision-pinned:

`JunjoSick/bookforge@23f8c9d3c97a06f48e13424698441bfb4b037844`

Phase 20 deliberately reuses BookForge instead of introducing another EPUB framework.

Observed upstream capabilities used by the project:

- stable block IDs and DOM-backed block reconstruction;
- inline marker tokens for preserving nested inline markup during translation;
- deterministic ZIP rewriting;
- source resource preservation for files that do not require modification;
- replacement-mode primary OPF language rewrite;
- replacement-mode XHTML `lang` / `xml:lang` rewrite;
- translated EPUB validation and block-marker validation.

Native `document-engine` remains the product boundary. BookForge IR is not persisted as project schema.

## Round-trip architecture

### Ingestion provenance

`SourceLocation` carries an optional `block_id`. BookForge EPUB ingestion records each native literary source block ID on the corresponding heading/paragraph provenance.

### Translation artifacts

`TranslatedParagraph` carries `source_block_id`. `TranslatedChapter` may carry a real source heading block ID and translated heading. Legacy/non-EPUB artifacts remain backward compatible through optional/defaulted fields.

### Export mapping and completeness ownership

EPUB reconstruction uses an explicit mapping:

`BookForge block ID -> reviewed translated text`

Completeness is intentionally owned at the native project/application boundary, not by blindly translating every block BookForge can model.

The project layer must fail closed when any native literary translation unit lacks exact source block provenance, when translated paragraph identity does not match source provenance, or when an EPUB heading with a source block has no translated heading. It never guesses by paragraph position or count.

The lower `document-engine`/BookForge boundary validates the explicit mappings it is given and rejects:

- unknown block IDs;
- duplicate provided block IDs;
- empty translated blocks;
- damaged structural marker tokens;
- invalid rebuilt XML/package structure.

BookForge may additionally expose package, navigation, or page-furniture text that is not a native literary translation unit. Unprovided BookForge-only blocks remain source-derived rather than being forced through the literary translator. This is deliberate: publication completeness means every native literary unit is accounted for, not that metadata/navigation must be rewritten as prose.

### Structural preservation

BookForge performs the source-aware XHTML rebuild. The native Phase 20 layer applies only publication metadata/directionality that BookForge does not own:

- `dir="rtl"` on XHTML root documents for RTL targets;
- `page-progression-direction="rtl"` on the OPF spine.

Images, CSS, navigation resources, links, footnotes/endnotes, and other non-translated archive resources must remain source-derived. The project must not regenerate the whole book from plain chapter text.

### Determinism

Rebuilt ZIP entry timestamps are normalized by the structural writer. The Phase 20 gate exports the same fixture twice and byte-compares outputs. The source EPUB is checksummed before and after the round trip to prove export does not mutate the imported source.

## Validation stack

Phase 20 requires all of the following before merge:

1. Rust formatting and Clippy `-D warnings`.
2. `document-engine`, `project-engine`, CLI, `translation-core`, and literary-review tests.
3. Explicit `document-engine --no-default-features` compatibility build.
4. BookForge internal translation/marker/rebuilt-package validation.
5. EPUBCheck 5.3.0 validation of a rights-safe generated EPUB 3.3 fixture before and after translation/export.
6. End-to-end `ApplicationService`/CLI project flow: create -> import EPUB -> analyze -> EchoProvider translate -> EPUB export.
7. Assertions that CSS/image bytes survive, links and inline markup survive, language becomes Persian, RTL metadata is present, and output is deterministic.
8. Apple Silicon (`macos-15`, arm64) compile/test coverage for publication surfaces.
9. Normal repository Rust/security/release gates on the final PR head.
10. No temporary write-enabled one-shot Phase 20 workflows/scripts in the final diff.

Lockfile validation is deliberately non-mutating: CI uses Cargo `--locked`/`cargo metadata --locked` to prove that committed lockfiles satisfy their manifests. It must not use `cargo generate-lockfile` as a freshness check, because that command refreshes otherwise compatible transitive dependencies and can create false CI failures unrelated to the branch. The standard Rust CI follows the same rule.

The generated fixture contains only synthetic project-owned text/assets and is not a proprietary manuscript.

## Adult-intimacy literary fidelity profile

Phase 20 branch work also closes a pre-existing literary-fidelity gap for explicit adult fiction. This is a supporting translation/review contract, not an EPUB feature and not automatic content classification.

Rules:

- default profile remains `literary`;
- `adult-intimacy` is explicit opt-in only;
- the caller must explicitly confirm that every participant in sexual content is an adult;
- the engine must not infer age confirmation from prose or character metadata;
- the profile preserves source explicitness/markedness, consent/hesitation/refusal/coercion and power cues, physical agency/referents, sensory channels, POV, emotional intensity, and pacing;
- it flags both sanitization and amplification;
- it must not sexualize nonsexual source material;
- `IntimacyFidelity` is review evidence only and never human approval/canon;
- the selected style profile participates in translation context fingerprinting so resume cannot silently reuse output produced under another profile.

This profile exists because literary translation fidelity can be materially damaged by euphemizing, censoring, escalating, or changing agency in confirmed-adult source material. It does not weaken any human-review or safety boundary.

## Dependency decisions

- No new EPUB framework: reuse the already approved revision-pinned BookForge boundary.
- `quick-xml` is exposed directly only for narrow native attribute patching; it was already present transitively through EPUB tooling and does not become a second document model.
- EPUBCheck remains an external CLI validator, not a Rust runtime dependency.
- No model/download requirement is added to publication export.
- No vector/database/memory dependency is added in Phase 20.

## Failure boundaries

- Missing BookForge feature: EPUB export returns an actionable error; DOCX and non-EPUB runtime remain usable.
- BookForge parse/rebuild/validation failure: fail closed; no legacy-parser fallback in the default feature path.
- Missing or mismatched native EPUB block provenance: fail closed; do not guess.
- BookForge-only metadata/navigation blocks not represented as native literary units: preserve source content rather than forcing a literary translation.
- EPUBCheck missing in a user environment: internal export can still exist, but publication certification is incomplete. CI installs the checksum-pinned validator for the Phase 20 gate.
- Java/EPUBCheck must not become an ingestion/translation/DOCX runtime requirement.

## Deferred work

- EPUB 3.4 migration and EPUBCheck 5.4.x as the authoritative gate: defer until the standard is stable or deliberately adopted.
- broader cross-reader visual rendering matrix: useful future distribution/product hardening, not required to own the core round-trip contract.
- publication accessibility authoring enhancements beyond structure preservation: treat separately from basic conformance so accessibility claims remain evidence-based.

## Completion rule

Do not call Phase 20 canonical until:

- the permanent Phase 20 workflow is green on the final branch/PR head;
- normal Rust CI/security/release checks are green;
- temporary mutation workflows are absent;
- docs reflect Phase 19 as canonical and Phase 20 as the current branch;
- the Phase 20 PR is merged to `main`;
- post-merge checks on `main` are green.

Only then move to Phase 21.
