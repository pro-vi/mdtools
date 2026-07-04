#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from collections import defaultdict
from pathlib import Path
from statistics import mean
from typing import Any


def _load_json(path: Path) -> Any:
    return json.loads(path.read_text())


def _bundle_rows(bundle: Path) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    run = _load_json(bundle / "run.json")
    rows = _load_json(bundle / "results.json")
    if not isinstance(rows, list):
        raise ValueError(f"{bundle}: results.json must contain a list")
    return run, rows


def _cost_proxy(row: dict[str, Any]) -> int:
    return int(row.get("bytes_prompt") or 0) + int(row.get("bytes_output") or 0) + int(row.get("bytes_observation") or 0)


def _md_adopted(row: dict[str, Any]) -> bool:
    tool_mix = row.get("tool_mix")
    if not isinstance(tool_mix, dict):
        return False
    return any(str(key).startswith("md ") for key in tool_mix)


def _row_model(run: dict[str, Any], row: dict[str, Any]) -> str:
    model = row.get("model") or run.get("model") or "unknown"
    return str(model)


def _summarize_rows(rows: list[dict[str, Any]]) -> dict[str, Any]:
    n = len(rows)
    passed = sum(1 for row in rows if row.get("correct") is True)
    costs = [_cost_proxy(row) for row in rows]
    elapsed = [float(row.get("elapsed_seconds") or 0.0) for row in rows]
    calls = [int(row.get("tool_calls") or 0) for row in rows]
    adopted = sum(1 for row in rows if _md_adopted(row))
    return {
        "n": n,
        "passed": passed,
        "pass_all": n > 0 and passed == n,
        "pass_rate": passed / n if n else 0.0,
        "avg_cost": mean(costs) if costs else 0.0,
        "avg_elapsed": mean(elapsed) if elapsed else 0.0,
        "avg_calls": mean(calls) if calls else 0.0,
        "md_adopted": adopted,
    }


def summarize_bundles(bundles: list[Path]) -> str:
    groups: dict[tuple[str, str, str], list[dict[str, Any]]] = defaultdict(list)
    model_mode_rows: dict[tuple[str, str], list[dict[str, Any]]] = defaultdict(list)

    for bundle in bundles:
        run, rows = _bundle_rows(bundle)
        for row in rows:
            model = _row_model(run, row)
            task_id = str(row.get("task_id"))
            mode = str(row.get("mode"))
            groups[(model, task_id, mode)].append(row)
            model_mode_rows[(model, mode)].append(row)

    lines = [
        "# Long-Document Regime Results",
        "",
        "| model | task | mode | n | pass | pass^n | avg byte proxy | avg calls | avg sec | md adopted |",
        "| --- | --- | --- | ---: | ---: | --- | ---: | ---: | ---: | ---: |",
    ]

    for key in sorted(groups):
        model, task_id, mode = key
        summary = _summarize_rows(groups[key])
        lines.append(
            "| "
            + " | ".join(
                [
                    model,
                    task_id,
                    mode,
                    str(summary["n"]),
                    str(summary["passed"]),
                    "yes" if summary["pass_all"] else "no",
                    f"{summary['avg_cost']:.0f}",
                    f"{summary['avg_calls']:.1f}",
                    f"{summary['avg_elapsed']:.1f}",
                    str(summary["md_adopted"]),
                ]
            )
            + " |"
        )

    lines.extend(["", "## Model-Level Verdicts", ""])
    models = sorted({model for model, _mode in model_mode_rows})
    any_candidate = False
    for model in models:
        native = _summarize_rows(model_mode_rows.get((model, "native"), []))
        md = _summarize_rows(model_mode_rows.get((model, "native+md"), []))
        no_md = _summarize_rows(model_mode_rows.get((model, "native+md-no-md"), []))
        correctness_lift = md["pass_rate"] > native["pass_rate"]
        cost_advantage = md["avg_cost"] < native["avg_cost"]
        candidate = correctness_lift or cost_advantage
        any_candidate = any_candidate or candidate
        verdict = "CANDIDATE" if candidate else "CLOSED"
        reasons: list[str] = []
        if correctness_lift:
            reasons.append(f"correctness {md['pass_rate']:.0%} vs native {native['pass_rate']:.0%}")
        if cost_advantage:
            reasons.append(f"byte proxy {md['avg_cost']:.0f} vs native {native['avg_cost']:.0f}")
        if not reasons:
            reasons.append("no correctness lift and no byte-cost proxy advantage")
        control = (
            f"no-md control pass={no_md['pass_rate']:.0%}, "
            f"byte proxy={no_md['avg_cost']:.0f}, md-attempt rows={no_md['md_adopted']}/{no_md['n']}"
        )
        lines.append(f"- `{model}`: {verdict} — {'; '.join(reasons)}; {control}.")

    lines.extend(["", "## Probe Verdict", ""])
    if any_candidate:
        lines.append("CANDIDATE: native+md won on correctness or byte-cost proxy for at least one model.")
    else:
        lines.append("CLOSED: native+md showed no correctness lift and no byte-cost proxy advantage on both models.")

    return "\n".join(lines) + "\n"


def main() -> None:
    parser = argparse.ArgumentParser(description="Summarize committed long-document probe bundles.")
    parser.add_argument("bundles", nargs="+", type=Path, help="bench/runs bundle directories")
    parser.add_argument("--output", type=Path, help="Write markdown readout to this path")
    args = parser.parse_args()

    readout = summarize_bundles(args.bundles)
    if args.output:
        args.output.write_text(readout)
    print(readout, end="")


if __name__ == "__main__":
    main()
