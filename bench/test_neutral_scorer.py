from __future__ import annotations

import unittest
from unittest.mock import patch

from bench import harness
from bench.harness import StructuralDiffPolicy, score_normalized_text_neutral, score_task
from bench.neutral_scorer import neutral_block_texts, neutral_heading_tree


def text_policy() -> StructuralDiffPolicy:
    return StructuralDiffPolicy(
        kind="normalized_text",
        normalize_line_endings=True,
        ignore_trailing_whitespace=True,
        compare_frontmatter_json=False,
        compare_heading_tree=False,
        compare_block_order=False,
        compare_link_destinations=False,
        compare_block_text=True,
    )


class NeutralScorerTests(unittest.TestCase):
    def test_known_good_document_scores_with_neutral_helpers(self) -> None:
        doc = "# Title\n\n- first\n- second\n"
        self.assertEqual(neutral_heading_tree(doc), [(1, "Title")])
        self.assertEqual(neutral_block_texts(doc), ["# Title", "- first\n- second"])

    def test_negative_control_dropped_heading_fails_neutral_scorer(self) -> None:
        expected = "# Configuration\n\nSettings live here.\n"
        actual = "Configuration\n\nSettings live here.\n"
        self.assertFalse(score_normalized_text_neutral(text_policy(), actual, expected, []))

    def test_divergent_verdict_creates_quarantine_entry(self) -> None:
        correct, verdict, quarantine = harness._verdict_fields(
            task_id="T8",
            mode="hybrid",
            primary=False,
            diagnostic=True,
        )
        self.assertIsNone(correct)
        self.assertEqual(verdict, "divergent")
        self.assertEqual(quarantine["task_id"], "T8")
        self.assertEqual(quarantine["reason"], "scorer_divergence")

    def test_diagnostic_scorer_error_is_divergent(self) -> None:
        expected = "# Configuration\n\nSettings live here.\n"
        with patch.object(harness, "score_normalized_text_md", side_effect=RuntimeError("boom")):
            primary, diagnostic, report = score_task(
                type("Task", (), {"scorer": text_policy(), "expected_artifact": "file_contents"})(),
                expected,
                expected,
                md_binary="md",
            )
        self.assertTrue(primary)
        self.assertIsNone(diagnostic)
        self.assertIn("diagnostic_md_error", report)
        correct, verdict, quarantine = harness._verdict_fields(
            task_id="T1",
            mode="hybrid",
            primary=primary,
            diagnostic=diagnostic,
        )
        self.assertIsNone(correct)
        self.assertEqual(verdict, "divergent")
        self.assertIsNotNone(quarantine)


if __name__ == "__main__":
    unittest.main()
