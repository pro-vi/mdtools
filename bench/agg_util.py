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
from collections import defaultdict
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
    # An aggregated record carries `_total_tokens` = median of per-run (in+out)
    # totals; prefer it. median(in)+median(out) != median(in+out), and cost is the
    # total — summing component medians can invent a cost no replicate had. (Codex P2.)
    if rec.get("_total_tokens") is not None:
        return int(rec["_total_tokens"])
    return int(rec.get("tokens_in", 0) or 0) + int(rec.get("tokens_out", 0) or 0)


def _calls(rec: dict[str, Any]) -> int:
    return int(rec.get("tool_calls", rec.get("calls", 0)) or 0)


def _cost(rec: dict[str, Any], basis: str) -> int:
    return _tokens(rec) if basis == "tokens" else _calls(rec)


def _median(values: list[float]) -> float | None:
    return statistics.median(values) if values else None


class InvalidCellError(ValueError):
    """Raised when v3 cell aggregation would hide too many infrastructure errors."""


def _is_tool_error(rec: dict[str, Any]) -> bool:
    return str(rec.get("runner_error") or "").startswith("tool_error:")


def _is_error_trial(rec: dict[str, Any]) -> bool:
    if rec.get("verdict") == "error":
        return True
    return bool(rec.get("runner_error")) and not _is_tool_error(rec)


def cell_trials(
    records: list[dict[str, Any]],
    *,
    mode: str | None = None,
    model: str | None = None,
    runner: str | None = None,
    thinking_level: str | None = None,
) -> list[dict[str, Any]]:
    """Return raw v3 trials for one logical cell without majority-vote collapse."""
    trials = []
    for rec in records:
        if mode is not None and rec.get("mode") != mode:
            continue
        if model is not None and rec.get("model") != model:
            continue
        if runner is not None and rec.get("runner") != runner:
            continue
        if thinking_level is not None and rec.get("thinking_level") != thinking_level:
            continue
        trials.append(rec)
    return sorted(trials, key=lambda r: (_task_id(r), int(r.get("run_index") or 0)))


def _validate_error_rate(trials: list[dict[str, Any]], *, max_error_rate: float) -> None:
    by_task: dict[Any, list[dict[str, Any]]] = defaultdict(list)
    for trial in trials:
        by_task[_task_id(trial)].append(trial)
    bad = []
    for task, task_trials in by_task.items():
        if not task_trials:
            continue
        errors = sum(1 for trial in task_trials if _is_error_trial(trial))
        if errors / len(task_trials) > max_error_rate:
            bad.append(f"{task}: {errors}/{len(task_trials)} error trials")
    if bad:
        raise InvalidCellError(
            "cell invalid because error-trial rate exceeds "
            f"{max_error_rate:.0%}: " + "; ".join(sorted(bad))
        )


def pass_at_1_mean(
    trials: list[dict[str, Any]],
    *,
    max_error_rate: float = 0.10,
) -> float:
    """Mean per-task pass@1 over raw trials; error trials count as failed trials."""
    if not trials:
        return 0.0
    _validate_error_rate(trials, max_error_rate=max_error_rate)
    by_task: dict[Any, list[dict[str, Any]]] = defaultdict(list)
    for trial in trials:
        by_task[_task_id(trial)].append(trial)
    task_rates = [
        sum(1 for trial in task_trials if _passed(trial) and not _is_error_trial(trial))
        / len(task_trials)
        for task_trials in by_task.values()
    ]
    return sum(task_rates) / len(task_rates)


