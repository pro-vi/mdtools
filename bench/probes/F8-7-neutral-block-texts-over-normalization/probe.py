#!/usr/bin/env python3
"""
F8-7 attribution probe — `neutral_block_texts` over-normalizes block content
in two distinct ways: (1) it hardcodes `"---"` for any horizontal rule
regardless of source style (`---`/`***`/`___`), and (2) it drops the heading
marker prefix (`# `, `## `, etc.) and underline (setext) so heading blocks are
indistinguishable from paragraph blocks in the output. `_md_block_texts`
preserves both via byte slicing of the raw source. The two scorers therefore
disagree on `compare_block_text` tasks where actual and expected differ in
hr style or heading-vs-paragraph structure: md correctly reports MISMATCH
while neutral falsely reports OK.

Severity: SCORER DIVERGENCE on the cross-check direction. The harness's
`BenchResult.correct = ok_md` (harness.py:1456) means the gating scorer (md)
correctly catches both classes today, so the binary correct flag is preserved
on a wrong agent answer. But the neutral scorer's role is to provide an
independent cross-check; over-normalization defeats that purpose and would
mask any future md-side regression on hr fidelity or heading prefix bytes.
The asymmetry is the mirror of F8-6 (which had md as the over-lenient side
on heading inline formatting); F8-7 puts the over-lenient side on neutral
for block-level structural representation.

Stage A — direct extractor divergence on a single mixed document containing
multiple hr styles and headings of different levels.

Stage B — end-to-end SCORER DIVERGENCE on a `compare_block_text` task where
the agent has dropped a heading and emitted a plain-paragraph version (a
realistic doc-maintenance failure mode); md catches via byte slicing,
neutral falsely reports OK because it loses the heading marker.

Run:
    python3 bench/probes/F8-7-neutral-block-texts-over-normalization/probe.py

Exit 1 = live failing trace. Exit 0 = inert (closure or false alarm).
"""

import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(
    os.path.abspath(__file__)))))
sys.path.insert(0, ROOT)

from bench.harness import (  # noqa: E402
    StructuralDiffPolicy,
    _md_block_texts,
    neutral_block_texts,
    score_normalized_text_md,
    score_normalized_text_neutral,
)

MD_BINARY = os.environ.get("MD_BINARY", os.path.join(ROOT, "target/release/md"))


def stage_a_direct_extractor_divergence() -> bool:
    """Two scorers return different block-text lists on identical input.

    `_md_block_texts` byte-slices the raw source: hr blocks preserve the
    actual marker (`---`/`***`/`___`), heading blocks include the level
    marker prefix or setext underline. `neutral_block_texts` hardcodes
    `"---"` for any hr token and emits only the inline content of heading
    tokens (no `# ` prefix, no `=====` underline).

    Returns True if divergence is live (failure observable), False if inert.
    """
    sample = (
        "# Hello\n"
        "\n"
        "---\n"
        "\n"
        "## Subsection\n"
        "\n"
        "***\n"
        "\n"
        "foo\n"
        "\n"
        "___\n"
        "\n"
        "bar\n"
    )
    md = _md_block_texts(sample, MD_BINARY)
    neutral = neutral_block_texts(sample)
    print("=== Stage A: direct extractor divergence ===")
    print(f"  md      = {md}")
    print(f"  neutral = {neutral}")
    if md == neutral:
        print("  [INERT] both extractors agree (closure or false alarm)")
        return False
    diffs = []
    if md[1] != neutral[1]:
        diffs.append(f"hr#1: md={md[1]!r} vs neutral={neutral[1]!r}")
    if md[3] != neutral[3]:
        diffs.append(f"hr#2: md={md[3]!r} vs neutral={neutral[3]!r}")
    if md[5] != neutral[5]:
        diffs.append(f"hr#3: md={md[5]!r} vs neutral={neutral[5]!r}")
    if md[0] != neutral[0]:
        diffs.append(f"h1: md={md[0]!r} vs neutral={neutral[0]!r}")
    if md[2] != neutral[2]:
        diffs.append(f"h2: md={md[2]!r} vs neutral={neutral[2]!r}")
    print(f"  [LIVE] extractors disagree on {len(diffs)} blocks:")
    for d in diffs:
        print(f"    {d}")
    return True


