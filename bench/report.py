#!/usr/bin/env python3
"""Generate a benchmark report from N=3 harness JSON output files.

Usage:
    python3 bench/report.py /tmp/bench_p5_haiku_*.txt
    python3 bench/report.py /tmp/bench_p5_haiku_*.txt --markdown
"""

import json
import re
import sys
from collections import defaultdict

TASK_FAMILIES = {
    "Extraction":       ["T1", "T5", "T9", "T11", "T16", "T19"],
    "Targeted mutation": ["T7", "T10", "T13", "T20"],
    "Batch mutation":   ["T12"],
    "Multi-step":       ["T15", "T18"],
    "Content delivery": ["T2", "T3", "T8", "T17"],
    "Safe-fail":        ["T14"],
    "Text manipulation": ["T4", "T6"],
    "Metadata":         ["T21", "T22", "T24"],
    "Table projection": ["T23"],
}


def parse_json_results(filepath):
    """Extract JSON results array from harness output."""
    content = open(filepath).read()
    # Try parsing the whole file as JSON first
    try:
        data = json.loads(content)
        if isinstance(data, list):
            return data
    except json.JSONDecodeError:
        pass
    # Fallback: find the last valid JSON array by trying json.loads
    # on progressively larger substrings from each top-level [
    candidates = []
    depth = 0
    in_string = False
    escape = False
    start = -1
    for i, ch in enumerate(content):
        if escape:
            escape = False
            continue
        if ch == '\\' and in_string:
            escape = True
            continue
        if ch == '"' and not escape:
            in_string = not in_string
            continue
        if in_string:
            continue
        if ch == '[':
            if depth == 0:
                start = i
            depth += 1
        elif ch == ']':
            depth -= 1
            if depth == 0 and start >= 0:
                candidates.append((start, i + 1))
                start = -1
    # Try candidates from last to first
    for s, e in reversed(candidates):
        try:
            return json.loads(content[s:e])
        except json.JSONDecodeError:
            pass
    return []


def parse_text_results(filepath):
    """Fallback: parse from text output."""
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
            m2 = re.match(r"\s+(T\d+)(?:\s+run \d+/\d+)?:", line)
            if m2:
                pending_task = m2.group(1)
            m3 = re.match(
                r"\s+md=(PASS|FAIL).*\| ([\d.]+)s \| ~(\d+)B out \| obs:(\d+)B \| ~(\d+) calls \| (\d+) mut\s*(↻?)",
                line,
            )
            if m3 and pending_task:
                results.append({
                    "task_id": pending_task, "mode": mode,
                    "correct": m3.group(1) == "PASS",
                    "elapsed_seconds": float(m3.group(2)),
                    "bytes_output": int(m3.group(3)),
                    "bytes_observation": int(m3.group(4)),
                    "tool_calls": int(m3.group(5)),
                    "mutations": int(m3.group(6)),
                    "requeried": m3.group(7) == "↻",
                })
                pending_task = None
    return results


