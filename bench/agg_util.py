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


# --- U2: md-attribution gate (BENCH_V2_ATTRIBUTION.md) ---

# Categories where md should add CAUSAL value (gate on md-lift over hybrid-no-md).
# Everything else (Text manipulation) is tie-acceptable: hybrid >= unix is the win.
STRUCTURAL_CATEGORIES = {
    "Extraction",
    "Targeted mutation",
    "Batch mutation",
    "Multi-step",
    "Content delivery",
    "Safe-fail",
    "Metadata",
    "Table projection",
}


def _aggregate_replicates(records: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Collapse N runs of the same (task, model, thinking, mode) into one record:
    pass = majority of runs passed; tokens/calls = median across runs. Lets N>=3
    metrics survive intersection_cost's (task, model, thinking) keying — which
    otherwise keeps only the last replicate per mode."""
    groups: dict[tuple, list[dict]] = {}
    for r in records:
        key = (_task_id(r), r.get("model"), r.get("thinking_level"), r.get("mode"))
        groups.setdefault(key, []).append(r)
    out: list[dict] = []
    for (task, model, thinking, mode), runs in groups.items():
        passed = sum(1 for r in runs if _passed(r)) * 2 >= len(runs)  # majority
        out.append({
            "task_id": task, "model": model, "thinking_level": thinking, "mode": mode,
            "correct": passed,
            "tokens_in": statistics.median([int(r.get("tokens_in", 0) or 0) for r in runs]),
            "tokens_out": statistics.median([int(r.get("tokens_out", 0) or 0) for r in runs]),
            "tool_calls": statistics.median([_calls(r) for r in runs]),
            # probe count is MAX-across-runs, not median: one stuck run (e.g. [1,1,5])
            # is a dirty cell even if the median is ≤1. See BENCH_V2_CLEAN_ABLATION.md.
            "md_probe_count": max([int(r.get("md_probe_count", 0) or 0) for r in runs]),
            "_n_runs": len(runs),
        })
    return out


def _cell_records(records, tier, category):
    return [r for r in records
            if extract_model_tier(r.get("model")) == tier
            and category_for(_task_id(r)) == category]


def _pass_rate(records, mode):
    cells = [r for r in records if r.get("mode") == mode]
    if not cells:
        return None
    return sum(1 for r in cells if _passed(r)) / len(cells)


def _probe_count(records, mode):
    """MAX md_probe_count over the mode's records (clean-ablation: hybrid-no-md).
    Max, not median: any task whose ablation banged into the md stub >1× makes the
    cell dirty — a median would hide a single stuck task in a multi-task category."""
    vals = [int(r.get("md_probe_count", 0) or 0) for r in records if r.get("mode") == mode]
    return max(vals) if vals else 0


def attribution_verdict(records, tier, category,
                        cost_tol=0.05, lift_margin=0.05, baseline_tol=0.20,
                        parity_tol=0.10, min_overlap=1):
    """The md-attribution gate for one (tier x category) cell. A structural cell
    CLOSES only when md is *causally* responsible — hybrid beats unix (Pareto) AND
    beats the hybrid-no-md baseline (md-lift), AND the baseline didn't flail.

    Verdicts: CLOSES | OPEN:loses-unix | OPEN:no-lift |
    OPEN:insufficient-evidence | SUSPECT:baseline-flails.
    """
    sub = _aggregate_replicates(_cell_records(records, tier, category))
    structural = category in STRUCTURAL_CATEGORIES

    unix_pass = _pass_rate(sub, "unix")
    hybrid_pass = _pass_rate(sub, "hybrid")
    nomd_pass = _pass_rate(sub, "hybrid-no-md")

    # Pareto front vs unix (the product claim)
    pu = intersection_cost(sub, "unix", "hybrid")          # a=unix, b=hybrid
    correctness_ok = (hybrid_pass is not None and unix_pass is not None
                      and hybrid_pass >= unix_pass - 1e-9)
    cost_ok = None
    if pu["n"] > 0:
        cost_ok = pu["median_b"] <= pu["median_a"] * (1 + cost_tol) + 1e-9
    pareto = bool(correctness_ok) and (cost_ok is None or cost_ok)

    # --- clean-ablation baseline validity: correctness parity + <=1 probe + cost parity ---
    # The baseline (hybrid-no-md) must be a COMPETENT unix fallback, not a sabotaged
    # mode — else hybrid "beats" it for the wrong reason. See BENCH_V2_CLEAN_ABLATION.md.
    nomd_probe = _probe_count(sub, "hybrid-no-md")
    correctness_parity = (nomd_pass is None or unix_pass is None
                          or nomd_pass >= unix_pass - parity_tol)
    probe_ok = nomd_probe <= 1
    bu = intersection_cost(sub, "unix", "hybrid-no-md")    # a=unix, b=no-md
    cost_parity = (bu["n"] == 0) or (bu["median_b"] <= bu["median_a"] * (1 + baseline_tol))
    baseline_ok = bool(correctness_parity and probe_ok and cost_parity)
    baseline_reason = ("correctness" if not correctness_parity
                       else "probes" if not probe_ok else "cost")

    # --- md-lift: hybrid must beat the BETTER of unix and hybrid-no-md ---
    # Not just the clean ablation. Else a hybrid-no-md merely degraded WITHIN
    # baseline tolerance (≤10pp pass, ≤20% cost) becomes the *source* of "lift"
    # while hybrid only ties unix — tolerance arbitrage. Requiring hybrid to beat
    # max(unix,no-md) on correctness OR min(unix,no-md) on cost makes md-lift mean
    # md produced a real, attributable win over a unix-only agent. See
    # BENCH_V2_CLEAN_ABLATION.md (round-3 /second-opinion).
    lf = intersection_cost(sub, "hybrid-no-md", "hybrid")  # a=no-md, b=hybrid
    pass_baselines = [p for p in (unix_pass, nomd_pass) if p is not None]
    corr_lift = bool(hybrid_pass is not None and pass_baselines
                     and hybrid_pass > max(pass_baselines) + 1e-9)
    # cost: hybrid strictly under unix (pu) AND under the clean no-md (lf), each on
    # its own valid paired overlap == hybrid < min(unix, no-md) by margin.
    beats_unix_cost = (pu["n"] >= min_overlap
                       and pu["median_b"] < pu["median_a"] * (1 - lift_margin))
    beats_nomd_cost = (lf["n"] >= min_overlap and probe_ok
                       and lf["median_b"] < lf["median_a"] * (1 - lift_margin))
    cost_lift = bool(beats_unix_cost and beats_nomd_cost)
    lift_positive = bool(corr_lift or cost_lift)

    if not pareto:
        verdict = "OPEN:loses-unix"
    elif not structural:
        verdict = "CLOSES"                       # tie-acceptable: Pareto is enough
    elif not baseline_ok:
        verdict = f"SUSPECT:baseline-flails({baseline_reason})"
    elif nomd_pass is None:
        verdict = "OPEN:insufficient-evidence"   # no clean baseline to attribute against
    elif lift_positive:
        verdict = "CLOSES"
    elif lf["n"] == 0 and not corr_lift:
        verdict = "OPEN:insufficient-evidence"
    else:
        verdict = "OPEN:no-lift"

    return {
        "verdict": verdict,
        "structural": structural,
        "correctness_ok": correctness_ok,
        "cost_ok": cost_ok,
        "lift_positive": lift_positive,
        "baseline_ok": baseline_ok,
        "baseline_reason": None if baseline_ok else baseline_reason,
        "nomd_probe": nomd_probe,
        "pareto": {"n": pu["n"], "unix": pu["median_a"], "hybrid": pu["median_b"]},
        "lift": {"n": lf["n"], "hybrid_no_md": lf["median_a"], "hybrid": lf["median_b"]},
    }