def stage_b_scorer_divergence_heading_dropped() -> bool:
    """End-to-end: md scorer correctly reports MISMATCH when the agent has
    dropped a heading (heading→paragraph conversion); neutral falsely says OK
    because it drops the `# ` prefix from the expected and the actual is
    already a plain paragraph with the same inline content.

    This is the SCORER DIVERGENCE direction where neutral is over-lenient
    (vs F8-6 where md was over-lenient). The current `BenchResult.correct =
    ok_md` gates correctly today, so the binary correct flag preserves
    benchmark integrity on this trace. The fileable defect is the broken
    cross-check: a future md-side regression that mirrored neutral's
    over-leniency would not be caught.
    """
    expected = (
        "# Configuration\n"
        "\n"
        "Settings live here.\n"
    )
    actual = (
        "Configuration\n"
        "\n"
        "Settings live here.\n"
    )

    policy = StructuralDiffPolicy(
        kind="normalized_text",
        normalize_line_endings=True,
        ignore_trailing_whitespace=True,
        compare_frontmatter_json=False,
        compare_heading_tree=False,
        compare_block_order=False,
        compare_link_destinations=False,
        compare_block_text=True,
    )

    md_report: list[str] = []
    neutral_report: list[str] = []
    ok_md = score_normalized_text_md(policy, actual, expected, MD_BINARY, md_report)
    ok_neutral = score_normalized_text_neutral(policy, actual, expected, neutral_report)

    print()
    print("=== Stage B: end-to-end SCORER DIVERGENCE on compare_block_text ===")
    for line in md_report:
        print(f"  md:      {line}")
    for line in neutral_report:
        print(f"  neutral: {line}")
    print(f"  ok_md={ok_md}  ok_neutral={ok_neutral}")

    # Live failing trace shape: scorers disagree.
    # F8-7's specific shape is md=False (correctly catches dropped heading)
    # while neutral=True (falsely accepts heading→paragraph conversion).
    if ok_md != ok_neutral:
        if not ok_md and ok_neutral:
            print("  [LIVE] neutral falsely accepts dropped-heading (over-leniency)")
        else:
            print(f"  [LIVE] unexpected divergence shape: md={ok_md} neutral={ok_neutral}")
        return True
    print("  [INERT] scorers agree (closure or false alarm)")
    return False


def stage_c_scorer_divergence_hr_style() -> bool:
    """End-to-end: agent emits `***` for a horizontal rule where expected
    has `---`. md scorer correctly says MISMATCH (byte slice preserves the
    actual marker); neutral scorer says OK because both hr tokens normalize
    to hardcoded `"---"`.

    Same SCORER DIVERGENCE class as Stage B but on a different surface
    feature (hr style preservation vs heading prefix preservation).
    """
    expected = (
        "# Hello\n"
        "\n"
        "---\n"
        "\n"
        "foo\n"
    )
    actual = (
        "# Hello\n"
        "\n"
        "***\n"
        "\n"
        "foo\n"
    )

    policy = StructuralDiffPolicy(
        kind="normalized_text",
        normalize_line_endings=True,
        ignore_trailing_whitespace=True,
        compare_frontmatter_json=False,
        compare_heading_tree=False,
        compare_block_order=False,
        compare_link_destinations=False,
        compare_block_text=True,
    )

    md_report: list[str] = []
    neutral_report: list[str] = []
    ok_md = score_normalized_text_md(policy, actual, expected, MD_BINARY, md_report)
    ok_neutral = score_normalized_text_neutral(policy, actual, expected, neutral_report)

    print()
    print("=== Stage C: end-to-end SCORER DIVERGENCE on hr style ===")
    for line in md_report:
        print(f"  md:      {line}")
    for line in neutral_report:
        print(f"  neutral: {line}")
    print(f"  ok_md={ok_md}  ok_neutral={ok_neutral}")

    if ok_md != ok_neutral:
        if not ok_md and ok_neutral:
            print("  [LIVE] neutral falsely accepts hr style mismatch (over-leniency)")
        else:
            print(f"  [LIVE] unexpected divergence shape: md={ok_md} neutral={ok_neutral}")
        return True
    print("  [INERT] scorers agree (closure or false alarm)")
    return False


def main() -> int:
    a_live = stage_a_direct_extractor_divergence()
    b_live = stage_b_scorer_divergence_heading_dropped()
    c_live = stage_c_scorer_divergence_hr_style()
    if a_live or b_live or c_live:
        print("\nRESULT: F8-7 is LIVE (exit 1)")
        return 1
    print("\nRESULT: F8-7 is inert (exit 0)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
