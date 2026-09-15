#!/usr/bin/env python3
"""Bounded Persian surface diagnostics for Phase 19.

This tool reports evidence only. It never returns rewritten Persian text and does
not load Hazm pretrained POS/parser/embedding models.
"""

from __future__ import annotations

import difflib
import importlib.metadata
import json
import re
import sys
from typing import Any

from hazm import Normalizer, sent_tokenize, word_tokenize

SCHEMA_VERSION = 1
MAX_CHARS = 40_000
ARABIC_VARIANTS = {"ي", "ك", "ى"}


def fail(message: str, code: int = 2) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(code)


def read_request() -> dict[str, Any]:
    try:
        payload = json.load(sys.stdin)
    except Exception as exc:  # boundary error must be actionable, not hidden
        fail(f"invalid JSON request: {exc}")
    if not isinstance(payload, dict):
        fail("request must be a JSON object")
    if payload.get("schema_version") != SCHEMA_VERSION:
        fail(f"unsupported schema_version: {payload.get('schema_version')!r}")
    unit_id = payload.get("unit_id")
    text = payload.get("text")
    if not isinstance(unit_id, str) or not unit_id.strip():
        fail("unit_id must be a non-empty string")
    if not isinstance(text, str):
        fail("text must be a string")
    if len(text) > MAX_CHARS:
        fail(f"text exceeds {MAX_CHARS}-character sidecar limit")
    return payload


def diagnose(text: str) -> dict[str, Any]:
    # Normalizer is used only to measure how much standard orthography would
    # change. The normalized text itself is intentionally not returned.
    normalized = Normalizer().normalize(text)
    similarity = difflib.SequenceMatcher(a=text, b=normalized, autojunk=False).ratio()
    edit_ratio = min(1.0, max(0.0, 1.0 - similarity))

    return {
        "sentence_count": len(sent_tokenize(text)) if text.strip() else 0,
        "word_count": len(word_tokenize(text)) if text.strip() else 0,
        "normalized_changed": normalized != text,
        "normalization_edit_ratio": edit_ratio,
        "arabic_variant_count": sum(character in ARABIC_VARIANTS for character in text),
        "repeated_whitespace_count": len(re.findall(r"[ \t]{2,}", text)),
        "zwnj_count": text.count("\u200c"),
    }


def main() -> None:
    request = read_request()
    result = diagnose(request["text"])
    response = {
        "schema_version": SCHEMA_VERSION,
        "unit_id": request["unit_id"],
        "tool": "hazm",
        "tool_version": importlib.metadata.version("hazm"),
        **result,
    }
    json.dump(response, sys.stdout, ensure_ascii=False, separators=(",", ":"))
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
