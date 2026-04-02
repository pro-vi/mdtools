#!/usr/bin/env python3
"""Analyze benchmark results from harness output files."""

import re
import sys
from collections import defaultdict

def parse_results(filepath):
    """Parse harness output into structured results."""
    results = []
    mode = None
    model = None
    with open(filepath) as f:
        for line in f:
            # === MODE: hybrid (N=1, model=claude-haiku-4-5-20251001) ===
            m = re.match(r"=== MODE: (\w+) \(N=(\d+)(?:, model=([^)]+))?\)", line)
            if m:
                mode = m.group(1)
                model = m.group(3) or "opus"
                continue
            # md=PASS neutral=PASS | 24.82s | ~18292B out | ~1 calls
            m = re.match(r"\s+md=(PASS|FAIL).*\| ([\d.]+)s \| ~(\d+)B out \| ~(\d+) calls", line)
            if m:
                results.append({
                    "mode": mode,
                    "model": model,
                    "pass": m.group(1) == "PASS",
                    "time": float(m.group(2)),
                    "bytes": int(m.group(3)),
                    "calls": int(m.group(4)),
                })
            # T1: description...
            m2 = re.match(r"\s+(T\d+):", line)
            if m2 and results:
                results[-1]["task"] = m2.group(1) if not results[-1].get("task") else results[-1]["task"]
    return results


def parse_with_task_ids(filepath):
    """Parse harness output, matching task IDs to their results."""
    results = []
    mode = None
    model = None
    pending_task = None
    with open(filepath) as f:
        for line in f:
            m = re.match(r"=== MODE: (\w+) \(N=(\d+)(?:, model=([^)]+))?\)", line)
            if m:
                mode = m.group(1)
                model = m.group(3) or "opus"
                continue
            # Task line: "  T1: description..."
            m2 = re.match(r"\s+(T\d+): ", line)
            if m2:
                pending_task = m2.group(1)
                continue
            # Result line — new format with obs and mut
            m3 = re.match(
                r"\s+md=(PASS|FAIL).*\| ([\d.]+)s \| ~(\d+)B out \| obs:(\d+)B \| ~(\d+) calls \| (\d+) mut\s*(↻?)",
                line,
            )
            if not m3:
                # Fallback: old format
                m3 = re.match(r"\s+md=(PASS|FAIL).*\| ([\d.]+)s \| ~(\d+)B out \| ~(\d+) calls", line)
                if m3 and pending_task:
                    results.append({
                        "task": pending_task, "mode": mode, "model": model,
                        "pass": m3.group(1) == "PASS",
                        "time": float(m3.group(2)), "bytes": int(m3.group(3)),
                        "calls": int(m3.group(4)), "obs": 0, "mut": 0, "rq": False,
                    })
                    pending_task = None
            elif pending_task:
                results.append({
                    "task": pending_task, "mode": mode, "model": model,
                    "pass": m3.group(1) == "PASS",
                    "time": float(m3.group(2)), "bytes": int(m3.group(3)),
                    "obs": int(m3.group(4)), "calls": int(m3.group(5)),
                    "mut": int(m3.group(6)), "rq": m3.group(7) == "↻",
                })
                pending_task = None
    return results


def main():
    files = sys.argv[1:]
    if not files:
        print("Usage: python3 bench/analyze.py /tmp/bench_*.txt")
        return

    all_results = []
    for f in files:
        all_results.extend(parse_with_task_ids(f))

    if not all_results:
        print("No results found.")
        return

    # Group by model
    models = sorted(set(r["model"] for r in all_results))
    modes = ["unix", "mdtools", "hybrid"]
    tasks = sorted(set(r["task"] for r in all_results), key=lambda t: int(t[1:]))

    for model in models:
        print(f"\n{'='*70}")
        print(f"MODEL: {model}")
        print(f"{'='*70}")

        # Per-task comparison
        print(f"\n{'Task':<6}", end="")
        for mode in modes:
            print(f"  {'':>2}{mode:^20}", end="")
        print()
        print(f"{'':6}", end="")
        for _ in modes:
            print(f"  {'pass':>5} {'time':>6} {'calls':>5}", end="")
        print()
        print("-" * 72)

        mode_stats = {m: {"pass": 0, "total": 0, "time": 0, "calls": 0, "bytes": 0} for m in modes}

        for task in tasks:
            print(f"{task:<6}", end="")
            for mode in modes:
                matches = [r for r in all_results if r["task"] == task and r["mode"] == mode and r["model"] == model]
                if matches:
                    r = matches[0]
                    mark = "✓" if r["pass"] else "✗"
                    print(f"  {mark:>5} {r['time']:>5.0f}s {r['calls']:>5}", end="")
                    mode_stats[mode]["total"] += 1
                    if r["pass"]:
                        mode_stats[mode]["pass"] += 1
                    mode_stats[mode]["time"] += r["time"]
                    mode_stats[mode]["calls"] += r["calls"]
                    mode_stats[mode]["bytes"] += r["bytes"]
                else:
                    print(f"  {'—':>5} {'—':>6} {'—':>5}", end="")
            print()

        print("-" * 72)
        print(f"{'TOTAL':<6}", end="")
        for mode in modes:
            s = mode_stats[mode]
            if s["total"] > 0:
                pct = s["pass"] / s["total"] * 100
                avg_t = s["time"] / s["total"]
                avg_c = s["calls"] / s["total"]
                print(f"  {pct:>4.0f}% {avg_t:>5.0f}s {avg_c:>5.1f}", end="")
            else:
                print(f"  {'—':>5} {'—':>6} {'—':>5}", end="")
        print()

    # Cross-model comparison
    if len(models) > 1:
        print(f"\n{'='*70}")
        print("CROSS-MODEL SUMMARY")
        print(f"{'='*70}")
        print(f"\n{'Model':<30} {'Mode':<10} {'Pass%':>6} {'Time':>6} {'Calls':>6} {'ObsKB':>6} {'RQ%':>5}")
        print("-" * 72)
        for model in models:
            for mode in modes:
                matches = [r for r in all_results if r["mode"] == mode and r["model"] == model]
                if matches:
                    n = len(matches)
                    pct = sum(1 for r in matches if r["pass"]) / n * 100
                    avg_t = sum(r["time"] for r in matches) / n
                    avg_c = sum(r["calls"] for r in matches) / n
                    avg_obs = sum(r.get("obs", 0) for r in matches) / n / 1024
                    rq_pct = sum(1 for r in matches if r.get("rq")) / n * 100
                    print(f"{model:<30} {mode:<10} {pct:>5.0f}% {avg_t:>5.0f}s {avg_c:>5.1f} {avg_obs:>5.0f}K {rq_pct:>4.0f}%")


if __name__ == "__main__":
    main()
