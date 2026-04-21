from __future__ import annotations

import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from bench.harness import BenchResult, build_run_metadata, write_run_artifacts


class AnalyzeInputTests(unittest.TestCase):
    def test_analyze_accepts_run_bundle_directory(self) -> None:
        repo_root = Path(__file__).resolve().parent.parent
        results = [
            BenchResult(
                task_id="T1",
                mode="hybrid",
                correct=True,
                correct_neutral=True,
                model="claude-haiku-test",
                tool_calls=2,
                elapsed_seconds=1.5,
            )
        ]
        metadata = build_run_metadata(
            run_kind="agent-track",
            tasks_path="bench/tasks/tasks.json",
            task_ids_path="bench/search/task_ids.json",
            selected_task_ids=["T1"],
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

        with tempfile.TemporaryDirectory(prefix="bench_analyze_bundle_") as tmpdir:
            write_run_artifacts(
                tmpdir,
                metadata=metadata,
                results=results,
                selected_task_ids=["T1"],
            )

            completed = subprocess.run(
                [sys.executable, "bench/analyze.py", tmpdir],
                capture_output=True,
                text=True,
                cwd=repo_root,
                check=False,
            )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("Run context:", completed.stdout)
        self.assertIn("selection=bench/search/task_ids.json", completed.stdout)
        self.assertIn("MODEL: claude-haiku-test", completed.stdout)
        self.assertIn("T1", completed.stdout)

    def test_analyze_labels_unresolved_persisted_models_as_unspecified(self) -> None:
        repo_root = Path(__file__).resolve().parent.parent
        results = [
            BenchResult(
                task_id="T1",
                mode="mdtools",
                correct=True,
                correct_neutral=True,
                elapsed_seconds=0.0,
            )
        ]
        metadata = build_run_metadata(
            run_kind="dry-run",
            tasks_path="bench/tasks/tasks.json",
            task_ids_path="bench/search/task_ids.json",
            selected_task_ids=["T1"],
            modes=["mdtools"],
            md_binary="target/debug/md",
            runner=None,
            executor=None,
            model=None,
            runs_per_task=1,
            results=results,
            started_at=0,
            finished_at=1,
        )

        with tempfile.TemporaryDirectory(prefix="bench_analyze_unspecified_") as tmpdir:
            write_run_artifacts(
                tmpdir,
                metadata=metadata,
                results=results,
                selected_task_ids=["T1"],
            )

            completed = subprocess.run(
                [sys.executable, "bench/analyze.py", tmpdir],
                capture_output=True,
                text=True,
                cwd=repo_root,
                check=False,
            )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("MODEL: unspecified", completed.stdout)
        self.assertNotIn("MODEL: opus", completed.stdout)

    def test_analyze_accepts_dry_run_text_output(self) -> None:
        repo_root = Path(__file__).resolve().parent.parent
        dry_run_output = """=== DRY RUN: dual scorer validation ===

  T1: md=PASS neutral=PASS
    heading_tree [md]: OK
    heading_tree [neutral]: OK
  T2: md=PASS neutral=FAIL ⚠ DIVERGENCE
    heading_tree [md]: OK
    heading_tree [neutral]: FAIL

SCORER ISSUES DETECTED.
"""

        with tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            prefix="bench_analyze_dry_run_",
            suffix=".txt",
            delete=False,
        ) as handle:
            tmp_path = Path(handle.name)
            handle.write(dry_run_output)

        try:
            completed = subprocess.run(
                [sys.executable, "bench/analyze.py", str(tmp_path)],
                capture_output=True,
                text=True,
                cwd=repo_root,
                check=False,
            )
        finally:
            tmp_path.unlink(missing_ok=True)

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("MODEL: unspecified", completed.stdout)
        self.assertNotIn("No results found.", completed.stdout)
        self.assertNotIn("MODEL: opus", completed.stdout)
        self.assertRegex(completed.stdout, re.compile(r"^T2\s+—\s+—\s+—\s+0%\s+0s\s+0\.0", re.MULTILINE))

    def test_analyze_surfaces_runner_errors_from_run_bundle(self) -> None:
        repo_root = Path(__file__).resolve().parent.parent
        results = [
            BenchResult(
                task_id="T1",
                mode="mdtools",
                correct=False,
                correct_neutral=False,
                elapsed_seconds=0.38,
                bytes_output=2372,
                runner_error="authentication_failed: Not logged in · Please run /login",
            )
        ]
        metadata = build_run_metadata(
            run_kind="agent-track",
            tasks_path="bench/tasks/tasks.json",
            task_ids_path=None,
            selected_task_ids=["T1"],
            modes=["mdtools"],
            md_binary="target/debug/md",
            runner="claude-cli",
            executor="guarded",
            model=None,
            runs_per_task=1,
            results=results,
            started_at=0,
            finished_at=1,
        )

        with tempfile.TemporaryDirectory(prefix="bench_analyze_runner_error_") as tmpdir:
            write_run_artifacts(
                tmpdir,
                metadata=metadata,
                results=results,
                selected_task_ids=["T1"],
            )

            completed = subprocess.run(
                [sys.executable, "bench/analyze.py", tmpdir],
                capture_output=True,
                text=True,
                cwd=repo_root,
                check=False,
            )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("Runner errors:", completed.stdout)
        self.assertIn("authentication_failed: Not logged in · Please run /login", completed.stdout)
        self.assertIn("mdtools x1 [T1]", completed.stdout)

    def test_analyze_results_json_without_metadata_uses_unspecified(self) -> None:
        repo_root = Path(__file__).resolve().parent.parent
        results = [
            {
                "task_id": "T1",
                "mode": "mdtools",
                "correct": True,
                "elapsed_seconds": 0.5,
                "tool_calls": 1,
            }
        ]

        with tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            prefix="bench_analyze_results_json_",
            suffix=".json",
            delete=False,
        ) as handle:
            tmp_path = Path(handle.name)
            handle.write(json.dumps(results))

        try:
            completed = subprocess.run(
                [sys.executable, "bench/analyze.py", str(tmp_path)],
                capture_output=True,
                text=True,
                cwd=repo_root,
                check=False,
            )
        finally:
            tmp_path.unlink(missing_ok=True)

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("MODEL: unspecified", completed.stdout)
        self.assertNotIn("MODEL: opus", completed.stdout)

    def test_analyze_separates_same_model_across_runners(self) -> None:
        repo_root = Path(__file__).resolve().parent.parent
        model_id = "claude-haiku-4-5-20251001"

        def build_bundle(runner: str, passed: bool) -> tuple[list[BenchResult], dict]:
            results = [
                BenchResult(
                    task_id="T1",
                    mode="hybrid",
                    correct=passed,
                    correct_neutral=passed,
                    model=model_id,
                    tool_calls=2,
                    elapsed_seconds=1.0,
                )
            ]
            metadata = build_run_metadata(
                run_kind="agent-track",
                tasks_path="bench/tasks/tasks.json",
                task_ids_path="bench/search/task_ids.json",
                selected_task_ids=["T1"],
                modes=["hybrid"],
                md_binary="target/debug/md",
                runner=runner,
                executor="guarded",
                model=None,
                runs_per_task=1,
                results=results,
                started_at=0,
                finished_at=1,
            )
            return results, metadata

        claude_results, claude_metadata = build_bundle("claude-cli", passed=True)
        oai_results, oai_metadata = build_bundle("oai-loop", passed=False)

        with tempfile.TemporaryDirectory(prefix="bench_analyze_claude_") as claude_dir, \
                tempfile.TemporaryDirectory(prefix="bench_analyze_oai_") as oai_dir:
            write_run_artifacts(
                claude_dir,
                metadata=claude_metadata,
                results=claude_results,
                selected_task_ids=["T1"],
            )
            write_run_artifacts(
                oai_dir,
                metadata=oai_metadata,
                results=oai_results,
                selected_task_ids=["T1"],
            )

            completed = subprocess.run(
                [sys.executable, "bench/analyze.py", claude_dir, oai_dir],
                capture_output=True,
                text=True,
                cwd=repo_root,
                check=False,
            )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        stdout = completed.stdout
        self.assertIn(f"MODEL: {model_id} (runner=claude-cli)", stdout)
        self.assertIn(f"MODEL: {model_id} (runner=oai-loop)", stdout)
        self.assertIn("CROSS-MODEL SUMMARY", stdout)
        self.assertIn(f"{model_id} [claude-cli]", stdout)
        self.assertIn(f"{model_id} [oai-loop]", stdout)

    def test_analyze_renders_unique_invalid_responses_column(self) -> None:
        """The InvU column distinguishes a deterministic-lock signature
        (invalid_responses=30, unique=1) from a varied-strategy signature
        (invalid_responses=25, unique=15). It must render in the
        CROSS-MODEL SUMMARY beside the existing Inv column so future
        model comparisons can read the adaptation signal at a glance."""
        repo_root = Path(__file__).resolve().parent.parent

        def build_bundle(runner: str, invalid: int, unique: int) -> tuple[list[BenchResult], dict]:
            results = [
                BenchResult(
                    task_id="T16",
                    mode="mdtools",
                    correct=False,
                    correct_neutral=False,
                    model="test-model",
                    tool_calls=0,
                    turns=30,
                    invalid_responses=invalid,
                    unique_invalid_responses=unique,
                    elapsed_seconds=1.0,
                )
            ]
            metadata = build_run_metadata(
                run_kind="agent-track",
                tasks_path="bench/tasks/tasks.json",
                task_ids_path="bench/search/task_ids.json",
                selected_task_ids=["T16"],
                modes=["mdtools"],
                md_binary="target/debug/md",
                runner=runner,
                executor="guarded",
                model=None,
                runs_per_task=1,
                results=results,
                started_at=0,
                finished_at=1,
            )
            return results, metadata

        stuck_results, stuck_meta = build_bundle("oai-loop-stuck", invalid=30, unique=1)
        trying_results, trying_meta = build_bundle("oai-loop-trying", invalid=25, unique=15)

        with tempfile.TemporaryDirectory(prefix="bench_analyze_stuck_") as stuck_dir, \
                tempfile.TemporaryDirectory(prefix="bench_analyze_trying_") as trying_dir:
            write_run_artifacts(
                stuck_dir, metadata=stuck_meta, results=stuck_results, selected_task_ids=["T16"]
            )
            write_run_artifacts(
                trying_dir, metadata=trying_meta, results=trying_results, selected_task_ids=["T16"]
            )

            completed = subprocess.run(
                [sys.executable, "bench/analyze.py", stuck_dir, trying_dir],
                capture_output=True,
                text=True,
                cwd=repo_root,
                check=False,
            )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("InvU", completed.stdout)
        # The cross-model summary should surface 30.0/1.0 for the stuck
        # group and 25.0/15.0 for the trying group on the same row.
        self.assertRegex(
            completed.stdout, re.compile(r"oai-loop-stuck\].*mdtools.*30\.0\s+1\.0", re.DOTALL)
        )
        self.assertRegex(
            completed.stdout, re.compile(r"oai-loop-trying\].*mdtools.*25\.0\s+15\.0", re.DOTALL)
        )

    def test_analyze_accepts_text_runner_error_suffixes(self) -> None:
        repo_root = Path(__file__).resolve().parent.parent
        agent_output = """=== MODE: mdtools (N=1, model=claude-sonnet-test) ===

  T1: summarize headings
    md=FAIL neutral=FAIL | 0.38s | ~2372B out | obs:0B | ~0 calls | 0 mut | deny:0 | err:authentication_failed: Not logged in · Please run /login
"""

        with tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            prefix="bench_analyze_runner_error_",
            suffix=".txt",
            delete=False,
        ) as handle:
            tmp_path = Path(handle.name)
            handle.write(agent_output)

        try:
            completed = subprocess.run(
                [sys.executable, "bench/analyze.py", str(tmp_path)],
                capture_output=True,
                text=True,
                cwd=repo_root,
                check=False,
            )
        finally:
            tmp_path.unlink(missing_ok=True)

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("MODEL: claude-sonnet-test", completed.stdout)
        self.assertIn("Runner errors:", completed.stdout)
        self.assertIn("authentication_failed: Not logged in · Please run /login", completed.stdout)
        self.assertIn("mdtools x1 [T1]", completed.stdout)


if __name__ == "__main__":
    unittest.main()
