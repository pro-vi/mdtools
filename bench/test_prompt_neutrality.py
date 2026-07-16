from __future__ import annotations

import json
import re
import unittest
from pathlib import Path


TASKS_PATH = Path("bench/tasks/tasks.json")
ADVERSARIALLY_MINED = {"C-T10-15", "C-T10-28", "C-AR-040", "C-AR-041"}
PROVENANCE_VALUES = {"core", "adversarially-mined"}

MD_SUBCOMMANDS = {
    "replace-section",
    "set-task",
    "move-section",
    "insert-block",
    "delete-block",
    "delete-section",
    "replace-block",
    "replace-table-row",
}
FORBIDDEN_LITERALS = {
    "block index",
    "loc ",
    "re-read the file structure",
    "--from",
    "-i ",
}


def _load_tasks() -> list[dict]:
    return json.loads(TASKS_PATH.read_text())


def prompt_neutrality_violations(tasks: list[dict]) -> list[str]:
    violations: list[str] = []
    md_command = re.compile(
        r"\bmd\s+(?:" + "|".join(re.escape(cmd) for cmd in sorted(MD_SUBCOMMANDS)) + r")\b"
    )
    for task in tasks:
        task_id = task.get("id", "<unknown>")
        description = task.get("description", "")
        lower = description.lower()
        if md_command.search(lower):
            violations.append(f"{task_id}: md command named in prompt")
        for literal in sorted(FORBIDDEN_LITERALS | MD_SUBCOMMANDS):
            if literal in lower:
                violations.append(f"{task_id}: forbidden prompt token {literal!r}")
    return violations


class PromptNeutralityTests(unittest.TestCase):
    def test_default_corpus_prompts_are_neutral(self) -> None:
        self.assertEqual(prompt_neutrality_violations(_load_tasks()), [])

    def test_lint_catches_planted_md_command(self) -> None:
        fixture = [
            {
                "id": "BAD",
                "description": 'Use md set-task 1.2 file.md -i to complete this task.',
            }
        ]
        self.assertIn("BAD: md command named in prompt", prompt_neutrality_violations(fixture))

    def test_every_task_has_v3_provenance(self) -> None:
        wrong = []
        for task in _load_tasks():
            expected = "adversarially-mined" if task["id"] in ADVERSARIALLY_MINED else "core"
            actual = task.get("provenance")
            if actual != expected or actual not in PROVENANCE_VALUES:
                wrong.append((task["id"], actual, expected))
        self.assertEqual(wrong, [])


if __name__ == "__main__":
    unittest.main()
