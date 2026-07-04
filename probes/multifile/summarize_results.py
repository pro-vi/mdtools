#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from bench.multifile_drift import load_drift_specs, summarize_drift_proof  # noqa: E402
from bench.pi_audit_adapter import load_pi_audit_events  # noqa: E402


MATCHED_RE = re.compile(r"multi_file_contents_any: matched ([^\s]+)")
MD_COMMAND_RE = re.compile(r"(^|[;&|()\s./])md(\s|$)")
DRIFT_SPECS = load_drift_specs(REPO_ROOT / "probes/multifile/drift_specs.json")
AUDIT_SUMMARY_SCHEMA = "multifile-drift-audit-summary.v1"


def _load_json(path: Path) -> Any:
    return json.loads(path.read_text())


def _bundle_rows(bundle: Path) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    run = _load_json(bundle / "run.json")
    rows = _load_json(bundle / "results.json")
    if not isinstance(rows, list):
        raise ValueError(f"{bundle}: results.json must contain a list")
    return run, rows


def _matched_alternative(row: dict[str, Any]) -> str:
    diff_report = str(row.get("diff_report") or "")
    match = MATCHED_RE.search(diff_report)
    return match.group(1) if match else "none"


def _proof_valid_from_diff(row: dict[str, Any]) -> bool:
    diff_report = str(row.get("diff_report") or "")
    return "multifile_drift_proof: OK" in diff_report


def _log_dirs(bundle: Path, task_id: str, mode: str) -> list[Path]:
    logs_root = bundle / "logs"
    if not logs_root.exists():
        return []
    prefix = f"{task_id}_{mode}_"
    return sorted(path for path in logs_root.iterdir() if path.is_dir() and path.name.startswith(prefix))


def _md_before_target_mutation(events: list[dict[str, Any]]) -> bool:
    for event in events:
        if event.get("event") == "multifile_drift_target_mutation":
            return False
        if event.get("event") != "tool_call" or event.get("toolName") != "bash":
            continue
        input_obj = event.get("input")
        command = input_obj.get("command") if isinstance(input_obj, dict) else None
        if isinstance(command, str) and MD_COMMAND_RE.search(command):
            return True
    return False


def _task_key(task_id: str) -> str:
    return "mf01" if task_id.endswith("01") else "mf02" if task_id.endswith("02") else task_id.lower()


def _clobbered(log_dir: Path, task_id: str) -> bool | None:
    spec = DRIFT_SPECS[_task_key(task_id)]
    target = log_dir / "final" / spec.target
    if not target.exists():
        return None
    return spec.new not in target.read_text()


def _audit_stats(log_dirs: list[Path], task_id: str) -> dict[str, int]:
    stats = Counter()
    stats["log_dirs"] = len(log_dirs)
    for log_dir in log_dirs:
        audit_path = log_dir / "pi-audit.jsonl"
        if not audit_path.exists():
            stats["missing_audit"] += 1
            continue
        events = load_pi_audit_events(audit_path)
        proof = summarize_drift_proof(events)
        if proof.valid:
            stats["proof_ok"] += 1
        if _md_before_target_mutation(events):
            stats["md_before_mutation"] += 1
        clobbered = _clobbered(log_dir, task_id)
        if clobbered is True:
            stats["clobber"] += 1
        elif clobbered is None:
            stats["missing_final"] += 1
    return dict(stats)


def _audit_summary_key(task_id: str, mode: str) -> str:
    return f"{task_id}|{mode}"


def _saved_audit_summary(bundle: Path) -> dict[str, dict[str, int]]:
    path = bundle / "audit_summary.json"
    if not path.exists():
        return {}
    payload = _load_json(path)
    if not isinstance(payload, dict) or payload.get("schema") != AUDIT_SUMMARY_SCHEMA:
        raise ValueError(f"{path}: unsupported audit summary schema")
    groups = payload.get("groups")
    if not isinstance(groups, dict):
        raise ValueError(f"{path}: missing groups object")
    out: dict[str, dict[str, int]] = {}
    for key, value in groups.items():
        if not isinstance(key, str) or not isinstance(value, dict):
            continue
        out[key] = {str(metric): int(count) for metric, count in value.items()}
    return out


def _audit_stats_for_group(bundle: Path, task_id: str, mode: str) -> dict[str, int]:
    saved = _saved_audit_summary(bundle).get(_audit_summary_key(task_id, mode))
    if saved is not None:
        return saved
    return _audit_stats(_log_dirs(bundle, task_id, mode), task_id)


