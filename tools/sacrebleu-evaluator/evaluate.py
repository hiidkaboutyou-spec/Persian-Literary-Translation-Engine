#!/usr/bin/env python3
"""Bounded JSON bridge for optional Phase 21 chrF++ evidence.

This tool deliberately exposes only SacreBLEU chrF2++ evidence. It does not
approve translations, rewrite text, or download benchmark corpora.
"""

from __future__ import annotations

import json
import math
import sys


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(2)


def main() -> None:
    try:
        request = json.load(sys.stdin)
    except Exception as exc:  # noqa: BLE001
        fail(f"invalid SacreBLEU request JSON: {exc}")

    items = request.get("items")
    if not isinstance(items, list) or not items:
        fail("request must contain a non-empty items list")

    ids: list[str] = []
    hypotheses: list[str] = []
    references: list[str] = []
    for index, item in enumerate(items):
        if not isinstance(item, dict):
            fail(f"item {index} must be an object")
        item_id = str(item.get("id", "")).strip()
        translation = item.get("translation")
        reference = item.get("reference")
        if not item_id:
            fail(f"item {index} has no id")
        if not isinstance(translation, str) or not translation.strip():
            fail(f"item {item_id} has no translation")
        if not isinstance(reference, str) or not reference.strip():
            fail(f"item {item_id} has no reference")
        ids.append(item_id)
        hypotheses.append(translation)
        references.append(reference)

    try:
        import sacrebleu
        from sacrebleu.metrics import CHRF

        metric = CHRF(char_order=6, word_order=2, beta=2)
        corpus_score = float(metric.corpus_score(hypotheses, [references]).score)
        signature = str(metric.get_signature())
        sentence_scores = [
            float(metric.sentence_score(hypothesis, [reference]).score)
            for hypothesis, reference in zip(hypotheses, references)
        ]
    except Exception as exc:  # noqa: BLE001
        fail(f"SacreBLEU evaluation failed: {exc}")

    if not math.isfinite(corpus_score) or any(
        not math.isfinite(score) for score in sentence_scores
    ):
        fail("SacreBLEU returned a non-finite score")

    json.dump(
        {
            "tool": "sacrebleu",
            "version": sacrebleu.__version__,
            "metric": "chrF2++",
            "signature": signature,
            "corpus_chrf2pp": corpus_score,
            "items": [
                {"id": item_id, "chrf2pp": score}
                for item_id, score in zip(ids, sentence_scores)
            ],
        },
        sys.stdout,
        ensure_ascii=False,
        separators=(",", ":"),
    )
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
