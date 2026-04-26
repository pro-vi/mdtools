from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from bench.command_policy import GuardEvent, classify_command_kind, load_guard_events
from bench.pi_audit_adapter import load_pi_audit_events, summarize_pi_audit_events
from bench.pi_runner import build_pi_json_command, default_audit_extension_path, parse_pi_json_output


class PiAuditAdapterTests(unittest.TestCase):
    def test_load_pi_audit_events_ignores_malformed_lines(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "audit.jsonl"
            path.write_text('{"schema":"pi-audit.v1","event":"tool_call"}\nnot-json\n')

            events = load_pi_audit_events(path)

        self.assertEqual(len(events), 1)
        self.assertEqual(events[0]["event"], "tool_call")

    def test_summarize_prefers_guard_sequence_for_requery_metrics(self) -> None:
        events = [
            {
                "schema": "pi-audit.v1",
                "event": "tool_call",
                "toolCallId": "1",
                "toolName": "bash",
                "input": {"command": "./md replace-block 1 doc.md -i --from patch.md"},
            },
            {
                "schema": "pi-audit.v1",
                "event": "tool_result",
                "toolCallId": "1",
                "toolName": "bash",
                "outputBytes": 12,
            },
        ]
        guard_events = [
            GuardEvent("allow", "md", "./md replace-block 1 doc.md -i --from patch.md"),
            GuardEvent("allow", "md", "./md blocks doc.md --json"),
        ]

        counters = summarize_pi_audit_events(events, guard_events=guard_events)

        self.assertEqual(counters.tool_calls, 1)
        self.assertEqual(counters.bytes_observation, 12)
        self.assertEqual(counters.mutations, 1)
        self.assertTrue(counters.requeried)

    def test_summarize_counts_guard_denials_as_policy_violations(self) -> None:
        counters = summarize_pi_audit_events([], guard_events=[GuardEvent("deny", "python", "python hack.py")])

        self.assertEqual(counters.policy_violations, 1)

    def test_summarize_extracts_model_and_thinking_level(self) -> None:
        counters = summarize_pi_audit_events([
            {
                "schema": "pi-audit.v1",
                "event": "model_change",
                "details": {"provider": "openai-codex", "modelId": "gpt-5.3-codex-spark"},
            },
            {
                "schema": "pi-audit.v1",
                "event": "thinking_level_change",
                "details": {"thinkingLevel": "off"},
            },
        ])

        self.assertEqual(counters.model, "openai-codex/gpt-5.3-codex-spark")
        self.assertEqual(counters.thinking_level, "off")


class PiRunnerTests(unittest.TestCase):
    def test_default_audit_extension_path_uses_global_pi_agent_dir(self) -> None:
        with patch.dict("os.environ", {"PI_CODING_AGENT_DIR": "/tmp/pi-agent"}, clear=True):
            self.assertEqual(
                default_audit_extension_path(),
                Path("/tmp/pi-agent/extensions/audit/index.ts"),
            )

        with patch.dict("os.environ", {"BENCH_PI_AUDIT_EXTENSION": "/tmp/audit/index.ts"}, clear=True):
            self.assertEqual(default_audit_extension_path(), Path("/tmp/audit/index.ts"))

    def test_build_pi_json_command_uses_pi_when_agent_default_is_claude(self) -> None:
        cmd = build_pi_json_command(
            agent_cmd="claude -p",
            model="sonnet",
            audit_extension_path=Path("/home/me/.pi/agent/extensions/audit/index.ts"),
            session_dir=Path("/tmp/sessions"),
        )

        self.assertEqual(cmd[0], "pi")
        self.assertIn("--mode", cmd)
        self.assertIn("json", cmd)
        self.assertIn("--model", cmd)
        self.assertIn("sonnet", cmd)

        cmd_with_thinking = build_pi_json_command(
            agent_cmd="pi --verbose",
            model="openai-codex/gpt-5.3-codex-spark",
            audit_extension_path=Path("/home/me/.pi/agent/extensions/audit/index.ts"),
            session_dir=Path("/tmp/sessions"),
            thinking_level="off",
        )
        self.assertIn("--thinking", cmd_with_thinking)
        self.assertIn("off", cmd_with_thinking)

    def test_parse_pi_json_output_collects_tool_and_text_outputs(self) -> None:
        lines = [
            {"type": "turn_start"},
            {"type": "tool_execution_start", "toolCallId": "1", "toolName": "bash", "args": {"command": "echo hi"}},
            {
                "type": "tool_execution_end",
                "toolCallId": "1",
                "toolName": "bash",
                "isError": False,
                "result": {"content": [{"type": "text", "text": "hi\n"}]},
            },
            {
                "type": "message_end",
                "message": {
                    "role": "assistant",
                    "provider": "anthropic",
                    "model": "claude-test",
                    "thinkingLevel": "off",
                    "content": [{"type": "text", "text": "DONE"}],
                    "stopReason": "stop",
                },
            },
        ]
        raw = "\n".join(json.dumps(line) for line in lines)

        parsed = parse_pi_json_output(raw)

        self.assertEqual(parsed.model, "anthropic/claude-test")
        self.assertEqual(parsed.thinking_level, "off")
        self.assertEqual(parsed.turns, 1)
        self.assertEqual(parsed.tool_calls, 1)
        self.assertEqual(parsed.tool_outputs, ["hi\n"])
        self.assertEqual(parsed.text_outputs, ["DONE"])
        self.assertIn("DONE", parsed.stdout)


class T10CanonicalReQueryCycleTests(unittest.TestCase):
    """Canonical re-query mutation moat (iter 43) — promotes iter-41's
    prose-only ledger claim ('T10 demonstrates the re-query moat in 3 tool
    calls: ./md tasks --status pending --json → ./md set-task 5.1 -i
    --status done → ./md tasks --status done --json — the canonical pattern
    from CLAUDE.md realized end-to-end under PI for the first time') to a
    typed cheap-channel assertion against the iter-41 T10 PI bundle. Pins
    the canonical query → mutation → query pattern detection via
    summarize_pi_audit_events on a structurally orthogonal axis from
    F4ClosureBundleReplayTests / F4PreFixCounterfactualTests (re-query
    detection vs scorer selection; raw_bytes branch vs json_envelope
    branch). If the adapter's query/mutation classifier or
    _has_query_after_mutation logic ever drifts, this test fails."""

    BUNDLE_DIR = (
        Path(__file__).resolve().parents[1]
        / "bench/runs/checkpoint-pi-T10-mdtools-gpt5.4mini-2026-04-26"
        / "logs/T10_mdtools_1777232433"
    )
    AUDIT_PATH = BUNDLE_DIR / "pi-audit.jsonl"
    GUARD_PATH = BUNDLE_DIR / "guard.log"

    def test_audit_only_summary_detects_canonical_recquery_cycle(self) -> None:
        if not self.AUDIT_PATH.exists():
            self.skipTest(f"iter-41 T10 PI bundle audit not present at {self.AUDIT_PATH}")
        events = load_pi_audit_events(self.AUDIT_PATH)
        # 8 events: model_change + thinking_level_change + 3×(tool_call + tool_result)
        self.assertEqual(len(events), 8)

        counters = summarize_pi_audit_events(events)
        self.assertEqual(counters.tool_calls, 3)
        self.assertEqual(counters.tool_results, 3)
        self.assertEqual(counters.tool_errors, 0)
        self.assertEqual(counters.mutations, 1)
        self.assertTrue(counters.requeried)
        self.assertEqual(counters.policy_violations, 0)
        self.assertEqual(counters.blocked, 0)
        self.assertEqual(counters.model, "openai-codex/gpt-5.4-mini")
        self.assertEqual(counters.thinking_level, "minimal")
        # The canonical 3-call re-query mutation cycle from CLAUDE.md
        self.assertEqual(len(counters.bash_commands), 3)
        self.assertIn("./md tasks", counters.bash_commands[0])
        self.assertIn("--status pending", counters.bash_commands[0])
        self.assertTrue(counters.bash_commands[1].startswith("./md set-task 5.1"))
        self.assertIn("--status done", counters.bash_commands[1])
        self.assertIn("./md tasks", counters.bash_commands[2])
        self.assertIn("--status done", counters.bash_commands[2])

    def test_guard_events_preserve_recquery_detection(self) -> None:
        if not self.AUDIT_PATH.exists() or not self.GUARD_PATH.exists():
            self.skipTest(f"iter-41 T10 PI bundle artifacts not present at {self.BUNDLE_DIR}")
        events = load_pi_audit_events(self.AUDIT_PATH)
        guard_events = load_guard_events(self.GUARD_PATH)
        # 3 guard.log entries — all allow, all md base-command (no policy violations)
        self.assertEqual(len(guard_events), 3)
        self.assertTrue(all(e.decision == "allow" for e in guard_events))
        self.assertTrue(all(e.base_command == "md" for e in guard_events))

        counters = summarize_pi_audit_events(events, guard_events=guard_events)
        # Per pi_audit_adapter.py:113 the guard sequence wins over the call
        # sequence when present, but both paths must produce mutations=1 and
        # requeried=True for the canonical query → mutation → query trajectory.
        self.assertEqual(counters.mutations, 1)
        self.assertTrue(counters.requeried)
        self.assertEqual(counters.policy_violations, 0)


class T15ParallelMutationFailureTests(unittest.TestCase):
    """Parallel-mutation FAIL pattern (iter 47) — promotes iter-45's
    prose-only ledger claim about T15's trajectory ('the agent
    parallelized two dependent mutations in the same turn —
    delete-section "Notes" and set-task 9.1 with stale loc 9.1 from
    the pre-delete query — then re-queried and observed the failure
    but did not recover with a third mutation') to a typed
    cheap-channel assertion against the iter-45 T15 PI bundle. Pins
    the dependent-mutation parallelization detection (two consecutive
    "mutation" entries in the kind sequence with no intervening
    "query") — the negative-shape counterpart to
    T10CanonicalReQueryCycleTests' positive-shape canonical re-query
    mutation cycle. Both tests anchor the F4-orthogonal closure trail
    on the raw_bytes scorer branch (as opposed to F4ClosureBundle/
    PreFixCounterfactual on the json_envelope branch). If the
    adapter's classify_command_kind ever drifts such that
    delete-section or set-task no longer classifies as "mutation",
    or if pi_audit_adapter no longer preserves bash_command order in
    PiAuditCounters, this test fails."""

    BUNDLE_DIR = (
        Path(__file__).resolve().parents[1]
        / "bench/runs/checkpoint-pi-T15-mdtools-gpt5.4mini-2026-04-26"
        / "logs/T15_mdtools_1777234559"
    )
    AUDIT_PATH = BUNDLE_DIR / "pi-audit.jsonl"
    GUARD_PATH = BUNDLE_DIR / "guard.log"

    def test_audit_only_summary_pins_parallel_mutation_pattern(self) -> None:
        if not self.AUDIT_PATH.exists():
            self.skipTest(f"iter-45 T15 PI bundle audit not present at {self.AUDIT_PATH}")
        events = load_pi_audit_events(self.AUDIT_PATH)
        # 16 events: model_change + thinking_level_change + 7×(tool_call + tool_result)
        self.assertEqual(len(events), 16)

        counters = summarize_pi_audit_events(events)
        self.assertEqual(counters.tool_calls, 7)
        self.assertEqual(counters.tool_results, 7)
        self.assertEqual(counters.tool_errors, 0)
        self.assertEqual(counters.mutations, 2)
        self.assertTrue(counters.requeried)
        self.assertEqual(counters.policy_violations, 0)
        self.assertEqual(counters.blocked, 0)
        self.assertEqual(counters.bytes_observation, 4987)
        self.assertEqual(counters.model, "openai-codex/gpt-5.4-mini")
        self.assertEqual(counters.thinking_level, "minimal")
        # 7-call FAIL trajectory (4-turn): turn-1 outline+tasks (queries),
        # turn-2 delete-section + set-task (parallel mutations on stale loc),
        # turn-3 outline + tasks --status pending + cat (queries), turn-4 stop.
        self.assertEqual(len(counters.bash_commands), 7)
        self.assertIn("./md outline", counters.bash_commands[0])
        self.assertIn("--json", counters.bash_commands[0])
        self.assertIn("./md tasks", counters.bash_commands[1])
        self.assertIn("./md delete-section 'Notes'", counters.bash_commands[2])
        self.assertIn("-i", counters.bash_commands[2])
        self.assertTrue(counters.bash_commands[3].startswith("./md set-task 9.1"))
        self.assertIn("--status done", counters.bash_commands[3])
        self.assertIn("./md outline", counters.bash_commands[4])
        self.assertIn("--status pending", counters.bash_commands[5])
        self.assertTrue(counters.bash_commands[6].startswith("cat "))
        # Parallel-mutation anti-pattern: classify each bash_command and assert
        # the kind sequence is exactly the [q,q,m,m,q,q,q] shape, with positions
        # 2,3 forming an adjacent-mutation pair — the structural signature
        # distinguishing this FAIL trace from T10's canonical PASS [q,m,q].
        kinds = [classify_command_kind(cmd) for cmd in counters.bash_commands]
        self.assertEqual(
            kinds,
            ["query", "query", "mutation", "mutation", "query", "query", "query"],
        )
        self.assertTrue(
            any(kinds[i] == "mutation" and kinds[i + 1] == "mutation" for i in range(len(kinds) - 1)),
            "expected adjacent mutation pair (parallel-execution anti-pattern) in kind sequence",
        )

    def test_guard_events_preserve_parallel_mutation_pattern(self) -> None:
        if not self.AUDIT_PATH.exists() or not self.GUARD_PATH.exists():
            self.skipTest(f"iter-45 T15 PI bundle artifacts not present at {self.BUNDLE_DIR}")
        events = load_pi_audit_events(self.AUDIT_PATH)
        guard_events = load_guard_events(self.GUARD_PATH)
        # 7 guard.log entries — all allow, base_command split = 6×md + 1×cat
        # (the cat appears at chronological position 5 in the guard.log because
        # the turn-3 cat verification fired before the turn-3 outline + tasks
        # --status pending re-queries were guarded; this differs from the
        # audit-event order, which lists cat last among turn-3 tool_calls).
        self.assertEqual(len(guard_events), 7)
        self.assertTrue(all(e.decision == "allow" for e in guard_events))
        base_commands = [e.base_command for e in guard_events]
        self.assertEqual(base_commands.count("md"), 6)
        self.assertEqual(base_commands.count("cat"), 1)

        counters = summarize_pi_audit_events(events, guard_events=guard_events)
        # Per pi_audit_adapter.py:113 the guard sequence wins over the call
        # sequence when present. Both paths must produce mutations=2 and
        # requeried=True for this trajectory; the kind sequence in guard
        # chronological order is [q,q,m,m,q,q,q] — same shape as the audit-
        # only path despite the different mid-trajectory ordering of cat.
        self.assertEqual(counters.mutations, 2)
        self.assertTrue(counters.requeried)
        self.assertEqual(counters.policy_violations, 0)
        guard_kinds = [
            classify_command_kind(e.raw_command, e.base_command) for e in guard_events
        ]
        self.assertEqual(
            guard_kinds,
            ["query", "query", "mutation", "mutation", "query", "query", "query"],
        )
        self.assertTrue(
            any(
                guard_kinds[i] == "mutation" and guard_kinds[i + 1] == "mutation"
                for i in range(len(guard_kinds) - 1)
            ),
            "expected adjacent mutation pair (parallel-execution anti-pattern) in guard sequence",
        )


class T12BatchMutationCycleTests(unittest.TestCase):
    """Batch-mutation moat at scale (iter 51) — promotes iter-49's
    prose-only ledger claim about T12's trajectory ('15 mutations across
    14 turns at 41.72s with 26 tool calls, organized as three
    query+mutation-batch sub-cycles per the kind sequence
    [q, q, m×8, q×5, m×6, q×3, m, q] … first PI bundle exercising the
    policy-deny guard path, first PI bundle with tool_errors=1, first PI
    bundle where audit-only and guard-augmented summarize_pi_audit_events
    paths produce distinct policy_violations values') to a typed
    cheap-channel assertion against the iter-49 T12 PI bundle. Pins the
    at-scale positive-shape pattern detection (batch-mutation moat
    invariant scales to N=15 mutations across three sub-cycles)
    complementing T10CanonicalReQueryCycleTests' single-mutation positive
    shape and T15ParallelMutationFailureTests' parallel-mutation negative
    shape — completing the F4-orthogonal closure trail's structural
    triad on the raw_bytes scorer branch. The audit-only-vs-guard-
    augmented policy_violations asymmetry is a previously-untested
    structural invariant in pi_audit_adapter.summarize_pi_audit_events
    (audit events have no native policy field; guard.log decision='deny'
    counts trigger only on the guard-augmented branch). If the adapter's
    classify_command_kind ever drifts on set-task batches, if the
    re-query detection logic stops counting query-after-mutation across
    multiple batches, or if the audit-only path starts inferring
    policy violations from non-guard sources, this test fails."""

    BUNDLE_DIR = (
        Path(__file__).resolve().parents[1]
        / "bench/runs/checkpoint-pi-T12-mdtools-gpt5.4mini-2026-04-26"
        / "logs/T12_mdtools_1777237571"
    )
    AUDIT_PATH = BUNDLE_DIR / "pi-audit.jsonl"
    GUARD_PATH = BUNDLE_DIR / "guard.log"

    EXPECTED_KIND_SEQUENCE = (
        ["query"] * 2
        + ["mutation"] * 8
        + ["query"] * 5
        + ["mutation"] * 6
        + ["query"] * 3
        + ["mutation"]
        + ["query"]
    )

    def test_audit_only_summary_pins_batch_mutation_at_scale(self) -> None:
        if not self.AUDIT_PATH.exists():
            self.skipTest(f"iter-49 T12 PI bundle audit not present at {self.AUDIT_PATH}")
        events = load_pi_audit_events(self.AUDIT_PATH)
        # 54 events: model_change + thinking_level_change + 26 tool_call +
        # 25 tool_result + 1 tool_error (the md block 5 out-of-range error
        # at bash_commands[13] routes to a tool_error event, not tool_result).
        self.assertEqual(len(events), 54)

        counters = summarize_pi_audit_events(events)
        self.assertEqual(counters.tool_calls, 26)
        self.assertEqual(counters.tool_results, 25)
        self.assertEqual(counters.tool_errors, 1)
        self.assertEqual(counters.mutations, 15)
        self.assertTrue(counters.requeried)
        # Audit-only path: no policy field on audit events themselves, so
        # policy_violations is structurally 0 even though the agent attempted
        # a denied sed command at bash_commands[12] mid-trajectory.
        self.assertEqual(counters.policy_violations, 0)
        self.assertEqual(counters.blocked, 0)
        self.assertEqual(counters.bytes_observation, 19161)
        self.assertEqual(counters.model, "openai-codex/gpt-5.4-mini")
        self.assertEqual(counters.thinking_level, "minimal")
        # 26-call PASS trajectory across 14 turns: three query+mutation-batch
        # sub-cycles separated by re-queries — the at-scale positive-shape
        # complement to T10's single-mutation [q,m,q] PASS and T15's
        # parallel-mutation [q,q,m,m,q,q,q] FAIL.
        self.assertEqual(len(counters.bash_commands), 26)
        # First sub-cycle: turn-1 outline+tasks queries, then 8-mutation
        # set-task batch covering top-level + nested + grandchild + blockquote
        # tasks under "Phase 0".
        self.assertIn("./md outline", counters.bash_commands[0])
        self.assertIn("--json", counters.bash_commands[0])
        self.assertIn("./md tasks", counters.bash_commands[1])
        for idx in range(2, 10):
            self.assertTrue(
                counters.bash_commands[idx].startswith("./md set-task "),
                f"expected set-task at bash_commands[{idx}], got {counters.bash_commands[idx][:80]!r}",
            )
        # Second sub-cycle: tasks re-queries observe the post-batch state,
        # the agent attempts sed for line-range verification (denied by guard
        # in mdtools mode — appears at audit position 12 with full command
        # preserved verbatim by the audit extension), recovers via md block
        # queries (the first md block returns a tool_error for wrong arg
        # shape, but the audit event still records the bash_command).
        self.assertIn("./md tasks", counters.bash_commands[10])
        self.assertIn("./md tasks", counters.bash_commands[11])
        self.assertTrue(
            counters.bash_commands[12].startswith("sed "),
            f"expected sed at bash_commands[12], got {counters.bash_commands[12][:80]!r}",
        )
        self.assertIn("./md block 5", counters.bash_commands[13])
        self.assertIn("./md block 6", counters.bash_commands[14])
        # Then 6-mutation set-task batch picking up tasks the first batch
        # missed (including the blockquote task at loc 6.0.0).
        for idx in range(15, 21):
            self.assertTrue(
                counters.bash_commands[idx].startswith("./md set-task "),
                f"expected set-task at bash_commands[{idx}], got {counters.bash_commands[idx][:80]!r}",
            )
        # Third sub-cycle: tasks + md block re-queries observe the
        # post-second-batch state, single final set-task at index 24,
        # final tasks re-query at index 25 to verify the mutation landed.
        self.assertIn("./md tasks", counters.bash_commands[21])
        self.assertIn("./md block 5", counters.bash_commands[22])
        self.assertIn("./md block 6", counters.bash_commands[23])
        self.assertTrue(counters.bash_commands[24].startswith("./md set-task "))
        self.assertIn("./md tasks", counters.bash_commands[25])
        # Batch-mutation moat-at-scale: classify each bash_command and assert
        # the kind sequence is exactly the [q×2, m×8, q×5, m×6, q×3, m, q]
        # shape (26 entries) — the structural signature distinguishing this
        # PASS-at-scale from T10's single-mutation [q,m,q] PASS and T15's
        # parallel-mutation [q,q,m,m,q,q,q] FAIL.
        kinds = [classify_command_kind(cmd) for cmd in counters.bash_commands]
        self.assertEqual(kinds, self.EXPECTED_KIND_SEQUENCE)
        # Three mutation→query transitions (one per sub-cycle boundary), at
        # positions 9→10, 20→21, and 24→25 — proving the canonical re-query
        # mutation cycle scales cleanly across multiple batches.
        mutation_to_query_boundaries = [
            i for i in range(len(kinds) - 1)
            if kinds[i] == "mutation" and kinds[i + 1] == "query"
        ]
        self.assertEqual(mutation_to_query_boundaries, [9, 20, 24])

    def test_guard_events_expose_policy_violations_asymmetry(self) -> None:
        if not self.AUDIT_PATH.exists() or not self.GUARD_PATH.exists():
            self.skipTest(f"iter-49 T12 PI bundle artifacts not present at {self.BUNDLE_DIR}")
        events = load_pi_audit_events(self.AUDIT_PATH)
        guard_events = load_guard_events(self.GUARD_PATH)
        # 26 guard.log entries — decision split = 25 allow + 1 deny;
        # base_command split = 25 md + 1 sed. The sole deny is the
        # sed -n '11,26p' line-range probe at chronological index 12,
        # blocked by command_policy in mdtools mode (sed is not in the
        # mdtools-mode allowlist).
        self.assertEqual(len(guard_events), 26)
        decisions = [e.decision for e in guard_events]
        self.assertEqual(decisions.count("allow"), 25)
        self.assertEqual(decisions.count("deny"), 1)
        base_commands = [e.base_command for e in guard_events]
        self.assertEqual(base_commands.count("md"), 25)
        self.assertEqual(base_commands.count("sed"), 1)
        # The denied sed is at chronological position 12, matching the audit
        # bash_commands[12] position (no skew between audit and guard
        # ordering because all audit tool_calls were guarded in turn).
        self.assertEqual(guard_events[12].decision, "deny")
        self.assertEqual(guard_events[12].base_command, "sed")

        counters = summarize_pi_audit_events(events, guard_events=guard_events)
        # Per pi_audit_adapter.py:113 the guard sequence wins over the call
        # sequence when present. Both paths must produce mutations=15 and
        # requeried=True for this trajectory.
        self.assertEqual(counters.mutations, 15)
        self.assertTrue(counters.requeried)
        # Audit-only-vs-guard-augmented policy_violations asymmetry: the
        # audit-only path returns 0 (audit events have no native policy
        # field), while the guard-augmented path returns 1 (counting the
        # single decision='deny' entry from guard.log). T12 is the first PI
        # bundle to surface this previously-untested structural asymmetry —
        # T10 and T15 both had policy_violations=0 on both paths because the
        # agent never attempted a denied command in those trajectories.
        self.assertEqual(counters.policy_violations, 1)
        self.assertEqual(counters.blocked, 0)
        # The guard-derived kind sequence must match the audit-derived one
        # bit-exact (chronological ordering is identical because every audit
        # tool_call in this trajectory was guarded — the sed deny did not
        # short-circuit the guard pipeline; the deny was recorded and the
        # audit tool_call still surfaced the attempted command).
        guard_kinds = [
            classify_command_kind(e.raw_command, e.base_command) for e in guard_events
        ]
        self.assertEqual(guard_kinds, self.EXPECTED_KIND_SEQUENCE)


class T1HybridModeBaselineTests(unittest.TestCase):
    """Cross-mode hybrid coverage baseline (iter 56) — promotes iter-53's
    prose-only ledger claim about the T1 hybrid trajectory ('first PI
    hybrid-mode bundle … T1 hybrid dual-scorer PASS in 11.97s with 1
    tool call (./md outline ... --json), 0 mutations, requeried=False,
    policy_violations=0 on both audit-only and guard-augmented paths
    … apples-to-apples cross-mode comparable cell with iter-4 T1
    mdtools bundle on all six normalization axes') to a typed cheap-
    channel assertion. Opens the cross-mode hybrid coverage trail's
    typed-assertion line as a third structural axis (cross-mode)
    parallel to the F4 closure trail (json_envelope branch / scorer
    selection invariants, iters 28/30/32/35/39) and the F4-orthogonal
    closure trail (raw_bytes branch / re-query mutation moat invariants,
    iters 43/47/51). Pins the +355-byte bytes_prompt delta evidencing
    the HYBRID_DOCS prompt template's expanded tool-docs section vs
    MDTOOLS_DOCS at bench/harness.py:282-283. Pins the audit-vs-guard
    symmetry on policy_violations (both 0 because all guard decisions
    are 'allow') as the structural counterpart to T12's audit-only-
    vs-guard-augmented asymmetry (0 vs 1). The iter-4 T1 mdtools
    bundle deliberately lacks the holdout_version field (predates
    iter-17 stamping); iter-17's forward-compat 'absence-of-field
    reads as v1 by date inference' convention converts to a typed-
    checkable assertion via .get('holdout_version', 1) yielding 1 for
    both bundles, plus an explicit assertNotIn check on the iter-4
    bundle that pins the absent state bit-exact and prevents
    retroactive edits (which would themselves be holdout-repair-
    shaped artifact edits). If the adapter's classify_command_kind
    drifts on ./md outline, if HYBRID_DOCS routing stops adding the
    +355-byte tool-docs expansion, or if the cross-mode comparable
    cell six-axis match is broken by a future bundle edit, this test
    fails."""

    HYBRID_BUNDLE_DIR = (
        Path(__file__).resolve().parents[1]
        / "bench/runs/checkpoint-pi-T1-hybrid-gpt5.4mini-2026-04-26"
        / "logs/T1_hybrid_1777240915"
    )
    HYBRID_AUDIT_PATH = HYBRID_BUNDLE_DIR / "pi-audit.jsonl"
    HYBRID_GUARD_PATH = HYBRID_BUNDLE_DIR / "guard.log"
    HYBRID_RUN_JSON = (
        Path(__file__).resolve().parents[1]
        / "bench/runs/checkpoint-pi-T1-hybrid-gpt5.4mini-2026-04-26/run.json"
    )
    HYBRID_RESULTS_JSON = (
        Path(__file__).resolve().parents[1]
        / "bench/runs/checkpoint-pi-T1-hybrid-gpt5.4mini-2026-04-26/results.json"
    )

    MDTOOLS_BUNDLE_DIR = (
        Path(__file__).resolve().parents[1]
        / "bench/runs/checkpoint-pi-T1-mdtools-gpt5.4mini-2026-04-26"
        / "logs/T1_mdtools_1777209684"
    )
    MDTOOLS_AUDIT_PATH = MDTOOLS_BUNDLE_DIR / "pi-audit.jsonl"
    MDTOOLS_GUARD_PATH = MDTOOLS_BUNDLE_DIR / "guard.log"
    MDTOOLS_RUN_JSON = (
        Path(__file__).resolve().parents[1]
        / "bench/runs/checkpoint-pi-T1-mdtools-gpt5.4mini-2026-04-26/run.json"
    )
    MDTOOLS_RESULTS_JSON = (
        Path(__file__).resolve().parents[1]
        / "bench/runs/checkpoint-pi-T1-mdtools-gpt5.4mini-2026-04-26/results.json"
    )

    EXPECTED_KIND_SEQUENCE = ["query"]

    def test_audit_only_summary_pins_t1_hybrid_baseline(self) -> None:
        if not self.HYBRID_AUDIT_PATH.exists():
            self.skipTest(f"iter-53 T1 hybrid PI bundle audit not present at {self.HYBRID_AUDIT_PATH}")
        events = load_pi_audit_events(self.HYBRID_AUDIT_PATH)
        # 4 events: model_change + thinking_level_change + 1 tool_call + 1 tool_result.
        self.assertEqual(len(events), 4)

        counters = summarize_pi_audit_events(events)
        self.assertEqual(counters.tool_calls, 1)
        self.assertEqual(counters.tool_results, 1)
        self.assertEqual(counters.tool_errors, 0)
        self.assertEqual(counters.mutations, 0)
        self.assertFalse(counters.requeried)
        self.assertEqual(counters.policy_violations, 0)
        self.assertEqual(counters.blocked, 0)
        self.assertEqual(counters.bytes_observation, 2265)
        self.assertEqual(counters.model, "openai-codex/gpt-5.4-mini")
        self.assertEqual(counters.thinking_level, "minimal")
        # Single-tool-call PASS trajectory: ./md outline --json. The hybrid
        # prompt's expanded tool-docs section (+355 bytes vs MDTOOLS_DOCS)
        # does not push the agent toward unix tools when an md command
        # aligns with the structural contract.
        self.assertEqual(len(counters.bash_commands), 1)
        self.assertIn("./md outline", counters.bash_commands[0])
        self.assertIn("--json", counters.bash_commands[0])
        self.assertIn("t1_mixed_headings.md", counters.bash_commands[0])
        kinds = [classify_command_kind(cmd) for cmd in counters.bash_commands]
        self.assertEqual(kinds, self.EXPECTED_KIND_SEQUENCE)

    def test_guard_events_preserve_t1_hybrid_audit_vs_guard_symmetry(self) -> None:
        if not self.HYBRID_AUDIT_PATH.exists() or not self.HYBRID_GUARD_PATH.exists():
            self.skipTest(f"iter-53 T1 hybrid PI bundle artifacts not present at {self.HYBRID_BUNDLE_DIR}")
        events = load_pi_audit_events(self.HYBRID_AUDIT_PATH)
        guard_events = load_guard_events(self.HYBRID_GUARD_PATH)
        # 1 guard.log entry — allow + md + ./md outline ... --json.
        self.assertEqual(len(guard_events), 1)
        self.assertEqual(guard_events[0].decision, "allow")
        self.assertEqual(guard_events[0].base_command, "md")
        self.assertIn("./md outline", guard_events[0].raw_command)
        self.assertIn("--json", guard_events[0].raw_command)

        # Audit-vs-guard symmetry on policy_violations: both 0 because all
        # guard decisions are 'allow' (no decision='deny' entries to count).
        # This is the structural counterpart to T12's asymmetry (0 audit-
        # only, 1 guard-augmented) — when no deny event exists in guard.log,
        # both code paths produce identical counters.
        audit_only = summarize_pi_audit_events(events)
        guard_augmented = summarize_pi_audit_events(events, guard_events=guard_events)
        self.assertEqual(audit_only.policy_violations, 0)
        self.assertEqual(guard_augmented.policy_violations, 0)
        self.assertEqual(audit_only.mutations, 0)
        self.assertEqual(guard_augmented.mutations, 0)
        self.assertFalse(audit_only.requeried)
        self.assertFalse(guard_augmented.requeried)
        # Guard-derived kind sequence matches audit-derived bit-exact.
        guard_kinds = [
            classify_command_kind(e.raw_command, e.base_command) for e in guard_events
        ]
        self.assertEqual(guard_kinds, self.EXPECTED_KIND_SEQUENCE)

    def test_t1_hybrid_pairs_apples_to_apples_with_t1_mdtools(self) -> None:
        if not (
            self.HYBRID_RESULTS_JSON.exists()
            and self.HYBRID_RUN_JSON.exists()
            and self.MDTOOLS_RESULTS_JSON.exists()
            and self.MDTOOLS_RUN_JSON.exists()
        ):
            self.skipTest("T1 hybrid or T1 mdtools PI bundle metadata not present")

        hybrid_results = json.loads(self.HYBRID_RESULTS_JSON.read_text())
        mdtools_results = json.loads(self.MDTOOLS_RESULTS_JSON.read_text())
        hybrid_run = json.loads(self.HYBRID_RUN_JSON.read_text())
        mdtools_run = json.loads(self.MDTOOLS_RUN_JSON.read_text())

        self.assertEqual(len(hybrid_results), 1)
        self.assertEqual(len(mdtools_results), 1)
        h_row = hybrid_results[0]
        m_row = mdtools_results[0]

        # Six-axis apples-to-apples match: task_id, model, thinking_level,
        # executor (runner), runs_per_task, task-set version (selected_task_ids).
        # Mode (mdtools vs hybrid) is the only varying axis. This is the
        # first PI-internal cross-mode comparable cell on all six normalization
        # axes from the spec's comparability rule.
        self.assertEqual(h_row["task_id"], "T1")
        self.assertEqual(m_row["task_id"], "T1")
        self.assertEqual(h_row["model"], m_row["model"])
        self.assertEqual(h_row["model"], "openai-codex/gpt-5.4-mini")
        self.assertEqual(h_row["thinking_level"], m_row["thinking_level"])
        self.assertEqual(h_row["thinking_level"], "minimal")
        self.assertEqual(hybrid_run["runner"], mdtools_run["runner"])
        self.assertEqual(hybrid_run["runner"], "pi-json")
        self.assertEqual(hybrid_run["runs_per_task"], mdtools_run["runs_per_task"])
        self.assertEqual(hybrid_run["runs_per_task"], 1)
        self.assertEqual(hybrid_run["selected_task_ids"], mdtools_run["selected_task_ids"])
        self.assertEqual(hybrid_run["selected_task_ids"], ["T1"])

        # Mode is the varying axis.
        self.assertEqual(h_row["mode"], "hybrid")
        self.assertEqual(m_row["mode"], "mdtools")
        self.assertEqual(hybrid_run["modes"], ["hybrid"])
        self.assertEqual(mdtools_run["modes"], ["mdtools"])

        # holdout_version: hybrid bundle has explicit 1 (post-iter-17 stamping),
        # mdtools bundle predates iter-17 stamping and lacks the field.
        # iter-17's forward-compat 'absence reads as v1 by date inference'
        # convention: .get('holdout_version', 1) yields 1 for both.
        self.assertEqual(hybrid_run["holdout_version"], 1)
        self.assertNotIn("holdout_version", mdtools_run)
        self.assertEqual(hybrid_run.get("holdout_version", 1), mdtools_run.get("holdout_version", 1))

        # Both PASS dual-scorer with the same trajectory shape (1 tool call,
        # 0 mutations, requeried=False, policy_violations=0, tool_errors=0
        # implicit via runner_error=null + no error counters).
        self.assertTrue(h_row["correct"] and h_row["correct_neutral"])
        self.assertTrue(m_row["correct"] and m_row["correct_neutral"])
        self.assertEqual(h_row["tool_calls"], m_row["tool_calls"])
        self.assertEqual(h_row["tool_calls"], 1)
        self.assertEqual(h_row["mutations"], m_row["mutations"])
        self.assertEqual(h_row["mutations"], 0)
        self.assertFalse(h_row["requeried"])
        self.assertFalse(m_row["requeried"])
        self.assertEqual(h_row["policy_violations"], m_row["policy_violations"])
        self.assertEqual(h_row["policy_violations"], 0)
        self.assertIsNone(h_row["runner_error"])
        self.assertIsNone(m_row["runner_error"])

        # The +355-byte bytes_prompt delta evidences the HYBRID_DOCS prompt
        # template's expanded tool-docs section vs MDTOOLS_DOCS at
        # bench/harness.py:282-283. This is a typed-checkable invariant
        # converting iter-53's prose claim into a mechanical assertion.
        self.assertEqual(h_row["bytes_prompt"], 4545)
        self.assertEqual(m_row["bytes_prompt"], 4190)
        self.assertEqual(h_row["bytes_prompt"] - m_row["bytes_prompt"], 355)


if __name__ == "__main__":
    unittest.main()
