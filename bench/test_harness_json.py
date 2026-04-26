from __future__ import annotations

import json
import unittest

from bench.harness import (
    StructuralDiffPolicy,
    extract_last_json,
    parse_agent_output,
    score_structural_json,
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


if __name__ == "__main__":
    unittest.main()
