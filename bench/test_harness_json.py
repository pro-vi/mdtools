from __future__ import annotations

import json
import unittest
from pathlib import Path

from bench.harness import (
    StructuralDiffPolicy,
    _md_block_texts,
    _md_heading_tree,
    extract_last_json,
    load_tasks,
    neutral_block_texts,
    neutral_heading_tree,
    parse_agent_output,
    score_normalized_text_md,
    score_normalized_text_neutral,
    score_structural_json,
    score_task,
    select_json_envelope_actual,
)
from bench.pi_runner import parse_pi_json_output

REPO_ROOT = Path(__file__).resolve().parents[1]
MD_BIN = str(REPO_ROOT / "target" / "release" / "md")


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

    def test_extract_last_json_prefers_wrapping_envelope_over_nested_array(self) -> None:
        """F8-2 closure (T8 iter 5) — when the agent's wrapping envelope
        is embedded in prose, the inner `entries` array is contained
        within the outer object's source span; the highest-end-position
        rule selects the wrapping envelope. Pre-fix the legacy
        "prefer last array" rule returned the nested array, which the
        text-output branch of `select_json_envelope_actual` propagated
        unchecked into a false-NEGATIVE on json_envelope tasks
        (`bench/probes/F8-2-extract-prefers-nested-array/probe.py`)."""
        wrapping_envelope = (
            '{"file":"doc.md","entries":[{"heading":{"level":1,"text":"Top"}}]}'
        )
        agent_text = (
            "The headings for doc.md are:\n"
            "- # Top\n"
            "\n"
            "Here is the JSON:\n"
            f"{wrapping_envelope}\n"
        )
        result = extract_last_json(agent_text)
        self.assertEqual(json.loads(result), json.loads(wrapping_envelope))

    def test_extract_last_json_prefers_latest_sibling_when_no_containment(self) -> None:
        """Non-regression for the highest-end-position rule under the
        sibling case: two independent top-level JSON documents emitted
        in agent text, neither contained in the other. The latest
        (highest end) wins, which represents the agent's final answer
        emitted after intermediate analysis."""
        intermediate = '[1,2,3]'
        final_answer = '{"total":6}'
        agent_text = (
            f"First I tabulated the inputs: {intermediate}.\n"
            f"The result envelope is: {final_answer}\n"
        )
        result = extract_last_json(agent_text)
        self.assertEqual(json.loads(result), json.loads(final_answer))

    def test_extract_last_json_honors_string_boundaries_in_depth_scan(self) -> None:
        """F8-3 closure (T8 iter 7) — the depth scanner must skip
        brace/bracket characters that appear inside JSON string
        values, so the wrapping envelope is enumerated as a candidate
        even when a heading.text or other string field contains
        `}`/`]`/`{`/`[`. Pre-fix, the {/} pass closed prematurely on
        the brace inside the string, the truncated candidate failed
        json.loads, start reset to -1, and the outer envelope was
        never recorded. The probe lives at
        `bench/probes/F8-3-brace-in-string-value/probe.py`."""
        envelope = (
            '{"schema_version":"mdtools.v1","file":"/tmp/x.md",'
            '"entries":[{"heading":{"level":1,"text":"Heading with } brace",'
            '"block_index":0}}]}'
        )
        agent_text = (
            "I ran `md outline --json /tmp/x.md` and got this answer:\n"
            f"{envelope}\n"
            "That is the document outline.\n"
        )
        result = extract_last_json(agent_text)
        parsed = json.loads(result)
        self.assertIsInstance(parsed, dict)
        self.assertEqual(parsed.get("schema_version"), "mdtools.v1")
        self.assertEqual(json.loads(result), json.loads(envelope))

    def test_extract_last_json_handles_escaped_quotes_in_string_value(self) -> None:
        """Non-regression for the string-aware depth scanner — escape
        sequences inside string values must not be mis-detected as
        string terminators. Pins that F8-3's fix doesn't regress on
        payloads that include escaped quotes alongside non-string
        brace characters elsewhere in the envelope."""
        envelope = (
            '{"file":"doc.md","entries":[{"heading":{"text":"says \\"hi\\"",'
            '"level":1}}]}'
        )
        agent_text = (
            "Here is the answer:\n"
            f"{envelope}\n"
        )
        result = extract_last_json(agent_text)
        self.assertEqual(json.loads(result), json.loads(envelope))

    def test_extract_last_json_preserves_backticks_in_string_value(self) -> None:
        """F8-4 closure (T8 iter 9) — when a JSON string value contains a
        backtick triplet (e.g. an `entries[].heading.text` that names the
        language of a code-fence example), the candidate must round-trip
        byte-exact through extract_last_json. Pre-fix, a global
        ` ``` `-stripping regex preprocessor mutated the string content
        silently before the depth scanner ran; the candidate parsed but
        carried corrupted string fields, so score_structural_json FAILed
        a byte-exact correct agent answer. The probe lives at
        `bench/probes/F8-4-fence-regex-strips-string-content/probe.py`."""
        envelope = (
            '{"schema_version":"mdtools.v1","file":"/tmp/spec.md",'
            '"entries":[{"heading":{"level":1,"text":"Configuration","block_index":0}},'
            '{"heading":{"level":2,"text":"Example: ```python block","block_index":1}}]}'
        )
        agent_text = (
            "Sure, here is the document outline:\n"
            "\n"
            "```json\n"
            f"{envelope}\n"
            "```\n"
            "\n"
            "That is what `md outline --json /tmp/spec.md` produced.\n"
        )
        result = extract_last_json(agent_text)
        parsed = json.loads(result)
        self.assertEqual(parsed["entries"][1]["heading"]["text"], "Example: ```python block")
        self.assertEqual(json.loads(result), json.loads(envelope))

    def test_extract_last_json_handles_fenced_json_via_depth_scanner(self) -> None:
        """Non-regression for the canonical LLM output style: a JSON
        envelope wrapped in a top-level ` ```json ... ``` ` fence with
        no surrounding prose. Pins that F8-4's removal of the fence-strip
        preprocessor did not regress the fenced-JSON case — the depth
        scanner still finds and returns the inner JSON because backticks
        are not structural JSON characters and the brace tracker enters
        cleanly at the inner `{`."""
        envelope = '{"file":"doc.md","total":42}'
        agent_text = f"```json\n{envelope}\n```\n"
        result = extract_last_json(agent_text)
        self.assertEqual(json.loads(result), json.loads(envelope))

    def test_extract_last_json_ignores_unmatched_prose_quote_before_json(self) -> None:
        """PR #4 review (Codex P2): pre-fix, a prose prefix containing an
        unmatched `"` would latch `in_string=True` forever, causing every
        subsequent `{`/`[` to be skipped and returning the raw text. The
        fix scopes quote tracking to active JSON spans (depth > 0) so
        prose quotes are ignored."""
        text = 'The agent says: "almost done\n\n{"answer":42,"score":1}\n'
        result = extract_last_json(text)
        self.assertEqual(json.loads(result), {"answer": 42, "score": 1})

    def test_extract_last_json_handles_multiple_unmatched_prose_quotes(self) -> None:
        """Stress case: several unmatched quotes scattered through prose
        before the trailing envelope. Pre-fix each quote toggled in_string
        unpredictably; post-fix prose quotes are ignored entirely outside
        any candidate."""
        text = (
            "First line with a stray \" quote.\n"
            "Second line says \"hello without closing.\n"
            "Then more prose, and \"another opener.\n"
            '{"final":"envelope"}\n'
        )
        result = extract_last_json(text)
        self.assertEqual(json.loads(result), {"final": "envelope"})

    def test_extract_last_json_handles_balanced_prose_quoted_brace(self) -> None:
        """PR #4 second-round Codex finding: a balanced prose `"}"` before
        the JSON envelope. The first fix gated `in_string` on `depth > 0`,
        which made the `}` inside `"}"` visible to the depth scanner,
        driving depth negative and missing the trailing `{...}` candidate.
        The second fix removes the depth gate and aborts in_string at
        newlines, so balanced quoted braces in prose stay shielded while
        unmatched prose quotes can no longer latch across line breaks."""
        text = 'He said "}" before the answer.\n{"key":"value"}\n'
        result = extract_last_json(text)
        self.assertEqual(json.loads(result), {"key": "value"})

    def test_extract_last_json_handles_balanced_quoted_bracket_in_prose(self) -> None:
        """Symmetric case for the [/] pass: a balanced prose `"["` and
        `"]"` before the JSON array. Pins both opener/closer passes."""
        text = 'Note: see "[item]" for details.\n[1,2,3]\n'
        result = extract_last_json(text)
        self.assertEqual(json.loads(result), [1, 2, 3])

    def test_extract_last_json_handles_unmatched_prose_quote_same_line_as_json(self) -> None:
        """PR #4 third-round Codex finding: the JSON envelope appears on the
        same line as an unmatched prose quote, with no newline between.
        Newline-abort cannot resolve this because string mode never aborts
        before the envelope. The two-pass scanner catches it via the
        unshielded pass (which ignores quotes entirely and validates
        candidates with json.loads)."""
        text = 'The agent says: "almost done {"answer":42}'
        result = extract_last_json(text)
        self.assertEqual(json.loads(result), {"answer": 42})

    def test_extract_last_json_handles_stray_prose_closer_before_envelope(self) -> None:
        """Stray prose `}` before any opener used to drive depth negative,
        preventing the next `{` from being recorded. The opener-stack
        scanner ignores closers with no matching opener, so the trailing
        envelope still records cleanly."""
        text = 'leftover prose with stray } closer\n{"k":"v"}\n'
        result = extract_last_json(text)
        self.assertEqual(json.loads(result), {"k": "v"})

    def test_extract_last_json_handles_stray_close_then_open_prose(self) -> None:
        """PR #4 round 4 Codex finding: prose contains `}{` immediately
        before the real envelope. With a depth counter + clamp, the stray
        `}` was ignored but the stray `{` set start at the wrong position,
        and the real envelope's outer `}` only brought depth from 2 to 1
        — so the candidate never recorded. The opener-stack approach pops
        on the inner close, emitting the correct candidate, and leaves
        the orphaned outer opener harmlessly on the stack at EOF."""
        text = '}{{"schema_version":"mdtools.v1","entries":[1,2,3]}'
        result = extract_last_json(text)
        self.assertEqual(
            json.loads(result),
            {"schema_version": "mdtools.v1", "entries": [1, 2, 3]},
        )

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

    def test_schema_version_only_overlap_rejected(self) -> None:
        """F8-1 closure (T8 iter 3) — schema_version-only overlap with the
        expected envelope is not sufficient for shape match. Without the
        subset check, every mdtools envelope would shape-match every other
        mdtools envelope on the universal `schema_version` key, so an
        intermediate `md tasks --json` call would be selected as `actual`
        when the task asks for `md outline --json`. The probe lives at
        `bench/probes/F8-1-schema-version-overlap/probe.py`."""
        outline_envelope = json.dumps({
            "schema_version": "mdtools.v1",
            "file": "x.md",
            "entries": [{"heading": {"level": 1, "text": "H1"}}],
        })
        tasks_envelope = json.dumps({
            "schema_version": "mdtools.v1",
            "results": [{"file": "x.md", "tasks": []}],
        })
        expected = outline_envelope
        # Chronological: outline first (correct), tasks second (intermediate).
        # Reverse iteration sees tasks first; pre-F8-1 it accepted on
        # schema_version overlap. Post-fix the subset check rejects it
        # and the loop continues to the outline envelope.
        actual = select_json_envelope_actual(
            [outline_envelope, tasks_envelope], [], "", expected
        )
        self.assertEqual(json.loads(actual), json.loads(expected))

    def test_subset_check_preserves_extra_keys_on_actual(self) -> None:
        """Subset check must still accept an actual whose top-level keys
        are a *superset* of expected keys (e.g. an envelope adding a
        debug field). Pins that the F8-1 fix did not over-tighten."""
        expected = json.dumps({
            "schema_version": "mdtools.v1",
            "file": "x.md",
            "entries": [{"heading": {"level": 1, "text": "H1"}}],
        })
        tool_with_extra = json.dumps({
            "schema_version": "mdtools.v1",
            "file": "x.md",
            "entries": [{"heading": {"level": 1, "text": "H1"}}],
            "warnings": [],
        })
        actual = select_json_envelope_actual(
            [tool_with_extra], [], "", expected
        )
        self.assertEqual(json.loads(actual), json.loads(tool_with_extra))


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


