"""Probe for F8-1: select_json_envelope_actual prefers schema_version-only overlap.

Failure-class hypothesis (scorer false-negative on existing surface):

  T7 iter 31 closed F4 by replacing "first non-empty JSON wins" with a
  shape-aware preference: tool outputs whose top-level keys intersect
  the expected JSON's top-level keys are preferred. The intersection
  check (`_json_top_keys(parsed) & expected_top_keys`) accepts any
  non-empty overlap.

  All `mdtools.v1` envelopes share the top-level key `schema_version`.
  So when the expected output is one mdtools envelope (e.g. outline)
  and the agent's last tool call emits a *different* mdtools envelope
  (e.g. `md tasks --json`), the scorer accepts the wrong envelope as
  the agent's `actual` answer purely on `schema_version` overlap.
  Downstream structural comparison then mismatches the wrong fields
  and the task is marked FAIL — but the agent's correct answer is in
  an earlier tool output that the F4 selector skipped over.

This script is a standalone demonstration, not a unit test. Run with:

    python3 bench/probes/F8-1-schema-version-overlap/probe.py

Exit code 0 = scorer behaves correctly (probe inert / hardening landed).
Exit code 1 = scorer false-negative reproduced (probe live).

Companion artifacts (added in a follow-up iteration when the finding is
closed): a unit test in bench/test_harness_json.py pinning the
discriminating-key requirement, plus the harness.py change itself.
"""

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", ".."))

from bench.harness import select_json_envelope_actual

OUTLINE_ENVELOPE = (
    '{"schema_version":"mdtools.v1","file":"x.md",'
    '"entries":[{"heading":{"level":1,"text":"H1"}}]}'
)
TASKS_ENVELOPE = (
    '{"schema_version":"mdtools.v1","results":[{"file":"x.md","tasks":[]}]}'
)


def main() -> int:
    # T1-shaped scenario:
    #   Agent invokes `md outline --json` (correct, produces OUTLINE_ENVELOPE),
    #   then `md tasks --json` (exploratory, produces TASKS_ENVELOPE).
    # Expected output is the outline envelope shape.
    expected = OUTLINE_ENVELOPE
    all_tool_outputs = [OUTLINE_ENVELOPE, TASKS_ENVELOPE]  # chronological
    all_text_outputs: list[str] = []
    stdout = ""

    actual = select_json_envelope_actual(
        all_tool_outputs, all_text_outputs, stdout, expected
    )

    print("expected envelope keys: schema_version, file, entries")
    print("tasks envelope keys:    schema_version, results")
    print("intersection:           {schema_version}  (single universal key)")
    print()
    print(f"actual selected (first 100 chars): {actual[:100]}")
    print()

    if actual == OUTLINE_ENVELOPE:
        print("VERDICT: PASS — scorer correctly preferred outline envelope.")
        return 0
    if actual == TASKS_ENVELOPE:
        print(
            "VERDICT: FAIL — scorer selected tasks envelope on "
            "schema_version-only overlap."
        )
        print(
            "  Downstream structural comparison will see entries=missing "
            "on the tasks envelope and mark the task FAIL even though the "
            "agent's correct answer is in all_tool_outputs[0]."
        )
        return 1
    print(f"VERDICT: UNEXPECTED — actual is neither candidate: {actual!r}")
    return 2


if __name__ == "__main__":
    sys.exit(main())
