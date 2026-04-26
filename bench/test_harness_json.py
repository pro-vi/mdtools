from __future__ import annotations

import json
import unittest
from pathlib import Path

from bench.harness import (
    StructuralDiffPolicy,
    extract_last_json,
    load_tasks,
    parse_agent_output,
    score_structural_json,
    score_task,
    select_json_envelope_actual,
)
from bench.pi_runner import parse_pi_json_output


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


class F4ClosureBundleReplayTests(unittest.TestCase):
    """F4 closure end-to-end (iter 32) — replay the iter-29 T16 PI bundle's
    durable agent_output.txt through parse_pi_json_output +
    select_json_envelope_actual + score_task and assert PASS on both md and
    neutral scorers. Promotes iter-30 + iter-31 ledger-prose REPL replay
    claims to a typed cheap-channel assertion against the actual failing
    artifact, complementing the synthetic JsonEnvelopeActualSelectionTests
    above by pinning behaviour on the real-world bundle that motivated F4."""

    BUNDLE_LOG = (
        Path(__file__).resolve().parents[1]
        / "bench/runs/checkpoint-pi-T16-mdtools-gpt5.4mini-2026-04-26"
        / "logs/T16_mdtools_1777224275/agent_output.txt"
    )

    def test_iter_29_t16_bundle_replays_to_dual_scorer_pass(self) -> None:
        if not self.BUNDLE_LOG.exists():
            self.skipTest(f"iter-29 T16 PI bundle agent_output not present at {self.BUNDLE_LOG}")
        raw = self.BUNDLE_LOG.read_text()
        trace = parse_pi_json_output(raw)
        self.assertEqual(trace.tool_calls, 4, "iter-29 bundle has 4 tool calls; trace shape changed")
        self.assertEqual(len(trace.text_outputs), 1, "iter-29 bundle has 1 assistant text message")

        repo_root = Path(__file__).resolve().parents[1]
        tasks = load_tasks(repo_root / "bench/tasks/tasks.json")
        task = next(t for t in tasks if t.id == "T16")
        expected = (repo_root / task.expected_output).read_text()

        actual = select_json_envelope_actual(
            trace.tool_outputs, trace.text_outputs, trace.stdout, expected
        )
        # Schema-aware selector must skip the schema-mismatched md tasks --json
        # envelope (top keys ['schema_version','results']) and pick the agent's
        # text answer (top keys ['total_pending','files']).
        self.assertEqual(json.loads(actual), json.loads(expected))

        ok_md, ok_neutral, report = score_task(task, actual, expected, md_binary="md")
        self.assertTrue(ok_md, f"md scorer should PASS post-F4; report: {report}")
        self.assertTrue(ok_neutral, f"neutral scorer should PASS post-F4; report: {report}")
        self.assertIn("json_canonical: OK", report)


def _pre_iter30_select_json_envelope_actual(
    all_tool_outputs: list[str],
    all_text_outputs: list[str],
    stdout: str,
) -> str:
    """Faithful reproduction of the pre-iter-30 json_envelope actual selector
    from `git show 7b36502:bench/harness.py:1404-1429` — the loop that F4
    identified as preferring any non-empty JSON tool output (last wins) over
    the agent's matching text answer. Used by F4PreFixCounterfactualTests to
    pin the regression-protection invariant: if the schema-aware selector at
    bench/harness.py:1481 is ever reverted to this loop shape, both T11 and
    T16 PI bundles fail dual-scorer with json_canonical: MISMATCH."""
    actual = ""
    for tool_out in reversed(all_tool_outputs):
        try:
            parsed = json.loads(tool_out.strip())
            if isinstance(parsed, (list, dict)) and len(parsed) > 0:
                actual = tool_out.strip()
                break
        except (json.JSONDecodeError, TypeError):
            continue
    if not actual:
        for text_out in reversed(all_text_outputs):
            candidate = extract_last_json(text_out)
            try:
                parsed = json.loads(candidate)
                if isinstance(parsed, (list, dict)) and len(parsed) > 0:
                    actual = candidate
                    break
            except (json.JSONDecodeError, TypeError):
                continue
    if not actual:
        actual = extract_last_json(stdout)
    return actual


