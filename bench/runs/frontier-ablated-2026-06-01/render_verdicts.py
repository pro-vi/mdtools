#!/usr/bin/env python3
"""Render the FIRST valid md-attribution verdicts: prior-clean unix/hybrid
(frontier-clean-2026-06-01) + NEW ablated hybrid-no-md (frontier-ablated-2026-06-01,
the 611c2c3 ./md-stub fix). Per-model, because both models are tier 'frontier'."""
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
from bench.agg_util import attribution_verdict, category_for  # noqa: E402

ROOT = Path(__file__).resolve().parents[3]
CLEAN = ROOT / "bench/runs/frontier-clean-2026-06-01"
ABLATED = ROOT / "bench/runs/frontier-ablated-2026-06-01"


def load(path):
    recs = []
    txt = Path(path).read_text()
    for m in re.finditer(r"\[\s*{.*?}\s*\]", txt, re.S):
        try:
            recs.extend(json.loads(m.group(0)))
        except Exception:
            pass
    return recs


def pct(b, a):
    if not a:
        return "n/a"
    return f"{(b - a) / a * 100:+.0f}%"


for label, sub in (("Sonnet", "sonnet"), ("Opus", "opus")):
    recs = (
        load(CLEAN / sub / "unix.txt")
        + load(CLEAN / sub / "hybrid.txt")
        + load(ABLATED / sub / "hybrid-no-md.txt")
    )
    modes = {r.get("mode") for r in recs}
    print(f"\n========== {label} (records={len(recs)}, modes={sorted(modes)}) ==========")
    for cat in ("Targeted mutation", "Batch mutation"):
        v = attribution_verdict(recs, "frontier", cat)
        p, lf = v["pareto"], v["lift"]
        print(f"\n  [{cat}]  VERDICT = {v['verdict']}")
        print(f"    pareto(unix->hybrid): n={p['n']} unix={p['unix']} hybrid={p['hybrid']} ({pct(p['hybrid'], p['unix'])})")
        print(f"    lift(no-md->hybrid):  n={lf['n']} no-md={lf['hybrid_no_md']} hybrid={lf['hybrid']} ({pct(lf['hybrid'], lf['hybrid_no_md'])})")
        print(f"    baseline_ok={v['baseline_ok']} reason={v['baseline_reason']} nomd_probe={v['nomd_probe']} lift_positive={v['lift_positive']}")
