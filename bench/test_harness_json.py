from __future__ import annotations

import json
import unittest

from bench.harness import (
    StructuralDiffPolicy,
    extract_last_json,
    parse_agent_output,
    score_structural_json,
    select_json_envelope_actual,
)


def _link_only_policy() -> StructuralDiffPolicy:
    return StructuralDiffPolicy(
        kind="structural",
        normalize_line_endings=True,
        ignore_trailing_whitespace=True,
        compare_frontmatter_json=False,
        compare_heading_tree=False,
        compare_block_order=False,
        compare_link_destinations=True,
        compare_block_text=False,
    )


class StructuralLinkEnvelopeTests(unittest.TestCase):
    """F3 closure — link-extraction scorer accepts both list and dict shapes."""

    def test_top_level_list_equivalent_to_links_envelope(self) -> None:
        expected = json.dumps({"links": [
            {"kind": "Inline", "destination": "https://a"},
            {"kind": "Inline", "destination": "https://b"},
        ]})
        actual = json.dumps([
            {"kind": "Inline", "destination": "https://a"},
            {"kind": "Inline", "destination": "https://b"},
        ])
        report: list[str] = []
        ok_md, ok_neutral = score_structural_json(
            _link_only_policy(), actual, expected, report
        )
        self.assertTrue(ok_md, report)
        self.assertTrue(ok_neutral, report)

    def test_top_level_list_with_mismatched_links_still_fails(self) -> None:
        expected = json.dumps({"links": [
            {"kind": "Inline", "destination": "https://a"},
        ]})
        actual = json.dumps([
            {"kind": "Inline", "destination": "https://wrong"},
        ])
        report: list[str] = []
        ok_md, ok_neutral = score_structural_json(
            _link_only_policy(), actual, expected, report
        )
        self.assertFalse(ok_md)
        self.assertFalse(ok_neutral)
        self.assertTrue(any("link_destinations" in line for line in report))

    def test_dict_envelope_still_accepted(self) -> None:
        payload = {"links": [{"kind": "Inline", "destination": "https://a"}]}
        report: list[str] = []
        ok_md, ok_neutral = score_structural_json(
            _link_only_policy(), json.dumps(payload), json.dumps(payload), report
        )
        self.assertTrue(ok_md, report)
        self.assertTrue(ok_neutral, report)

    def test_top_level_list_rejected_when_other_checks_required(self) -> None:
        policy = StructuralDiffPolicy(
            kind="structural",
            normalize_line_endings=True,
            ignore_trailing_whitespace=True,
            compare_frontmatter_json=False,
            compare_heading_tree=True,  # additional check
            compare_block_order=False,
            compare_link_destinations=True,
            compare_block_text=False,
        )
        actual = json.dumps([{"kind": "Inline", "destination": "https://a"}])
        expected = json.dumps({"links": [{"kind": "Inline", "destination": "https://a"}]})
        report: list[str] = []
        ok_md, ok_neutral = score_structural_json(policy, actual, expected, report)
        self.assertFalse(ok_md)
        self.assertFalse(ok_neutral)
        self.assertTrue(
            any("expected top-level JSON object" in line for line in report),
            report,
        )


class HarnessJsonExtractionTests(unittest.TestCase):
    def test_extract_last_json_preserves_top_level_object(self) -> None:
        payload = json.dumps({"file": "doc.md", "entries": [{"heading": {"text": "One"}}]})
        self.assertEqual(json.loads(extract_last_json(payload)), json.loads(payload))

    def test_parse_agent_output_surfaces_runner_auth_failure(self) -> None:
        payload = json.dumps([
            {
                "type": "system",
                "subtype": "init",
                "model": "claude-sonnet-4-6",
            },
            {
                "type": "assistant",
                "message": {
                    "content": [
                        {"type": "text", "text": "Not logged in · Please run /login"},
                    ]
                },
                "error": "authentication_failed",
            },
            {
                "type": "result",
                "is_error": True,
                "num_turns": 1,
                "result": "Not logged in · Please run /login",
            },
        ])

        parsed = parse_agent_output(payload)

        self.assertEqual(parsed.model, "claude-sonnet-4-6")
        self.assertEqual(
            parsed.runner_error,
            "authentication_failed: Not logged in · Please run /login",
        )
        self.assertEqual(parsed.turns, 1)
        self.assertEqual(parsed.tool_calls, 0)
        self.assertIn("Not logged in", parsed.stdout)