class MdBlockTextsUtf8Tests(unittest.TestCase):
    """F8-5 closure (T8 iter 11) — `_md_block_texts` slices UTF-8 byte
    offsets from `md blocks --json` against the source text. The pre-fix
    implementation indexed the Python `str` directly with byte offsets,
    which drifted by the cumulative byte-excess from every preceding
    multi-byte character. The probe lives at
    `bench/probes/F8-5-md-block-texts-utf8-byte-vs-char-slice/probe.py`."""

    def test_md_block_texts_honors_utf8_byte_boundaries(self) -> None:
        """The F8-5 stage A trace: a heading with one multi-byte `é`
        produces 1-byte drift that compounds across later blocks. Pre-fix
        result was `['# Héllo', ': foo', ': bar']` (leading char of every
        non-first block dropped); post-fix result is byte-exact."""
        content = "# Héllo\n\nA: foo\n\nB: bar\n"
        self.assertEqual(
            _md_block_texts(content, MD_BIN),
            ["# Héllo", "A: foo", "B: bar"],
        )

    def test_score_normalized_text_md_rejects_wrong_answer_with_utf8(self) -> None:
        """The F8-5 stage B trace: SCORER DIVERGENCE on a wrong agent
        answer. Pre-fix, the broken slicer dropped the first char of the
        affected block in BOTH actual and expected, so the differing char
        was never compared; ok_md=True (false-POSITIVE), ok_neutral=False.
        Post-fix, ok_md=False (correct), agreeing with the neutral scorer."""
        expected = "# café\n\nFOO\n\nbar\n"
        actual = "# café\n\nFOO\n\nXar\n"
        policy = StructuralDiffPolicy(
            kind="normalized_text",
            normalize_line_endings=True,
            ignore_trailing_whitespace=True,
            compare_frontmatter_json=False,
            compare_heading_tree=True,
            compare_block_order=False,
            compare_link_destinations=False,
            compare_block_text=True,
        )
        ok_md = score_normalized_text_md(policy, actual, expected, MD_BIN, [])
        ok_neutral = score_normalized_text_neutral(policy, actual, expected, [])
        self.assertFalse(ok_md, "md scorer must reject wrong UTF-8 answer post-fix")
        self.assertFalse(ok_neutral, "neutral scorer already rejects this answer")
        self.assertEqual(ok_md, ok_neutral, "scorers must agree on UTF-8 content")


