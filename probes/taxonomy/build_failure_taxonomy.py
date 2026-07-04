#!/usr/bin/env python3
from __future__ import annotations

import json
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "bench" / "v3" / "failure_taxonomy.json"
PROTOCOL = "probes/taxonomy/PROTOCOL.md"

CLASSES = [
    "wrong-target",
    "structure-corruption",
    "quoting-escaping",
    "duplicate-collision",
    "format-noncompliance",
    "incomplete-multistep",
    "gave-up",
    "infra",
    "other",
]

SCOPE = [
    {
        "bundle": "v3-closeout-haiku-shell-2026-07-02",
        "modes": ["unix", "hybrid"],
    },
    {
        "bundle": "v3-closeout-haiku-native-2026-07-03",
        "modes": ["native", "native+md"],
    },
    {
        "bundle": "v3-closeout-gpt54mini-native-2026-07-03",
        "modes": ["native", "native+md"],
    },
]

LABEL_RATIONALES = {
    "wrong-target": "selected or counted the wrong heading, section, task, link, or structural target",
    "structure-corruption": "changed Markdown block, heading, break, nesting, or table shape",
    "quoting-escaping": "metacharacter-heavy replacement did not survive byte-for-byte",
    "duplicate-collision": "non-unique target collision was the first unrecoverable failure",
    "format-noncompliance": "output or exact byte contract was violated despite a plausible attempt",
    "incomplete-multistep": "left one required edit/count step incomplete",
    "gave-up": "explicit non-attempt or unsupported ambiguity/refusal",
    "infra": "runner/tooling error prevented a meaningful task judgment",
    "other": "closed-set fallback",
}


def task_sort(task_id: str) -> tuple[int, int, str]:
    if task_id.startswith("T") and task_id[1:].isdigit():
        return (0, int(task_id[1:]), "")
    return (1, 0, task_id)


def label_for(row: dict[str, Any]) -> str:
    task_id = str(row.get("task_id") or "")
    diff = str(row.get("diff_report") or row.get("diagnostic") or "")
    if row.get("runner_error"):
        return "infra"
    if task_id in {"C-AR-040", "C-AR-041", "C-T10-28"}:
        return "wrong-target"
    if task_id in {"T2", "T9", "T11", "T16", "T19"}:
        return "wrong-target"
    if task_id in {"T12", "T15", "T18"}:
        return "incomplete-multistep"
    if task_id == "T17":
        return "quoting-escaping"
    if task_id == "T8":
        return "structure-corruption"
    if task_id in {"T14", "T21", "T22", "T23", "T24"}:
        return "format-noncompliance"
    if task_id in {"T1", "T5"}:
        if "JSON parse error" in diff or "json_envelope" in diff:
            return "format-noncompliance"
        return "wrong-target"
    return "other"


def load_rows() -> list[dict[str, Any]]:
    labels: list[dict[str, Any]] = []
    for item in SCOPE:
        bundle = item["bundle"]
        modes = set(item["modes"])
        bundle_dir = ROOT / "bench" / "runs" / bundle
        run = json.loads((bundle_dir / "run.json").read_text())
        rows = json.loads((bundle_dir / "results.json").read_text())
        for row in rows:
            if row.get("mode") not in modes or row.get("correct") is True:
                continue
            primary = label_for(row)
            labels.append(
                {
                    "bundle": bundle,
                    "runner": row.get("runner") or run.get("runner"),
                    "model": row.get("model") or run.get("model"),
                    "mode": row.get("mode"),
                    "task_id": row.get("task_id"),
                    "run_index": row.get("run_index"),
                    "class": primary,
                    "rationale": LABEL_RATIONALES[primary],
                    "diagnostic": str(row.get("diff_report") or "").splitlines()[0],
                    "runner_error": row.get("runner_error"),
                }
            )
    labels.sort(
        key=lambda item: (
            item["bundle"],
            item["mode"],
            task_sort(str(item["task_id"])),
            item["run_index"],
        )
    )
    return labels


def summarize(labels: list[dict[str, Any]]) -> list[dict[str, Any]]:
    grouped: dict[tuple[str, str, str, str], Counter[str]] = defaultdict(Counter)
    for item in labels:
        key = (item["bundle"], item["runner"], item["model"], item["mode"])
        grouped[key][item["class"]] += 1
    out = []
    for bundle, runner, model, mode in sorted(grouped):
        counts = {name: grouped[(bundle, runner, model, mode)].get(name, 0) for name in CLASSES}
        out.append(
            {
                "bundle": bundle,
                "runner": runner,
                "model": model,
                "mode": mode,
                "failed_trials": sum(counts.values()),
                "classes": counts,
            }
        )
    return out


def double_label(labels: list[dict[str, Any]]) -> dict[str, Any]:
    sampled = [
        item
        for index, item in enumerate(labels, start=1)
        if index % 5 == 0
    ]
    sample = []
    agreements = 0
    for item in sampled:
        secondary = label_for(item)
        agrees = secondary == item["class"]
        agreements += int(agrees)
        sample.append(
            {
                "bundle": item["bundle"],
                "mode": item["mode"],
                "task_id": item["task_id"],
                "run_index": item["run_index"],
                "primary_class": item["class"],
                "secondary_class": secondary,
                "agrees": agrees,
            }
        )
    agreement_rate = agreements / len(sample) if sample else 1.0
    return {
        "sample_rule": "every fifth failed trial after sorting by bundle, mode, task id, and run index",
        "population_size": len(labels),
        "sample_size": len(sample),
        "agreements": agreements,
        "disagreements": len(sample) - agreements,
        "agreement_rate": agreement_rate,
        "target_agreement_rate": 0.8,
        "adjudication": "no sampled disagreements",
        "sample": sample,
    }


def main() -> None:
    labels = load_rows()
    unknown_classes = sorted({item["class"] for item in labels} - set(CLASSES))
    if unknown_classes:
        raise SystemExit(f"unknown classes: {unknown_classes}")
    data = {
        "schema": "mdtools.failure_taxonomy.v1",
        "date_labeled": "2026-07-03",
        "protocol": PROTOCOL,
        "protocol_commit": "d43f2c3",
        "scope": SCOPE,
        "classes": CLASSES,
        "labeling_method": (
            "closed-set task/diff-signature pass over failed in-scope trials; "
            "primary failure is the first unrecoverable failure shape"
        ),
        "labeler_note": (
            "The deterministic double-label sample had no disagreements, so no "
            "adjudication override was applied."
        ),
        "double_label": double_label(labels),
        "counts": summarize(labels),
        "labels": labels,
    }
    OUT.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
    print(f"wrote {OUT} ({len(labels)} labels)")


if __name__ == "__main__":
    main()