class JsonEnvelopeActualSelectionTests(unittest.TestCase):
    """F4 closure (iter 30) — schema-aware tool-output preference for the
    json_envelope branch. Pins that the iter-29 T16 PI-bundle FAIL would
    now PASS, while existing PASS shapes (T1, T9 jq projection, T22) are
    preserved."""

    def test_t16_shape_text_wins_over_intermediate_md_tasks_envelope(self) -> None:
        tool_envelope = json.dumps({
            "schema_version": "mdtools.v1",
            "results": [{"task_id": "x", "status": "pending"}],
        })
        text_answer = json.dumps({
            "total_pending": 9,
            "files": [{"file": "backend.md", "pending": 3}],
        })
        expected = json.dumps({
            "total_pending": 9,
            "files": [{"file": "backend.md", "pending": 3}],
        })
        actual = select_json_envelope_actual(
            [tool_envelope], [text_answer], "", expected
        )
        self.assertEqual(json.loads(actual), json.loads(expected))

    def test_t9_shape_jq_projected_tool_output_wins(self) -> None:
        tool_envelope = json.dumps({
            "schema_version": "mdtools.v1",
            "results": [{"loc": "5.1", "summary_text": "x"}],
        })
        tool_projected = json.dumps([
            {"loc": "5.1", "nearest_heading": "Phase 0", "summary_text": "x"},
        ])
        expected = json.dumps([
            {"loc": "5.1", "nearest_heading": "Phase 0", "summary_text": "x"},
        ])
        # jq projection arrives after the raw envelope in tool-output order.
        actual = select_json_envelope_actual(
            [tool_envelope, tool_projected], [], "", expected
        )
        self.assertEqual(json.loads(actual), json.loads(expected))

    def test_t1_shape_matching_tool_output_wins_over_text_noise(self) -> None:
        tool_out = json.dumps({
            "schema_version": "mdtools.v1",
            "file": "x.md",
            "entries": [{"heading": {"text": "A"}}],
        })
        text_noise = "I extracted the outline above."
        expected = json.dumps({
            "schema_version": "mdtools.v1",
            "file": "x.md",
            "entries": [{"heading": {"text": "A"}}],
        })
        actual = select_json_envelope_actual(
            [tool_out], [text_noise], "", expected
        )
        self.assertEqual(json.loads(actual), json.loads(expected))

    def test_t22_top_level_list_tool_output_falls_through_to_fallback(self) -> None:
        # T22 unix-mode case: agent emits a top-level list (no key overlap
        # with expected dict), no text answer. Must still surface the list
        # so the F3-closure scorer can wrap it.
        tool_list = json.dumps([
            {"kind": "Inline", "destination": "https://example.com/guide"},
        ])
        expected = json.dumps({
            "schema_version": "mdtools.v1",
            "file": "x.md",
            "links": [
                {"kind": "Inline", "destination": "https://example.com/guide"},
            ],
        })
        actual = select_json_envelope_actual(
            [tool_list], [], "", expected
        )
        self.assertEqual(json.loads(actual), json.loads(tool_list))

    def test_text_only_answer_works(self) -> None:
        text_answer = json.dumps({"total_pending": 9, "files": []})
        expected = json.dumps({"total_pending": 9, "files": []})
        actual = select_json_envelope_actual(
            [], [text_answer], "", expected
        )
        self.assertEqual(json.loads(actual), json.loads(expected))

    def test_no_shape_match_falls_back_to_most_recent_tool_output(self) -> None:
        # Neither tool output overlaps expected; no text candidate.
        # Must surface the most-recent tool output (reversed order) so
        # the existing scorer can still emit a meaningful MISMATCH.
        tool_a = json.dumps({"a": 1})
        tool_b = json.dumps({"b": 2})
        expected = json.dumps({"x": 0})
        actual = select_json_envelope_actual(
            [tool_a, tool_b], [], "", expected
        )
        self.assertEqual(json.loads(actual), json.loads(tool_b))

    def test_unknown_expected_shape_preserves_first_tool_output_rule(self) -> None:
        # Expected has no observable key shape (list of strings) → revert
        # to pre-F4 behavior: first non-empty parseable JSON wins.
        tool_dict = json.dumps({"x": 1})
        tool_list = json.dumps([1, 2, 3])
        expected = json.dumps(["a", "b"])
        actual = select_json_envelope_actual(
            [tool_dict, tool_list], [], "", expected
        )
        self.assertEqual(json.loads(actual), json.loads(tool_list))

    def test_empty_outputs_falls_back_to_extract_last_json_of_stdout(self) -> None:
        expected = json.dumps({"x": 1})
        actual = select_json_envelope_actual(
            [], [], json.dumps({"x": 1}), expected
        )
        self.assertEqual(json.loads(actual), {"x": 1})


if __name__ == "__main__":
    unittest.main()
