from __future__ import annotations

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


if __name__ == "__main__":
    unittest.main()
