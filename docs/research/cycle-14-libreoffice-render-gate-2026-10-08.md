# Cycle 14 LibreOffice DOCX render gate

Date: 2026-10-07 22:03 UTC. Run:
`2026-10-07T22:03Z-cycle14-docx-render`. Tracks #138.

Stacked base: PR #167 head
`0efd67c0a3cefe7c81b4f6757eb067ecd1bfb5a8`; its prerequisite is PR #166.
Canonical `main` at selection time:
`963707d3ca61dc97d6835982e2fb82e228709144`.

## Problem and acceptance contract

The existing Cycle 14 regressions inspect OOXML and re-open exported DOCX files
with the project parser. That proves structure and text preservation, but not
that an independent office application can lay out the package and produce
searchable/selectable mixed Persian/Latin text.

This bounded gate is accepted when a rights-safe fixture traverses the real
application path (`create -> import -> analyze -> EchoProvider translate ->
DOCX export`), LibreOffice opens and renders the DOCX under a bounded timeout
and isolated user profile, the produced PDF has at least one page, and Poppler
finds the URL, email and Persian ZWNJ token in its selectable text.

This is a render smoke test, not visual typography approval. The DOCX artifact
is retained briefly so a human can inspect pagination, fonts and BiDi layout.

## Evidence and options

- Context7 resolved the high-reputation official LibreOffice core documentation
  as `/libreoffice/core`. Current command-line source documents
  `--convert-to`, automatic headless mode and
  `-env:UserInstallation=file:///...`; conversion errors can otherwise be
  reported only on stderr after a failed load.
- Firecrawl developer search found upstream Docling issue #3819 and fix #3820:
  concurrent LibreOffice conversions without a timeout and isolated profile
  can hang or collide on the shared profile. This gate therefore uses a 60
  second timeout, a forced-kill grace period, a fresh profile URI and cleanup.
  Sources: https://github.com/docling-project/docling/issues/3819 and
  https://github.com/docling-project/docling/issues/3820.
- PostHog runtime guidance loaded the Performance & Reliability review
  perspective. Its applicable requirements are explicit timeout, actionable
  failure output and cleanup. Current privacy documentation says sensitive data
  should be filtered before it reaches PostHog:
  https://posthog.com/docs/privacy/data-collection. No connected project,
  event schema or metric definition was available, so no product metric was
  queried or invented. The gate emits ordinary CI logs only and uses synthetic
  text; it adds no manuscript telemetry.

| Option | Architectural fit | Maintenance, license, security, cost and limits | Decision |
| --- | --- | --- | --- |
| Keep parser/OOXML tests only | Fast but does not exercise a real office renderer | Existing false-confidence gap remains | Rejected |
| Automate Microsoft Word | Closest to one target editor | Proprietary, unavailable on Linux CI, credential/platform burden | Deferred to human QA |
| LibreOffice headless + Poppler in dedicated CI | Independent renderer and selectable-text evidence; no runtime coupling | Distro packages are CI-only; fixed Ubuntu 24.04 runner, observed versions logged; timeout/profile isolation required; adds CI minutes but no service cost | Selected |
| Add a Rust/Node conversion dependency | Could wrap the same CLI | Adds runtime/dependency and supply-chain surface without new capability | Rejected |

LibreOffice is MPL-2.0/LGPLv3+ and Poppler tools are GPL-licensed system tools;
they are installed only on the isolated CI runner, not linked, redistributed or
made a production prerequisite. There is no new Cargo/npm dependency, network
service, provider, model, credential, font or recurring service cost.

## Implementation, validation and rollback

Changed surfaces:

- `tools/docx-render-validator/validate.sh`: validates inputs/tools, isolates
  the LibreOffice profile, bounds execution, checks PDF page count, extracts
  UTF-8 text and fails with the missing tokens.
- `.github/workflows/cycle14-docx-render.yml`: fixed Ubuntu runner, real
  application-path fixture, render validation and short-lived rights-safe DOCX
  artifact.
- this research/handoff record.

Local validation must include shell syntax, a generated synthetic DOCX through
the validator, failure on a deliberately missing token, and
`git diff --check`. Rust/application evidence must come from exact-head GitHub
Actions because the local runner has no Rust toolchain. Production deployment,
Microsoft Word rendering and human visual approval are not claimed.

