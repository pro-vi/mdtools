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


if __name__ == "__main__":
    unittest.main()
