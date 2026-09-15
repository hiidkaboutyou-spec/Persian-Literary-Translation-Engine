#!/usr/bin/env python3
"""JSON stdin/stdout bridge for optional COMET quality evaluation.

The Rust engine remains the orchestration authority. This process only returns
machine-evaluation evidence; it never approves or rewrites literary output.
"""

from __future__ import annotations

import json
import math
import sys
from typing import Any


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(2)


def as_jsonable(value: Any) -> Any:
    if value is None or isinstance(value, (str, int, bool)):
        return value
    if isinstance(value, float):
        return value if math.isfinite(value) else None
    if isinstance(value, dict):
        return {str(key): as_jsonable(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [as_jsonable(item) for item in value]
    if hasattr(value, "to_dict"):
        return as_jsonable(value.to_dict())
    if hasattr(value, "__dict__"):
        return as_jsonable(vars(value))
    return str(value)


def main() -> None:
    try:
        request = json.load(sys.stdin)
    except Exception as exc:  # noqa: BLE001 - CLI boundary must report malformed input.
        fail(f"invalid COMET request JSON: {exc}")

    model_name = str(request.get("model", "")).strip()
    items = request.get("items")
    batch_size = request.get("batch_size", 8)
    gpus = request.get("gpus", 0)

    if not model_name:
        fail("COMET request is missing a model name")
    if not isinstance(items, list) or not items:
        fail("COMET request must contain at least one item")
    if not isinstance(batch_size, int) or batch_size < 1:
        fail("batch_size must be a positive integer")
    if not isinstance(gpus, int) or gpus < 0:
        fail("gpus must be a non-negative integer")

    data: list[dict[str, str]] = []
    ids: list[str] = []
    for index, item in enumerate(items):
        if not isinstance(item, dict):
            fail(f"item {index} must be an object")
        item_id = str(item.get("id", "")).strip()
        source = item.get("source")
        translation = item.get("translation")
        reference = item.get("reference")
        if not item_id:
            fail(f"item {index} has no id")
        if not isinstance(source, str) or not isinstance(translation, str):
            fail(f"item {item_id} must contain string source and translation fields")
        record = {"src": source, "mt": translation}
        if reference is not None:
            if not isinstance(reference, str):
                fail(f"item {item_id} reference must be a string when present")
            record["ref"] = reference
        ids.append(item_id)
        data.append(record)

    try:
        from comet import download_model, load_from_checkpoint

        model_path = download_model(model_name)
        model = load_from_checkpoint(model_path)
        output = model.predict(data, batch_size=batch_size, gpus=gpus)
    except Exception as exc:  # noqa: BLE001 - preserve model/runtime error for Rust caller.
        fail(f"COMET evaluation failed: {exc}")

    raw_scores = getattr(output, "scores", None)
    system_score = getattr(output, "system_score", None)
    if raw_scores is None or system_score is None:
        fail("COMET model returned no scores")

    scores = [float(score) for score in raw_scores]
    if len(scores) != len(ids):
        fail(f"COMET returned {len(scores)} scores for {len(ids)} items")

    metadata = getattr(output, "metadata", None)
    raw_spans = getattr(metadata, "error_spans", None) if metadata is not None else None
    spans = raw_spans if isinstance(raw_spans, (list, tuple)) else [None] * len(ids)
    if len(spans) != len(ids):
        spans = [None] * len(ids)

    response_items = []
    for item_id, score, item_spans in zip(ids, scores, spans, strict=True):
        response_items.append(
            {
                "id": item_id,
                "score": score,
                "error_spans": as_jsonable(item_spans) if item_spans is not None else [],
            }
        )

    json.dump(
        {
            "model": model_name,
            "system_score": float(system_score),
            "items": response_items,
        },
        sys.stdout,
        ensure_ascii=False,
        separators=(",", ":"),
    )
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
