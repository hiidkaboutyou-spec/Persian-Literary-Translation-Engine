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
