# Cycle 14 protected runs to DOCX

Date: 2026-10-07 11:04 UTC. Run: `2026-10-07T11:04Z-book-ir-protected-runs`.
Base: `main@c8f0a5da71d7a8037a01a68e4f7c0600e9ba68c4`. Tracks #138.

## Problem and acceptance contract

The merged Book IR and `Manuscript` adapter preserve stable block identity, but
each imported paragraph still becomes one editable run. The legacy DOCX
exporter also accepts flattened chapter strings, so it cannot distinguish
Persian prose from a URL, email address, ISBN, code fragment, file path or
technical identifier. As a result, mixed-direction content can be rendered in
the wrong visual order or changed by a later translation step.

This slice is accepted when:

1. the adapter identifies only high-confidence technical spans and preserves
   every input byte, including Persian ZWNJ and surrounding punctuation;
2. ordinary English literary prose remains editable rather than being
   incorrectly frozen as protected Latin text;
3. a Book-IR-aware DOCX path writes RTL paragraphs structurally and writes
   protected/LTR runs with `w:rtl w:val="0"`;
4. the existing `Chapter`-based application path remains API-compatible and
   emits explicit LTR display runs for embedded ASCII text;
5. tests inspect the produced OOXML and re-open the DOCX, rather than treating
   successful ZIP creation as proof of a healthy document.

## Evidence and options

Repository evidence inspected before implementation:

- PR #149 introduced the versioned Book IR and explicit `RunProtection` /
  `TextDirection` model.
- PR #153 merged the deterministic `Manuscript` adapter while deliberately
  leaving protected-run detection for the next gate.
- Draft PR #147 splits broad ASCII spans inside the legacy DOCX exporter. It
  demonstrates the immediate `w:bidi` paragraph / `w:rtl` run distinction but
  is not a canonical semantic model and can treat ordinary English prose as
  untranslatable.
- The existing Cycle 14 research records Pandoc, boko, booktrans, opendoc,
  docx-rs and docx4j as architecture references. No new dependency is needed
  to complete this bounded vertical slice.

Options compared:

| Option | Fit | Maintenance and risk | Decision |
| --- | --- | --- | --- |
| Keep an ASCII heuristic only in DOCX | Quickly affects DOCX, but EPUB/PDF/editor cannot share the semantics | Broad matching can freeze prose; duplicates logic at every exporter | Rejected as canonical path |
| Replace the OOXML writer now | Could provide a richer document API | New license/security/API surface and round-trip regression risk before a proven need | Deferred |
| Detect conservative spans in the adapter and consume Book IR in the existing writer | One semantic model; no runtime/network/provider change | Small scanner to maintain; ambiguous tokens intentionally remain editable | Selected |

The selected scanner is dependency-free and recognizes URLs, email addresses,
ISBN-10/13 labels, backtick-delimited code, explicit file paths and identifiers
that combine letters, digits and an identifier separator. It does **not**
protect generic Latin words. Unsupported or ambiguous input fails open to an
editable run, while Book IR validation still fails closed on invalid structure.

No external code, package, service, credential, font or paid provider is
introduced. License, privacy, dependency and recurring-cost impact are
therefore unchanged.

## Implementation and verification boundary

Changed files:

- `engine/crates/document-engine/src/book_ir_adapter.rs`: conservative span
  segmentation with exact-text and false-positive regression tests.
- `engine/crates/document-engine/src/export.rs`: validated Book IR export path,
  explicit inline direction in OOXML, a display-only compatibility bridge for
  the current flattened-chapter application path, package inspection and
  re-open tests.
- `engine/crates/document-engine/src/lib.rs`: public Book IR exporter.

Local verification available in this runner is limited to repository/static
checks because no Rust toolchain is installed. `git diff --check` must pass;
Rust format, compile, clippy and test results must come from the exact-head CI
run and are not represented here as a local pass. Production or visual Word /
LibreOffice rendering is also not claimed: the automated test proves OOXML
structure, Unicode/ZWNJ preservation and parser re-open, while a later fixture
gate must check pagination, fonts and visual BiDi in real office applications.

Rollback is a revert of this isolated change: the legacy exporter API remains
available and unchanged. The next step after exact-head CI is green is to pass
translated Book IR through the application export boundary, then add a real
fixture opened in Word or LibreOffice before retiring draft PR #147's legacy
heuristic direction.

### Convergence addendum

Exact-head review found that the application still calls the legacy
`export_persian_docx` API. Merely adding the Book IR exporter would therefore
leave the active user path unchanged. The same PR now includes a bounded
compatibility bridge: at DOCX serialization time only, ASCII graphic spans
with letters or digits receive LTR direction while every run remains editable.
This cannot freeze source prose or affect translation because it executes after
translation and does not assign `RunProtection::Protected`. A regression opens
the legacy-exported DOCX and verifies mixed Persian, Latin, URL, email, ISBN and
ZWNJ text. Canonical future behavior still belongs to explicit Book IR runs.

## Application-boundary integration addendum

Date: 2026-10-07 16:04 UTC. Run:
`2026-10-07T16:04Z-book-ir-application-export`. Stacked base:
`12ddd51eddf27c18a13aa6f29f1bffae4ab44025` from PR #166.

Fresh frontier inspection confirmed that the persisted application path still
flattened each translated chapter and called the compatibility exporter. The
canonical exporter therefore existed and was tested, but real application
exports did not consume it. The bounded acceptance contract for this addendum
is:

1. rebuild translated document structure from plan-validated chapter
   artifacts without changing their persistence schema;
2. preserve parser-owned chapter/scene/paragraph IDs when artifact paragraph
   IDs still match exactly;
3. make an explicit flattened compatibility representation when an older or
   provider-collapsed artifact cannot be mapped honestly, preserving all
   translated text rather than fabricating provenance;
4. route application DOCX export through
   `manuscript_to_book_ir -> export_book_ir_persian_docx`;
5. cover the real `ApplicationService` route with a DOCX re-open regression
   containing Latin tokens and Persian ZWNJ.

Options considered:

| Option | Result | Decision |
| --- | --- | --- |
| Continue calling the legacy exporter | Leaves Book IR semantics outside the real product path | Rejected |
| Change the persisted chapter-artifact schema | Could store Book IR directly but requires migration and invalidates compatible projects | Deferred |
| Rebuild a translated manuscript in memory, preserving exact IDs where supported | Uses existing validated artifacts and schemas; bounded rollback | Selected |
| Fail every unaligned legacy artifact | Strongest structure guarantee but would make previously exportable completed projects unusable | Rejected for DOCX compatibility |

The compatibility case clears source scenes before adapting the translated
chapter, which makes the loss of exact source alignment explicit and causes
the existing adapter to derive deterministic structural IDs. It never claims
source block provenance. Exact mappings retain parser-owned paragraph IDs and
scene breaks. Both paths are converted to Book IR before DOCX serialization,
so protected-token semantics and explicit inline direction reach the real
application boundary.

Changed surfaces for this addendum:

- `engine/crates/project-engine/src/application/translation.rs`: translated
  manuscript reconstruction, identity checks, canonical Book IR export call,
  and focused structure/protected-run regressions;
- `engine/crates/project-engine/tests/application_workflow.rs`: end-to-end
  `ApplicationService` export and DOCX re-open with URL, email and ZWNJ;
- this research/handoff record.

No provider, model, persisted schema, dependency, credential, network service,
font or cost changes are introduced. Rollback is the application-boundary
commit only; PR #166's lower-level Book IR and compatibility exporter remain
separately reviewable. Automated re-open is not visual Word/LibreOffice QA and
does not prove production deployment.