class NeutralHeadingTreeInlineRenderingTests(unittest.TestCase):
    """F8-6 closure (T8 iter 13) — `neutral_heading_tree` returned the raw
    inline source `tokens[i+1].content` while `_md_heading_tree` returned
    `md outline --json`'s already-rendered plaintext, creating SCORER
    DIVERGENCE on any heading containing inline markdown (inline code,
    emphasis, links, images, html). The probe lives at
    `bench/probes/F8-6-heading-tree-inline-formatting-divergence/probe.py`.
    Closure renders inline children to plaintext to match md outline."""

    def test_neutral_heading_tree_renders_inline_to_plaintext(self) -> None:
        """The F8-6 stage A trace: a sample with inline code, bold, and
        link headings. Pre-fix neutral returned raw markdown source like
        '`md tasks` command' / '**Important** notes' / '[docs](url)';
        post-fix neutral matches md outline's stripped plaintext."""
        sample = (
            "# The `md tasks` command\n"
            "\n"
            "## **Important** notes\n"
            "\n"
            "### See [docs](https://example.com)\n"
        )
        self.assertEqual(
            neutral_heading_tree(sample),
            [
                (1, "The md tasks command"),
                (2, "Important notes"),
                (3, "See docs"),
            ],
        )
        self.assertEqual(
            neutral_heading_tree(sample),
            _md_heading_tree(sample, MD_BIN),
            "neutral and md heading_tree must agree on inline-markdown "
            "rendering after F8-6 closure",
        )

    def test_score_normalized_text_md_and_neutral_agree_on_emphasis_diff(self) -> None:
        """The F8-6 stage B trace: SCORER DIVERGENCE pre-fix. expected has
        plain '# Configuration', actual injected '# **Configuration**'.
        Pre-fix ok_md=True (renders both to 'Configuration'),
        ok_neutral=False (raw source differs). Post-fix both scorers agree
        that the two heading texts are heading-tree-equivalent — the md
        outline contract treats them as the same heading."""
        expected = "# Configuration\n\nSettings live here.\n"
        actual = "# **Configuration**\n\nSettings live here.\n"
        policy = StructuralDiffPolicy(
            kind="normalized_text",
            normalize_line_endings=True,
            ignore_trailing_whitespace=True,
            compare_frontmatter_json=False,
            compare_heading_tree=True,
            compare_block_order=False,
            compare_link_destinations=False,
            compare_block_text=False,
        )
        ok_md = score_normalized_text_md(policy, actual, expected, MD_BIN, [])
        ok_neutral = score_normalized_text_neutral(policy, actual, expected, [])
        self.assertEqual(
            ok_md,
            ok_neutral,
            "scorers must agree on inline-markdown heading-tree equivalence",
        )

    def test_neutral_heading_tree_handles_image_alt_and_html_inline(self) -> None:
        """Non-regression on the broader inline-markdown surface: image
        renders to alt text (via children recursion), html_inline tags
        drop while their bracketed text content survives via sibling text
        tokens. Validates the closure generalizes beyond emphasis/code/link."""
        sample = (
            "# Heading with ![alt text](img.png) suffix\n"
            "\n"
            "## Heading with <span>html</span> body\n"
        )
        self.assertEqual(
            neutral_heading_tree(sample),
            _md_heading_tree(sample, MD_BIN),
            "neutral must match md outline on image+html_inline headings",
        )

    def test_neutral_heading_tree_preserves_softbreak_in_setext_heading(self) -> None:
        """PR #4 review (Codex P2): pre-fix, softbreak/hardbreak tokens
        were dropped from `_render_inline_to_plaintext`, concatenating
        words across line breaks in multi-line setext headings. That
        partially undid F8-6 by reintroducing scorer divergence on any
        heading whose inline content spans multiple physical lines. The
        fix emits a single space for break tokens. This test pins both
        the rendered output and parity with `_md_heading_tree`."""
        sample = "Multi\nLine Heading\n=================\n\nbody text\n"
        result = neutral_heading_tree(sample)
        self.assertEqual(result, [(1, "Multi Line Heading")])
        self.assertEqual(
            result,
            _md_heading_tree(sample, MD_BIN),
            "neutral must match md outline on multi-line setext heading",
        )


