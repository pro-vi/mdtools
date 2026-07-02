from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from bench import harness
from bench.harness import (
    BenchResult,
    BenchTask,
    StructuralDiffPolicy,
    build_run_metadata,
    load_resume_results,
    load_tasks,
    run_agent,
    select_tasks,
    write_run_artifacts,
)
from bench.oai_loop import LoopError, LoopTrace


class HarnessRunArtifactTests(unittest.TestCase):
    def test_write_run_artifacts_persists_metadata_results_and_task_ids(self) -> None:
        results = [
            BenchResult(
                task_id="T2",
                mode="hybrid",
                correct=True,
                correct_neutral=True,
                run_index=0,
                model="claude-sonnet-4-6",
                tool_calls=3,
                elapsed_seconds=1.25,
            ),
            BenchResult(
                task_id="T1",
                mode="hybrid",
                correct=False,
                correct_neutral=True,
                run_index=0,
                model="claude-sonnet-4-6",
                policy_violations=1,
                requeried=True,
                invalid_responses=4,
                unique_invalid_responses=1,
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
        self.assertEqual(run_data["trials_per_cell"], 1)
        self.assertEqual(
            run_data["temperature_policy"],
            "provider default (temperature not exposed by claude-cli)",
        )
        self.assertEqual(run_data["started_at"], "1970-01-01T00:00:00Z")
        self.assertEqual(run_data["finished_at"], "1970-01-01T00:00:01Z")
        self.assertEqual(run_data["model"], "claude-sonnet-4-6")

        self.assertEqual([item["task_id"] for item in result_data], ["T2", "T1"])
        self.assertEqual([item["run_index"] for item in result_data], [0, 0])
        self.assertEqual(result_data[0]["model"], "claude-sonnet-4-6")
        self.assertEqual(result_data[1]["policy_violations"], 1)
        self.assertEqual(result_data[0]["invalid_responses"], 0)
        self.assertEqual(result_data[1]["invalid_responses"], 4)
        self.assertEqual(result_data[0]["unique_invalid_responses"], 0)
        self.assertEqual(result_data[1]["unique_invalid_responses"], 1)
        self.assertAlmostEqual(
            run_data["aggregates"]["overall"]["avg_invalid_responses"], 2.0
        )
        self.assertAlmostEqual(
            run_data["aggregates"]["by_mode"]["hybrid"]["avg_invalid_responses"], 2.0
        )
        # InvU aggregates separately: 0 + 1 = 1, /2 = 0.5 — distinguishes the
        # (4 invalid, 1 unique) deterministic-lock signature from a hypothetical
        # (4 invalid, 4 unique) varied-strategy signature.
        self.assertAlmostEqual(
            run_data["aggregates"]["overall"]["avg_unique_invalid_responses"], 0.5
        )
        self.assertAlmostEqual(
            run_data["aggregates"]["by_mode"]["hybrid"]["avg_unique_invalid_responses"],
            0.5,
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

    def test_load_resume_results_accepts_matching_partial_bundle(self) -> None:
        results = [
            BenchResult(
                task_id="T1",
                mode="unix",
                correct=False,
                correct_neutral=True,
                run_index=0,
                model="Qwen3.6-35B-A3B-8bit",
                tool_calls=29,
            )
        ]
        metadata = build_run_metadata(
            run_kind="agent-track",
            tasks_path="bench/tasks/tasks.json",
            task_ids_path=None,
            selected_task_ids=["T1", "T2"],
            modes=["unix", "mdtools", "hybrid"],
            md_binary="target/release/md",
            runner="oai-loop",
            executor="guarded",
            model="Qwen3.6-35B-A3B-8bit",
            runs_per_task=5,
            results=results,
            started_at=0,
            finished_at=1,
        )

        with tempfile.TemporaryDirectory(prefix="bench_resume_") as tmpdir:
            write_run_artifacts(
                tmpdir,
                metadata=metadata,
                results=results,
                selected_task_ids=["T1", "T2"],
            )
            loaded = load_resume_results(
                tmpdir,
                selected_task_ids=["T1", "T2"],
                modes=["unix", "mdtools", "hybrid"],
                runs_per_task=5,
                runner="oai-loop",
                executor="guarded",
                model="Qwen3.6-35B-A3B-8bit",
            )

        self.assertEqual(len(loaded), 1)
        self.assertEqual(loaded[0].task_id, "T1")
        self.assertEqual(loaded[0].mode, "unix")
        self.assertEqual(loaded[0].run_index, 0)
        self.assertEqual(loaded[0].tool_calls, 29)

    def test_load_resume_results_rejects_duplicate_cells(self) -> None:
        results = [
            BenchResult(task_id="T1", mode="unix", correct=False, run_index=0),
            BenchResult(task_id="T1", mode="unix", correct=True, run_index=0),
        ]
        metadata = build_run_metadata(
            run_kind="agent-track",
            tasks_path="bench/tasks/tasks.json",
            task_ids_path=None,
            selected_task_ids=["T1"],
            modes=["unix"],
            md_binary="target/release/md",
            runner="oai-loop",
            executor="guarded",
            model="Qwen3.6-35B-A3B-8bit",
            runs_per_task=5,
            results=results,
            started_at=0,
            finished_at=1,
        )

        with tempfile.TemporaryDirectory(prefix="bench_resume_dupe_") as tmpdir:
            write_run_artifacts(
                tmpdir,
                metadata=metadata,
                results=results,
                selected_task_ids=["T1"],
            )
            with self.assertRaisesRegex(ValueError, "duplicate result"):
                load_resume_results(
                    tmpdir,
                    selected_task_ids=["T1"],
                    modes=["unix"],
                    runs_per_task=5,
                    runner="oai-loop",
                    executor="guarded",
                    model="Qwen3.6-35B-A3B-8bit",
                )

    def test_load_resume_results_rejects_metadata_mismatch(self) -> None:
        results = [BenchResult(task_id="T1", mode="unix", correct=False, run_index=0)]
        metadata = build_run_metadata(
            run_kind="agent-track",
            tasks_path="bench/tasks/tasks.json",
            task_ids_path=None,
            selected_task_ids=["T1"],
            modes=["unix"],
            md_binary="target/release/md",
            runner="oai-loop",
            executor="guarded",
            model="Qwen3.6-35B-A3B-8bit",
            runs_per_task=1,
            results=results,
            started_at=0,
            finished_at=1,
        )

        with tempfile.TemporaryDirectory(prefix="bench_resume_mismatch_") as tmpdir:
            write_run_artifacts(
                tmpdir,
                metadata=metadata,
                results=results,
                selected_task_ids=["T1"],
            )
            with self.assertRaisesRegex(ValueError, "trials_per_cell"):
                load_resume_results(
                    tmpdir,
                    selected_task_ids=["T1"],
                    modes=["unix"],
                    runs_per_task=5,
                    runner="oai-loop",
                    executor="guarded",
                    model="Qwen3.6-35B-A3B-8bit",
                )

    def test_run_metadata_records_qwen_temperature_policy(self) -> None:
        results = [
            BenchResult(
                task_id="T1",
                mode="hybrid",
                correct=True,
                correct_neutral=True,
                run_index=0,
                model="Qwen3.6-35B-A3B-8bit",
            )
        ]
        metadata = build_run_metadata(
            run_kind="agent-track",
            tasks_path="bench/tasks/tasks.json",
            task_ids_path=None,
            selected_task_ids=["T1"],
            modes=["hybrid"],
            md_binary="target/debug/md",
            runner="oai-loop",
            executor="guarded",
            model="Qwen3.6-35B-A3B-8bit",
            runs_per_task=5,
            results=results,
            started_at=0,
            finished_at=1,
        )

        self.assertEqual(metadata["trials_per_cell"], 5)
        self.assertEqual(
            metadata["temperature_policy"],
            "temperature=0; chat_template_kwargs.enable_thinking=false",
        )

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


    def test_run_agent_preserves_partial_trace_when_loop_error_raised(self) -> None:
        """When run_oai_loop raises LoopError (watchdog/HTTP/etc.) mid-task,
        the harness populates the BenchResult's per-turn counters from the
        attached partial trace instead of defaulting to all zeros, so the
        durable bundle carries diagnostic signal (tool_calls, invalid_responses,
        turns, bytes_*) alongside the recorded runner_error."""
        md_binary = "target/release/md"
        if not Path(md_binary).exists():
            self.skipTest(f"{md_binary} not built")

        tasks = load_tasks("bench/tasks/tasks.json")
        task = select_tasks(tasks, ["T7"])[0]

        partial = LoopTrace(
            raw_output="[partial transcript]",
            text_outputs=["first attempt"],
            tool_outputs=["tool output 1", "tool output 2"],
            bytes_output=512,
            bytes_observation=1024,
            tool_calls=2,
            turns=4,
            invalid_responses=1,
            unique_invalid_responses=1,
        )

        def raise_loop_error(**_kwargs):
            cause = TimeoutError(
                "OAI request to /chat/completions exceeded wall-time deadline of 90s"
            )
            raise LoopError(cause, partial)

        with patch.object(harness, "run_oai_loop", side_effect=raise_loop_error):
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
        # Partial trace counters MUST survive the error path.
        self.assertEqual(result.tool_calls, 2)
        self.assertEqual(result.invalid_responses, 1)
        self.assertEqual(result.unique_invalid_responses, 1)
        self.assertEqual(result.turns, 4)
        self.assertEqual(result.bytes_output, 512)
        self.assertEqual(result.bytes_observation, 1024)

    def test_run_agent_scores_input_file_copied_under_non_inputs_parent(self) -> None:
        """Candidate tasks under bench/search/candidates keep their parent
        directory in the temp workspace; prompt and scorer must point at that
        copied path instead of workdir/<basename>."""
        md_binary = "target/release/md"
        if not Path(md_binary).exists():
            self.skipTest(f"{md_binary} not built")

        with tempfile.TemporaryDirectory(prefix="bench_candidate_path_") as tmpdir:
            tmp = Path(tmpdir)
            candidate_dir = tmp / "candidate-family"
            candidate_dir.mkdir()
            input_path = candidate_dir / "input.md"
            expected_path = candidate_dir / "expected.md"
            input_path.write_text("# Tasks\n\n- [ ] Ship docs\n")
            expected_path.write_text("# Tasks\n\n- [x] Ship docs\n")

            task = BenchTask(
                id="C-PATH",
                description="Mark the Ship docs task as done.",
                input_files=[str(input_path)],
                expected_output=str(expected_path),
                expected_artifact="file_contents",
                difficulty="intermediate",
                scorer=StructuralDiffPolicy(
                    kind="normalized_text",
                    normalize_line_endings=True,
                    ignore_trailing_whitespace=True,
                    compare_frontmatter_json=False,
                    compare_heading_tree=True,
                    compare_block_order=True,
                    compare_link_destinations=False,
                    compare_block_text=True,
                ),
            )

            def complete_task(**kwargs):
                workdir = Path(kwargs["workdir"])
                copied_input = workdir / "candidate-family" / "input.md"
                self.assertIn(str(copied_input), kwargs["prompt"])
                self.assertNotIn(f"{workdir}/input.md", kwargs["prompt"])
                copied_input.write_text(expected_path.read_text())
                return LoopTrace(
                    raw_output="done",
                    text_outputs=["done"],
                    tool_outputs=[],
                    bytes_output=4,
                    bytes_observation=0,
                    tool_calls=1,
                    turns=1,
                    invalid_responses=0,
                    unique_invalid_responses=0,
                )

            with patch.object(harness, "run_oai_loop", side_effect=complete_task):
                result = run_agent(
                    task,
                    "hybrid",
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

        self.assertTrue(result.correct, result.diff_report)
        self.assertTrue(result.correct_neutral, result.diff_report)
        self.assertIsNone(result.runner_error)


class HoldoutVersionMetadataTests(unittest.TestCase):
    """build_run_metadata stamps holdout_version onto run.json metadata.

    Cross-version comparability of holdout bundles requires the version to be
    recorded on each bundle, per the frontier-loop spec's holdout-repair
    exception path ("stamp the new version onto subsequent run bundles").
    """

    def _make_metadata(self, *, holdout_version: int | None) -> dict:
        return build_run_metadata(
            run_kind="dry-run",
            tasks_path="bench/tasks/tasks.json",
            task_ids_path=None,
            selected_task_ids=["T1"],
            modes=["mdtools"],
            md_binary="target/release/md",
            runner=None,
            executor=None,
            model=None,
            runs_per_task=1,
            results=[],
            started_at=0,
            finished_at=1,
            holdout_version=holdout_version,
        )

    def test_metadata_includes_holdout_version_when_provided(self) -> None:
        metadata = self._make_metadata(holdout_version=1)
        self.assertEqual(metadata["holdout_version"], 1)

    def test_metadata_holdout_version_is_none_by_default(self) -> None:
        metadata = build_run_metadata(
            run_kind="dry-run",
            tasks_path="bench/tasks/tasks.json",
            task_ids_path=None,
            selected_task_ids=["T1"],
            modes=["mdtools"],
            md_binary="target/release/md",
            runner=None,
            executor=None,
            model=None,
            runs_per_task=1,
            results=[],
            started_at=0,
            finished_at=1,
        )
        self.assertIn("holdout_version", metadata)
        self.assertIsNone(metadata["holdout_version"])

    def test_metadata_propagates_future_version_bumps(self) -> None:
        metadata = self._make_metadata(holdout_version=2)
        self.assertEqual(metadata["holdout_version"], 2)


if __name__ == "__main__":
    unittest.main()
