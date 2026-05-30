#!/usr/bin/env python3
"""Shared aggregation helpers for bench-v2: model tiering, task categorization,
and cost-on-both-passed-intersection.

Pure functions, no I/O. Consumed by BOTH report.py and analyze.py so the
slice/intersection logic lives in exactly one place (no drift). See
bench/BENCH_V2.md (U3).

Accepts records in either shape: the raw persisted BenchResult dict
(`task_id` / `correct` / `tokens_in` / `tokens_out` / `tool_calls`) or the
analyze.normalize_result shape (`task` / `pass` / `calls`).
"""
from __future__ import annotations

import statistics
from typing import Any

# Canonical task -> family map. report.py imports this (single source of truth).
TASK_FAMILIES: dict[str, list[str]] = {
    "Extraction":        ["T1", "T5", "T9", "T11", "T16", "T19"],
    "Targeted mutation": ["T7", "T10", "T13", "T20"],
    "Batch mutation":    ["T12"],
    "Multi-step":        ["T15", "T18"],
    "Content delivery":  ["T2", "T3", "T8", "T17"],
    "Safe-fail":         ["T14"],
    "Text manipulation": ["T4", "T6"],
    "Metadata":          ["T21", "T22", "T24"],
    "Table projection":  ["T23"],
}

_FRONTIER_HINTS = ("claude", "gpt-4", "gpt-5", "opus", "sonnet", "haiku", "gemini", "grok", "o1", "o3")
_LOCAL_HINTS = ("qwen", "llama", "mistral", "gemma", "hermes", "phi", "deepseek", "mlx", "4bit", "8bit")


def extract_model_tier(model: str | None) -> str:
    """frontier | local | unspecified, inferred from the model id string."""
    if not model:
        return "unspecified"
    m = model.lower()
    if any(h in m for h in _FRONTIER_HINTS):
        return "frontier"
    if any(h in m for h in _LOCAL_HINTS):
        return "local"
    return "unspecified"


def category_for(task_id: str | None) -> str:
    """Task family from TASK_FAMILIES, or 'other'."""
    for family, ids in TASK_FAMILIES.items():
        if task_id in ids:
            return family
    return "other"


def _task_id(rec: dict[str, Any]) -> Any:
    return rec.get("task_id") or rec.get("task")


def _passed(rec: dict[str, Any]) -> bool:
    if "correct" in rec:
        return bool(rec["correct"])
    return bool(rec.get("pass"))


def _tokens(rec: dict[str, Any]) -> int:
    return int(rec.get("tokens_in", 0) or 0) + int(rec.get("tokens_out", 0) or 0)


def _calls(rec: dict[str, Any]) -> int:
    return int(rec.get("tool_calls", rec.get("calls", 0)) or 0)


def _cost(rec: dict[str, Any], basis: str) -> int:
    return _tokens(rec) if basis == "tokens" else _calls(rec)


def _median(values: list[float]) -> float | None:
    return statistics.median(values) if values else None


def intersection_cost(records: list[dict[str, Any]], mode_a: str, mode_b: str) -> dict[str, Any]:
    """Median cost per mode over the tasks BOTH modes PASSED.

    Keyed by (task_id, model, thinking_level) so the comparison is apples-to-
    apples across modes. Cost basis is tokens when every intersection record has
    tokens > 0, else falls back to tool_calls (a single number never mixes token
    and proxy units). Empty intersection -> n=0, medians None — never a
    misleading number computed over a non-shared subset.
    """
    by_key: dict[tuple, dict[Any, dict]] = {}
    for r in records:
        key = (_task_id(r), r.get("model"), r.get("thinking_level"))
        by_key.setdefault(key, {})[r.get("mode")] = r

    a_recs: list[dict] = []
    b_recs: list[dict] = []
    for modes in by_key.values():
        ra, rb = modes.get(mode_a), modes.get(mode_b)
        if ra is not None and rb is not None and _passed(ra) and _passed(rb):
            a_recs.append(ra)
            b_recs.append(rb)

    n = len(a_recs)
    both = a_recs + b_recs
    basis = "tokens" if both and all(_tokens(r) > 0 for r in both) else "tool_calls"
    median_a = _median([_cost(r, basis) for r in a_recs]) if n else None
    median_b = _median([_cost(r, basis) for r in b_recs]) if n else None
    delta = (median_a - median_b) if (median_a is not None and median_b is not None) else None
    return {
        "mode_a": mode_a,
        "mode_b": mode_b,
        "n": n,
        "basis": basis if n else None,
        "median_a": median_a,
        "median_b": median_b,
        "delta": delta,  # mode_a cost minus mode_b cost on the shared set
    }
