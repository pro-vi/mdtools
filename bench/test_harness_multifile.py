import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from bench.harness import BenchTask, StructuralDiffPolicy, score_multifile_any
from bench.multifile_drift import format_drift_proof, summarize_drift_proof


REPO = Path(__file__).resolve().parent.parent


def raw_policy() -> StructuralDiffPolicy:
    return StructuralDiffPolicy(
        kind="raw_bytes",
        normalize_line_endings=True,
        ignore_trailing_whitespace=True,
        compare_frontmatter_json=False,
        compare_heading_tree=False,
        compare_block_order=False,
        compare_link_destinations=False,
        compare_block_text=False,
    )


class HarnessMultifileTests(unittest.TestCase):
    def test_multifile_scorer_accepts_any_expected_alternative(self) -> None:
        with tempfile.TemporaryDirectory(prefix="bench_multifile_scorer_") as tmpdir:
            root = Path(tmpdir)
            actual = root / "actual" / "mf"
            safe = root / "expected" / "safe-fail" / "mf"
            retry = root / "expected" / "retry" / "mf"
            actual.mkdir(parents=True)
            safe.mkdir(parents=True)
            retry.mkdir(parents=True)

            (actual / "alpha.md").write_text("# Alpha\n\nretry\n")
            (actual / "beta.md").write_text("# Beta\n\nretry\n")
            (safe / "alpha.md").write_text("# Alpha\n\nsafe\n")
            (safe / "beta.md").write_text("# Beta\n\nsafe\n")
            (retry / "alpha.md").write_text("# Alpha\n\nretry\n")
            (retry / "beta.md").write_text("# Beta\n\nretry\n")

            task = BenchTask(
                id="MF-TEST",
                description="test",
                input_files=[
                    "bench/regimes/multifile/inputs/mf/alpha.md",
                    "bench/regimes/multifile/inputs/mf/beta.md",
                ],
                expected_output=str(root / "expected"),
                expected_artifact="multi_file_contents_any",
                difficulty="test",
                scorer=raw_policy(),
            )

            ok_primary, ok_diagnostic, report = score_multifile_any(
                task,
                actual_root=root / "actual",
                expected_output=task.expected_output,
                md_binary="md",
            )

        self.assertTrue(ok_primary, report)
        self.assertTrue(ok_diagnostic, report)
        self.assertIn("matched retry", report)

    def test_multifile_scorer_rejects_partial_match(self) -> None:
        with tempfile.TemporaryDirectory(prefix="bench_multifile_scorer_") as tmpdir:
            root = Path(tmpdir)
            actual = root / "actual" / "mf"
            retry = root / "expected" / "retry" / "mf"
            actual.mkdir(parents=True)
            retry.mkdir(parents=True)

            (actual / "alpha.md").write_text("# Alpha\n\nretry\n")
            (actual / "beta.md").write_text("# Beta\n\nwrong\n")
            (retry / "alpha.md").write_text("# Alpha\n\nretry\n")
            (retry / "beta.md").write_text("# Beta\n\nretry\n")

            task = BenchTask(
                id="MF-TEST",
                description="test",
                input_files=[
                    "bench/regimes/multifile/inputs/mf/alpha.md",
                    "bench/regimes/multifile/inputs/mf/beta.md",
                ],
                expected_output=str(root / "expected"),
                expected_artifact="multi_file_contents_any",
                difficulty="test",
                scorer=raw_policy(),
            )

            ok_primary, ok_diagnostic, report = score_multifile_any(
                task,
                actual_root=root / "actual",
                expected_output=task.expected_output,
                md_binary="md",
            )

        self.assertFalse(ok_primary, report)
        self.assertFalse(ok_diagnostic, report)
        self.assertIn("no alternative matched", report)

    def test_multifile_drift_injector_fires_once(self) -> None:
        injector = REPO / "probes" / "multifile" / "drift_injector.py"
        for task_id in ("mf01", "mf02"):
            with self.subTest(task_id=task_id):
                with tempfile.TemporaryDirectory(prefix="bench_multifile_drift_") as tmpdir:
                    root = Path(tmpdir)
                    shutil.copytree(
                        REPO / "bench" / "regimes" / "multifile" / "inputs" / task_id,
                        root / task_id,
                    )

                    first = subprocess.run(
                        [
                            sys.executable,
                            str(injector),
                            "--root",
                            str(root),
                            "--task",
                            task_id,
                        ],
                        check=True,
                        capture_output=True,
                        text=True,
                    )
                    after_first = {
                        path.relative_to(root): path.read_text()
                        for path in sorted((root / task_id).glob("*.md"))
                    }
                    second = subprocess.run(
                        [
                            sys.executable,
                            str(injector),
                            "--root",
                            str(root),
                            "--task",
                            task_id,
                        ],
                        check=True,
                        capture_output=True,
                        text=True,
                    )
                    after_second = {
                        path.relative_to(root): path.read_text()
                        for path in sorted((root / task_id).glob("*.md"))
                    }

                    expected_root = (
                        REPO / "bench" / "regimes" / "multifile" / "drifted" / task_id
                    )
                    expected = {
                        path.relative_to(expected_root): path.read_text()
                        for path in sorted((expected_root / task_id).glob("*.md"))
                    }

                self.assertEqual(first.stdout.strip(), "fired")
                self.assertEqual(second.stdout.strip(), "already-fired")
                self.assertEqual(after_first, expected)
                self.assertEqual(after_second, expected)

    def test_multifile_drift_proof_requires_read_drift_mutation_order(self) -> None:
        proof = summarize_drift_proof(
            [
                {"event": "multifile_drift_config", "details": {"enabled": True}},
                {"event": "tool_result", "toolName": "read"},
                {"event": "multifile_drift_observed_read"},
                {"event": "multifile_drift_fired", "details": {"outcome": "fired"}},
                {
                    "event": "multifile_drift_target_mutation",
                    "details": {"afterObservedRead": True, "afterDrift": True},
                },
            ]
        )

        self.assertTrue(proof.valid, format_drift_proof(proof))
        self.assertIn("multifile_drift_proof: OK", format_drift_proof(proof))

    def test_multifile_drift_proof_rejects_mutation_before_read(self) -> None:
        proof = summarize_drift_proof(
            [
                {"event": "multifile_drift_config", "details": {"enabled": True}},
                {
                    "event": "multifile_drift_target_mutation",
                    "details": {"afterObservedRead": False, "afterDrift": False},
                },
                {
                    "event": "multifile_drift_invalid",
                    "reason": "target mutation before observed target read",
                },
            ]
        )

        self.assertFalse(proof.valid)
        self.assertIn("target mutation before observed target read", format_drift_proof(proof))


if __name__ == "__main__":
    unittest.main()
