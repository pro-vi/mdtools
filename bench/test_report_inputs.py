from __future__ import annotations

import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from bench.harness import BenchResult, build_run_metadata, write_run_artifacts


class ReportInputTests(unittest.TestCase):
    def test_report_accepts_run_bundle_directory(self) -> None:
        repo_root = Path(__file__).resolve().parent.parent
        results = [
            BenchResult(
                task_id="T1",
                mode="mdtools",
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

        with tempfile.TemporaryDirectory(prefix="bench_report_bundle_") as tmpdir:
            write_run_artifacts(
                tmpdir,
                metadata=metadata,
                results=results,
                selected_task_ids=["T1"],
            )

            completed = subprocess.run(
                [sys.executable, "bench/report.py", tmpdir],
                capture_output=True,
                text=True,
                cwd=repo_root,
                check=False,
            )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("Run context:", completed.stdout)
        self.assertIn("model=claude-haiku-test", completed.stdout)
        self.assertIn("selection=bench/search/task_ids.json", completed.stdout)
        self.assertIn("T1", completed.stdout)
        self.assertIn("Per-task sample count: N=1 per task/mode", completed.stdout)

    def test_report_markdown_uses_real_sample_count(self) -> None:
        repo_root = Path(__file__).resolve().parent.parent
        results = [
            BenchResult(
                task_id="T1",
                mode="mdtools",
                correct=True,
                correct_neutral=True,
                tool_calls=2,
                elapsed_seconds=1.5,
            )
        ]
        metadata = build_run_metadata(
            run_kind="dry-run",
            tasks_path="bench/tasks/tasks.json",
            task_ids_path=None,
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

        with tempfile.TemporaryDirectory(prefix="bench_report_markdown_n1_") as tmpdir:
            write_run_artifacts(
                tmpdir,
                metadata=metadata,
                results=results,
                selected_task_ids=["T1"],
            )

            completed = subprocess.run(
                [sys.executable, "bench/report.py", tmpdir, "--markdown"],
                capture_output=True,
                text=True,
                cwd=repo_root,
                check=False,
            )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("### Per-task results (N=1 per task/mode)", completed.stdout)
        self.assertNotIn("N=3", completed.stdout)

    def test_report_surfaces_runner_errors_from_run_bundle(self) -> None:
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

        with tempfile.TemporaryDirectory(prefix="bench_report_runner_error_") as tmpdir:
            write_run_artifacts(
                tmpdir,
                metadata=metadata,
                results=results,
                selected_task_ids=["T1"],
            )

            completed = subprocess.run(
                [sys.executable, "bench/report.py", tmpdir, "--markdown"],
                capture_output=True,
                text=True,
                cwd=repo_root,
                check=False,
            )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("### Runner errors", completed.stdout)
        self.assertIn("authentication_failed: Not logged in · Please run /login", completed.stdout)
        self.assertIn("| mdtools | 1 | T1 |", completed.stdout)

    def test_report_accepts_dry_run_text_output(self) -> None:
        repo_root = Path(__file__).resolve().parent.parent
        dry_run_output = """=== DRY RUN: dual scorer validation ===

  T1: md=PASS neutral=PASS
    heading_tree [md]: OK
    heading_tree [neutral]: OK
  T2: md=FAIL neutral=FAIL
    heading_tree [md]: FAIL
    heading_tree [neutral]: FAIL

SCORER ISSUES DETECTED.
"""

        with tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            prefix="bench_report_dry_run_",
            suffix=".txt",
            delete=False,
        ) as handle:
            tmp_path = Path(handle.name)
            handle.write(dry_run_output)

        try:
            completed = subprocess.run(
                [sys.executable, "bench/report.py", str(tmp_path)],
                capture_output=True,
                text=True,
                cwd=repo_root,
                check=False,
            )
        finally:
            tmp_path.unlink(missing_ok=True)

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertNotIn("No results found.", completed.stdout)
        self.assertIn("mdtools", completed.stdout)
        self.assertIn("T1", completed.stdout)
        self.assertIn("T2", completed.stdout)
        self.assertRegex(completed.stdout, re.compile(r"^T2\s+0%\s+0s\s+0\.0", re.MULTILINE))

    def test_report_separates_same_model_across_runners(self) -> None:
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

        with tempfile.TemporaryDirectory(prefix="bench_report_claude_") as claude_dir, \
                tempfile.TemporaryDirectory(prefix="bench_report_oai_") as oai_dir:
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
                [sys.executable, "bench/report.py", claude_dir, oai_dir],
                capture_output=True,
                text=True,
                cwd=repo_root,
                check=False,
            )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        stdout = completed.stdout
        self.assertIn(f"GROUP: {model_id} [claude-cli]", stdout)
        self.assertIn(f"GROUP: {model_id} [oai-loop]", stdout)
        self.assertIn("CROSS-GROUP SUMMARY", stdout)

        # Assert per-group T1 lines are distinct (one PASS at 100%, one FAIL at 0%).
        claude_section = stdout.split("GROUP: " + f"{model_id} [claude-cli]")[1]
        claude_section = claude_section.split("GROUP: ")[0]
        oai_section = stdout.split("GROUP: " + f"{model_id} [oai-loop]")[1]
        oai_section = oai_section.split("GROUP: ")[0].split("CROSS-GROUP SUMMARY")[0]
        self.assertRegex(claude_section, re.compile(r"^T1\s+100%", re.MULTILINE))
        self.assertRegex(oai_section, re.compile(r"^T1\s+0%", re.MULTILINE))

    def test_report_accepts_text_runner_error_suffixes(self) -> None:
        repo_root = Path(__file__).resolve().parent.parent
        agent_output = """=== MODE: mdtools (N=1, model=claude-sonnet-test) ===

  T1: summarize headings
    md=FAIL neutral=FAIL | 0.38s | ~2372B out | obs:0B | ~0 calls | 0 mut | deny:0 | err:authentication_failed: Not logged in · Please run /login
"""

        with tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            prefix="bench_report_runner_error_",
            suffix=".txt",
            delete=False,
        ) as handle:
            tmp_path = Path(handle.name)
            handle.write(agent_output)

        try:
            completed = subprocess.run(
                [sys.executable, "bench/report.py", str(tmp_path)],
                capture_output=True,
                text=True,
                cwd=repo_root,
                check=False,
            )
        finally:
            tmp_path.unlink(missing_ok=True)

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("Runner errors:", completed.stdout)
        self.assertIn("authentication_failed: Not logged in · Please run /login", completed.stdout)
        self.assertIn("mdtools x1 [T1]", completed.stdout)


if __name__ == "__main__":
    unittest.main()
