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

    def test_text_parse_keeps_native_md_mode_intact(self) -> None:
        # FRAC-194 review #3 (the regex fix), pinned through the actual text parser:
        # a "=== MODE: native+md" header must yield mode "native+md", not truncate to
        # "native" at the '+'. The OLD [\w-]+ regex dropped the '+'; existing fixtures
        # only used hybrid-no-md (which [\w-]+ already matched), so nothing caught it.
        from bench.report import parse_text_results
        text = (
            "=== MODE: native+md (N=3, model=claude-sonnet-4-6, thinking=off) ===\n"
            "  T7 run 1/3:\n"
            "    md=PASS | 1.5s | ~100B out | obs:50B | ~3 calls | 1 mut | deny:0\n"
        )
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", prefix="bench_native_md_mode_",
                                         delete=False, encoding="utf-8") as h:
            tmp_path = Path(h.name)
            h.write(text)
        try:
            recs = parse_text_results(str(tmp_path))
        finally:
            tmp_path.unlink(missing_ok=True)
        self.assertEqual([r["mode"] for r in recs], ["native+md"])   # not "native"

    def test_native_only_report_suppresses_spurious_posix_verdict_row(self) -> None:
        # FRAC-194 review #5: a native-only run has no unix/hybrid data, so the
        # POSIX-rooted verdict would render a spurious "OPEN:loses-unix" row against a
        # baseline that was never run. It must be suppressed; the ·native-arm row stays.
        from bench.report import render_cost_slice

        def rec(task: str, mode: str, cost: int) -> dict:
            return {"task_id": task, "mode": mode, "model": "claude-sonnet-4-6",
                    "thinking_level": None, "correct": True,
                    "tokens_in": cost, "tokens_out": 0, "tool_calls": 0}

        recs: list[dict] = []
        for t in ("T7", "T10"):
            recs += [rec(t, "native", 80000), rec(t, "native+md", 50000),
                     rec(t, "native+md-no-md", 80000)]
        out = render_cost_slice(recs, ["native", "native+md", "native+md-no-md"], markdown=True)
        self.assertIn("·native-arm", out)        # the real native verdict still renders
        self.assertNotIn("loses-unix", out)       # the spurious POSIX row is gone

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

    def test_report_parses_hyphenated_hybrid_no_md_mode_header(self) -> None:
        # Regression (PR#10 Codex P2): the attribution ablation mode name contains a
        # hyphen, so the text-log MODE-header regex must accept it. The old `(\w+)`
        # matched only `hybrid`, the record parsed mode=None, and the no-md baseline
        # silently dropped out of the attribution verdicts (or crashed mode sorting).
        repo_root = Path(__file__).resolve().parent.parent
        agent_output = """=== MODE: hybrid-no-md (N=3, model=claude-sonnet-test) ===

  T1: summarize headings
    md=PASS neutral=PASS | 1.00s | ~100B out | obs:0B | ~2 calls | 0 mut | deny:0
"""

        with tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            prefix="bench_report_hybrid_no_md_",
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
        self.assertNotIn("No results found.", completed.stdout)
        self.assertIn("hybrid-no-md", completed.stdout)
        self.assertIn("T1", completed.stdout)

    def test_cost_slice_aggregates_replicates_not_last(self) -> None:
        # Regression (PR#10 Codex P2): render_cost_slice must aggregate -N>1 replicates
        # (median per task/mode) BEFORE the both-passed intersection, mirroring
        # attribution_verdict. The old direct intersection_cost kept only the LAST
        # replicate per (task,mode), making the headline cost table order-dependent and
        # inconsistent with the gating verdict for N>=3 runs.
        from bench.report import render_cost_slice

        def rec(mode: str, cost: int) -> dict:
            return {
                "task_id": "T1",
                "mode": mode,
                "model": "claude-sonnet-test",
                "thinking_level": None,
                "correct": True,
                "tokens_in": cost,
                "tokens_out": 0,
                "tool_calls": 0,
            }

        # hybrid replicates [100, 100, 999]: median 100, last 999.
        records = [rec("unix", 200) for _ in range(3)] + [
            rec("hybrid", c) for c in (100, 100, 999)
        ]
        out = render_cost_slice(records, ["unix", "hybrid"])
        cost_line = next(line for line in out.splitlines() if "Extraction" in line)
        self.assertIn(
            " 100 ", cost_line,
            f"hybrid cost should be the replicate median (100), got: {cost_line}",
        )
        self.assertNotIn(
            "999", cost_line, "must not report the last replicate (999)"
        )

    def _verdict_rows(self, mode_costs: list[tuple[str, int]]) -> str:
        from bench.report import render_cost_slice
        recs = []
        for t in ("T7", "T10"):
            for m, c in mode_costs:
                recs.append({"task_id": t, "task": t, "mode": m, "model": "claude-opus-4-8",
                             "correct": True, "tokens_in": c, "tokens_out": 0,
                             "md_probe_count": 1 if m.endswith("no-md") else 0,
                             "thinking_level": None, "runner": "claude-cli"})
        modes = [m for m, _ in mode_costs]
        return render_cost_slice(recs, modes, markdown=True)

    def test_native_arm_renders_distinct_verdict_row(self) -> None:
        # U6 (FRAC-194): POSIX no-lift + native CLOSES (md cheaper than native) →
        # two distinct rows, the native one tagged ·native-arm.
        out = self._verdict_rows([
            ("unix", 80000), ("hybrid", 80000), ("hybrid-no-md", 80000),
            ("native", 80000), ("native+md", 50000), ("native+md-no-md", 80000),
        ])
        self.assertIn("·native-arm", out)
        self.assertIn("CLOSES", out)
        self.assertIn("no-lift", out)

    def test_no_native_data_has_no_native_arm_row(self) -> None:
        out = self._verdict_rows([("unix", 80000), ("hybrid", 50000), ("hybrid-no-md", 80000)])
        self.assertNotIn("·native-arm", out)


if __name__ == "__main__":
    unittest.main()