class NeutralBlockTextsSourceFidelityTests(unittest.TestCase):
    """F8-7 closure (T8 iter 15) — `neutral_block_texts` was over-normalizing
    block content vs `_md_block_texts`'s byte-slice contract: hr tokens
    hardcoded "---" regardless of source style, and heading tokens dropped
    the level marker prefix. The probe lives at
    `bench/probes/F8-7-neutral-block-texts-over-normalization/probe.py`.
    Closure source-slices hr and heading via `tok.map` line ranges."""

    def test_neutral_block_texts_preserves_hr_style(self) -> None:
        """The F8-7 stage A trace, hr subset: three different hr styles
        (---/***/___) in the same document. Pre-fix neutral collapsed all
        three to hardcoded "---"; post-fix neutral matches md byte-slicing."""
        sample = (
            "---\n"
            "\n"
            "foo\n"
            "\n"
            "***\n"
            "\n"
            "bar\n"
            "\n"
            "___\n"
            "\n"
            "baz\n"
        )
        self.assertEqual(
            neutral_block_texts(sample),
            ["---", "foo", "***", "bar", "___", "baz"],
        )
        self.assertEqual(
            neutral_block_texts(sample),
            _md_block_texts(sample, MD_BIN),
            "neutral and md block_texts must agree on hr style fidelity",
        )

    def test_neutral_block_texts_preserves_heading_prefix(self) -> None:
        """The F8-7 stage A trace, heading subset: atx and setext headings
        across multiple levels. Pre-fix neutral dropped both the `# `/`## `
        prefix (atx) and the `=====` underline (setext); post-fix neutral
        preserves the source-fidelity heading representation."""
        sample = (
            "# Hello\n"
            "\n"
            "## Subsection\n"
            "\n"
            "Setext H1\n"
            "=========\n"
            "\n"
            "Setext H2\n"
            "---------\n"
            "\n"
            "body\n"
        )
        self.assertEqual(
            neutral_block_texts(sample),
            [
                "# Hello",
                "## Subsection",
                "Setext H1\n=========",
                "Setext H2\n---------",
                "body",
            ],
        )
        self.assertEqual(
            neutral_block_texts(sample),
            _md_block_texts(sample, MD_BIN),
            "neutral and md block_texts must agree on heading prefix fidelity",
        )

    def test_score_normalized_text_md_and_neutral_agree_on_dropped_heading(self) -> None:
        """The F8-7 stage B trace: SCORER DIVERGENCE pre-fix on a heading→
        paragraph collapse (a realistic doc-maintenance failure mode). md
        correctly reported MISMATCH via byte slicing; neutral falsely OK
        because it dropped the `# ` prefix from the expected, making the
        heading indistinguishable from a paragraph. Post-fix both scorers
        agree on MISMATCH."""
        expected = "# Configuration\n\nSettings live here.\n"
        actual = "Configuration\n\nSettings live here.\n"
        policy = StructuralDiffPolicy(
            kind="normalized_text",
            normalize_line_endings=True,
            ignore_trailing_whitespace=True,
            compare_frontmatter_json=False,
            compare_heading_tree=False,
            compare_block_order=False,
            compare_link_destinations=False,
            compare_block_text=True,
        )
        ok_md = score_normalized_text_md(policy, actual, expected, MD_BIN, [])
        ok_neutral = score_normalized_text_neutral(policy, actual, expected, [])
        self.assertFalse(ok_md, "md scorer correctly catches dropped heading")
        self.assertFalse(ok_neutral, "neutral must reject dropped heading post-fix")
        self.assertEqual(ok_md, ok_neutral, "scorers must agree on dropped heading")

    def test_score_normalized_text_md_and_neutral_agree_on_hr_style(self) -> None:
        """The F8-7 stage C trace: SCORER DIVERGENCE pre-fix on an hr style
        swap. md correctly reported MISMATCH (--- vs ***) via byte slicing;
        neutral falsely OK because it hardcoded "---" for both. Post-fix
        both scorers agree on MISMATCH."""
        expected = "# Hello\n\n---\n\nfoo\n"
        actual = "# Hello\n\n***\n\nfoo\n"
        policy = StructuralDiffPolicy(
            kind="normalized_text",
            normalize_line_endings=True,
            ignore_trailing_whitespace=True,
            compare_frontmatter_json=False,
            compare_heading_tree=False,
            compare_block_order=False,
            compare_link_destinations=False,
            compare_block_text=True,
        )
        ok_md = score_normalized_text_md(policy, actual, expected, MD_BIN, [])
        ok_neutral = score_normalized_text_neutral(policy, actual, expected, [])
        self.assertFalse(ok_md, "md scorer correctly catches hr style swap")
        self.assertFalse(ok_neutral, "neutral must reject hr style swap post-fix")
        self.assertEqual(ok_md, ok_neutral, "scorers must agree on hr style swap")


