from __future__ import annotations

import json
import unittest

from bench.harness import extract_last_json


class HarnessJsonExtractionTests(unittest.TestCase):
    def test_extract_last_json_preserves_top_level_object(self) -> None:
        payload = json.dumps({"file": "doc.md", "entries": [{"heading": {"text": "One"}}]})
        self.assertEqual(json.loads(extract_last_json(payload)), json.loads(payload))


if __name__ == "__main__":
    unittest.main()
