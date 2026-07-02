from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any


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
    if reasons:
        return BundleConformance(status="exploratory", reasons=tuple(reasons))
    return BundleConformance(status="headline-eligible")


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
