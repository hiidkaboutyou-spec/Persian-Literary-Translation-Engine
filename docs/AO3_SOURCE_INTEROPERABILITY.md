# AO3 source interoperability research and adoption

Date: 2026-10-02. Branch: `cycle-14-ao3-source-interoperability`.
Base: canonical Book IR after PR #149. Tracks the source/provenance part of #138.

## Goal

Reuse the useful engineering ideas from mature AO3 tooling without turning the Persian Literary
Translation Engine into an AO3 scraper, account client, or credential store.

The translation engine already accepts EPUB as a first-class manuscript format. The safest useful
integration is therefore **local downloaded-artifact interoperability**: preserve source identity and
fanfiction metadata when an AO3/FanFicFare EPUB is imported, while keeping all account/network
behavior outside the runtime.

## Repositories reviewed

### JimmXinu/FanFicFare

Useful ideas adopted:

- treat the AO3 work ID as stable source identity;
- normalize accepted AO3 aliases to one canonical work URL;
- preserve story/source metadata alongside downloaded ebook content;
- keep source metadata separate from translated chapter text.

Reference examined at commit `c804f5d8daaa4bcc1178907b745ac549b84cd425`, including
`adapter_archiveofourownorg.py` and `base_otw_adapter.py`.

Not adopted:

- site login;
- username/password configuration;
- live chapter fetching/updating;
- cookies/browser-cache behavior.

No FanFicFare code is copied into the runtime. The project keeps its existing Rust EPUB parser and
implements the small source-normalization rule independently.

### nianeyna/ao3downloader

Useful ideas adopted:

- downloaded files are the interoperability boundary;
- metadata should be exportable/preservable independently from the story body;
- source/update identity should be explicit rather than inferred from translated text.

Reference examined around commit `72bb4048807ca47974c51ab6f8058fd1bb92b556`.

Not adopted:

- bulk crawling/downloading;
- Marked for Later account automation;
- credential persistence;
- remote update checks as translation-engine behavior.

A future project-source refresh feature may reuse the *concept* of checking a stable source identity,
but it must remain explicit, rate-limited, and outside translation execution.

### wendytg/ao3_api

Useful idea adopted:

- AO3 metadata has a useful structured taxonomy (work ID, authors, fandoms, relationships, tags,
  warnings, status, dates, word/chapter counts).

Reference examined at commit `02e349985d927bd8693f905f440e1ef0539f1984`.

Not adopted:

- `Session` login/authentication;
- live scraping/fetching;
- kudos/comments/bookmarks/subscriptions;
- any operation requiring AO3 authenticity tokens.

The current implementation keeps a generic bounded EPUB metadata map instead of coupling the domain
model to AO3-specific account APIs.

### Zaki-1052/AO3-History-Exporter

The local-only/privacy model is compatible with the project's security direction, but reading history
is user-library data rather than manuscript translation data. No runtime integration is justified.
Exported history can remain an external personal-library tool.

### serpentineegg/fictrail

Searchable local reading history is useful for readers, not for translation execution. No runtime
integration is justified.

### marciepublishes/AO3-Find-That-Bookmark-

Deleted-bookmark detection and local bookmark snapshots are valuable archival features, but they do
not belong in the translation engine. No runtime integration is justified.

### spin-drift/ao3-bunker

Local reading-list and scroll-position tracking are reader UX features. They remain outside the
translation engine.

## Implemented slice

`document-engine/src/epub.rs` now:

1. preserves bounded OPF values for identifier, publisher, source, description, rights, date, and
   subjects;
2. scans package metadata for recognized AO3 work URLs;
3. normalizes AO3 aliases (including `ao3.org` and download domains) to
   `https://archiveofourown.org/works/<id>`;
4. stores `source_site`, `source_url`, and `ao3_work_id` in the existing manuscript metadata map;
5. performs no network request and introduces no new dependency.

The metadata value bound is 16,384 Unicode scalar values per imported field to avoid allowing package
metadata to grow project state without limit.

## Security and privacy boundary

The literary runtime must not:

- request or persist AO3 passwords;
- store login cookies or CSRF/authenticity tokens;
- automate kudos, comments, bookmarks, subscriptions, history, or Marked for Later;
- crawl AO3 listings;
- make translation correctness depend on AO3 availability.

Restricted works may still be translated if the user has lawfully obtained a local file and imports
that file through the normal project workflow. Authentication remains the user's/browser's concern,
not the engine's.

## Regression coverage

Added tests require:

- known AO3 download aliases normalize to one canonical work URL;
- leading-zero work IDs normalize deterministically;
- non-AO3 `/works/<id>` URLs are not misclassified;
- subjects and description survive OPF metadata extraction;
- the feature works from package metadata only, with no network/session dependency.

## Next safe follow-ups

1. Carry `Manuscript.book.metadata` unchanged into the canonical Book IR adapter when that adapter is
   implemented.
2. Add a local sidecar metadata import format only if real FanFicFare/ao3downloader exports need fields
   not represented in OPF.
3. Add explicit source-refresh UX only as a separate opt-in capability; do not couple it to translation.
4. Keep history/bookmark/reading-progress integrations outside the translation runtime.
