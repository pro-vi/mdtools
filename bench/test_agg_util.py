#!/usr/bin/env python3
"""Tests for bench/agg_util.py (bench-v2 U3 — the load-bearing pure functions).

Runnable via pytest OR standalone (pytest is not always installed):
    python3 bench/test_agg_util.py
"""
from __future__ import annotations

import pathlib as _pathlib
import sys as _sys

# Support the documented standalone invocation `python3 bench/test_agg_util.py`:
# run from the repo root, Python puts `bench/` (the script's dir) on sys.path, not
# the repo root, so the `bench.agg_util` package import below can't resolve without
# putting the repo root on the path first. (`python3 -m bench.test_agg_util` works
# without this; this makes the docstring's command work too.)
_sys.path.insert(0, str(_pathlib.Path(__file__).resolve().parent.parent))

from bench.agg_util import (
    STRUCTURAL_CATEGORIES,
    attribution_verdict,
    category_for,
    extract_model_tier,
    intersection_cost,
)


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


# --- U2: md-attribution gate (BENCH_V2_ATTRIBUTION.md) ---

def _ar(task, mode, model, passed, cost, n=1):
    """n replicates of one (task, mode, model) cell; cost carried via tokens_in."""
    return [_rec(task, mode, model, passed, tin=cost) for _ in range(n)]


def test_structural_categories():
    assert "Targeted mutation" in STRUCTURAL_CATEGORIES
    assert "Extraction" in STRUCTURAL_CATEGORIES
    assert "Text manipulation" not in STRUCTURAL_CATEGORIES  # tie-acceptable


def test_attribution_neutering_stays_open():
    # THE load-bearing test: structural cell where hybrid ≈ hybrid-no-md ≈ unix
    # (the loop neutered the prompt → md unused → removing md changes nothing).
    recs = (_ar("T7", "unix", "claude", True, 80000)
            + _ar("T7", "hybrid", "claude", True, 80000)
            + _ar("T7", "hybrid-no-md", "claude", True, 80000))
    v = attribution_verdict(recs, "frontier", "Targeted mutation")
    assert v["verdict"] == "OPEN:no-lift", v


def test_attribution_genuine_value_closes():
    # md makes hybrid cheaper than hybrid-no-md AND hybrid ≤ unix → md earns it.
    recs = (_ar("T7", "unix", "claude", True, 80000)
            + _ar("T7", "hybrid", "claude", True, 50000)
            + _ar("T7", "hybrid-no-md", "claude", True, 80000))
    v = attribution_verdict(recs, "frontier", "Targeted mutation")
    assert v["verdict"] == "CLOSES", v


def test_attribution_loses_unix():
    recs = (_ar("T7", "unix", "claude", True, 80000)
            + _ar("T7", "hybrid", "claude", True, 100000)  # > unix +5%
            + _ar("T7", "hybrid-no-md", "claude", True, 80000))
    v = attribution_verdict(recs, "frontier", "Targeted mutation")
    assert v["verdict"] == "OPEN:loses-unix", v


def test_attribution_counter_game_suspect():
    # baseline flails far above unix (prompt pushed md so no-md can't cope) → suspect lift
    recs = (_ar("T7", "unix", "claude", True, 80000)
            + _ar("T7", "hybrid", "claude", True, 50000)
            + _ar("T7", "hybrid-no-md", "claude", True, 200000))
    v = attribution_verdict(recs, "frontier", "Targeted mutation")
    assert v["verdict"] == "SUSPECT:baseline-flails(cost)", v


def test_attribution_tie_acceptable_closes():
    # Text manipulation: hybrid ≥ unix is enough; no lift required, no hybrid-no-md needed.
    recs = (_ar("T4", "unix", "claude", True, 80000)
            + _ar("T4", "hybrid", "claude", True, 80000))
    v = attribution_verdict(recs, "frontier", "Text manipulation")
    assert v["verdict"] == "CLOSES", v


def test_attribution_insufficient_evidence():
    # structural cell with NO hybrid-no-md records → can't attribute → not closed
    recs = (_ar("T7", "unix", "claude", True, 80000)
            + _ar("T7", "hybrid", "claude", True, 50000))
    v = attribution_verdict(recs, "frontier", "Targeted mutation")
    assert v["verdict"] == "OPEN:insufficient-evidence", v


def test_attribution_n3_aggregation():
    # N=3 replicates per (task,mode) must survive intersection keying (majority pass, median cost)
    recs = (_ar("T7", "unix", "claude", True, 80000, n=3)
            + _ar("T7", "hybrid", "claude", True, 50000, n=3)
            + _ar("T7", "hybrid-no-md", "claude", True, 80000, n=3))
    v = attribution_verdict(recs, "frontier", "Targeted mutation")
    assert v["verdict"] == "CLOSES", v


# --- clean-ablation validity gate (BENCH_V2_CLEAN_ABLATION.md) ---

def _rpb(task, mode, model, passed, cost, probe=0, n=1):
    """n replicates of (task,mode) with a per-run md_probe_count."""
    out = []
    for _ in range(n):
        r = _rec(task, mode, model, passed, tin=cost)
        r["md_probe_count"] = probe
        out.append(r)
    return out


