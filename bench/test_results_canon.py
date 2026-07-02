import unittest
from pathlib import Path

from bench import results_canon


REPO = Path(__file__).resolve().parent.parent


class BenchV3RetractionTests(unittest.TestCase):
    def test_results_canon_renders_archived_v2_banner(self) -> None:
        doc = results_canon.render("2026-07-01")
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


if __name__ == "__main__":
    unittest.main()
