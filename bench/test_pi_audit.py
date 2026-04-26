from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from bench.command_policy import GuardEvent
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


if __name__ == "__main__":
    unittest.main()