def test_attribution_correctness_poisoning_suspect():
    # The worst exploit: hybrid-no-md FAILS a task unix passes (sabotaged baseline)
    # but cost is fine. Must be SUSPECT(correctness), NOT CLOSES.
    recs = (_rpb("T7", "unix", "claude", True, 100) + _rpb("T10", "unix", "claude", True, 100)
            + _rpb("T7", "hybrid", "claude", True, 100) + _rpb("T10", "hybrid", "claude", True, 100)
            + _rpb("T7", "hybrid-no-md", "claude", True, 105, probe=1)
            + _rpb("T10", "hybrid-no-md", "claude", False, 105, probe=1))
    v = attribution_verdict(recs, "frontier", "Targeted mutation")
    assert v["verdict"] == "SUSPECT:baseline-flails(correctness)", v


def test_attribution_multi_probe_suspect():
    # clean cost+correctness but the agent hit the md stub >1× (prompt over-pushes md) → suspect.
    recs = (_rpb("T7", "unix", "claude", True, 100) + _rpb("T10", "unix", "claude", True, 100)
            + _rpb("T7", "hybrid", "claude", True, 80) + _rpb("T10", "hybrid", "claude", True, 80)
            + _rpb("T7", "hybrid-no-md", "claude", True, 100, probe=3)
            + _rpb("T10", "hybrid-no-md", "claude", True, 100, probe=3))
    v = attribution_verdict(recs, "frontier", "Targeted mutation")
    assert v["verdict"] == "SUSPECT:baseline-flails(probes)", v


def test_attribution_clean_baseline_closes():
    # clean ablation: hybrid-no-md ≈ unix on pass+cost, ≤1 probe; hybrid genuinely cheaper → CLOSES.
    recs = (_rpb("T7", "unix", "claude", True, 100) + _rpb("T10", "unix", "claude", True, 100)
            + _rpb("T7", "hybrid", "claude", True, 50) + _rpb("T10", "hybrid", "claude", True, 50)
            + _rpb("T7", "hybrid-no-md", "claude", True, 100, probe=1)
            + _rpb("T10", "hybrid-no-md", "claude", True, 100, probe=1))
    v = attribution_verdict(recs, "frontier", "Targeted mutation")
    assert v["verdict"] == "CLOSES", v


def test_attribution_neutering_clean_probe_still_open():
    # neutered prompt → agent never tries md (probe 0) → hybrid≈hybrid-no-md → no lift.
    recs = (_rpb("T7", "unix", "claude", True, 100) + _rpb("T10", "unix", "claude", True, 100)
            + _rpb("T7", "hybrid", "claude", True, 100) + _rpb("T10", "hybrid", "claude", True, 100)
            + _rpb("T7", "hybrid-no-md", "claude", True, 100, probe=0)
            + _rpb("T10", "hybrid-no-md", "claude", True, 100, probe=0))
    v = attribution_verdict(recs, "frontier", "Targeted mutation")
    assert v["verdict"] == "OPEN:no-lift", v


# --- round-3 tightening: lift vs the BETTER baseline (no tolerance arbitrage) ---

def test_attribution_cost_tolerance_arbitrage_open():
    # The round-3 hole: hybrid TIES unix on cost (100 == 100) but "beats" a
    # hybrid-no-md degraded WITHIN baseline tol (106, ≤20%, ≤1 probe). Old rule
    # fired cost_lift (100 < 106·0.95) → CLOSES; but md added NO win over unix —
    # the "lift" was the degraded ablation. Must be OPEN:no-lift (lift vs the
    # better of unix and no-md, not the weaker one).
    recs = (_rpb("T7", "unix", "claude", True, 100) + _rpb("T10", "unix", "claude", True, 100)
            + _rpb("T7", "hybrid", "claude", True, 100) + _rpb("T10", "hybrid", "claude", True, 100)
            + _rpb("T7", "hybrid-no-md", "claude", True, 106, probe=1)
            + _rpb("T10", "hybrid-no-md", "claude", True, 106, probe=1))
    v = attribution_verdict(recs, "frontier", "Targeted mutation")
    assert v["verdict"] == "OPEN:no-lift", v


def test_attribution_probe_tail_suspect():
    # round-3: a probe TAIL ([1,1,5] across runs of one task) must trip
    # SUSPECT(probes) — a median-≤1 gate would let it pass (median([1,1,5])==1).
    # Probe validity is max-across-runs, not median: one stuck run is a dirty cell.
    recs = (_rpb("T7", "unix", "claude", True, 100) + _rpb("T10", "unix", "claude", True, 100)
            + _rpb("T7", "hybrid", "claude", True, 80) + _rpb("T10", "hybrid", "claude", True, 80)
            # T7/no-md runs probe [1, 1, 5]: median 1 (would pass), max 5 (dirty)
            + _rpb("T7", "hybrid-no-md", "claude", True, 100, probe=1, n=2)
            + _rpb("T7", "hybrid-no-md", "claude", True, 100, probe=5)
            + _rpb("T10", "hybrid-no-md", "claude", True, 100, probe=1))
    v = attribution_verdict(recs, "frontier", "Targeted mutation")
    assert v["verdict"] == "SUSPECT:baseline-flails(probes)", v


if __name__ == "__main__":
    fns = [v for k, v in sorted(globals().items()) if k.startswith("test_") and callable(v)]
    for fn in fns:
        fn()
        print(f"  ok {fn.__name__}")
    print(f"U3 OK: {len(fns)} tests passed")
