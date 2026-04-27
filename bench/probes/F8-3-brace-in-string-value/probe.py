#!/usr/bin/env python3
"""F8-3 attribution probe: extract_last_json's depth scanner does not
honor JSON string boundaries.

When an mdtools envelope embedded in agent prose carries a string
value (e.g. heading.text) that contains a `}`, `]`, `{`, or `[`
character, the scanner counts the brace inside the string as a real
closer. The first wrapping-envelope candidate it tries to record is
truncated mid-envelope, fails json.loads, and resets `start = -1`.
The scanner then never re-anchors on the actual outer envelope, so
the only candidate that survives is the inner `entries`/`results`
array (recorded by the separate `[`/`]` pass, which is brace-blind
in a benign way).

The candidate returned to the harness is therefore a list, not the
expected object envelope. score_structural_json early-outs FAIL on
"expected top-level JSON object."

This is a fresh failing trace symmetric to F8-2 but on a different
axis: F8-2 was about the preference rule (array vs object) when both
candidates are valid; F8-3 is about candidate enumeration itself —
the wrapping envelope never enters the candidate set.

Run: `python3 bench/probes/F8-3-brace-in-string-value/probe.py`
Exit 0 = inert (closure landed). Exit 1 = live failing trace.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2].parent
sys.path.insert(0, str(REPO_ROOT / "bench"))

from harness import extract_last_json, select_json_envelope_actual  # noqa: E402

# An mdtools.v1 envelope where heading.text contains a single `}`.
# The shape mirrors what `md outline --json` would emit on a document
# whose heading is e.g. "Configuration `{key: value}` syntax".
ENVELOPE_WITH_BRACE_IN_STRING = (
    '{"schema_version":"mdtools.v1","file":"/tmp/x.md",'
    '"entries":[{"heading":{"level":1,"text":"Heading with } brace",'
    '"block_index":0}}]}'
)
EXPECTED_ENVELOPE_SHAPE = (
    '{"schema_version":"mdtools.v1","file":"...","entries":[]}'
)


def stage_a_direct_extractor() -> int:
    """Stage A: extract_last_json on agent prose embedding the envelope.

    Pre-fix: the `{`/`}` depth scanner gets confused by the `}` inside
    `"Heading with } brace"` and never records the wrapping envelope as
    a candidate. The only surviving candidate is the inner `entries`
    array (parsed by the `[`/`]` pass). The function returns the array.
    Post-fix: extract_last_json honors JSON string boundaries and
    returns the wrapping envelope object."""
    agent_text = (
        "I ran `md outline --json /tmp/x.md` and got this answer:\n"
        f"{ENVELOPE_WITH_BRACE_IN_STRING}\n"
        "That is the document outline.\n"
    )

    candidate = extract_last_json(agent_text)
    try:
        parsed = json.loads(candidate)
    except json.JSONDecodeError as exc:
        print(f"[FAIL stage A] candidate is unparseable: {exc}")
        print(f"  candidate prefix: {candidate[:80]!r}")
        return 1

    if isinstance(parsed, dict) and "schema_version" in parsed:
        print("[OK stage A] extract_last_json returned the wrapping envelope")
        return 0

    print("[FAIL stage A] extract_last_json returned the wrong candidate")
    print(f"  type: {type(parsed).__name__}")
    print(f"  candidate prefix: {candidate[:80]!r}")
    print(f"  expected: dict with schema_version key")
    return 1


def stage_b_harness_text_branch() -> int:
    """Stage B: select_json_envelope_actual text-output branch.

    The harness reaches the text-output branch when no tool-output is
    shape-matching. extract_last_json's misextraction propagates here:
    the inner array passes the parseability/non-empty check at lines
    1525–1532 unchanged, and is returned as `actual` to
    score_structural_json. The downstream comparison fails."""
    agent_prose = (
        "Here is the answer envelope:\n"
        f"{ENVELOPE_WITH_BRACE_IN_STRING}\n"
    )
    actual = select_json_envelope_actual(
        all_tool_outputs=[],
        all_text_outputs=[agent_prose],
        stdout=agent_prose,
        expected_str=EXPECTED_ENVELOPE_SHAPE,
    )
    try:
        parsed = json.loads(actual)
    except json.JSONDecodeError as exc:
        print(f"[FAIL stage B] actual is unparseable: {exc}")
        print(f"  actual prefix: {actual[:80]!r}")
        return 1

    if isinstance(parsed, dict) and "schema_version" in parsed:
        print("[OK stage B] select_json_envelope_actual surfaced the envelope")
        return 0

    print("[FAIL stage B] select_json_envelope_actual surfaced the wrong shape")
    print(f"  type: {type(parsed).__name__}")
    print(f"  actual prefix: {actual[:80]!r}")
    print(f"  expected: dict envelope with schema_version")
    return 1


def main() -> int:
    rc_a = stage_a_direct_extractor()
    rc_b = stage_b_harness_text_branch()
    if rc_a == 0 and rc_b == 0:
        print("\n[F8-3 inert] both stages returned the wrapping envelope")
        return 0
    print("\n[F8-3 live] depth scanner is brace-in-string blind")
    return 1


if __name__ == "__main__":
    sys.exit(main())
