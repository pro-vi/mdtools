#!/usr/bin/env python3
"""F8-4 attribution probe: extract_last_json's code-fence regex
strips backtick triplets that appear INSIDE JSON string values.

`extract_last_json` runs `re.sub(r"```(?:json)?\\s*\\n?", "", text)`
as a preprocessing step (bench/harness.py:1560) to handle the common
LLM output style where the JSON answer is wrapped in a ```json …
``` fence. The substitution is global and string-blind: any ``` it
encounters is stripped, regardless of whether it is a top-level
markdown fence boundary or a literal substring inside a JSON string
value (e.g. an `entries[].heading.text` or `body` field that contains
a code-fence sample).

Result: the wrapping envelope still parses (JSON syntax is preserved
because ``` are not quote characters), but the string-valued field
is corrupted — three backticks are silently removed from every
appearance inside a string. Downstream structural compare in
`score_structural_json` then FAILs on a correct agent answer because
the actual heading text or body diverges from expected by exactly
the stripped backtick characters.

This is fresh and symmetric to F8-3 but at a different layer:
F8-3 hardened the depth scanner inside extract_last_json against
brace/bracket characters in JSON strings. F8-4 hits the regex
preprocessor that runs *before* the depth scanner, on the
backtick character — same gap shape (string-blind), different layer.

Run: `python3 bench/probes/F8-4-fence-regex-strips-string-content/probe.py`
Exit 0 = inert (closure landed). Exit 1 = live failing trace.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2].parent
sys.path.insert(0, str(REPO_ROOT / "bench"))

from harness import (  # noqa: E402
    StructuralDiffPolicy,
    extract_last_json,
    score_structural_json,
    select_json_envelope_actual,
)

# Realistic envelope: a heading whose text mentions ```python (e.g. a
# spec section talking about code-fence syntax, or a tutorial doc
# whose H2s name the language of each example block). The outer
# md outline --json envelope echoes the heading text verbatim.
ENVELOPE_WITH_FENCE_IN_STRING = {
    "schema_version": "mdtools.v1",
    "file": "/tmp/spec.md",
    "entries": [
        {"heading": {"level": 1, "text": "Configuration", "block_index": 0}},
        {"heading": {"level": 2, "text": "Example: ```python block", "block_index": 1}},
    ],
}
ENVELOPE_STR = json.dumps(ENVELOPE_WITH_FENCE_IN_STRING)


def stage_a_direct_extractor() -> int:
    """Stage A: extract_last_json on agent prose embedding the envelope
    inside a top-level ```json … ``` fence (the canonical LLM output style).

    Pre-fix: the regex strips every occurrence of ``` from the text,
    including the two embedded inside the heading.text value. The
    candidate is still parseable JSON (syntax intact), but the string
    value is corrupted: "Example: ```python block" becomes
    "Example: python block".

    Post-fix: the preprocessor honors string boundaries (or the
    fence-strip is anchored to top-level markdown fence boundaries
    only), so the heading.text round-trips byte-exactly through
    extract_last_json."""
    agent_text = (
        "Sure, here is the document outline:\n"
        "\n"
        "```json\n"
        f"{ENVELOPE_STR}\n"
        "```\n"
        "\n"
        "That is what `md outline --json /tmp/spec.md` produced.\n"
    )

    candidate = extract_last_json(agent_text)
    try:
        parsed = json.loads(candidate)
    except json.JSONDecodeError as exc:
        print(f"[FAIL stage A] candidate is unparseable: {exc}")
        print(f"  candidate prefix: {candidate[:120]!r}")
        return 1

    actual_h2_text = parsed["entries"][1]["heading"]["text"]
    expected_h2_text = ENVELOPE_WITH_FENCE_IN_STRING["entries"][1]["heading"]["text"]
    if actual_h2_text == expected_h2_text:
        print("[OK stage A] extract_last_json preserved heading.text byte-exact")
        return 0

    print("[FAIL stage A] extract_last_json corrupted heading.text")
    print(f"  expected: {expected_h2_text!r}")
    print(f"  actual:   {actual_h2_text!r}")
    return 1


def stage_b_harness_text_branch() -> int:
    """Stage B: end-to-end harness path. The agent emits the CORRECT
    answer in text output (wrapped in a ```json fence). The harness
    selects the envelope via the text-output branch of
    select_json_envelope_actual, then scores via score_structural_json.

    Pre-fix: the regex corruption silently mutates heading.text inside
    extract_last_json. The actual envelope has "Example: python block"
    where expected has "Example: ```python block". score_structural_json
    compares heading_tree, finds a mismatch, and returns FAIL — even
    though the agent's answer was byte-exact correct.

    Post-fix: the actual envelope round-trips heading.text byte-exact,
    score_structural_json returns OK."""
    agent_text = (
        "Looking at the document outline:\n"
        "\n"
        "```json\n"
        f"{ENVELOPE_STR}\n"
        "```\n"
    )
    actual = select_json_envelope_actual(
        all_tool_outputs=[],
        all_text_outputs=[agent_text],
        stdout=agent_text,
        expected_str=ENVELOPE_STR,
    )

    policy = StructuralDiffPolicy(
        kind="structural",
        normalize_line_endings=True,
        ignore_trailing_whitespace=True,
        compare_frontmatter_json=False,
        compare_heading_tree=True,
        compare_block_order=False,
        compare_link_destinations=False,
        compare_block_text=False,
    )
    report: list[str] = []
    ok_md, ok_neutral = score_structural_json(policy, actual, ENVELOPE_STR, report)
    if ok_md and ok_neutral:
        print("[OK stage B] harness scored agent's correct answer as PASS")
        return 0

    print("[FAIL stage B] harness scored agent's correct answer as FAIL")
    print(f"  ok_md={ok_md} ok_neutral={ok_neutral}")
    for line in report:
        print(f"  {line}")
    return 1


def main() -> int:
    rc_a = stage_a_direct_extractor()
    rc_b = stage_b_harness_text_branch()
    if rc_a == 0 and rc_b == 0:
        print("\n[F8-4 inert] fence-strip preserves backticks inside string values")
        return 0
    print("\n[F8-4 live] fence-strip regex is string-blind on backticks")
    return 1


if __name__ == "__main__":
    sys.exit(main())