Risk is limited to added CI time and possible upstream package-rendering
variation. The fixed runner, logged versions, timeout, isolated profile and
token-level assertions bound that risk. Rollback is removal of this workflow,
validator and record; the runtime/exporter is unchanged.

Exact next step after green exact-head CI: inspect the retained DOCX manually
in Word and LibreOffice for paragraph BiDi, line wrapping and pagination, then
land the stack in order (#166, #167, this gate) if the head-bound review and
required checks permit it.

## Visual-artifact checkpoint

Date: 2026-10-08 02:06 UTC. Run:
`2026-10-08T02:06Z-cycle14-docx-visual-artifact`. Base/head before this
checkpoint: `960b2faad782a10d5d2b578a6e9f20c610e180ad` on PR #169.

Acceptance for this bounded follow-up is stricter than successful conversion:
the same rights-safe application export must produce non-empty PDF, extracted
text, cover-page PNG and final content-page PNG artifacts; the content PNG must
be inspected for obvious glyph clipping, overlap and mixed-script ordering;
exact-head CI must retain the bundle for independent human review. It still does not claim Microsoft Word
compatibility, professional typography approval or complete-book pagination.

Current evidence and decisions:

- Context7 resolved `node-poppler` as `/fdawgs/node-poppler`, the closest
  documented wrapper around the already-installed Poppler CLI because no core
  Poppler entry was returned. Its current API documentation maps first/last
  page, `singleFile`, PNG and resolution controls to the required one-page
  render. No Node dependency was added; the existing system CLI is used.
- Firecrawl developer search and a full scrape of
  https://github.com/freedomofpress/dangerzone/issues/524 found a concrete
  `pdftoppm` failure mode: a full temporary filesystem produced empty files
  despite exit status zero. The validator therefore verifies every retained
  artifact is a non-empty regular file instead of trusting process status.
  The upstream project later stopped using `pdftoppm`; that does not justify a
  new library here because this gate renders one synthetic page and validates
  its output explicitly.
- PostHog's `triaging-visual-review-runs` runtime guidance was loaded. Its
  applicable boundary is that visual changes require human confirmation before
  baseline/finalization; one clean render is not general proof. This repository
  has no connected PostHog Visual Review run or known product telemetry schema,
  so no event, property, metric or baseline was queried or invented. The PNG is
  a short-lived GitHub Actions artifact, not PostHog telemetry.

The workflow uploads artifacts only after the render gate succeeds and only
from the hard-coded synthetic fixture. Reusing this upload path for a private
manuscript is prohibited without an explicit privacy design and authorization.
Rollback is removal of the optional third validator argument and the additional
artifact paths; application/runtime behavior and dependencies remain unchanged.

Local validation on the exact pre-push tree:

- `bash -n tools/docx-render-validator/validate.sh` and `git diff --check`:
  passed;
- workflow YAML parse with PyYAML: passed;
- synthetic RTL DOCX -> LibreOffice PDF -> Poppler text/PNG: passed with one
  page; PDF 15,161 bytes, extracted UTF-8 text 111 bytes and PNG 18,503 bytes
  at 1224 x 1584;
- required URL, email and ZWNJ token: present; deliberately missing-token
  negative control: rejected with the expected diagnostic;
- local renderer versions: LibreOfficeDev 26.8.0.0.alpha0 and Poppler
  `pdftoppm` 26.05.0 (CI logs remain authoritative for Ubuntu versions);
- direct inspection of the local content-page PNG found one right-aligned line,
  joined Persian glyphs, intact `https://example.com` and `user@example.org`,
  and no obvious clipping or overlap. This is a bounded smoke observation, not
  professional Persian layout approval.

Exact-head CI and its retained bundle are still required before this checkpoint
is considered complete. Production deployment and Microsoft Word were not
tested.

Exact-head run #2 initially passed and retained a one-page preview, but artifact
inspection exposed a coverage defect: the application export is two pages and
the first page contains only the generated title. The Persian paragraph is on
page two. The gate was therefore corrected before merge to retain two bounded
previews: page one (`cover-page.png`) and the final page
(`content-page.png`). This avoids an unbounded image set for future fixtures
while ensuring this tiny rights-safe smoke traverses the page containing the
tested paragraph. Run #2 is evidence of the caught gap, not final acceptance.

As a diagnostic before the corrected CI rerun, page two was also rasterized
from run #2's retained PDF with the same Poppler options. Inspection showed the
`Chapter 1` heading and the complete right-aligned Persian/mixed-script line,
with joined glyphs, intact URL/email and no obvious clipping or overlap. The
corrected exact-head artifact must reproduce this evidence directly.

## Page-to-content binding checkpoint

Date: 2026-10-08 03:10 UTC. Run:
`2026-10-08T03:10Z-cycle14-docx-page-binding`. Base/head before this
checkpoint: `135a377946260e15bc8ede398b2f71d7aa5dbd19` on PR #169;
canonical `main` remained
`963707d3ca61dc97d6835982e2fb82e228709144`. PRs #166, #167 and #169
were still open, non-draft and mergeable with unchanged heads; #169 had no
review or unresolved thread, so the existing owned branch was extended instead
of opening a parallel PR.

The retained `content-page.png` was non-empty and was manually inspected, but
the automated gate only proved that expected tokens existed somewhere in the
PDF. A later title, note or blank final page could therefore leave a valid PNG
that was not the page carrying the tested paragraph. Acceptance now requires
page-range extraction of the exact final page, all expected URL/email/ZWNJ
tokens on that page, a retained `content-page.txt`, and the existing bounded
PNG of that same page. A missing token on the selected page must fail even when
the token exists elsewhere in the document.

Evidence and dependency decision:

- Context7 resolved Poppler's closest maintained documentation as
  `/fdawgs/node-poppler`; its current `pdfToText` API maps
  `firstPageToConvert`, `lastPageToConvert` and `maintainLayout` to the same
  already-installed Poppler CLI. No Node wrapper or new dependency is needed.
- Firecrawl developer search followed by a full scrape of Debian testing's
  Poppler manpage confirmed `pdftotext -f` selects the first converted page,
  `-l` the last, `-layout` best preserves physical layout, and UTF-8 is the
  default. Source:
  https://manpages.debian.org/testing/poppler-utils/pdftotext.1.en.html.
- PostHog's `triaging-visual-review-runs` guidance was loaded. The applicable
  principle is to bind each reviewed screenshot to the changed content and
  inspect the image rather than infer correctness from metadata. There is no
  connected PostHog Visual Review run or known product event schema for this
  repository, so no event, property, metric or baseline was queried or guessed.

This is a CI-only evidence improvement using the existing Poppler package. It
adds no runtime, provider, model, credential, network service, telemetry, font
or cost. The fixture remains synthetic/rights-safe. Rollback is removal of the
page-scoped extraction/assertion and its text artifact; application export is
unchanged. Exact-head CI and inspection of the resulting content PNG remain
required before completion; Microsoft Word and production are not claimed.

Local verification on the implementation branch:

- `bash -n tools/docx-render-validator/validate.sh`, YAML parsing with PyYAML
  and `git diff --check`: passed.
- Positive two-page DOCX: expected URL, email and both ZWNJ words were placed
  only on page 2. Validation passed with `pages=2`; `content-page.txt` retained
  all four tokens and all five generated render files were non-empty.
- Page-binding negative control: the same tokens were placed only on page 1
  and an unrelated page 2 was selected for the screenshot. Full-document text
  validation passed first, then the new page-scoped assertion failed as
  required with `rendered content-page text is missing expected tokens`.
- The positive `content-page.png` was opened at original resolution. It showed
  the page-2 Persian/mixed-script sentence with joined Persian glyphs, visible
  URL/email and no obvious clipping or overlap. This is local LibreOffice
  evidence only; exact-head CI inspection is still pending.

Exact-head completion evidence:

- Published implementation commit:
  `da8e8d31110d3c4318cc9bee4694cc4ead5bfb0d`.
- Cycle 14 DOCX Render run #4 completed successfully on that exact SHA:
  https://github.com/hiidkaboutyou-spec/Persian-Literary-Translation-Engine/actions/runs/37741187702.
- Artifact `11533708786` is 50,518 bytes with digest
  `sha256:ecc5fc42e098113bb56b3cf0572b7c54eeb603b165724bb58cbae4f758c0b8c0`
  and expires 2026-10-15:
  https://github.com/hiidkaboutyou-spec/Persian-Literary-Translation-Engine/actions/runs/37741187702/artifacts/11533708786.
- The exact-head bundle contains six non-empty files, including the new
  125-byte `content-page.txt`. Its URL, email and ZWNJ token assertions passed.
  The 1191x1684 exact-head content PNG was opened at original resolution and
  showed `Chapter 1` plus the complete right-aligned mixed Persian/Latin line,
  with joined Persian glyphs and no obvious clipping or overlap.

This closes only the page-to-content evidence gap. It does not establish
Microsoft Word compatibility, professional typography approval, complete-book
pagination or a production deployment. PR #169 remains stacked behind #166 and
#167; head-bound review and Word inspection remain the next external gates.

## Durable page-binding regression checkpoint

Date: 2026-10-08 08:14 UTC. Run:
`2026-10-08T08:14Z-cycle14-docx-page-binding-regression`. Base:
`ce70ae88ebf3a2838d81dc7297a2fb4ab42bd337` on the existing PR #169;
canonical `main` remained
`963707d3ca61dc97d6835982e2fb82e228709144`. PRs #166, #167 and #169
were still open, non-draft and mergeable with unchanged prerequisite heads.
The final-head Cycle 14 run #5 was successful and its retained full text proved
that `Cycle 14 DOCX Render` appears only on the cover, while the extracted
content page begins with `Chapter 1`.

The previous page-binding behavior had a strong local negative control, but CI
only exercised the positive path. Removing the page-scoped assertion could
therefore leave CI green. This follow-up adds a permanent workflow regression:
it runs the same validator and application-exported DOCX with the cover-only
title as the expected token. Full-document extraction must find the title, the
exact content-page assertion must reject it with the expected diagnostic, and
no misleading negative artifact directory may be created.

Current evidence and dependency decision:

- Context7 again resolved Poppler's closest maintained documentation as
  `/fdawgs/node-poppler`. Its current `firstPageToConvert`,
  `lastPageToConvert` and `maintainLayout` options document the same bounded
  page extraction already used by the installed CLI; no wrapper is needed.
- Firecrawl developer search found concrete Bash `errexit` failure modes when
  functions are evaluated under `if`, `&&` or `||`. The workflow therefore
  invokes the validator directly in the `if` condition, checks that the branch
  is the failure branch, and separately verifies both diagnostic lines instead
  of relying on `set -e` alone. Relevant upstream analysis:
  https://github.com/koalaman/shellcheck/issues/2303.
- PostHog's `review-hog-perspective-performance-reliability` guidance was
  loaded. The relevant requirements are actionable error output, no swallowed
  failures, bounded resources and operationally useful logs. No connected
  PostHog project or event schema exists for this repository, so no metric,
  event or property was queried or invented.

This change adds no package, provider, model, credential, telemetry, service,
font or cost. It reuses the rights-safe application fixture and installed
LibreOffice/Poppler tools. Rollback is removal of the single negative-control
workflow step. Local syntax/YAML/behavior checks and exact-head CI remain
required before this checkpoint is complete; Word and production are not
claimed.

Local verification used the rights-safe DOCX downloaded from exact-head run #5:

- validator shell syntax, workflow YAML parsing and `git diff --check`: passed;
- the cover-only title was found by full-document extraction, then rejected by
  the exact content-page assertion with both expected diagnostic lines;
- the negative artifact directory was not created, proving the expected failure
  cannot leave a bundle that looks reviewable.

Exact-head negative-control evidence:

- Published implementation commit:
  `8ed013e5be331981dc7c1e132b1008b4e1b59f9f`.
- Cycle 14 DOCX Render run #6 completed successfully on that exact SHA,
  including the new `Prove page-bound validation rejects a cover-only token`
  step:
  https://github.com/hiidkaboutyou-spec/Persian-Literary-Translation-Engine/actions/runs/37747688528.
- Artifact `11536249024` is 50,513 bytes with digest
  `sha256:9730329c5962e831abc2519448e0596417e03d99996082c1e9b7919525239a52`
  and expires 2026-10-15:
  https://github.com/hiidkaboutyou-spec/Persian-Literary-Translation-Engine/actions/runs/37747688528/artifacts/11536249024.
- The downloaded exact-head bundle contains six non-empty files. Its
  `content-page.txt` contains the URL, email and ZWNJ token but not the
  cover-only title. The 1191x1684 `content-page.png` was opened at original
  resolution and showed `Chapter 1` plus the complete mixed Persian/Latin
  line, with joined Persian glyphs and no obvious clipping or overlap.

This proves the CI gate now exercises both acceptance and rejection paths for
page-bound content. It still does not establish Microsoft Word compatibility,
professional typography approval, complete-book pagination or a production
deployment. PR #169 remains stacked behind #166 and #167; head-bound review
and Word inspection remain the next external gates.
