import json
import tempfile
import unittest
from pathlib import Path

from bench import results_canon
from bench.v3_manifest import load_manifest


REPO = Path(__file__).resolve().parent.parent


class BenchV3RetractionTests(unittest.TestCase):
    def test_results_canon_renders_archived_v2_banner(self) -> None:
        doc = results_canon.render("2026-07-01")
        self.assertIn("mdtools benchmark v3", doc)
        self.assertIn("ARCHIVED (v2)", doc)
        self.assertIn("should not be cited as current evidence", doc)
        self.assertIn("bench/V3.md", doc)

    def test_readme_retracts_pre_v3_headlines(self) -> None:
        readme = (REPO / "README.md").read_text()
        self.assertIn("BENCH_V3_RETRACTION", readme)
        self.assertIn("No v3 headline numbers have shipped yet", readme)
        self.assertNotIn("+43pp", readme)
        self.assertNotIn("54%", readme)

    def test_v3_protocol_exists(self) -> None:
        v3 = (REPO / "bench" / "V3.md").read_text()
        self.assertIn("at least five trials per task/mode", v3)
        self.assertIn("Scorer divergence blocks headline publication", v3)

    def test_v3_renderer_renders_synthetic_bundle_sections(self) -> None:
        with tempfile.TemporaryDirectory(prefix="bench_v3_canon_") as tmpdir:
            manifest = load_manifest()
            bundle = Path(tmpdir) / "bundle"
            bundle.mkdir()
            (bundle / "run.json").write_text(json.dumps({
                "runner": "claude-cli",
                "model": "claude-haiku-4-5-20251001",
                "modes": ["unix", "hybrid", "hybrid-no-md"],
                "trials_per_cell": 5,
                "temperature_policy": "provider default (temperature not exposed by claude-cli)",
                "holdout_version": 2,
                "task_file_sha256": manifest["task_file_sha256"],
                "prompt_template_sha256": manifest["prompt_template_sha256"],
                "scorer_version": manifest["scorer_version"],
            }))
            rows = []
            for task_id in ["T1", "T2", "C-T10-15"]:
                for mode in ["unix", "hybrid"]:
                    for run_index in range(5):
                        rows.append({
                            "task_id": task_id,
                            "mode": mode,
                            "model": "claude-haiku-4-5-20251001",
                            "run_index": run_index,
                            "correct": mode == "hybrid" or run_index < 2,
                            "verdict": "pass" if mode == "hybrid" or run_index < 2 else "fail",
                            "tool_calls": run_index + 1,
                        })
            (bundle / "results.json").write_text(json.dumps(rows))

            doc = results_canon.render_v3("2026-07-01", [bundle])

        self.assertIn("## Core Tasks", doc)
        self.assertIn("## Adversarially Mined Tasks", doc)
        self.assertIn("Mean pass@1", doc)
        self.assertIn("pass^k", doc)
        self.assertIn("Cost-vs-Success Frontier", doc)
        self.assertIn("Harness Card", doc)
        self.assertIn("claude-haiku-4-5-20251001", doc)
        self.assertIn("| claude-haiku-4-5-20251001 | `claude-cli` | `hybrid` |", doc)

    def test_v3_renderer_blocks_unadjudicated_quarantine(self) -> None:
        with tempfile.TemporaryDirectory(prefix="bench_v3_quarantine_") as tmpdir:
            root = Path(tmpdir)
            bundle = root / "bundle"
            bundle.mkdir()
            (bundle / "run.json").write_text(json.dumps({
                "runner": "oai-loop",
                "model": "Qwen3.6-35B-A3B-8bit",
                "trials_per_cell": 5,
            }))
            (bundle / "results.json").write_text(json.dumps([
                {
                    "task_id": "T1",
                    "mode": "hybrid",
                    "model": "Qwen3.6-35B-A3B-8bit",
                    "runner": "oai-loop",
                    "run_index": 0,
                    "correct": None,
                    "verdict": "divergent",
                }
            ]))
            adjudications = root / "adjudications.json"
            adjudications.write_text("[]")
            with self.assertRaisesRegex(results_canon.CanonBlockedError, "T1:hybrid:run0"):
                results_canon.render_v3(
                    "2026-07-01",
                    [bundle],
                    adjudications_path=adjudications,
                )

    def test_v3_renderer_routes_wrong_n_to_exploratory(self) -> None:
        manifest = load_manifest()
        with tempfile.TemporaryDirectory(prefix="bench_v3_exploratory_") as tmpdir:
            bundle = Path(tmpdir) / "bundle"
            bundle.mkdir()
            (bundle / "run.json").write_text(json.dumps({
                "runner": "oai-loop",
                "model": "Qwen3.6-35B-A3B-8bit",
                "modes": ["unix", "mdtools", "hybrid"],
                "trials_per_cell": 1,
                "task_file_sha256": manifest["task_file_sha256"],
                "prompt_template_sha256": manifest["prompt_template_sha256"],
                "scorer_version": manifest["scorer_version"],
            }))
            (bundle / "results.json").write_text(json.dumps([
                {
                    "task_id": "T1",
                    "mode": "hybrid",
                    "model": "Qwen3.6-35B-A3B-8bit",
                    "runner": "oai-loop",
                    "run_index": 0,
                    "correct": True,
                    "verdict": "pass",
                }
            ]))
            doc = results_canon.render_v3("2026-07-01", [bundle])
        self.assertIn("## Exploratory Bundles", doc)
        self.assertIn("wrong N", doc)


if __name__ == "__main__":
    unittest.main()
