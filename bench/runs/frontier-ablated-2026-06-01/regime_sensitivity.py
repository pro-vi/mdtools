#!/usr/bin/env python3
"""Cache-regime sensitivity of the md-attribution verdicts.

Prompt caching is OUTPUT-INVARIANT: the model cannot see cache state, so token
counts and pass/fail are identical across regimes — only the PRICE of the
input-side prefix moves. So the "which cost basis" question is really "which
cache regime", and we can reconstruct every regime EXACTLY by re-pricing the
logged per-run usage (usage_breakdown.json) — no re-run needed.

cost(r) = input + cache_creation + r * cache_read + output

  r = 1.0  -> cold / cache-disabled  == raw tokens_in  == a ONE-OFF invocation
  r = 0.1  -> Anthropic warm read price == REPEATED / cached production use
  r = 0.0  -> cache fully free (lower bound)

We sweep r in [0,1], recompute attribution_verdict per (model x category), and
report the breakpoint r* where each cell's verdict flips. A verdict that holds
across all r is regime-robust; one that flips is regime-dependent.
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
from bench.agg_util import attribution_verdict  # noqa: E402

ROWS = json.loads((Path(__file__).parent / "usage_breakdown.json").read_text())
MODELS = ["claude-sonnet-4-6", "claude-opus-4-8[1m]"]
CATS = ["Targeted mutation", "Batch mutation"]


def records_at_r(model, r):
    recs = []
    for x in ROWS:
        if x["model"] != model:
            continue
        cost = x["input"] + x["cache_creation"] + r * x["cache_read"] + x["output"]
        recs.append({
            "task_id": x["task"], "mode": x["mode"], "model": model, "correct": True,
            "tokens_in": cost, "tokens_out": 0, "_total_tokens": cost,
            "md_probe_count": 1 if x["mode"] == "hybrid-no-md" else 0,
            "thinking_level": None, "runner": "claude-cli",
        })
    return recs


def verdict(model, cat, r):
    return attribution_verdict(records_at_r(model, r), "frontier", cat)["verdict"]


GRID = [round(i / 100, 2) for i in range(0, 101, 5)]
for model in MODELS:
    print(f"\n========== {model} ==========")
    for cat in CATS:
        seq = [(r, verdict(model, cat, r)) for r in GRID]
        segs = []
        for r, v in seq:
            if not segs or segs[-1][1] != v:
                segs.append([r, v, r])
            else:
                segs[-1][2] = r
        print(f"  {cat}:")
        for lo, v, hi in segs:
            tag = ("  <- COLD/one-off (raw tokens)" if lo <= 1.0 <= hi
                   else "  <- WARM/repeated (Anthropic r=0.1)" if lo <= 0.10 <= hi else "")
            print(f"      r in [{lo:.2f},{hi:.2f}]  {v}{tag}")
