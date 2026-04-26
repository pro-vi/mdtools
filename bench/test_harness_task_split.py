from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from bench.harness import (
    BenchTask,
    check_holdout_integrity,
    compute_task_fingerprint,
    load_holdout_fingerprints,
    load_task_ids,
    load_tasks,
    read_holdout_version,
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


class HarnessRuntimeHoldoutGuardTests(unittest.TestCase):
    """Runtime mechanical invocation of the holdout-immutability guard.

    The L1 closure landed verify_holdout_fingerprints as a function plus
    a cheap-channel unit test (above). This class pins the harness's
    runtime wrapper check_holdout_integrity, so any harness invocation
    that runs against a configured holdout split is mechanically gated
    on holdout integrity, not just on the unit test having been run
    first.
    """

    def test_clean_repo_returns_none(self) -> None:
        self.assertIsNone(check_holdout_integrity())

    def test_drift_returns_breach_message(self) -> None:
        with tempfile.TemporaryDirectory(prefix="bench_runtime_drift_") as tmpdir:
            tmp = Path(tmpdir)
            raw_tasks = json.loads(Path("bench/tasks/tasks.json").read_text())
            for entry in raw_tasks:
                if entry["id"] == "T22":
                    entry["description"] = entry["description"] + "  (sneak edit)"
            tasks_path = tmp / "tasks.json"
            tasks_path.write_text(json.dumps(raw_tasks))
            breach = check_holdout_integrity(
                tasks_path=str(tasks_path),
                holdout_ids_path="bench/holdout/task_ids.json",
                fingerprints_path="bench/holdout/fingerprints.json",
            )
            self.assertIsNotNone(breach)
            self.assertIn("holdout-immutability breach", breach)
            self.assertIn("T22", breach)

    def test_missing_holdout_files_skipped_silently(self) -> None:
        with tempfile.TemporaryDirectory(prefix="bench_no_holdout_") as tmpdir:
            tmp = Path(tmpdir)
            self.assertIsNone(
                check_holdout_integrity(
                    tasks_path="bench/tasks/tasks.json",
                    holdout_ids_path=str(tmp / "missing_task_ids.json"),
                    fingerprints_path=str(tmp / "missing_fingerprints.json"),
                )
            )


class HoldoutVersionStampTests(unittest.TestCase):
    """Cross-version comparability: stamp holdout_version onto run.json bundles.

    The frontier-loop spec's holdout-repair exception path requires bundles
    to carry the holdout_version under which they were produced, so future
    cross-version comparability is mechanical rather than inferred from
    bundle dates. read_holdout_version is the single authoritative read at
    run start; build_run_metadata threads the value into run.json metadata.
    """

    def test_read_holdout_version_returns_one_for_live_repo(self) -> None:
        self.assertEqual(read_holdout_version(), 1)

    def test_read_holdout_version_returns_none_when_missing(self) -> None:
        with tempfile.TemporaryDirectory(prefix="bench_no_fingerprints_") as tmpdir:
            self.assertIsNone(
                read_holdout_version(
                    fingerprints_path=str(Path(tmpdir) / "missing_fingerprints.json"),
                )
            )

    def test_read_holdout_version_returns_none_for_malformed(self) -> None:
        with tempfile.TemporaryDirectory(prefix="bench_malformed_fingerprints_") as tmpdir:
            path = Path(tmpdir) / "fingerprints.json"
            path.write_text("{}")
            self.assertIsNone(read_holdout_version(fingerprints_path=str(path)))


class ScorerDispatcherBranchTests(unittest.TestCase):
    """Pin the corpus-vacuous property of the score_task else-arm.

    iter 27 surfaced (forward-pointing in bench/ledger.md) that
    bench/harness.py:378's else-arm fallback ("ok = a.strip() == e.strip()")
    is corpus-vacuous in the current 24-task corpus: every task routes to
    one of four explicit dispatcher branches at lines 340 (raw_bytes), 363
    (structural + json_canonical), 367 (structural + json_envelope), or
    371 (normalized_text). This class promotes that prose claim to a
    typed cheap-channel assertion so future task additions or scorer
    edits that would silently route through an unvalidated string-equality
    fallback fire here rather than being discovered only by manual prose
    re-reading.
    """

    BRANCH_RAW_BYTES = "raw_bytes"
    BRANCH_JSON_CANONICAL = "json_canonical"
    BRANCH_STRUCTURAL_JSON = "structural_json"
    BRANCH_NORMALIZED_TEXT = "normalized_text"
    BRANCH_ELSE_FALLBACK = "else_fallback"

    @classmethod
    def _classify(cls, task: BenchTask) -> str:
        """Return the dispatcher branch this task routes to in score_task.

        Mirrors the predicate order in bench/harness.py:340-378 exactly:
        raw_bytes early-return is first; among kind=="structural", the
        json_canonical branch wins over the json_envelope branch.
        """
        policy = task.scorer
        if policy.kind == "raw_bytes":
            return cls.BRANCH_RAW_BYTES
        if policy.kind == "structural" and policy.json_canonical:
            return cls.BRANCH_JSON_CANONICAL
        if policy.kind == "structural" and task.expected_artifact == "json_envelope":
            return cls.BRANCH_STRUCTURAL_JSON
        if policy.kind == "normalized_text":
            return cls.BRANCH_NORMALIZED_TEXT
        return cls.BRANCH_ELSE_FALLBACK

    def test_no_corpus_task_reaches_else_fallback(self) -> None:
        tasks = load_tasks("bench/tasks/tasks.json")
        unrouted = [
            {
                "id": task.id,
                "kind": task.scorer.kind,
                "expected_artifact": task.expected_artifact,
                "json_canonical": task.scorer.json_canonical,
            }
            for task in tasks
            if self._classify(task) == self.BRANCH_ELSE_FALLBACK
        ]
        self.assertEqual(
            unrouted,
            [],
            "Tasks reach bench/harness.py:378's corpus-vacuous else-arm "
            "(`ok = a.strip() == e.strip()`); fix scorer.kind / "
            "expected_artifact / json_canonical or extend the dispatcher: "
            f"{unrouted}",
        )

    def test_corpus_exercises_all_four_explicit_branches(self) -> None:
        tasks = load_tasks("bench/tasks/tasks.json")
        branches_hit = {self._classify(task) for task in tasks}
        for branch in (
            self.BRANCH_RAW_BYTES,
            self.BRANCH_JSON_CANONICAL,
            self.BRANCH_STRUCTURAL_JSON,
            self.BRANCH_NORMALIZED_TEXT,
        ):
            self.assertIn(
                branch,
                branches_hit,
                f"Corpus no longer exercises the {branch} dispatcher branch; "
                "either restore a task that hits this branch or remove the "
                "branch from bench/harness.py:score_task",
            )


if __name__ == "__main__":
    unittest.main()
