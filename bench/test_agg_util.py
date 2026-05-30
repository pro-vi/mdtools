#!/usr/bin/env python3
"""Tests for bench/agg_util.py (bench-v2 U3 — the load-bearing pure functions).

Runnable via pytest OR standalone (pytest is not always installed):
    python3 bench/test_agg_util.py
"""
from __future__ import annotations

from bench.agg_util import category_for, extract_model_tier, intersection_cost


def _rec(task, mode, model, passed, tin=0, tout=0, calls=0, thinking=None):
    return {
        "task_id": task, "mode": mode, "model": model, "thinking_level": thinking,
        "correct": passed, "tokens_in": tin, "tokens_out": tout, "tool_calls": calls,
    }


def test_tier():
    assert extract_model_tier("claude-opus-4-8") == "frontier"
    assert extract_model_tier("Qwen3.5-27B-4bit") == "local"
    assert extract_model_tier("some-unknown-model") == "unspecified"
    assert extract_model_tier(None) == "unspecified"


def test_category():
    assert category_for("T12") == "Batch mutation"
    assert category_for("T1") == "Extraction"
    assert category_for("T999") == "other"


def test_happy_path_intersection():
    # T1,T2,T3 pass in BOTH modes; T4 only mdtools, T5 only unix.
    data = {
        ("T1", "unix"): (True, 100), ("T1", "mdtools"): (True, 60),
        ("T2", "unix"): (True, 200), ("T2", "mdtools"): (True, 80),
        ("T3", "unix"): (True, 300), ("T3", "mdtools"): (True, 100),
        ("T4", "unix"): (False, 50), ("T4", "mdtools"): (True, 40),
        ("T5", "unix"): (True, 70),  ("T5", "mdtools"): (False, 90),
    }
    recs = [_rec(t, mode, "claude", p, tin=tok) for (t, mode), (p, tok) in data.items()]
    out = intersection_cost(recs, "unix", "mdtools")
    assert out["n"] == 3, out
    assert out["basis"] == "tokens"
    assert out["median_a"] == 200   # unix median of [100,200,300]
    assert out["median_b"] == 80    # mdtools median of [60,80,100]
    assert out["delta"] == 120


def test_superset_does_not_average_over_nonshared():
    # unix passes {T1,T2,T3}; mdtools passes {T1,T2}. Intersection {T1,T2};
    # unix's median must NOT include T3's 9999 cost.
    recs = [
        _rec("T1", "unix", "claude", True, tin=100), _rec("T1", "mdtools", "claude", True, tin=10),
        _rec("T2", "unix", "claude", True, tin=200), _rec("T2", "mdtools", "claude", True, tin=20),
        _rec("T3", "unix", "claude", True, tin=9999), _rec("T3", "mdtools", "claude", False, tin=30),
    ]
    out = intersection_cost(recs, "unix", "mdtools")
    assert out["n"] == 2
    assert out["median_a"] == 150   # median([100,200]); 9999 excluded
    assert out["median_b"] == 15


def test_disjoint_empty_intersection():
    recs = [
        _rec("T1", "unix", "claude", True, tin=100), _rec("T1", "mdtools", "claude", False, tin=10),
        _rec("T2", "unix", "claude", False, tin=200), _rec("T2", "mdtools", "claude", True, tin=20),
    ]
    out = intersection_cost(recs, "unix", "mdtools")
    assert out["n"] == 0
    assert out["basis"] is None
    assert out["median_a"] is None and out["median_b"] is None and out["delta"] is None


def test_token_absent_falls_back_to_calls():
    recs = [
        _rec("T1", "unix", "claude", True, tin=0, calls=5), _rec("T1", "mdtools", "claude", True, tin=0, calls=2),
        _rec("T2", "unix", "claude", True, tin=0, calls=7), _rec("T2", "mdtools", "claude", True, tin=0, calls=4),
    ]
    out = intersection_cost(recs, "unix", "mdtools")
    assert out["n"] == 2
    assert out["basis"] == "tool_calls"
    assert out["median_a"] == 6     # median([5,7])
    assert out["median_b"] == 3     # median([2,4])


def test_accepts_normalized_shape():
    # records keyed task/pass/calls (analyze.normalize_result shape), tokens absent
    recs = [
        {"task": "T1", "mode": "unix", "model": "claude", "thinking_level": None, "pass": True, "tokens_in": 0, "tokens_out": 0, "calls": 5},
        {"task": "T1", "mode": "mdtools", "model": "claude", "thinking_level": None, "pass": True, "tokens_in": 0, "tokens_out": 0, "calls": 2},
    ]
    out = intersection_cost(recs, "unix", "mdtools")
    assert out["n"] == 1 and out["basis"] == "tool_calls"
    assert out["median_a"] == 5 and out["median_b"] == 2


if __name__ == "__main__":
    fns = [v for k, v in sorted(globals().items()) if k.startswith("test_") and callable(v)]
    for fn in fns:
        fn()
        print(f"  ok {fn.__name__}")
    print(f"U3 OK: {len(fns)} tests passed")
