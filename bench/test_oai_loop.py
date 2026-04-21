from __future__ import annotations

import json
import tempfile
import threading
import time
import unittest
from pathlib import Path
from unittest.mock import patch

from bench import oai_loop
from bench.oai_loop import format_final_output, normalize_api_base, parse_action_text, run_oai_loop


class OAILoopTests(unittest.TestCase):
    def test_normalize_api_base_appends_v1(self) -> None:
        self.assertEqual(normalize_api_base("http://127.0.0.1:10240"), "http://127.0.0.1:10240/v1")
        self.assertEqual(normalize_api_base("http://127.0.0.1:10240/v1"), "http://127.0.0.1:10240/v1")

    def test_parse_action_text_accepts_wrapped_json(self) -> None:
        action = parse_action_text("```json\n{\"type\":\"bash\",\"command\":\"md outline doc.md --json\"}\n```")
        self.assertEqual(action.action_type, "bash")
        self.assertEqual(action.command, "md outline doc.md --json")

    def test_parse_action_text_rejects_command_separators(self) -> None:
        with self.assertRaises(ValueError):
            parse_action_text(json.dumps({"type": "bash", "command": "cat doc.md && md outline doc.md --json"}))

    def test_format_final_output_serializes_json_values(self) -> None:
        rendered = format_final_output({"ok": True, "items": [1, 2]})
        self.assertEqual(json.loads(rendered), {"ok": True, "items": [1, 2]})

    def test_run_oai_loop_counts_invalid_responses(self) -> None:
        # Hermes-4-70B-4bit on T16 surfaced a failure mode where the model
        # emits a separator-bearing bash command for N consecutive turns and
        # never recovers. Track those parse_action_text rejections as a
        # first-class model-capability probe instead of hiding them inside turns/tool_calls.
        scripted_assistant_replies = [
            json.dumps({"type": "bash", "command": "cat doc.md && md outline doc.md --json"}),
            json.dumps({"type": "bash", "command": "cat doc.md; md outline doc.md --json"}),
            json.dumps({"type": "final", "output": "done"}),
        ]
        responses = iter(
            {"choices": [{"message": {"content": reply}}]}
            for reply in scripted_assistant_replies
        )

        def fake_request_json(_base, _key, _path, _payload, _timeout):
            return next(responses)

        with tempfile.TemporaryDirectory(prefix="bench_oai_loop_invalid_") as tmpdir, \
                patch.object(oai_loop, "_request_json", side_effect=fake_request_json):
            trace = run_oai_loop(
                api_base="http://127.0.0.1:10240",
                api_key="test-key",
                model="test-model",
                prompt="task",
                workdir=Path(tmpdir),
                env={"PATH": "/usr/bin"},
                max_turns=5,
                request_timeout_seconds=1,
                tool_timeout_seconds=1,
            )

        self.assertEqual(trace.invalid_responses, 2)
        self.assertEqual(trace.tool_calls, 0)
        self.assertEqual(trace.turns, 3)
        self.assertEqual(trace.text_outputs, ["done"])

    def test_request_json_watchdog_fires_when_blocking_exceeds_deadline(self) -> None:
        # urllib.urlopen(timeout=N) can fail to raise on streaming completions
        # when keepalives reset the socket read timer (observed on Hermes-4-70B
        # via OMLX: harness sat on a single POST for 10+ min). The watchdog
        # enforces a hard wall-time bound so a stuck request becomes a recorded
        # runner_error rather than a silent hang.
        release = threading.Event()

        def blocking(*_args, **_kwargs):
            release.wait(timeout=5)
            return {"ok": True}

        with patch.object(oai_loop, "_do_request_json", side_effect=blocking), \
                patch.object(oai_loop, "WATCHDOG_MARGIN_SECONDS", 0):
            started = time.monotonic()
            with self.assertRaises(TimeoutError) as ctx:
                oai_loop._request_json(
                    "http://127.0.0.1:10240/v1",
                    "test-key",
                    "/chat/completions",
                    {"model": "x"},
                    timeout_seconds=1,
                )
            elapsed = time.monotonic() - started
        release.set()  # let the abandoned worker finish

        self.assertIn("wall-time deadline", str(ctx.exception))
        self.assertLess(elapsed, 3.0)

    def test_request_json_returns_fast_response_without_tripping_watchdog(self) -> None:
        fixture = {"choices": [{"message": {"content": "ok"}}]}
        with patch.object(oai_loop, "_do_request_json", return_value=fixture):
            result = oai_loop._request_json(
                "http://127.0.0.1:10240/v1",
                "test-key",
                "/chat/completions",
                {"model": "x"},
                timeout_seconds=5,
            )
        self.assertEqual(result, fixture)

    def test_request_json_propagates_inner_error(self) -> None:
        with patch.object(
            oai_loop,
            "_do_request_json",
            side_effect=RuntimeError("OAI request failed with HTTP 500"),
        ):
            with self.assertRaises(RuntimeError) as ctx:
                oai_loop._request_json(
                    "http://127.0.0.1:10240/v1",
                    "test-key",
                    "/chat/completions",
                    {"model": "x"},
                    timeout_seconds=5,
                )
        self.assertIn("HTTP 500", str(ctx.exception))


if __name__ == "__main__":
    unittest.main()
