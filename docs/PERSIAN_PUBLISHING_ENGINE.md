# Persian Publishing Engine

## Product goal

The product must not stop at "translated text". It must turn a translated manuscript into a professional Persian book with correct RTL behavior, Persian orthography, reliable mixed Persian/Latin text, publication-grade layout, and editable outputs.

The web app is the canonical editing surface. A single structured **Book IR** is the source of truth for chapters, paragraphs, inline runs, scene breaks, notes, images, captions, headings, metadata, and approved typography decisions. DOCX, EPUB, and PDF are generated from that same model so they do not drift apart.

## Editability contract

### Canonical editing

All substantive editing happens against the persisted Book IR in the web app. Any edit to translated text, headings, notes, names, paragraph structure, or publication settings can regenerate every export format.

### DOCX / Microsoft Word

DOCX is the primary externally editable office format. The exporter must create real paragraph styles, headings, page/section settings, RTL paragraph properties, RTL Persian runs, LTR protected runs, page numbers, headers/footers, footnotes where present, bookmarks, hyperlinks, and a real table of contents where supported.

A DOCX exported by the product should be re-importable without losing the canonical chapter/paragraph identity that the application can reasonably preserve.

### EPUB

EPUB remains structurally editable and re-importable. The existing Phase 20 EPUB round-trip path is the canonical foundation. Persian output must use EPUB 3 semantics, `lang="fa"`, `dir="rtl"`, RTL spine progression, preserved links/assets, valid navigation, and deterministic EPUBCheck validation.

### PDF

PDF is a publication/final-render format, not a reliable rich-editing source format. The product must therefore make PDF **regenerable from the editable web project**, with immediate preview after edits. We should not misrepresent arbitrary PDF reverse-editing as lossless.

The PDF export should preserve selectable/searchable Persian Unicode text, embedded fonts, correct BiDi shaping, links, bookmarks where practical, and professional book pagination.

## Persian text and typography pipeline

Publishing normalization must be deterministic, configurable, and reviewable. It must not blindly rewrite protected content.

### Required normalization

- Persian Yeh/Kaf normalization (`ي/ى -> ی`, `ك -> ک`).
- Arabic/Persian digit policy by context.
- ZWNJ / نیم‌فاصله normalization for common Persian affixes.
- Persian punctuation: `، ؛ ؟` and Persian quote conventions such as `« »`.
- whitespace and punctuation spacing.
- Unicode normalization without destroying intentional literary typography.
- no manual kashida insertion as a justification hack.
- no letter-spacing/tracking on joining Persian/Arabic scripts.

### Protected spans

Normalization must not damage:
- URLs and email addresses;
- ISBNs, DOI-like identifiers, code, file paths, CLI fragments;
- explicit Latin quotations/names that the editor marks as protected;
- source-language evidence/provenance identifiers;
- inline code and machine-readable metadata.

### Mixed-script direction

"Right aligned" is not enough. Paragraph and inline direction must follow actual language/script semantics.

- Persian paragraphs default to RTL.
- English/code runs inside Persian paragraphs remain LTR.
- punctuation/brackets must preserve correct Unicode BiDi behavior.
- web components use `dir`, `lang`, CSS logical properties, and direction-aware inline wrappers rather than visual string reversal.
- book export formats carry equivalent semantic direction metadata.

## Open-source adoption map

Every direct adoption requires a pinned version, license record, security/maintenance review, and tests around the exact behavior we depend on.

### Approved for direct or selective adoption

#### persian-tools/rust-persian-tools — MIT

Use in the Rust publishing layer for carefully scoped Persian utilities such as:
- Persian character normalization;
- Arabic/Persian digit conversion;
- tested half-space helpers where behavior matches our literary fixtures.

Do not apply a broad normalizer blindly to the full manuscript. Wrap it behind our own publishing API and protect URLs, identifiers, Latin runs, and user-approved exceptions.

#### persian-tools/persian-tools — MIT

Use selectively in the web app for live non-authoritative Persian UI helpers and input feedback. Canonical publishing normalization remains server-side so browser and export behavior cannot diverge.

#### brothersincode/virastar — MIT

