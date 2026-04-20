#!/usr/bin/env python3
"""Analyze benchmark results from harness output files or persisted run bundles."""

import json
import re
import sys
from collections import defaultdict
from pathlib import Path


def parse_json_results(filepath):
    """Load a persisted results.json array when present."""
    try:
        data = json.loads(Path(filepath).read_text())
    except (OSError, json.JSONDecodeError):
        return []
    return data if isinstance(data, list) else []


def parse_run_metadata(filepath):
    """Load a persisted run.json metadata object when present."""
    try:
        data = json.loads(Path(filepath).read_text())
    except (OSError, json.JSONDecodeError):
        return None
    return data if isinstance(data, dict) else None


def normalize_result(result, default_model):
    """Normalize persisted or text-parsed results into the analysis shape."""
    if not isinstance(result, dict):
        return None

    task_id = result.get("task_id") or result.get("task")
    mode = result.get("mode")
    if not task_id or not mode:
        return None

    if "correct" in result:
        passed = bool(result["correct"])
    elif "pass" in result:
        passed = bool(result["pass"])
    else:
        return None

    return {
        "task": task_id,
        "mode": mode,
        "model": result.get("model") or default_model,
        "pass": passed,
        "time": float(result.get("elapsed_seconds", result.get("time", 0.0))),
        "bytes": int(result.get("bytes_output", result.get("bytes", 0))),
        "calls": int(result.get("tool_calls", result.get("calls", 0))),
        "obs": int(result.get("bytes_observation", result.get("obs", 0))),
        "mut": int(result.get("mutations", result.get("mut", 0))),
        "deny": int(result.get("policy_violations", result.get("deny", 0))),
        "rq": bool(result.get("requeried", result.get("rq", False))),
        "runner_error": result.get("runner_error"),
    }


def load_analysis_input(path):
    """Load normalized results plus optional run metadata from a file or run bundle."""
    input_path = Path(path)
    metadata = None

    if input_path.is_dir():
        results_path = input_path / "results.json"
        if not results_path.exists():
            raise FileNotFoundError(f"{input_path} does not contain results.json")
        raw_results = parse_json_results(results_path)
        metadata = parse_run_metadata(input_path / "run.json")
        source = str(input_path)
    else:
        raw_results = parse_json_results(input_path)
        if not raw_results:
            raw_results = parse_with_task_ids(input_path)
        if input_path.name == "results.json":
            metadata = parse_run_metadata(input_path.with_name("run.json"))
            source = str(input_path.parent)
        else:
            source = str(input_path)

    default_model = "opus"
    if metadata:
        default_model = metadata.get("model") or "unspecified"

    results = []
    for result in raw_results:
        normalized = normalize_result(result, default_model)
        if normalized:
            results.append(normalized)

    if metadata:
        metadata = dict(metadata)
        metadata["_source"] = source
    return results, metadata


def format_run_context(metadata):
    selected = metadata.get("selected_task_ids") or []
    modes = metadata.get("modes") or []
    selection = metadata.get("task_ids_path") or "inline selection"

    parts = [
        str(metadata.get("kind", "unknown")),
        f"{len(selected)} tasks",
        f"selection={selection}",
        f"modes={','.join(modes) if modes else '?'}",
        f"corpus={metadata.get('tasks_path', '?')}",
    ]

    if metadata.get("runner"):
        parts.append(f"runner={metadata['runner']}")
    if metadata.get("executor"):
        parts.append(f"executor={metadata['executor']}")
    if metadata.get("model"):
        parts.append(f"model={metadata['model']}")

    return f"{metadata.get('_source', '?')}: " + ", ".join(parts)


def print_run_context(metadata_list):
    if not metadata_list:
        return

    print("Run context:")
    for metadata in metadata_list:
        print(f"  - {format_run_context(metadata)}")
    print()


def collect_runner_errors(results):
    grouped = defaultdict(lambda: {"count": 0, "tasks": set()})
    for result in results:
        error = result.get("runner_error")
        task = result.get("task")
        mode = result.get("mode")
        if not error or not task or not mode:
            continue
        entry = grouped[(mode, error)]
        entry["count"] += 1
        entry["tasks"].add(task)

    rows = []
    for (mode, error), entry in sorted(grouped.items()):
        rows.append(
            {
                "mode": mode,
                "count": entry["count"],
                "tasks": sorted(entry["tasks"], key=lambda task_id: int(task_id[1:])),
                "error": error,
            }
        )
    return rows

