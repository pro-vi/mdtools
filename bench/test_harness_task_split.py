from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from bench.harness import load_task_ids, load_tasks, select_tasks


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

    def test_pilot_manifests_are_disjoint(self) -> None:
        extraction = set(load_task_ids("bench/search/extraction_pilot.json"))
        mutation = set(load_task_ids("bench/search/mutation_pilot.json"))
        overlap = extraction & mutation
        self.assertEqual(
            overlap,
            set(),
            f"pilot manifests overlap: {overlap} — each pilot anchors a distinct task family",
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


if __name__ == "__main__":
    unittest.main()