Use as a differential/reference implementation and regression corpus for Persian punctuation, quote, spacing, character, and ZWNJ cases. Prefer our Rust canonical implementation for production authority.

#### su6i/parsi-rtl — MIT

Use its mixed-script direction strategy and tests as research/reference for block language detection and BiDi fixtures. The web app should set real semantic `dir` attributes and preserve LTR code/Latin runs.

#### rastikerdar/vazirmatn — SIL OFL 1.1

Primary modern Persian UI/body candidate. Self-host; no production font CDN dependency.

#### rastikerdar/sahel-font — SIL OFL 1.1

Alternative heading/UI family and optional publishing profile. Prefer stable non-variable builds if the variable-font mark-placement caveat remains relevant.

#### rastikerdar/samim-font — SIL OFL 1.1, archived

Optional long-form/body publishing profile after glyph/render verification. Archived status means we treat it as stable font data, not an actively maintained dependency.

#### rastikerdar/parastoo-font — SIL OFL 1.1, discontinued

Optional literary/print profile after render verification. Keep optional rather than default because upstream development is discontinued.

#### bokuweb/docx-rs — MIT

Evaluate for the publication DOCX layer because it supports generation/parsing, styles, headers/footers, sections, hyperlinks, bookmarks, comments, footnotes, tracked changes, and tables of contents. Migration from the current handwritten minimal OOXML exporter must be incremental and round-trip tested.

#### pagedjs/pagedjs — MIT

Primary candidate for browser print preview and professional CSS Paged Media rendering. Use our own Persian book CSS and templates; do not copy example-book visual identities. Server PDF generation must run in a sandboxed renderer using only application-generated/sanitized HTML.

#### pagedjs/pagedjs-examples / electricbookworks/paged-design

Reference for print-book CSS techniques only. Recreate layouts with original project-specific Persian publication profiles and verified compatible source licenses.

### Reference / QA only

#### ali2000hos/persian-writing — MIT

Excellent QA/reference material for Persian orthography, RTL Word structure, font usage, pagination, and document verification. Use it to derive independent fixtures and acceptance tests. Do not make an external agent skill the runtime authority of the engine.

#### KiaroSama/Revayat-Novel-Skill — GPL-3.0

Reference only because its GPL license is intentionally stronger than the main project's current licensing strategy. Useful ideas to reproduce independently include Word-native footnotes, clickable TOC, image preservation, glossary continuity, Persian typography checks, and deterministic publication QA.

#### xepersian

Use as a Persian typesetting benchmark/reference where useful, not as a required runtime dependency. Its ecosystem is valuable for comparing expected Persian print behavior, but the web product should not require a full TeX installation to perform routine exports.

## Book design system

The product must ship original, professional Persian book profiles rather than one hard-coded template.

### Required profiles

1. **Literary Classic**
   - generous margins;
   - restrained chapter openings;
   - comfortable body measure and line height;
   - minimal running furniture.

2. **Modern Novel**
   - cleaner contemporary hierarchy;
   - slightly tighter page economy;
   - modern Persian font pairing.

3. **Compact Paperback**
   - smaller trim and economical page count;
   - stronger widow/orphan and paragraph-break protection.

4. **Large Reading**
   - larger type and line spacing for accessibility/comfortable reading.

5. **Digital EPUB**
   - reflow-first CSS;
   - user font-size friendly;
   - no fixed page assumptions;
   - semantic headings and navigation.

Each profile is a tokenized publication theme, not a separate exporter.

### User-adjustable settings

- book/trim size and margins;
- body font and heading font from approved bundled families;
- body size, leading, paragraph indent/spacing;
- chapter-opening style;
- scene-break ornament/text;
- page-number position;
- running header behavior;
- justification vs ragged-start/end policy where the format supports it;
- Persian vs Latin digit presentation rules by content type;
- cover/title-page metadata;
- EPUB reading theme metadata where applicable.

## Automatic book-layout rules

The engine must enforce professional layout without requiring the user to know publishing terminology:

