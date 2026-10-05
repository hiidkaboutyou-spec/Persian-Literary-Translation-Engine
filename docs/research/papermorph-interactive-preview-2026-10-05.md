# Papermorph interactive-preview adoption — 2026-10-05

## Upstream reviewed

- repository: `DozenTwelve/Papermorph`
- license: MIT
- observed purpose: convert reference PDFs into animated, narrated, interactive static web books.

Useful engineering patterns in the upstream skill:
- keep private source PDFs/page extraction outside the static deliverable and version control;
- map the book before chapter production;
- keep a self-contained static output;
- review chapter openings/blank states, clipping, overlap and text density;
- use targeted browser interaction tests for changed reader behavior;
- keep chapter work resumable through file-based state rather than requiring full chat history.

## Product fit

The Persian Literary Translation Engine does **not** need Papermorph's algebra-oriented SVG teaching engine, quizzes, beat choreography, Edge TTS dependency, or Opus-specific authoring workflow.

The useful gap in this project is narrower: after producing an EPUB, a reviewer should be able to open a quick, local, interactive reading surface and inspect chapter order, RTL reading flow, mixed-script readability and gross content loss before deeper publication validation.

## Adopted

Added `tools/epub_web_preview/`:

- parses the generated EPUB container/OPF/spine;
- follows spine order rather than guessing filesystem order;
- renders a self-contained RTL HTML reader;
- provides chapter navigation, keyboard navigation and text-size controls;
- binds the preview manifest to the exact source EPUB SHA-256;
- strips active/source XHTML content such as scripts, iframes, forms, event handlers and external links;
- adds no remote CSS/font/script/TTS/model dependency;
- writes only under ignored local output by default;
- does not mutate the EPUB or project Book IR.

The preview is deliberately **diagnostic only**. It is not an editor, publication renderer, EPUB conformance checker or source of canonical manuscript state.

## Rejected/deferred

- Papermorph SVG animation engine: wrong product abstraction for literary books.
- Embedded quizzes/practice: educational-book feature, not a translation/publishing requirement.
- Edge TTS: network/privacy dependency with no measured product gap yet.
- PDF-first reconstruction as product truth: this engine already owns structured Book IR and EPUB/DOCX boundaries.
- automatic public hosting/bookshelf: would create a privacy leak risk for user manuscripts.

## Verification contract

Permanent tests use a project-owned synthetic EPUB and prove:

- spine order is preserved;
- Persian RTL reader semantics are emitted;
- active XHTML content is removed;
- no HTTP/HTTPS dependency leaks into output;
- the same EPUB yields deterministic preview files;
- missing EPUB structural metadata fails closed.

Future visual/browser QA may add Playwright screenshots when the publication UI itself needs visual-regression evidence. That should remain a local/CI QA layer rather than a dependency of translation runtime.
