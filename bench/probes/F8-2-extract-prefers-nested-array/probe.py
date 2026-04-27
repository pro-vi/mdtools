"""Probe for F8-2: extract_last_json prefers a nested array over the wrapping object.

Failure-class hypothesis (scorer false-NEGATIVE on existing surface):

  `extract_last_json` (bench/harness.py:1539) walks the agent's text
  twice — once accumulating bracket-balanced `[...]` candidates, once
  accumulating brace-balanced `{...}` candidates — then returns
  `arrays[-1]` if any array was found, else `objects[-1]`.

  When the agent's text contains the correct envelope answer (a dict
  whose `entries`/`results`/`links` field is a non-empty list of dicts)
  embedded in prose, the inner list is one bracket-balanced candidate
  and the wrapping dict is one brace-balanced candidate. Because
  arrays are unconditionally preferred, the inner list is returned.

  Downstream `select_json_envelope_actual` (text-output branch,
  harness.py:1525-1532) does NOT shape-check the returned candidate:
  it only requires non-empty parseable JSON. So the inner list is
  passed through to `score_structural_json`, which immediately rejects
  any non-dict actual with the "expected top-level JSON object"
  message. The task is marked FAIL even though the agent's correct
  envelope answer was present in the text.

  This is the text-output mirror of F8-1 (which addressed the
  tool-output shape-match path). F8-1 ensured tool outputs are
  shape-checked via `_expected_json_top_keys` subset; F8-2 is the
  remaining gap on the text-output fallback path.

This script is a standalone demonstration, not a unit test. Run with:

    python3 bench/probes/F8-2-extract-prefers-nested-array/probe.py

Exit code 0 = scorer behaves correctly (probe inert / hardening landed).
Exit code 1 = scorer false-NEGATIVE reproduced (probe live).
Exit code 2 = unexpected output (neither candidate observed).

Companion artifacts (added in a follow-up iteration when the finding
is closed): a hardening to `extract_last_json` so a wrapping dict is
preferred over a nested array, plus a unit test in
`bench/test_harness_json.py` pinning the new precedence.
"""

import json
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", ".."))

from bench.harness import extract_last_json, select_json_envelope_actual

WRAPPING_ENVELOPE = (
    '{"file":"doc.md","entries":[{"heading":{"level":1,"text":"Top"}}]}'
)
NESTED_ARRAY = '[{"heading":{"level":1,"text":"Top"}}]'

AGENT_TEXT = (
    "The headings for doc.md are:\n"
    "- # Top\n"
    "\n"
    "Here is the JSON:\n"
    f"{WRAPPING_ENVELOPE}\n"
)


def _stage_a_extract_last_json() -> str:
    """Direct call: text containing wrapping envelope returns the nested array."""
    return extract_last_json(AGENT_TEXT)


def _stage_b_select_json_envelope_actual() -> str:
    """Harness path: select_json_envelope_actual on a text-only transcript
    where the expected envelope shape is the wrapping dict, but the agent
    only produced text outputs. The select path forwards extract_last_json's
    return value without a shape check, so the nested array propagates."""
    expected = WRAPPING_ENVELOPE
    return select_json_envelope_actual(
        all_tool_outputs=[],
        all_text_outputs=[AGENT_TEXT],
        stdout="",
        expected_str=expected,
    )


def main() -> int:
    a = _stage_a_extract_last_json()
    b = _stage_b_select_json_envelope_actual()

    print("Stage A — extract_last_json on raw agent text:")
    print(f"  returned (first 100 chars): {a[:100]}")
    parsed_a = json.loads(a)
    print(f"  parsed type: {type(parsed_a).__name__}")
    print()
    print("Stage B — select_json_envelope_actual (text-only transcript):")
    print(f"  returned (first 100 chars): {b[:100]}")
    parsed_b = json.loads(b)
    print(f"  parsed type: {type(parsed_b).__name__}")
    print()

    nested_array_returned = a.strip() == NESTED_ARRAY and b.strip() == NESTED_ARRAY
    wrapping_envelope_returned = (
        json.loads(a) == json.loads(WRAPPING_ENVELOPE)
        and json.loads(b) == json.loads(WRAPPING_ENVELOPE)
    )

    if wrapping_envelope_returned:
        print("VERDICT: PASS — extractor preferred wrapping envelope on both stages.")
        return 0
    if nested_array_returned:
        print(
            "VERDICT: FAIL — extractor returned the nested entries array on both "
            "stages. Downstream score_structural_json will mark the task FAIL "
            "because the actual is a list and expected is a dict, even though "
            "the agent's correct envelope answer was present in the text."
        )
        return 1
    print(
        "VERDICT: UNEXPECTED — neither candidate was returned cleanly: "
        f"stage_a={a!r}, stage_b={b!r}"
    )
    return 2


if __name__ == "__main__":
    sys.exit(main())
