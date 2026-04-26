from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from bench.harness import (
    compute_task_fingerprint,
    load_holdout_fingerprints,
    load_task_ids,
    load_tasks,
    select_tasks,
    verify_holdout_fingerprints,
)


class HarnessTaskSplitTests(unittest.TestCase):
    def test_load_task_ids_rejects_duplicates(self) -> None:
        with tempfile.TemporaryDirectory(prefix="bench_split_test_") as tmpdir:
            path = Path(tmpdir) / "dup.json"
            path.write_text(json.dumps(["T1", "T1"]))
            with self.assertRaisesRegex(ValueError, "duplicate task ID"):
                load_task_ids(str(path))

    def test_select_tasks_preserves_requested_order(self) -> None:
        tasks = load_tasks("bench/tasks/tasks.json")
        selected = select_tasks(tasks, ["T3", "T1", "T2"])
        self.assertEqual([task.id for task in selected], ["T3", "T1", "T2"])

    def test_search_and_holdout_partition_default_corpus(self) -> None:
        all_task_ids = {task.id for task in load_tasks("bench/tasks/tasks.json")}
        search_ids = set(load_task_ids("bench/search/task_ids.json"))
        holdout_ids = set(load_task_ids("bench/holdout/task_ids.json"))

        self.assertTrue(search_ids)
        self.assertTrue(holdout_ids)
        self.assertEqual(search_ids & holdout_ids, set())
        self.assertEqual(search_ids | holdout_ids, all_task_ids)

    def test_extraction_pilot_manifest_is_search_subset(self) -> None:
        self._assert_pilot_manifest("bench/search/extraction_pilot.json")

    def test_mutation_pilot_manifest_is_search_subset(self) -> None:
        self._assert_pilot_manifest("bench/search/mutation_pilot.json")

    def test_multistep_pilot_manifest_is_search_subset(self) -> None:
        self._assert_pilot_manifest("bench/search/multistep_pilot.json")

    def test_pilot_manifests_are_disjoint(self) -> None:
        manifests = {
            "extraction": set(load_task_ids("bench/search/extraction_pilot.json")),
            "mutation": set(load_task_ids("bench/search/mutation_pilot.json")),
            "multistep": set(load_task_ids("bench/search/multistep_pilot.json")),
        }
        names = sorted(manifests)
        for i, a in enumerate(names):
            for b in names[i + 1 :]:
                overlap = manifests[a] & manifests[b]
                self.assertEqual(
                    overlap,
                    set(),
                    f"pilot manifests {a} and {b} overlap: {overlap} — each pilot anchors a distinct task family",
                )

    def _assert_pilot_manifest(self, manifest_path: str) -> None:
        search_ids = set(load_task_ids("bench/search/task_ids.json"))
        pilot_ids = load_task_ids(manifest_path)

        self.assertTrue(pilot_ids, f"{manifest_path} must be non-empty")
        self.assertEqual(len(pilot_ids), len(set(pilot_ids)))
        self.assertTrue(
            set(pilot_ids).issubset(search_ids),
            f"{manifest_path} escapes the search split: {set(pilot_ids) - search_ids}",
        )

        tasks_by_id = {task.id: task for task in load_tasks("bench/tasks/tasks.json")}
        for task_id in pilot_ids:
            self.assertIn(task_id, tasks_by_id)


class HoldoutImmutabilityTests(unittest.TestCase):
    """L1 closure: mechanical guard for holdout-task drift.

    Detects silent edits to holdout task descriptions, scorer settings,
    or referenced files. Legitimate repairs must follow the holdout-repair
    exception path: bump holdout_version, regenerate fingerprints, and
    mark prior holdout results non-comparable.
    """

    def test_live_holdout_matches_recorded_fingerprints(self) -> None:
        verify_holdout_fingerprints()

    def test_holdout_fingerprint_manifest_has_required_shape(self) -> None:
        manifest = load_holdout_fingerprints()
        self.assertIsInstance(manifest["holdout_version"], int)
        self.assertGreaterEqual(manifest["holdout_version"], 1)
        holdout_ids = set(load_task_ids("bench/holdout/task_ids.json"))
        self.assertEqual(set(manifest["fingerprints"]), holdout_ids)

    def test_drift_in_task_description_is_detected(self) -> None:
        with tempfile.TemporaryDirectory(prefix="bench_holdout_drift_") as tmpdir:
            tmp = Path(tmpdir)
            raw_tasks = json.loads(Path("bench/tasks/tasks.json").read_text())
            for entry in raw_tasks:
                if entry["id"] == "T22":
                    entry["description"] = entry["description"] + "  (sneak edit)"
            tasks_path = tmp / "tasks.json"
            tasks_path.write_text(json.dumps(raw_tasks))
            with self.assertRaisesRegex(ValueError, "holdout-immutability breach"):
                verify_holdout_fingerprints(
                    tasks_path=str(tasks_path),
                    holdout_ids_path="bench/holdout/task_ids.json",
                    fingerprints_path="bench/holdout/fingerprints.json",
                )

    def test_drift_in_expected_output_bytes_is_detected(self) -> None:
        with tempfile.TemporaryDirectory(prefix="bench_holdout_bytes_") as tmpdir:
            tmp = Path(tmpdir)
            raw_tasks = json.loads(Path("bench/tasks/tasks.json").read_text())
            mutated_expected = tmp / "expected.md"
            mutated_expected.write_text("totally different expected output\n")
            for entry in raw_tasks:
                if entry["id"] == "T4":
                    entry["expected_output"] = str(mutated_expected)
            tasks_path = tmp / "tasks.json"
            tasks_path.write_text(json.dumps(raw_tasks))
            with self.assertRaisesRegex(ValueError, "holdout-immutability breach"):
                verify_holdout_fingerprints(
                    tasks_path=str(tasks_path),
                    holdout_ids_path="bench/holdout/task_ids.json",
                    fingerprints_path="bench/holdout/fingerprints.json",
                )

    def test_compute_task_fingerprint_is_deterministic(self) -> None:
        raw_tasks = json.loads(Path("bench/tasks/tasks.json").read_text())
        by_id = {t["id"]: t for t in raw_tasks}
        first = compute_task_fingerprint(by_id["T22"])
        second = compute_task_fingerprint(by_id["T22"])
        self.assertEqual(first, second)


if __name__ == "__main__":
    unittest.main()