def write_audit_summaries(bundles: list[Path]) -> None:
    for bundle in bundles:
        _run, rows = _bundle_rows(bundle)
        groups: dict[str, dict[str, int]] = {}
        for row in rows:
            task_id = str(row.get("task_id"))
            mode = str(row.get("mode"))
            key = _audit_summary_key(task_id, mode)
            if key not in groups:
                groups[key] = _audit_stats(_log_dirs(bundle, task_id, mode), task_id)
        payload = {"schema": AUDIT_SUMMARY_SCHEMA, "groups": groups}
        (bundle / "audit_summary.json").write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")


def summarize_bundles(bundles: list[Path]) -> str:
    groups: dict[tuple[str, str, str], list[dict[str, Any]]] = defaultdict(list)
    bundle_by_group: dict[tuple[str, str, str], Path] = {}

    for bundle in bundles:
        run, rows = _bundle_rows(bundle)
        model = str(run.get("model") or rows[0].get("model") if rows else "unknown")
        for row in rows:
            task_id = str(row.get("task_id"))
            mode = str(row.get("mode"))
            key = (model, task_id, mode)
            groups[key].append(row)
            bundle_by_group[key] = bundle

    lines = [
        "# Multifile Drift Probe Results",
        "",
        "| model | task | mode | n | pass | pass^n | alt counts | proof OK | md before mutation | clobbers | notes |",
        "| --- | --- | --- | ---: | ---: | --- | --- | ---: | ---: | ---: | --- |",
    ]

    model_mode_pass_all: dict[tuple[str, str], list[bool]] = defaultdict(list)
    for key in sorted(groups):
        model, task_id, mode = key
        rows = sorted(groups[key], key=lambda row: int(row.get("run_index") or 0))
        n = len(rows)
        passed = sum(1 for row in rows if row.get("correct") is True)
        pass_all = passed == n and n > 0
        model_mode_pass_all[(model, mode)].append(pass_all)

        alt_counts = Counter(_matched_alternative(row) for row in rows)
        diff_proofs = sum(1 for row in rows if _proof_valid_from_diff(row))
        audit_stats = _audit_stats_for_group(bundle_by_group[key], task_id, mode)
        proof_ok = max(diff_proofs, audit_stats.get("proof_ok", 0))
        notes: list[str] = []
        if audit_stats.get("log_dirs", 0) != n:
            notes.append(f"log_dirs={audit_stats.get('log_dirs', 0)}")
        if audit_stats.get("missing_audit"):
            notes.append(f"missing_audit={audit_stats['missing_audit']}")
        if audit_stats.get("missing_final"):
            notes.append(f"missing_final={audit_stats['missing_final']}")

        alt_text = ", ".join(f"{name}:{count}" for name, count in sorted(alt_counts.items()))
        lines.append(
            "| "
            + " | ".join(
                [
                    model,
                    task_id,
                    mode,
                    str(n),
                    str(passed),
                    "yes" if pass_all else "no",
                    alt_text,
                    str(proof_ok),
                    str(audit_stats.get("md_before_mutation", 0)),
                    str(audit_stats.get("clobber", 0)),
                    "; ".join(notes),
                ]
            )
            + " |"
        )

    lines.extend(["", "## Kill-Condition Check", ""])
    models = sorted({model for model, _mode in model_mode_pass_all})
    for model in models:
        native = model_mode_pass_all.get((model, "native"), [])
        md = model_mode_pass_all.get((model, "native+md"), [])
        if not native or not md:
            verdict = "INCOMPLETE: missing native or native+md rows."
        elif md == native and all(md):
            verdict = "CLOSED: native handled drift at pass^n parity with native+md."
        elif md == native:
            verdict = "CLOSED: neither arm reached pass^n reliability; no native+md edge was demonstrated."
        elif all(md) and not all(native):
            verdict = "CANDIDATE: native+md reached pass^n where native did not."
        else:
            verdict = "CLOSED: native+md did not reach pass^n reliability."
        lines.append(f"- `{model}`: {verdict}")

    return "\n".join(lines) + "\n"


def main() -> None:
    parser = argparse.ArgumentParser(description="Summarize committed multifile drift probe bundles.")
    parser.add_argument("bundles", nargs="+", type=Path, help="bench/runs bundle directories")
    parser.add_argument("--output", type=Path, help="Write markdown readout to this path")
    parser.add_argument(
        "--write-audit-summary",
        action="store_true",
        help="Persist compact per-bundle audit summaries derived from ignored run logs",
    )
    args = parser.parse_args()

    if args.write_audit_summary:
        write_audit_summaries(args.bundles)

    readout = summarize_bundles(args.bundles)
    if args.output:
        args.output.write_text(readout)
    print(readout, end="")


if __name__ == "__main__":
    main()
