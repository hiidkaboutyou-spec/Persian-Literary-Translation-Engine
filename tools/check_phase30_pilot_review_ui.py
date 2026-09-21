#!/usr/bin/env python3
"""Static Phase-30 pilot review UI contract checks.

These checks deliberately avoid parsing or storing manuscript text. They verify
that the local WebView surface is wired to the Rust-owned Phase 28/29 commands
without introducing browser persistence, remote fetches, inline event handlers,
or unsafe HTML rendering.
"""

from html.parser import HTMLParser
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
HTML = (ROOT / "desktop/ui/index.html").read_text(encoding="utf-8")
JS = (ROOT / "desktop/ui/app.js").read_text(encoding="utf-8")

REQUIRED_IDS = {
    "refresh-pilot-review",
    "pilot-max-targets",
    "pilot-summary-grid",
    "pilot-issues",
    "pilot-target-list",
    "pilot-context",
    "pilot-reviewer",
    "pilot-outcome",
    "pilot-note",
    "pilot-finding-dimension",
    "pilot-finding-severity",
    "pilot-finding-note",
    "pilot-source-start",
    "pilot-source-end",
    "pilot-target-start",
    "pilot-target-end",
    "record-pilot-review",
    "open-pilot-editor",
}


class SurfaceParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.ids = []
        self.inline_handlers = []

    def handle_starttag(self, tag, attrs):
        for key, value in attrs:
            if key == "id" and value:
                self.ids.append(value)
            if key.lower().startswith("on"):
                self.inline_handlers.append((tag, key))


parser = SurfaceParser()
parser.feed(HTML)

missing = sorted(REQUIRED_IDS - set(parser.ids))
if missing:
    raise SystemExit(f"missing Phase-30 DOM ids: {missing}")

duplicates = sorted({value for value in parser.ids if parser.ids.count(value) > 1})
if duplicates:
    raise SystemExit(f"duplicate DOM ids: {duplicates}")

if parser.inline_handlers:
    raise SystemExit(f"inline event handlers are forbidden: {parser.inline_handlers}")

required_js = [
    'call("get_pilot_review_summary"',
    'call("record_pilot_review"',
    'call("get_translated_chapter"',
    'nonNegativeNumberOrNull("pilot-max-targets")',
    'textContent',
]
for needle in required_js:
    if needle not in JS:
        raise SystemExit(f"missing pilot UI contract: {needle}")

for forbidden in ["innerHTML", "outerHTML", "localStorage", "sessionStorage", "indexedDB", "fetch("]:
    if forbidden in JS:
        raise SystemExit(f"forbidden browser surface introduced: {forbidden}")

if re.search(r"https?://", HTML) or re.search(r"https?://", JS):
    raise SystemExit("pilot UI must not introduce remote frontend URLs")

if 'data-view="pilot"' not in HTML or 'data-view-panel="pilot"' not in HTML:
    raise SystemExit("pilot navigation/view pairing is incomplete")

print("Phase 30 pilot review UI contract: OK")
