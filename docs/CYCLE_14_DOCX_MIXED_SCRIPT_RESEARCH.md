# Cycle 14 DOCX mixed-script repair (isolated slice)

Date: 2026-09-27. Base: `main@ed0f844f601733c655b2e5b84bbbad6e67f1a847`.
Tracks #138; independent of draft Phase 39 PR #146.

## Problem and result

`document-engine::export_persian_docx` emitted the entire paragraph as one `w:rtl` run. This incorrectly marks embedded English identifiers, URLs and ISBNs as complex-script RTL text. The existing paragraph `w:bidi` and document default `w:rtl` are still appropriate for Persian; the exporter now splits ASCII graphic spans into runs and explicitly overrides the inherited RTL setting with `w:rtl w:val="0"` for spans containing ASCII letters or numbers. It preserves exact source text, Persian runs, chapter styles and the existing importer contract. No normalization or URL rewriting is implied.

This is a bounded repair to the current DOCX path, not Book IR, Word rendering proof, or completion of Cycle 14. Explicit editor-owned protected spans and punctuation attachment remain for the Book IR phase. A final standalone ASCII punctuation mark stays in an RTL run; trailing punctuation attached to an ASCII identifier still requires visual Word/LibreOffice QA.

## Research log and options

Queries: GitHub code `w:bidi w:rtl docx mixed text`; repositories `docx-rs`; web `site:github.com/plutext/docx4j w:rtl w:bidi Arabic Latin mixed run issue`, `site:github.com/bokuweb/docx-rs rtl bidi mixed text issue`, `site:learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.wordprocessing.righttolefttext RTL run complex script Latin w:rtl`.

- [Microsoft Open XML `RightToLeftText`](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.wordprocessing.righttolefttext?view=openxml-3.0.1) defines `w:rtl` as a run property that applies complex-script characteristics to every character in that run. [BiDi](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.wordprocessing.bidi?view=openxml-3.0.1) is a separate paragraph property. This supports preserving `w:bidi` while changing only Latin runs.
- [docx4j `TextDirection.java`](https://github.com/plutext/docx4j/blob/master/docx4j-core/src/main/java/org/docx4j/model/properties/run/TextDirection.java) and its multilingual Arabic fixture use run-level direction. Apache-2.0 Java implementation is a reference only; copying or adding a JVM dependency would increase build/runtime cost without solving this small existing exporter defect.
- [bokuweb/docx-rs](https://github.com/bokuweb/docx-rs) is the larger MIT Rust DOCX candidate already listed in the publishing contract. Open [OOXML validator issue #724](https://github.com/bokuweb/docx-rs/issues/724) and [round-trip issue #597](https://github.com/bokuweb/docx-rs/issues/597) require direct validation before replacing the writer. A migration is deferred to the Book IR and full DOCX capability stage. No package was installed.
- Reusing the existing ZIP/OOXML writer was selected for this small change: no dependency/license/security delta, no schema migration, and a focused DOCX XML + re-import regression can check it. The existing `zip` dependency remains pinned through the workspace lockfile.

## Validation and next gate

Added mixed Persian/English/URL/email/ISBN/ZWNJ fixture, direct `word/document.xml` run assertions, exact-text re-import and XML escaping coverage. Local `git diff --check` passed. The current scratch environment has no `cargo`/`rustfmt`, so Rust fmt, Clippy, unit tests, release build, Phase 20 publication and Word visual rendering are **not yet verified**. Keep the PR draft until exact-head CI and an appropriate DOCX render check pass. This branch must not activate a hosted provider or alter EPUB/PDF output.

Next: check CI on the exact PR head; repair any failure; then build a versioned Book IR with explicit protected runs and identity preservation before expanding to common DOCX/EPUB/PDF publishing tokens.