- no orphaned chapter/section headings;
- widow/orphan controls where supported;
- keep scene separators with surrounding content sensibly;
- avoid single-line paragraphs stranded at page boundaries where practical;
- first paragraph after a chapter heading can use profile-specific no-indent rules;
- preserve intentional blank/scene-break semantics;
- prevent clipped Persian ascenders/diacritics with safe leading;
- preserve images, aspect ratio, captions, alt text, and logical anchoring;
- create correct front matter, title page, copyright/metadata page, TOC, body, and optional notes sections;
- running headers and page numbers suppress automatically on title/chapter-opening pages according to profile;
- do not fake Persian emphasis with synthetic italic or tracking.

## Word/DOCX requirements

The final Word file must be a file a human editor/publisher can actually continue editing.

Required:
- named paragraph/character styles instead of per-run formatting;
- proper `w:bidi` paragraph semantics and RTL Persian runs;
- explicit LTR runs for protected Latin content;
- Persian-compatible complex-script font assignment;
- real page/section breaks;
- real footnotes/endnotes when source semantics contain them;
- clickable hyperlinks/bookmarks;
- TOC field/bookmarks where supported;
- headers, footers, page numbers;
- comments/tracked-change compatibility where technically feasible;
- round-trip import test;
- deterministic OOXML structural QA.

## EPUB requirements

Build on the existing Phase 20 round-trip implementation:
- preserve source assets unless intentionally replaced;
- generate/retain semantic XHTML;
- `lang="fa"` and `dir="rtl"`;
- RTL page progression;
- accessible navigation and TOC;
- locally embedded licensed fonts only when profile requests embedding and the EPUB strategy allows it;
- reflow-safe CSS with logical properties;
- EPUBCheck as a blocking conformance gate;
- re-import/round-trip fixture.

## PDF requirements

Preferred architecture:
1. Book IR -> sanitized semantic HTML.
2. Original Persian publication CSS.
3. Paged.js preview in the browser.
4. isolated headless-browser/Paged.js render for final PDF.
5. deterministic QA over rendered output.

Blocking PDF QA:
- selected Persian fonts embedded;
- no fallback-tofu/missing glyphs;
- searchable/selectable Unicode text;
- mixed Persian/Latin copy/paste sanity fixtures;
- no clipped text;
- stable page count for the same canonical input/profile/toolchain;
- headings and scene breaks do not land in known-bad positions;
- links/bookmarks validated where supported;
- visual regression snapshots on representative Persian pages.

Rust-native PDF libraries such as `printpdf` can remain useful for low-level utilities, but current direct PDF book layout should not switch to a less-proven complex-script pipeline unless Persian/BiDi conformance is demonstrated.

## Web publishing studio

The web app must expose a visual **Book Design** surface.

On iPhone:
- preview one page/spread at a time;
- choose publication profile and font;
- edit text in the canonical manuscript editor;
- see layout warnings;
- export DOCX / EPUB / PDF from the same project.

On larger screens:
- manuscript/editor panel;
- live book/page preview;
- design inspector for typography/page tokens;
- publication QA sidebar.

The preview is not allowed to be a fake mockup. It must use the same semantic HTML/CSS token set as the production PDF path wherever technically possible.

## Verification matrix

Every release that changes publishing behavior must test at least:

### Persian text
- Arabic `ي/ك` -> Persian `ی/ک`;
- ZWNJ examples such as `می‌رود`, `کتاب‌ها`;
- Persian punctuation and quotes;
- Persian + English names;
- Persian sentence beginning with an English token;
- numbers, dates, ISBN/URL protected cases.

### DOCX
- re-open/re-import;
- RTL paragraph/run XML;
- font/style definitions;
- headings and TOC/bookmarks;
- page and section properties;
- notes/images where fixture contains them.

### EPUB
- EPUBCheck;
- `lang`, `dir`, spine progression;
- links/images/nav;
- deterministic re-export;
- re-import.

### PDF
- font embedding;
- text extraction/copy;
- visual page fixtures;
- mixed BiDi;
- pagination invariants.

## Completion condition

The publishing engine is complete only when a user can edit a translated book in the web app, choose a professional Persian book profile, and produce a coherent DOCX, EPUB, and PDF from the same canonical manuscript without manually repairing RTL, Persian punctuation, fonts, chapter layout, page numbering, or basic book structure afterward.