class F4PreFixCounterfactualTests(unittest.TestCase):
    """F4 closure trail counterfactual (iter 35 + iter 39) — promotes iter-33's
    prose-only ledger claim ('the pre-iter-30 selector logic against the
    iter-33 trajectory selects tool 1 with keys [results, schema_version]
    and FAILs dual-scorer') to a typed cheap-channel assertion across all
    three F4-relevant durable bundles (T16 from iter 29, T11 from iter 33,
    T19 from iter 37). If the schema-aware select_json_envelope_actual at
    bench/harness.py:1481 is ever reverted toward the pre-iter-30 loop
    shape, these tests fail."""

    REPO_ROOT = Path(__file__).resolve().parents[1]

    BUNDLES = [
        (
            "T16",
            REPO_ROOT
            / "bench/runs/checkpoint-pi-T16-mdtools-gpt5.4mini-2026-04-26"
            / "logs/T16_mdtools_1777224275/agent_output.txt",
            ["files", "total_pending"],
        ),
        (
            "T11",
            REPO_ROOT
            / "bench/runs/checkpoint-pi-T11-mdtools-gpt5.4mini-2026-04-26"
            / "logs/T11_mdtools_1777227478/agent_output.txt",
            ["phases", "totals"],
        ),
        (
            "T19",
            REPO_ROOT
            / "bench/runs/checkpoint-pi-T19-mdtools-gpt5.4mini-2026-04-26"
            / "logs/T19_mdtools_1777230034/agent_output.txt",
            ["phases", "totals"],
        ),
    ]

    def _replay_pre_fix(self, task_id: str, bundle_log: Path, expected_top_keys: list[str]) -> None:
        if not bundle_log.exists():
            self.skipTest(f"{task_id} PI bundle agent_output not present at {bundle_log}")
        raw = bundle_log.read_text()
        trace = parse_pi_json_output(raw)

        tasks = load_tasks(self.REPO_ROOT / "bench/tasks/tasks.json")
        task = next(t for t in tasks if t.id == task_id)
        expected = (self.REPO_ROOT / task.expected_output).read_text()

        self.assertEqual(
            sorted(json.loads(expected).keys()),
            sorted(expected_top_keys),
            f"{task_id} expected_output top-level keys drifted; counterfactual rationale stale",
        )

        pre_actual = _pre_iter30_select_json_envelope_actual(
            trace.tool_outputs, trace.text_outputs, trace.stdout
        )
        pre_parsed = json.loads(pre_actual)
        self.assertIsInstance(pre_parsed, dict, f"{task_id} pre-fix selector should pick a JSON-dict tool output")
        self.assertEqual(
            sorted(pre_parsed.keys()),
            ["results", "schema_version"],
            f"{task_id} pre-fix selector should pick the md tasks --json envelope, not the agent text answer",
        )

        ok_md, ok_neutral, report = score_task(task, pre_actual, expected, md_binary="md")
        self.assertFalse(ok_md, f"{task_id} md scorer must FAIL under pre-fix selector; report: {report}")
        self.assertFalse(ok_neutral, f"{task_id} neutral scorer must FAIL under pre-fix selector; report: {report}")
        self.assertIn("json_canonical: MISMATCH", report)

    def test_iter_29_t16_bundle_fails_under_pre_fix_selector(self) -> None:
        task_id, bundle_log, expected_top_keys = self.BUNDLES[0]
        self._replay_pre_fix(task_id, bundle_log, expected_top_keys)

    def test_iter_33_t11_bundle_fails_under_pre_fix_selector(self) -> None:
        task_id, bundle_log, expected_top_keys = self.BUNDLES[1]
        self._replay_pre_fix(task_id, bundle_log, expected_top_keys)

    def test_iter_37_t19_bundle_fails_under_pre_fix_selector(self) -> None:
        task_id, bundle_log, expected_top_keys = self.BUNDLES[2]
        self._replay_pre_fix(task_id, bundle_log, expected_top_keys)


if __name__ == "__main__":
    unittest.main()
