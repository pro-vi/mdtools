from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable


MANIFEST_PATH = Path("bench/v3/manifest.json")
TASKS_PATH = Path("bench/tasks/tasks.json")
SCORER_VERSION = "v3-neutral-primary"


class ManifestConformanceError(ValueError):
    pass


@dataclass(frozen=True)
class BundleConformance:
    status: str
    reasons: tuple[str, ...] = ()

    @property
    def headline_eligible(self) -> bool:
        return self.status == "headline-eligible"


def sha256_file(path: str | Path) -> str:
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def load_manifest(path: str | Path = MANIFEST_PATH) -> dict[str, Any]:
    return json.loads(Path(path).read_text())


def manifest_hash(path: str | Path = MANIFEST_PATH) -> str:
    return sha256_file(path)


def current_prompt_template_sha256() -> str:
    from bench.harness import SYSTEM_PROMPT_TEMPLATE

    return sha256_text(SYSTEM_PROMPT_TEMPLATE)


def validate_manifest_current(
    manifest: dict[str, Any],
    *,
    tasks_path: str | Path = TASKS_PATH,
) -> None:
    task_hash = sha256_file(tasks_path)
    if manifest.get("task_file_sha256") != task_hash:
        raise ManifestConformanceError(
            "manifest task_file_sha256 does not match current tasks.json: "
            f"{manifest.get('task_file_sha256')} != {task_hash}"
        )
    prompt_hash = current_prompt_template_sha256()
    if manifest.get("prompt_template_sha256") != prompt_hash:
        raise ManifestConformanceError(
            "manifest prompt_template_sha256 does not match current harness prompt template: "
            f"{manifest.get('prompt_template_sha256')} != {prompt_hash}"
        )


def comparison_modes(comparison: dict[str, Any]) -> tuple[str, str]:
    left, right = [part.strip() for part in comparison["comparison"].split("->", 1)]
    return left, right


def required_modes_for_comparison(comparison: dict[str, Any]) -> set[str]:
    left, right = comparison_modes(comparison)
    modes = {left, right}
    ablation = comparison.get("ablation")
    if ablation:
        modes.add(ablation)
    return modes


def matching_primary_comparisons(
    run: dict[str, Any],
    manifest: dict[str, Any],
) -> list[dict[str, Any]]:
    run_modes = set(run.get("modes") or [])
    return [
        comparison
        for comparison in manifest.get("primary_comparisons", [])
        if (
            run.get("runner") == comparison.get("runner")
            and run.get("model") == comparison.get("model")
            and required_modes_for_comparison(comparison).issubset(run_modes)
        )
    ]


def bundle_conformance(run: dict[str, Any], manifest: dict[str, Any]) -> BundleConformance:
    reasons: list[str] = []
    expected_k = int(manifest.get("primary_metric", {}).get("trials_per_task", 5))
    observed_k = run.get("trials_per_cell", run.get("runs_per_task"))
    if observed_k != expected_k:
        reasons.append(f"wrong N: expected {expected_k}, got {observed_k}")
    for field in ("task_file_sha256", "prompt_template_sha256", "scorer_version"):
        expected = manifest.get(field)
        observed = run.get(field)
        if expected and observed != expected:
            reasons.append(f"{field} mismatch")
    if not matching_primary_comparisons(run, manifest):
        reasons.append("not a preregistered primary comparison")
    if reasons:
        return BundleConformance(status="exploratory", reasons=tuple(reasons))
    return BundleConformance(status="headline-eligible")


def headline_completeness_reasons(
    run: dict[str, Any],
    rows: Iterable[dict[str, Any]],
    manifest: dict[str, Any],
    *,
    core_task_ids: Iterable[str],
) -> tuple[str, ...]:
    """Validate required core task × mode × run-index rows for headline bundles."""
    comparisons = matching_primary_comparisons(run, manifest)
    if not comparisons:
        return ()

    expected_k = int(manifest.get("primary_metric", {}).get("trials_per_task", 5))
    expected_indices = set(range(expected_k))
    core_ids = set(core_task_ids)
    required_modes = set().union(
        *(required_modes_for_comparison(comparison) for comparison in comparisons)
    )
    expected = {
        (task_id, mode, run_index)
        for task_id in core_ids
        for mode in required_modes
        for run_index in expected_indices
    }
    seen: set[tuple[str, str, int]] = set()
    duplicates: list[tuple[str, str, int]] = []
    invalid_indices: list[str] = []

    for index, row in enumerate(rows):
        task_id = row.get("task_id")
        mode = row.get("mode")
        if task_id not in core_ids or mode not in required_modes:
            continue
        row_model = row.get("model") or run.get("model")
        row_runner = row.get("runner") or run.get("runner")
        if row_model != run.get("model") or row_runner != run.get("runner"):
            continue
        run_index = row.get("run_index")
        if type(run_index) is not int or run_index not in expected_indices:
            invalid_indices.append(f"row {index}: {task_id}:{mode}:run{run_index!r}")
            continue
        key = (task_id, mode, run_index)
        if key in seen:
            duplicates.append(key)
            continue
        seen.add(key)

    def sample(cells: Iterable[tuple[str, str, int]]) -> str:
        ordered = sorted(cells, key=lambda item: (item[0], item[1], item[2]))
        return ", ".join(f"{task}:{mode}:run{run_index}" for task, mode, run_index in ordered[:5])

    reasons: list[str] = []
    missing = expected - seen
    if missing:
        reasons.append(
            "incomplete results: "
            f"missing {len(missing)} required core trials"
            + (f" (e.g. {sample(missing)})" if missing else "")
        )
    if duplicates:
        reasons.append(
            "incomplete results: "
            f"{len(duplicates)} duplicate required core trials"
            + (f" (e.g. {sample(duplicates)})" if duplicates else "")
        )
    if invalid_indices:
        reasons.append(
            "incomplete results: "
            f"{len(invalid_indices)} required core rows have invalid run_index"
            f" (e.g. {', '.join(invalid_indices[:5])})"
        )
    return tuple(reasons)


def evaluate_success_threshold(
    *,
    lift_ci_low_pp: float,
    exact_p: float,
    favorable_quarantines: int = 0,
    manifest: dict[str, Any] | None = None,
) -> str:
    manifest = manifest or load_manifest()
    threshold = manifest.get("success_threshold", {})
    min_lift = float(threshold.get("ci_lower_bound_lift_pp", 15.0))
    max_p = float(threshold.get("exact_p_lt", 0.05))
    max_quarantines = int(threshold.get("manual_review_if_md_favorable_quarantines_gt", 2))
    if (
        lift_ci_low_pp > min_lift
        and exact_p < max_p
        and favorable_quarantines <= max_quarantines
    ):
        return "confirmed"
    return "downgrade"


def validate_headline_run_request(
    *,
    manifest: dict[str, Any],
    runs_per_task: int,
    tasks_path: str | Path,
) -> None:
    validate_manifest_current(manifest, tasks_path=tasks_path)
    expected_k = int(manifest.get("primary_metric", {}).get("trials_per_task", 5))
    if runs_per_task != expected_k:
        raise ManifestConformanceError(
            f"--headline requires -N {expected_k}; got -N {runs_per_task}"
        )
