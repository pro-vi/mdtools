from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from bench import harness
from bench.harness import BenchResult, build_run_metadata, load_tasks, run_agent, select_tasks, write_run_artifacts


class HarnessRunArtifactTests(unittest.TestCase):
    def test_write_run_artifacts_persists_metadata_results_and_task_ids(self) -> None:
        results = [
            BenchResult(
                task_id="T2",
                mode="hybrid",
                correct=True,
                correct_neutral=True,
                model="claude-sonnet-4-6",
                tool_calls=3,
                elapsed_seconds=1.25,
            ),
            BenchResult(
                task_id="T1",
                mode="hybrid",
                correct=False,
                correct_neutral=True,
                model="claude-sonnet-4-6",
                policy_violations=1,
                requeried=True,
                invalid_responses=4,
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
            model=None,
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
        self.assertEqual(run_data["model"], "claude-sonnet-4-6")

        self.assertEqual([item["task_id"] for item in result_data], ["T2", "T1"])
        self.assertEqual(result_data[0]["model"], "claude-sonnet-4-6")
        self.assertEqual(result_data[1]["policy_violations"], 1)
        self.assertEqual(result_data[0]["invalid_responses"], 0)
        self.assertEqual(result_data[1]["invalid_responses"], 4)
        self.assertAlmostEqual(
            run_data["aggregates"]["overall"]["avg_invalid_responses"], 2.0
        )
        self.assertAlmostEqual(
            run_data["aggregates"]["by_mode"]["hybrid"]["avg_invalid_responses"], 2.0
        )
        self.assertEqual(
            result_data[1]["runner_error"],
            "authentication_failed: Not logged in · Please run /login",
        )
        self.assertEqual(task_id_data, ["T2", "T1"])

    def test_write_run_artifacts_is_atomic_and_repeatable(self) -> None:
        """Repeated calls cleanly overwrite and leave no .tmp residue, so the main loop can write an always-valid partial bundle after every task."""
        first = [
            BenchResult(
                task_id="T1",
                mode="hybrid",
                correct=True,
                correct_neutral=True,
                tool_calls=1,
            ),
        ]
        second = [
            BenchResult(
                task_id="T1",
                mode="hybrid",
                correct=True,
                correct_neutral=True,
                tool_calls=1,
            ),
            BenchResult(
                task_id="T2",
                mode="hybrid",
                correct=False,
                correct_neutral=False,
                tool_calls=5,
            ),
        ]

        def make_metadata(results: list[BenchResult], selected: list[str], finished_at: float) -> dict:
            return build_run_metadata(
                run_kind="agent-track",
                tasks_path="bench/tasks/tasks.json",
                task_ids_path="bench/search/extraction_pilot.json",
                selected_task_ids=selected,
                modes=["hybrid"],
                md_binary="target/debug/md",
                runner="oai-loop",
                executor="guarded",
                model="Qwen3.5-27B-4bit",
                runs_per_task=1,
                results=results,
                started_at=0,
                finished_at=finished_at,
            )

        with tempfile.TemporaryDirectory(prefix="bench_partial_") as tmpdir:
            selected = ["T1", "T2"]
            write_run_artifacts(
                tmpdir,
                metadata=make_metadata(first, selected, finished_at=1),
                results=first,
                selected_task_ids=selected,
            )
            # After the first write (partial), bundle must be fully valid JSON.
            partial_results = json.loads((Path(tmpdir) / "results.json").read_text())
            partial_run = json.loads((Path(tmpdir) / "run.json").read_text())
            self.assertEqual([r["task_id"] for r in partial_results], ["T1"])
            self.assertEqual(partial_run["aggregates"]["overall"]["runs"], 1)

            write_run_artifacts(
                tmpdir,
                metadata=make_metadata(second, selected, finished_at=2),
                results=second,
                selected_task_ids=selected,
            )
            final_results = json.loads((Path(tmpdir) / "results.json").read_text())
            final_run = json.loads((Path(tmpdir) / "run.json").read_text())
            self.assertEqual([r["task_id"] for r in final_results], ["T1", "T2"])
            self.assertEqual(final_run["aggregates"]["overall"]["runs"], 2)
            self.assertEqual(final_run["finished_at"], "1970-01-01T00:00:02Z")

            # No .tmp residue from the atomic write pattern.
            residue = sorted(p.name for p in Path(tmpdir).iterdir() if p.name.endswith(".tmp"))
            self.assertEqual(residue, [])

    def test_run_agent_records_runner_error_when_oai_loop_raises(self) -> None:
        """A hung or failed oai-loop request becomes a recorded runner_error
        on the BenchResult, so the outer harness loop can continue to the next
        task with per-task incremental durability (instead of the whole run
        aborting and losing prior tasks' results)."""
        md_binary = "target/release/md"
        if not Path(md_binary).exists():
            self.skipTest(f"{md_binary} not built")

        tasks = load_tasks("bench/tasks/tasks.json")
        task = select_tasks(tasks, ["T7"])[0]

        def boom(**_kwargs):
            raise TimeoutError(
                "OAI request to /chat/completions exceeded wall-time deadline of 90s"
            )

        with patch.object(harness, "run_oai_loop", side_effect=boom):
            result = run_agent(
                task,
                "mdtools",
                agent_cmd="unused",
                md_binary=md_binary,
                model="Hermes-4-70B-4bit",
                runner="oai-loop",
                executor="guarded",
                log_dir=None,
                max_turns=30,
                oai_api_base="http://127.0.0.1:10240/v1",
                oai_api_key="test-key",
                oai_request_timeout=60,
                oai_tool_timeout=30,
            )

        self.assertFalse(result.correct)
        self.assertIsNotNone(result.runner_error)
        self.assertIn("oai_loop_error", result.runner_error)
        self.assertIn("TimeoutError", result.runner_error)
        self.assertIn("wall-time deadline", result.runner_error)
        self.assertEqual(result.task_id, "T7")
        self.assertEqual(result.mode, "mdtools")
        self.assertEqual(result.tool_calls, 0)
        self.assertEqual(result.turns, 0)
        self.assertEqual(result.model, "Hermes-4-70B-4bit")


if __name__ == "__main__":
    unittest.main()