def parse_results(filepath):
    """Parse harness output into structured results."""
    results = []
    mode = None
    model = None
    in_dry_run = False
    with open(filepath) as f:
        for line in f:
            if line.startswith("=== DRY RUN:"):
                mode = "mdtools"
                model = "unspecified"
                in_dry_run = True
                continue
            # === MODE: hybrid (N=1, model=claude-haiku-4-5-20251001) ===
            m = re.match(r"=== MODE: (\w+) \(N=(\d+)(?:, model=([^)]+))?\)", line)
            if m:
                mode = m.group(1)
                model = m.group(3) or "unspecified"
                in_dry_run = False
                continue
            if in_dry_run:
                m = re.match(r"\s+(T\d+): md=(PASS|FAIL) neutral=(PASS|FAIL)(?: .*)?$", line)
                if m:
                    results.append({
                        "task": m.group(1),
                        "mode": mode,
                        "model": model,
                        "pass": m.group(2) == "PASS" and m.group(3) == "PASS",
                        "time": 0.0,
                        "bytes": 0,
                        "calls": 0,
                        "obs": 0,
                        "mut": 0,
                        "deny": 0,
                        "rq": False,
                    })
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
    in_dry_run = False
    with open(filepath) as f:
        for line in f:
            if line.startswith("=== DRY RUN:"):
                mode = "mdtools"
                model = "unspecified"
                pending_task = None
                in_dry_run = True
                continue
            m = re.match(r"=== MODE: (\w+) \(N=(\d+)(?:, model=([^)]+))?\)", line)
            if m:
                mode = m.group(1)
                model = m.group(3) or "unspecified"
                pending_task = None
                in_dry_run = False
                continue
            if in_dry_run:
                m = re.match(r"\s+(T\d+): md=(PASS|FAIL) neutral=(PASS|FAIL)(?: .*)?$", line)
                if m:
                    results.append({
                        "task": m.group(1),
                        "mode": mode,
                        "model": model,
                        "pass": m.group(2) == "PASS" and m.group(3) == "PASS",
                        "time": 0.0,
                        "bytes": 0,
                        "calls": 0,
                        "obs": 0,
                        "mut": 0,
                        "deny": 0,
                        "rq": False,
                    })
                continue
            # Task line: "  T1: ..." or "  T1 run 1/3: ..."
            m2 = re.match(r"\s+(T\d+)(?:\s+run \d+/\d+)?: ", line)
            if m2:
                pending_task = m2.group(1)
                continue
            # Result line — new format with obs and mut
            m3 = re.match(
                r"\s+md=(PASS|FAIL).*\| ([\d.]+)s \| ~(\d+)B out \| obs:(\d+)B \| ~(\d+) calls \| (\d+) mut \| deny:(\d+)\s*(↻?)(?: \| err:(.*))?$",
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
                    "mut": int(m3.group(6)), "deny": int(m3.group(7)),
                    "rq": m3.group(8) == "↻",
                    "runner_error": m3.group(9).strip() if m3.group(9) else None,
                })
                pending_task = None
    return results


def main():
    files = sys.argv[1:]
    if not files:
        print("Usage: python3 bench/analyze.py /tmp/bench_*.txt | bench/runs/<bundle>")
        return

    all_results = []
    run_metadata = []
    for f in files:
        results, metadata = load_analysis_input(f)
        all_results.extend(results)
        if metadata:
            run_metadata.append(metadata)

    if not all_results:
        print("No results found.")
        return

    print_run_context(run_metadata)

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
                    n = len(matches)
                    pass_rate = sum(1 for r in matches if r["pass"]) / n
                    avg_time = sum(r["time"] for r in matches) / n
                    avg_calls = sum(r["calls"] for r in matches) / n
                    pct = f"{pass_rate:.0%}"
                    print(f"  {pct:>5} {avg_time:>5.0f}s {avg_calls:>5.1f}", end="")
                    for r in matches:
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

        runner_errors = collect_runner_errors([r for r in all_results if r["model"] == model])
        if runner_errors:
            print("\nRunner errors:")
            for row in runner_errors:
                tasks_label = ", ".join(row["tasks"])
                print(f"  - {row['mode']} x{row['count']} [{tasks_label}]: {row['error']}")

    # Cross-model comparison
    if len(models) > 1:
        print(f"\n{'='*70}")
        print("CROSS-MODEL SUMMARY")
        print(f"{'='*70}")
        print(f"\n{'Model':<30} {'Mode':<10} {'Pass%':>6} {'Time':>6} {'Calls':>6} {'ObsKB':>6} {'Deny':>6} {'RQ%':>5}")
        print("-" * 80)
        for model in models:
            for mode in modes:
                matches = [r for r in all_results if r["mode"] == mode and r["model"] == model]
                if matches:
                    n = len(matches)
                    pct = sum(1 for r in matches if r["pass"]) / n * 100
                    avg_t = sum(r["time"] for r in matches) / n
                    avg_c = sum(r["calls"] for r in matches) / n
                    avg_obs = sum(r.get("obs", 0) for r in matches) / n / 1024
                    avg_deny = sum(r.get("deny", 0) for r in matches) / n
                    rq_pct = sum(1 for r in matches if r.get("rq")) / n * 100
                    print(f"{model:<30} {mode:<10} {pct:>5.0f}% {avg_t:>5.0f}s {avg_c:>5.1f} {avg_obs:>5.0f}K {avg_deny:>5.1f} {rq_pct:>4.0f}%")


if __name__ == "__main__":
    main()
