# EPUBCheck validator

This directory provides an optional, checksum-pinned wrapper around the official W3C/DAISY EPUBCheck distribution.

## Boundary

EPUBCheck validates EPUB package/markup conformance. It does **not** judge translation quality, literary fidelity, Persian naturalness, or human approval.

It is not vendored and is not required by the current translation runtime.

## Install

```bash
bash tools/epubcheck-validator/install.sh
```

The installer:

- downloads official EPUBCheck `5.3.0`;
- verifies SHA-256 `6c07e68584b2e2ce2f89fe06e1246dfead3eb36b46b340e7d93524f29dcff6c5` before installation;
- installs to `.tools/epubcheck/5.3.0` by default, which is ignored by Git;
- does not modify the Rust runtime or project persistence.

Set `EPUBCHECK_HOME` to use a different local install directory.

Java must be available on `PATH` when running the validator.

## Validate an EPUB

```bash
bash tools/epubcheck-validator/validate.sh path/to/book.epub
```

Additional EPUBCheck CLI options can be passed after the EPUB path.

Phase 20 should use this validator as one publication-conformance gate after the engine gains deterministic EPUB round-trip export.
