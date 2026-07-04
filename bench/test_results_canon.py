import json
import tempfile
import unittest
from pathlib import Path

from bench import results_canon
from bench.v3_manifest import load_manifest


REPO = Path(__file__).resolve().parent.parent


def _core_task_ids() -> list[str]:
    tasks = json.loads((REPO / "bench" / "tasks" / "tasks.json").read_text())
    return [
        task["id"]
        for task in tasks
        if task.get("provenance", "core") == "core"
    ]


def _complete_haiku_shell_rows() -> list[dict]:
    rows = []
    for task_id in _core_task_ids():
        for mode in ["unix", "hybrid", "hybrid-no-md"]:
            for run_index in range(5):
                correct = mode == "hybrid" and run_index == 0
                rows.append({
                    "task_id": task_id,
                    "mode": mode,
                    "model": "claude-haiku-4-5-20251001",
                    "runner": "claude-cli",
                    "run_index": run_index,
                    "correct": correct,
                    "verdict": "pass" if correct else "fail",
                    "tool_calls": run_index + 1,
                })
    for mode in ["unix", "hybrid"]:
        for run_index in range(5):
            rows.append({
                "task_id": "C-T10-15",
                "mode": mode,
                "model": "claude-haiku-4-5-20251001",
                "runner": "claude-cli",
                "run_index": run_index,
                "correct": mode == "hybrid",
                "verdict": "pass" if mode == "hybrid" else "fail",
                "tool_calls": run_index + 1,
            })
    return rows


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
        self.assertIn("directional/exploratory rather than confirmed", readme)
        self.assertIn("failed", readme)
        self.assertIn("+28.3pp", readme)
        self.assertIn("below the frozen +15pp floor", readme)
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
            rows = _complete_haiku_shell_rows()
            (bundle / "results.json").write_text(json.dumps(rows))

            doc = results_canon.render_v3("2026-07-01", [bundle])

        self.assertIn("## Core Tasks", doc)
        self.assertIn("## Headline Status", doc)
        self.assertIn("Manifest threshold", doc)
        self.assertIn("Lift pass@1 mean", doc)
        self.assertIn("Exact p", doc)
        self.assertIn("FAIL (downgrade)", doc)
        self.assertIn("## Interpretation Notes", doc)
        self.assertIn("## Variance Decomposition", doc)
        self.assertIn("Task variance term", doc)
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

    def test_v3_renderer_routes_incomplete_headline_bundle_to_exploratory(self) -> None:
        manifest = load_manifest()
        with tempfile.TemporaryDirectory(prefix="bench_v3_partial_") as tmpdir:
            bundle = Path(tmpdir) / "bundle"
            bundle.mkdir()
            (bundle / "run.json").write_text(json.dumps({
                "runner": "claude-cli",
                "model": "claude-haiku-4-5-20251001",
                "modes": ["unix", "hybrid", "hybrid-no-md"],
                "trials_per_cell": 5,
                "task_file_sha256": manifest["task_file_sha256"],
                "prompt_template_sha256": manifest["prompt_template_sha256"],
                "scorer_version": manifest["scorer_version"],
            }))
            (bundle / "results.json").write_text(json.dumps([
                {
                    "task_id": "T1",
                    "mode": "unix",
                    "model": "claude-haiku-4-5-20251001",
                    "runner": "claude-cli",
                    "run_index": 0,
                    "correct": True,
                    "verdict": "pass",
                }
            ]))
            doc = results_canon.render_v3("2026-07-01", [bundle])

        self.assertIn("No headline-eligible v3 run bundles", doc)
        self.assertIn("## Exploratory Bundles", doc)
        self.assertIn("incomplete results", doc)
        self.assertIn("missing 359 required core trials", doc)

    def test_v3_renderer_renders_interpretation_notes(self) -> None:
        with tempfile.TemporaryDirectory(prefix="bench_v3_notes_") as tmpdir:
            root = Path(tmpdir)
            bundle = root / "bundle"
            bundle.mkdir()
            manifest = load_manifest()
            (bundle / "run.json").write_text(json.dumps({
                "runner": "claude-cli",
                "model": "claude-haiku-4-5-20251001",
                "modes": ["unix", "hybrid", "hybrid-no-md"],
                "trials_per_cell": 5,
                "task_file_sha256": manifest["task_file_sha256"],
                "prompt_template_sha256": manifest["prompt_template_sha256"],
                "scorer_version": manifest["scorer_version"],
            }))
            rows = _complete_haiku_shell_rows()
            (bundle / "results.json").write_text(json.dumps(rows))
            notes = root / "adjudications.json"
            notes.write_text(json.dumps([
                {
                    "type": "interpretation_note",
                    "date": "2026-07-03",
                    "topic": "tool_error classification",
                    "summary": "tool_error rows are scored model behavior.",
                    "affected_rows": {"bundle": {"native": 1}},
                    "unaffected_bundles": ["haiku"],
                }
            ]))
            doc = results_canon.render_v3(
                "2026-07-03",
                [bundle],
                adjudications_path=notes,
            )
        self.assertIn("2026-07-03", doc)
        self.assertIn("tool_error rows are scored model behavior", doc)
        self.assertIn("bundle: native 1", doc)

    def test_v3_renderer_renders_failure_taxonomy(self) -> None:
        with tempfile.TemporaryDirectory(prefix="bench_v3_taxonomy_") as tmpdir:
            root = Path(tmpdir)
            bundle = root / "bundle"
            bundle.mkdir()
            manifest = load_manifest()
            (bundle / "run.json").write_text(json.dumps({
                "runner": "claude-cli",
                "model": "claude-haiku-4-5-20251001",
                "modes": ["unix", "hybrid", "hybrid-no-md"],
                "trials_per_cell": 5,
                "task_file_sha256": manifest["task_file_sha256"],
                "prompt_template_sha256": manifest["prompt_template_sha256"],
                "scorer_version": manifest["scorer_version"],
            }))
            (bundle / "results.json").write_text(json.dumps(_complete_haiku_shell_rows()))
            taxonomy = root / "failure_taxonomy.json"
            taxonomy.write_text(json.dumps({
                "classes": ["wrong-target", "format-noncompliance"],
                "double_label": {
                    "population_size": 15,
                    "sample_size": 3,
                    "agreements": 3,
                    "agreement_rate": 1.0,
                    "target_agreement_rate": 0.8,
                },
                "counts": [
                    {
                        "model": "claude-haiku-4-5-20251001",
                        "runner": "claude-cli",
                        "mode": "unix",
                        "failed_trials": 10,
                        "classes": {"wrong-target": 7, "format-noncompliance": 3},
                    },
                    {
                        "model": "claude-haiku-4-5-20251001",
                        "runner": "claude-cli",
                        "mode": "hybrid",
                        "failed_trials": 5,
                        "classes": {"wrong-target": 5, "format-noncompliance": 0},
                    },
                ],
            }))

            doc = results_canon.render_v3(
                "2026-07-03",
                [bundle],
                failure_taxonomy_path=taxonomy,
            )

        self.assertIn("## Mechanism Evidence (Exploratory)", doc)
        self.assertIn("Double-label agreement: 3/3 = 100.0%", doc)
        self.assertIn("### Failure Class Counts", doc)
        self.assertIn("Haiku 4.5 `unix` -> `hybrid`", doc)
        self.assertIn("format-noncompliance", doc)


if __name__ == "__main__":
    unittest.main()
