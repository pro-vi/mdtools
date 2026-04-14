from __future__ import annotations

import json
import unittest

from bench.oai_loop import format_final_output, normalize_api_base, parse_action_text


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


if __name__ == "__main__":
    unittest.main()