def main():
    files = [f for f in sys.argv[1:] if not f.startswith("--")]
    markdown = "--markdown" in sys.argv

    all_results = []
    for f in files:
        results = parse_json_results(f)
        if not results:
            results = parse_text_results(f)
        all_results.extend(results)

    if not all_results:
        print("No results found.")
        return

    modes = sorted(set(r.get("mode", "?") for r in all_results))
    tasks = sorted(set(r["task_id"] for r in all_results), key=lambda t: int(t[1:]))

    # Per-task aggregated results
    def agg(results_list):
        if not results_list:
            return None
        n = len(results_list)
        return {
            "n": n,
            "pass_rate": sum(1 for r in results_list if r["correct"]) / n,
            "avg_time": sum(r["elapsed_seconds"] for r in results_list) / n,
            "avg_calls": sum(r["tool_calls"] for r in results_list) / n,
            "avg_obs_kb": sum(r.get("bytes_observation", 0) for r in results_list) / n / 1024,
            "avg_mut": sum(r.get("mutations", 0) for r in results_list) / n,
            "rq_rate": sum(1 for r in results_list if r.get("requeried")) / n,
        }

    if markdown:
        # Markdown table output
        print("### Per-task results (N=3)\n")
        print(f"| Task |", end="")
        for mode in modes:
            print(f" {mode} pass | {mode} time | {mode} calls |", end="")
        print()
        print(f"|------|", end="")
        for _ in modes:
            print(f"---:|---:|---:|", end="")
        print()
    else:
        print(f"\n{'Task':<6}", end="")
        for mode in modes:
            print(f"  {mode:^18}", end="")
        print()
        print(f"{'':6}", end="")
        for _ in modes:
            print(f"  {'pass%':>5} {'time':>5} {'calls':>5}", end="")
        print()
        print("-" * (6 + 20 * len(modes)))

    for task in tasks:
        if markdown:
            print(f"| {task} |", end="")
        else:
            print(f"{task:<6}", end="")
        for mode in modes:
            task_results = [r for r in all_results if r["task_id"] == task and r.get("mode") == mode]
            a = agg(task_results)
            if a:
                pct = f"{a['pass_rate']:.0%}"
                if markdown:
                    print(f" {pct} | {a['avg_time']:.0f}s | {a['avg_calls']:.1f} |", end="")
                else:
                    print(f"  {pct:>5} {a['avg_time']:>4.0f}s {a['avg_calls']:>5.1f}", end="")
            else:
                if markdown:
                    print(f" — | — | — |", end="")
                else:
                    print(f"  {'—':>5} {'—':>5} {'—':>5}", end="")
        print()

    # Aggregate by mode
    print()
    if markdown:
        print("### Aggregate\n")
        print("| Mode | Pass% | Avg Time | Avg Calls | Avg Obs KB | Requery% |")
        print("|------|------:|--------:|---------:|----------:|--------:|")
    else:
        print(f"{'Mode':<10} {'Pass%':>6} {'Time':>6} {'Calls':>6} {'ObsKB':>7} {'RQ%':>5}")
        print("-" * 42)

    for mode in modes:
        mode_results = [r for r in all_results if r.get("mode") == mode]
        a = agg(mode_results)
        if a:
            if markdown:
                print(f"| {mode} | {a['pass_rate']:.0%} | {a['avg_time']:.0f}s | {a['avg_calls']:.1f} | {a['avg_obs_kb']:.0f}KB | {a['rq_rate']:.0%} |")
            else:
                print(f"{mode:<10} {a['pass_rate']:>5.0%} {a['avg_time']:>5.0f}s {a['avg_calls']:>5.1f} {a['avg_obs_kb']:>6.0f}K {a['rq_rate']:>4.0%}")

    # By task family — show per-mode sample count alongside pass rate
    print()
    if markdown:
        print("### By task family\n")
        print("| Family | Tasks |", end="")
        for mode in modes:
            print(f" {mode} |", end="")
        print()
        print("|--------|------:|", end="")
        for _ in modes:
            print(f"---:|", end="")
        print()
    else:
        print(f"{'Family':<20} {'Tasks':>5}", end="")
        for mode in modes:
            print(f"  {mode:>10}", end="")
        print()
        print("-" * (27 + 12 * len(modes)))

    for family, family_tasks in TASK_FAMILIES.items():
        present = [t for t in family_tasks if t in tasks]
        if not present:
            continue
        if markdown:
            print(f"| {family} | {len(present)} |", end="")
        else:
            print(f"{family:<20} {len(present):>5}", end="")
        for mode in modes:
            fam_results = [r for r in all_results if r["task_id"] in present and r.get("mode") == mode]
            a = agg(fam_results)
            if a:
                n = a["n"]
                if markdown:
                    print(f" {a['pass_rate']:.0%} (n={n}) |", end="")
                else:
                    print(f"  {a['pass_rate']:>4.0%} n={n:<3}", end="")
            else:
                if markdown:
                    print(f" — |", end="")
                else:
                    print(f"  {'—':>10}", end="")
        print()


if __name__ == "__main__":
    main()
