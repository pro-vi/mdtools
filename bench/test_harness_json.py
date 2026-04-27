from __future__ import annotations

import json
import unittest
from pathlib import Path

from bench.harness import (
    StructuralDiffPolicy,
    _md_block_texts,
    extract_last_json,
    load_tasks,
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


if __name__ == "__main__":
    unittest.main()
