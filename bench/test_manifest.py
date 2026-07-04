from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from bench.v3_manifest import (
    ManifestConformanceError,
    bundle_conformance,
    evaluate_success_threshold,
    headline_completeness_reasons,
    load_manifest,
    sha256_file,
    validate_headline_run_request,
    validate_manifest_current,
)


class BenchV3ManifestTests(unittest.TestCase):
    def test_manifest_hashes_match_current_repo(self) -> None:
        validate_manifest_current(load_manifest())

    def test_edited_tasks_file_fails_hash_check(self) -> None:
        manifest = load_manifest()
        with tempfile.TemporaryDirectory(prefix="bench_manifest_tasks_") as tmpdir:
            tasks = Path(tmpdir) / "tasks.json"
            tasks.write_text(Path("bench/tasks/tasks.json").read_text() + "\n")
            with self.assertRaisesRegex(ManifestConformanceError, "task_file_sha256"):
                validate_manifest_current(manifest, tasks_path=tasks)

    def test_bundle_conformance_routes_headline_and_wrong_n(self) -> None:
        manifest = load_manifest()
        run = {
            "trials_per_cell": 5,
            "runner": "claude-cli",
            "model": "claude-haiku-4-5-20251001",
            "modes": ["unix", "hybrid", "hybrid-no-md"],
            "task_file_sha256": manifest["task_file_sha256"],
            "prompt_template_sha256": manifest["prompt_template_sha256"],
            "scorer_version": manifest["scorer_version"],
        }
        self.assertTrue(bundle_conformance(run, manifest).headline_eligible)
        wrong_n = dict(run, trials_per_cell=1)
        exploratory = bundle_conformance(wrong_n, manifest)
        self.assertEqual(exploratory.status, "exploratory")
        self.assertIn("wrong N", exploratory.reasons[0])
        local = dict(run, runner="oai-loop", model="Qwen3.6-35B-A3B-8bit")
        self.assertEqual(bundle_conformance(local, manifest).status, "exploratory")

    def test_headline_completeness_detects_missing_duplicate_and_invalid_rows(self) -> None:
        manifest = {
            "primary_comparisons": [
                {
                    "runner": "runner",
                    "model": "model",
                    "comparison": "left -> right",
                    "ablation": "right-no-md",
                }
            ],
            "primary_metric": {"trials_per_task": 2},
        }
        run = {
            "runner": "runner",
            "model": "model",
            "modes": ["left", "right", "right-no-md"],
        }

        def row(mode: str, run_index: int) -> dict:
            return {
                "task_id": "T1",
                "mode": mode,
                "run_index": run_index,
                "runner": "runner",
                "model": "model",
            }

        rows = [
            row("left", 0),
            row("left", 0),
            row("left", 5),
            row("right", 0),
            row("right", 1),
            row("right-no-md", 0),
        ]

        reasons = headline_completeness_reasons(
            run,
            rows,
            manifest,
            core_task_ids=["T1"],
        )

        self.assertTrue(any("missing" in reason and "required core trials" in reason for reason in reasons))
        self.assertTrue(any("duplicate required core trials" in reason for reason in reasons))
        self.assertTrue(any("invalid run_index" in reason for reason in reasons))

    def test_success_threshold_confirm_and_downgrade(self) -> None:
        manifest = load_manifest()
        self.assertEqual(
            evaluate_success_threshold(
                lift_ci_low_pp=16.0,
                exact_p=0.01,
                favorable_quarantines=0,
                manifest=manifest,
            ),
            "confirmed",
        )
        self.assertEqual(
            evaluate_success_threshold(
                lift_ci_low_pp=14.0,
                exact_p=0.01,
                favorable_quarantines=0,
                manifest=manifest,
            ),
            "downgrade",
        )

    def test_headline_run_request_validates_n_and_hashes(self) -> None:
        manifest = load_manifest()
        validate_headline_run_request(
            manifest=manifest,
            runs_per_task=5,
            tasks_path="bench/tasks/tasks.json",
        )
        with self.assertRaisesRegex(ManifestConformanceError, "-N 5"):
            validate_headline_run_request(
                manifest=manifest,
                runs_per_task=1,
                tasks_path="bench/tasks/tasks.json",
            )

    def test_manifest_task_hash_is_literal_sha256(self) -> None:
        manifest = load_manifest()
        self.assertEqual(manifest["task_file_sha256"], sha256_file("bench/tasks/tasks.json"))


if __name__ == "__main__":
    unittest.main()