def pass_hat_k(
    trials: list[dict[str, Any]],
    *,
    max_error_rate: float = 0.10,
) -> float:
    """Fraction of tasks that pass all k raw trials; k may be 1 for legacy bundles."""
    if not trials:
        return 0.0
    _validate_error_rate(trials, max_error_rate=max_error_rate)
    by_task: dict[Any, list[dict[str, Any]]] = defaultdict(list)
    for trial in trials:
        by_task[_task_id(trial)].append(trial)
    return sum(
        1
        for task_trials in by_task.values()
        if task_trials and all(_passed(trial) and not _is_error_trial(trial) for trial in task_trials)
    ) / len(by_task)


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
        # runner is part of the cell identity: the same model under two runners
        # (e.g. claude-cli vs oai-loop) is NOT the same execution stack and must not
        # be paired/medianed together. `runner` is None on records that don't carry
        # it (text logs), which preserves prior behavior. (PR#10 Codex P2.)
        key = (_task_id(r), r.get("model"), r.get("thinking_level"), r.get("runner"))
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
    """Collapse N runs of the same (task, model, thinking, mode, runner) into one
    record: pass = majority of runs passed; tokens/calls = median across runs. Lets
    N>=3 metrics survive intersection_cost's keying — which otherwise keeps only the
    last replicate per mode. `runner` is part of the key so two runners on the same
    model aren't merged as replicates (PR#10 Codex P2); it's preserved on the output
    so the downstream intersection re-keys consistently."""
    groups: dict[tuple, list[dict]] = {}
    for r in records:
        key = (_task_id(r), r.get("model"), r.get("thinking_level"), r.get("mode"), r.get("runner"))
        groups.setdefault(key, []).append(r)
    out: list[dict] = []
    for (task, model, thinking, mode, runner), runs in groups.items():
        passed = sum(1 for r in runs if _passed(r)) * 2 > len(runs)  # strict majority (tie = fail)
        out.append({
            "task_id": task, "model": model, "thinking_level": thinking, "mode": mode,
            "runner": runner,
            "correct": passed,
            "tokens_in": statistics.median([int(r.get("tokens_in", 0) or 0) for r in runs]),
            "tokens_out": statistics.median([int(r.get("tokens_out", 0) or 0) for r in runs]),
            # Cost is the per-run TOTAL: median(in+out), NOT median(in)+median(out)
            # (those differ and can invent a cost no replicate had, flipping CLOSES/
            # OPEN). _tokens() reads this; the component medians above are display-only.
            "_total_tokens": statistics.median([_tokens(r) for r in runs]),
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


def _root_verdict(sub, structural, baseline, treatment, ablation, ablation_key,
                  *, cost_tol, lift_margin, baseline_tol, parity_tol, min_overlap):
    """The attribution gate for ONE baseline-family root. `treatment` CLOSES only
    when it beats `baseline` (Pareto) AND beats the better of (`baseline`,
    `ablation`) on the lift axis, AND the clean `ablation` baseline didn't flail.

    The logic is verbatim the historical hardcoded gate — only the mode names are
    parameters. POSIX root: (unix, hybrid, hybrid-no-md). Native root (FRAC-194):
    (native, native+md, native+md-no-md). `ablation_key` is the output dict key for
    the lift baseline ("hybrid_no_md" for POSIX — preserves byte-identical output).
    """
    base_pass = _pass_rate(sub, baseline)
    treat_pass = _pass_rate(sub, treatment)
    abl_pass = _pass_rate(sub, ablation)

    # Pareto front vs the baseline (the product claim)
    pu = intersection_cost(sub, baseline, treatment)        # a=baseline, b=treatment
    correctness_ok = (treat_pass is not None and base_pass is not None
                      and treat_pass >= base_pass - 1e-9)
    cost_ok = None
    if pu["n"] > 0:
        cost_ok = pu["median_b"] <= pu["median_a"] * (1 + cost_tol) + 1e-9
    pareto = bool(correctness_ok) and (cost_ok is None or cost_ok)

    # --- clean-ablation baseline validity: correctness parity + <=1 probe + cost parity ---
    # The ablation must be a COMPETENT fallback, not a sabotaged mode — else the
    # treatment "beats" it for the wrong reason. See BENCH_V2_CLEAN_ABLATION.md.
    abl_probe = _probe_count(sub, ablation)
    correctness_parity = (abl_pass is None or base_pass is None
                          or abl_pass >= base_pass - parity_tol)
    probe_ok = abl_probe <= 1
    bu = intersection_cost(sub, baseline, ablation)         # a=baseline, b=ablation
    cost_parity = (bu["n"] == 0) or (bu["median_b"] <= bu["median_a"] * (1 + baseline_tol))
    baseline_ok = bool(correctness_parity and probe_ok and cost_parity)
    baseline_reason = ("correctness" if not correctness_parity
                       else "probes" if not probe_ok else "cost")

    # --- lift: treatment must beat the BETTER of baseline and ablation ---
    # Not just the clean ablation. Else an ablation merely degraded WITHIN tolerance
    # (≤10pp pass, ≤20% cost) becomes the *source* of "lift" while the treatment only
    # ties the baseline — tolerance arbitrage. Requiring treatment to beat
    # max(baseline,ablation) on correctness OR min(...) on cost makes lift mean a
    # real, attributable win over a baseline-only agent. See BENCH_V2_CLEAN_ABLATION.md.
    lf = intersection_cost(sub, ablation, treatment)        # a=ablation, b=treatment
    pass_baselines = [p for p in (base_pass, abl_pass) if p is not None]
    corr_lift = bool(treat_pass is not None and pass_baselines
                     and treat_pass > max(pass_baselines) + 1e-9)
    beats_base_cost = (pu["n"] >= min_overlap
                       and pu["median_b"] < pu["median_a"] * (1 - lift_margin))
    beats_abl_cost = (lf["n"] >= min_overlap and probe_ok
                      and lf["median_b"] < lf["median_a"] * (1 - lift_margin))
    cost_lift = bool(beats_base_cost and beats_abl_cost)
    lift_positive = bool(corr_lift or cost_lift)

    if not pareto:
        verdict = f"OPEN:loses-{baseline}"
    elif not structural:
        verdict = "CLOSES"                       # tie-acceptable: Pareto is enough
    elif not baseline_ok:
        verdict = f"SUSPECT:baseline-flails({baseline_reason})"
    elif abl_pass is None:
        verdict = "OPEN:insufficient-evidence"   # no clean baseline to attribute against
    elif lift_positive:
        verdict = "CLOSES"
    elif lf["n"] < min_overlap and not corr_lift:
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
        "nomd_probe": abl_probe,
        "pareto": {"n": pu["n"], baseline: pu["median_a"], treatment: pu["median_b"]},
        "lift": {"n": lf["n"], ablation_key: lf["median_a"], treatment: lf["median_b"]},
    }


def attribution_verdict(records, tier, category,
                        cost_tol=0.05, lift_margin=0.05, baseline_tol=0.20,
                        parity_tol=0.10, min_overlap=2):
    """The md-attribution gate for one (tier x category) cell.

    Returns the POSIX-rooted verdict (unix, hybrid, hybrid-no-md) at the top level —
    byte-identical to the historical gate. When the cell also has native-arm data,
    a "native_root" key carries the deployment-true verdict rooted at
    (native, native+md, native+md-no-md) — "does md help an agent that already has
    native Edit?" (FRAC-194). native-root is present iff both native and native+md
    have ≥1 run; partial native data ⇒ OPEN:insufficient-evidence.

    Verdicts: CLOSES | OPEN:loses-<baseline> | OPEN:no-lift |
    OPEN:insufficient-evidence | SUSPECT:baseline-flails.
    """
    sub = _aggregate_replicates(_cell_records(records, tier, category))
    structural = category in STRUCTURAL_CATEGORIES
    tols = dict(cost_tol=cost_tol, lift_margin=lift_margin, baseline_tol=baseline_tol,
                parity_tol=parity_tol, min_overlap=min_overlap)

    result = _root_verdict(sub, structural, "unix", "hybrid", "hybrid-no-md", "hybrid_no_md", **tols)
    # Whether this cell actually has POSIX-arm data. The top-level verdict is always
    # computed (POSIX-rooted), but a native-only run has no unix/hybrid/hybrid-no-md
    # data — so the verdict is a spurious "loses-unix". Renderers gate the POSIX row on
    # this. (FRAC-194 review #5; default-True elsewhere preserves historical callers.)
    result["posix_present"] = any(
        _pass_rate(sub, m) is not None for m in ("unix", "hybrid", "hybrid-no-md"))

    # native-rooted arm (FRAC-194): only when the cell carries native-arm data.
    native_pass = _pass_rate(sub, "native")
    treat_pass = _pass_rate(sub, "native+md")
    if native_pass is not None and treat_pass is not None:
        result["native_root"] = _root_verdict(
            sub, structural, "native", "native+md", "native+md-no-md", "native_md_no_md", **tols)
    elif native_pass is not None or treat_pass is not None:
        # partial native data: can't attribute — don't conclude (invariant).
        result["native_root"] = {"verdict": "OPEN:insufficient-evidence",
                                 "structural": structural, "incomplete": True}

    return result