class NeutralBlockTextsCollectionFidelityTests(unittest.TestCase):
    """F8-8 closure (T8 follow-up) — symmetric mirror of F8-7 on the five
    collection-type tokens (paragraph_open, bullet_list_open, ordered_list_open,
    blockquote_open, table_open). Probe at
    `bench/probes/F8-8-neutral-block-texts-collection-over-normalization/probe.py`.
    Closure extends F8-7's `tok.map` line-slice fix to the remaining branch."""

    def test_neutral_block_texts_preserves_list_markers(self) -> None:
        """F8-8 stage A subset: bullet vs ordered lists. Pre-fix neutral
        collapsed both to inline `foo\\nbar`; post-fix neutral preserves the
        `- ` and `1. ` markers via byte-slicing."""
        sample = "- bullet a\n- bullet b\n\n1. ordered a\n2. ordered b\n"
        self.assertEqual(
            neutral_block_texts(sample),
            ["- bullet a\n- bullet b", "1. ordered a\n2. ordered b"],
        )
        self.assertEqual(
            neutral_block_texts(sample),
            _md_block_texts(sample, MD_BIN),
            "neutral and md block_texts must agree on list-marker fidelity",
        )

    def test_neutral_block_texts_preserves_blockquote_prefix_and_table(self) -> None:
        """F8-8 stage A subset: blockquote `> ` prefix and table `|`/`---`
        separators. Pre-fix neutral dropped both."""
        sample = (
            "> quoted line\n"
            "> still quoted\n"
            "\n"
            "| col1 | col2 |\n"
            "| --- | --- |\n"
            "| a | b |\n"
        )
        self.assertEqual(
            neutral_block_texts(sample),
            _md_block_texts(sample, MD_BIN),
            "neutral and md block_texts must agree on blockquote/table fidelity",
        )
        # Spot-check: blockquote prefix survives the round-trip.
        self.assertIn("> quoted line", neutral_block_texts(sample)[0])

    def test_score_normalized_text_md_and_neutral_agree_on_blockquote_flattening(self) -> None:
        """F8-8 stage C trace: SCORER DIVERGENCE pre-fix on a blockquote→
        paragraph flattening (a realistic doc-maintenance failure mode). md
        correctly reported MISMATCH via byte-slicing; neutral falsely OK
        pre-fix. Post-fix both scorers agree on MISMATCH."""
        expected = "# Notice\n\n> Important note\n> Do not forget.\n"
        actual = "# Notice\n\nImportant note\nDo not forget.\n"
        policy = StructuralDiffPolicy(
            kind="normalized_text",
            normalize_line_endings=True,
            ignore_trailing_whitespace=True,
            compare_frontmatter_json=False,
            compare_heading_tree=False,
            compare_block_order=False,
            compare_link_destinations=False,
            compare_block_text=True,
        )
        ok_md = score_normalized_text_md(policy, actual, expected, MD_BIN, [])
        ok_neutral = score_normalized_text_neutral(policy, actual, expected, [])
        self.assertFalse(ok_md, "md scorer correctly catches blockquote flattening")
        self.assertFalse(ok_neutral, "neutral must reject blockquote flattening post-fix")
        self.assertEqual(ok_md, ok_neutral, "scorers must agree on blockquote flattening")

    def test_score_normalized_text_md_and_neutral_agree_on_nested_list_flattening(self) -> None:
        """F8-8 stage D trace: SCORER DIVERGENCE pre-fix on a nested→flat
        list flattening (changes commonmark nesting semantics). md correctly
        reported MISMATCH via byte-slicing the indentation; neutral falsely
        OK pre-fix. Post-fix both scorers agree on MISMATCH."""
        expected = "- top\n  - inner\n  - inner2\n"
        actual = "- top\n- inner\n- inner2\n"
        policy = StructuralDiffPolicy(
            kind="normalized_text",
            normalize_line_endings=True,
            ignore_trailing_whitespace=True,
            compare_frontmatter_json=False,
            compare_heading_tree=False,
            compare_block_order=False,
            compare_link_destinations=False,
            compare_block_text=True,
        )
        ok_md = score_normalized_text_md(policy, actual, expected, MD_BIN, [])
        ok_neutral = score_normalized_text_neutral(policy, actual, expected, [])
        self.assertFalse(ok_md, "md scorer correctly catches nested list flattening")
        self.assertFalse(ok_neutral, "neutral must reject nested list flattening post-fix")
        self.assertEqual(ok_md, ok_neutral, "scorers must agree on nested list flattening")


if __name__ == "__main__":
    unittest.main()
