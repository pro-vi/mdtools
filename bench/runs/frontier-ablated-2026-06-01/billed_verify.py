#!/usr/bin/env python3
"""Billed-$ correction of regime_sensitivity.py (2026-06-02).

regime_sensitivity.py priced cost as `input + cc + r*cache_read + output` —
output at 1x, sweeping only cache_read. Real Anthropic billing weights output
~5x input and cache_creation ~1.25x; cache_read ~0.1x. Under those weights the
Sonnet-Batch "regime-conditional flip" (r*~0.42) disappears: it is regime-ROBUST.

Three cost models per (model x category):
  GT     = cost_usd directly (zero price assumptions, ground truth)
  billed = input + 1.25*cc + w*cache_read + 5*output  (Anthropic ratios, model-
           invariant; w = swept cache-read weight; real warm = 0.1, cold = 1.0,
           true cold one-off ~1.25 = reads billed as fresh creation)
  plan   = input + cc + r*cache_read + output          (the old output@1x formula)

Cross-check: billed@0.1 reproduces the cost_usd verdict on every cell.
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
from bench.agg_util import attribution_verdict, category_for, STRUCTURAL_CATEGORIES  # noqa: E402

ROWS = json.loads((Path(__file__).parent / "usage_breakdown.json").read_text())
MODELS = ["claude-sonnet-4-6", "claude-opus-4-8[1m]"]
GRID = [round(i / 100, 2) for i in range(0, 126, 2)]  # 0 .. 1.25 (incl. true cold one-off)


def recs(model, cost_fn):
    out = []
    for x in ROWS:
        if x["model"] != model:
            continue
        c = int(round(cost_fn(x)))
        out.append({"task_id": x["task"], "mode": x["mode"], "model": model, "correct": True,
                    "_total_tokens": c, "tokens_in": c, "tokens_out": 0,
                    "md_probe_count": 1 if x["mode"] == "hybrid-no-md" else 0,
                    "thinking_level": None, "runner": "claude-cli"})
    return out


def verdict(model, cat, cost_fn):
    r = attribution_verdict(recs(model, cost_fn), "frontier", cat)
    return r["verdict"], r["pareto"]["n"], r["pareto"]


c_usd = lambda x: x["cost_usd"] * 1e7
c_billed = lambda w: (lambda x: x["input"] + 1.25 * x["cache_creation"] + w * x["cache_read"] + 5 * x["output"])
c_plan = lambda r: (lambda x: x["input"] + x["cache_creation"] + r * x["cache_read"] + x["output"])

cats = sorted({category_for(x["task"]) for x in ROWS})
print(f"categories present: {cats}\n")

for model in MODELS:
    print(f"========== {model} ==========")
    for cat in cats:
        gt, n, _ = verdict(model, cat, c_usd)
        if n == 0:
            continue
        struct = "structural" if cat in STRUCTURAL_CATEGORIES else "tie-ok"
        bset = {verdict(model, cat, c_billed(w))[0] for w in GRID}
        b_real, _, parb = verdict(model, cat, c_billed(0.1))
        delta = f" (hyb {(parb['hybrid']/parb['unix']-1)*100:+.0f}% vs unix)" if parb["unix"] else ""
        pset = {verdict(model, cat, c_plan(r))[0] for r in GRID}
        match = "OK" if gt == b_real else f"MISMATCH(gt={gt})"
        print(f"  {cat:18s} [{struct:10s} n={n:2d}] GT={gt:28s} billed@0.1={b_real:28s}{delta}")
        print(f"  {'':18s}  billed-regime-dep={len(bset) > 1!s:5s}{('  ' + str(sorted(bset))) if len(bset) > 1 else '':40s}  cross-check={match}")
        print(f"  {'':18s}  PLAN-formula-regime-dep={len(pset) > 1!s:5s}{('  ' + str(sorted(pset))) if len(pset) > 1 else ''}")

print("\n========== Sonnet Batch: explicit regime points (billed-$) ==========")
for label, w in [("warm/repeated r=0.1", 0.1), ("cold cache-read r=1.0", 1.0), ("true cold one-off r=1.25", 1.25)]:
    vv, _, par = verdict("claude-sonnet-4-6", "Batch mutation", c_billed(w))
    sep = f"hyb={par['hybrid']:.0f} unix={par['unix']:.0f} ({(par['hybrid']/par['unix']-1)*100:+.0f}%)" if par["unix"] else ""
    print(f"  {label:26s} -> {vv:10s}  {sep}")
