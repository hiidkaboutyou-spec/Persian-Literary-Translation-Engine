#!/usr/bin/env python3
"""Export high-confidence blind human-review judgments as local preference pairs.

Adapted conceptually from preference-data pipelines in train-llm-from-scratch.
This is developer/evaluation tooling only: it performs no training, no network
calls, and no provider admission. Ties, defers, pending cases and reviewer
disagreements are excluded instead of being converted into artificial labels.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
from typing import Any


class PreferenceExportError(RuntimeError):
    pass


def _read_json(path: Path) -> tuple[bytes, dict[str, Any]]:
    raw = path.read_bytes()
    try:
        value = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise PreferenceExportError(f"invalid JSON: {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise PreferenceExportError(f"expected JSON object: {path}")
    return raw, value


def _nonempty(value: object, field: str) -> str:
    text = str(value or "").strip()
    if not text:
        raise PreferenceExportError(f"missing non-empty field: {field}")
    return text


def _decision(value: object) -> str:
    return str(value or "").strip().lower()


def _prompt(case: dict[str, Any]) -> str:
    source = _nonempty(case.get("source"), "case.source")
    before = str(case.get("context_before") or "").strip()
    after = str(case.get("context_after") or "").strip()
    dimensions = case.get("dimensions")
    if not isinstance(dimensions, list):
        dimensions = []
    parts = [
        "Translate the literary source into publication-quality Persian.",
        "Preserve meaning, character voice, register, subtext, cultural/pragmatic intent, and continuity.",
        "Source:\n" + source,
    ]
    if before:
        parts.append("Context before:\n" + before)
    if after:
        parts.append("Context after:\n" + after)
    if dimensions:
        parts.append("Review dimensions: " + ", ".join(str(item) for item in dimensions[:12]))
    return "\n\n".join(parts)


def build_preference_rows(
    bundle_bytes: bytes,
    bundle: dict[str, Any],
    key: dict[str, Any],
    ledgers: list[dict[str, Any]],
    *,
    min_reviews: int = 1,
) -> tuple[list[dict[str, Any]], dict[str, int]]:
    corpus_id = _nonempty(bundle.get("corpus_id"), "bundle.corpus_id")
    if _nonempty(key.get("corpus_id"), "key.corpus_id") != corpus_id:
        raise PreferenceExportError("bundle/key corpus_id mismatch")

    expected_sha = str(key.get("bundle_sha256") or "").strip().lower()
    actual_sha = hashlib.sha256(bundle_bytes).hexdigest()
    if expected_sha and expected_sha != actual_sha:
        raise PreferenceExportError("reveal key is not bound to the exact blind bundle bytes")

    cases = bundle.get("cases")
    assignments = key.get("assignments")
    if not isinstance(cases, list) or not isinstance(assignments, list):
        raise PreferenceExportError("bundle cases and key assignments must be arrays")

    case_by_id: dict[str, dict[str, Any]] = {}
    for raw in cases:
        if not isinstance(raw, dict):
            raise PreferenceExportError("blind bundle contains a non-object case")
        case_id = _nonempty(raw.get("case_id"), "case.case_id")
        if case_id in case_by_id:
            raise PreferenceExportError(f"duplicate case_id in bundle: {case_id}")
        case_by_id[case_id] = raw

    assignment_by_id: dict[str, dict[str, str]] = {}
    for raw in assignments:
        if not isinstance(raw, dict):
            raise PreferenceExportError("reveal key contains a non-object assignment")
        case_id = _nonempty(raw.get("case_id"), "assignment.case_id")
        if case_id in assignment_by_id:
            raise PreferenceExportError(f"duplicate case_id in reveal key: {case_id}")
        a_system = _nonempty(raw.get("candidate_a_system"), "assignment.candidate_a_system")
        b_system = _nonempty(raw.get("candidate_b_system"), "assignment.candidate_b_system")
        if a_system == b_system:
            raise PreferenceExportError(f"case {case_id} maps both candidates to the same system")
        assignment_by_id[case_id] = {"a": a_system, "b": b_system}

    if set(case_by_id) != set(assignment_by_id):
        raise PreferenceExportError("blind bundle/reveal-key case sets do not match")

    votes: dict[str, list[tuple[str, str]]] = {case_id: [] for case_id in case_by_id}
    uncertain: set[str] = set()
    for ledger in ledgers:
        if _nonempty(ledger.get("corpus_id"), "ledger.corpus_id") != corpus_id:
            raise PreferenceExportError("ledger corpus_id mismatch")
        ledger_sha = str(ledger.get("bundle_sha256") or "").strip().lower()
        if ledger_sha and ledger_sha != actual_sha:
            raise PreferenceExportError("review ledger is not bound to the supplied blind bundle")
        reviewer = _nonempty(ledger.get("reviewer"), "ledger.reviewer")
        ledger_cases = ledger.get("cases")
        if not isinstance(ledger_cases, list):
            raise PreferenceExportError("ledger cases must be an array")
        seen: set[str] = set()
        for raw_case in ledger_cases:
            if not isinstance(raw_case, dict):
                raise PreferenceExportError("ledger contains a non-object case")
            case_id = _nonempty(raw_case.get("case_id"), "ledger.case_id")
            if case_id not in case_by_id or case_id in seen:
                raise PreferenceExportError(f"unknown/duplicate ledger case: {case_id}")
            seen.add(case_id)
            decision = _decision(raw_case.get("decision"))
            if decision in {"candidate_a", "candidatea", "a"}:
                votes[case_id].append((reviewer, "a"))
            elif decision in {"candidate_b", "candidateb", "b"}:
                votes[case_id].append((reviewer, "b"))
            else:
                uncertain.add(case_id)

    min_reviews = max(1, int(min_reviews))
    rows: list[dict[str, Any]] = []
    summary = {
        "cases_total": len(case_by_id),
        "pairs_exported": 0,
        "skipped_uncertain": 0,
        "skipped_insufficient_reviews": 0,
        "skipped_disagreement": 0,
    }

    for case_id in case_by_id:
        case_votes = votes[case_id]
        if case_id in uncertain:
            summary["skipped_uncertain"] += 1
            continue
        if len(case_votes) < min_reviews:
            summary["skipped_insufficient_reviews"] += 1
            continue
        assignment = assignment_by_id[case_id]
        preferred_systems = {
            assignment[side]
            for _reviewer, side in case_votes
        }
        if len(preferred_systems) != 1:
            summary["skipped_disagreement"] += 1
            continue

        preferred_system = next(iter(preferred_systems))
        case = case_by_id[case_id]
        candidate_a = _nonempty(case.get("candidate_a"), "case.candidate_a")
        candidate_b = _nonempty(case.get("candidate_b"), "case.candidate_b")
        if preferred_system == assignment["a"]:
            chosen, rejected = candidate_a, candidate_b
            rejected_system = assignment["b"]
        else:
            chosen, rejected = candidate_b, candidate_a
            rejected_system = assignment["a"]
        if chosen == rejected:
            summary["skipped_disagreement"] += 1
            continue

        rows.append(
            {
                "schema_version": 1,
                "prompt": _prompt(case),
                "chosen": chosen,
                "rejected": rejected,
                "metadata": {
                    "corpus_id": corpus_id,
                    "case_id": case_id,
                    "review_count": len(case_votes),
                    "reviewers": sorted({reviewer for reviewer, _side in case_votes}),
                    "chosen_system": preferred_system,
                    "rejected_system": rejected_system,
                    "provenance": "blind_human_review_unanimous_preference",
                    "production_admission": "not_granted",
                },
            }
        )

    summary["pairs_exported"] = len(rows)
    return rows, summary


def write_jsonl_atomic(rows: list[dict[str, Any]], output: Path) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    staged = output.with_name(output.name + ".tmp")
    with staged.open("w", encoding="utf-8", newline="\n") as handle:
        for row in rows:
            handle.write(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n")
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(staged, output)
    try:
        output.chmod(0o600)
    except OSError:
        pass


def export_preferences(
    bundle_path: Path,
    key_path: Path,
    ledger_paths: list[Path],
    output_path: Path,
    *,
    min_reviews: int = 1,
) -> dict[str, int]:
    bundle_bytes, bundle = _read_json(bundle_path)
    _key_bytes, key = _read_json(key_path)
    ledgers = [_read_json(path)[1] for path in ledger_paths]
    rows, summary = build_preference_rows(
        bundle_bytes,
        bundle,
        key,
        ledgers,
        min_reviews=min_reviews,
    )
    write_jsonl_atomic(rows, output_path)
    return summary


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Export unanimous strict A/B blind-review decisions as local prompt/chosen/rejected JSONL. "
            "Run the canonical blind-review verification command first when authenticated evidence is required."
        )
    )
    parser.add_argument("bundle", type=Path)
    parser.add_argument("reveal_key", type=Path)
    parser.add_argument("ledgers", type=Path, nargs="+")
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--min-reviews", type=int, default=1)
    args = parser.parse_args()
    summary = export_preferences(
        args.bundle,
        args.reveal_key,
        args.ledgers,
        args.out,
        min_reviews=args.min_reviews,
    )
    print(json.dumps(summary, sort_keys=True))
    print(f"local preference export written: {args.out}")
    print("This export is evidence for offline evaluation/training research only; production admission remains NOT GRANTED.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
