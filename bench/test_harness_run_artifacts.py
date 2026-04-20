from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from bench.harness import BenchResult, build_run_metadata, write_run_artifacts


class HarnessRunArtifactTests(unittest.TestCase):
    def test_write_run_artifacts_persists_metadata_results_and_task_ids(self) -> None:
        results = [
            BenchResult(task_id="T2", mode="hybrid", correct=True, correct_neutral=True, tool_calls=3, elapsed_seconds=1.25),
            BenchResult(
                task_id="T1",
                mode="hybrid",
                correct=False,
                correct_neutral=True,
                policy_violations=1,
                requeried=True,
                runner_error="authentication_failed: Not logged in · Please run /login",
            ),
        ]
        metadata = build_run_metadata(
            run_kind="agent-track",
            tasks_path="bench/tasks/tasks.json",
            task_ids_path="bench/search/task_ids.json",
            selected_task_ids=["T2", "T1"],
            modes=["hybrid"],
            md_binary="target/debug/md",
            runner="claude-cli",
            executor="guarded",
            model="claude-haiku-test",
            runs_per_task=1,
            results=results,
            started_at=0,
            finished_at=1,
        )

        with tempfile.TemporaryDirectory(prefix="bench_run_artifacts_") as tmpdir:
            write_run_artifacts(
                tmpdir,
                metadata=metadata,
                results=results,
                selected_task_ids=["T2", "T1"],
            )

            run_data = json.loads((Path(tmpdir) / "run.json").read_text())
            result_data = json.loads((Path(tmpdir) / "results.json").read_text())
            task_id_data = json.loads((Path(tmpdir) / "task_ids.json").read_text())

        self.assertEqual(run_data["kind"], "agent-track")
        self.assertEqual(run_data["selected_task_ids"], ["T2", "T1"])
        self.assertEqual(run_data["aggregates"]["overall"]["runs"], 2)
        self.assertEqual(run_data["aggregates"]["by_mode"]["hybrid"]["pass_count"], 1)
        self.assertEqual(run_data["started_at"], "1970-01-01T00:00:00Z")
        self.assertEqual(run_data["finished_at"], "1970-01-01T00:00:01Z")

        self.assertEqual([item["task_id"] for item in result_data], ["T2", "T1"])
        self.assertEqual(result_data[1]["policy_violations"], 1)
        self.assertEqual(
            result_data[1]["runner_error"],
            "authentication_failed: Not logged in · Please run /login",
        )
        self.assertEqual(task_id_data, ["T2", "T1"])


if __name__ == "__main__":
    unittest.main()
