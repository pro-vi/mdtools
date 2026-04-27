#!/usr/bin/env python3
"""
F8-6 attribution probe — `_md_heading_tree` strips inline markdown formatting
(emphasis, inline code, links) while `neutral_heading_tree` returns the raw
markdown source. Two `compare_heading_tree` policy paths therefore disagree
silently, and the harness's `BenchResult.correct = ok_md` (harness.py:1437)
records a wrong agent answer as PASS while `score_normalized_text_neutral`
correctly reports MISMATCH.

Stage A — direct extractor divergence on a single heading with inline
markdown.

Stage B — end-to-end SCORER DIVERGENCE on a `compare_heading_tree` task
where the agent's answer differs from expected only in inline formatting.
The md scorer says OK (false-POSITIVE PASS), neutral correctly MISMATCH.

Run:
    python3 bench/probes/F8-6-heading-tree-inline-formatting-divergence/probe.py

Exit 1 = live failing trace. Exit 0 = inert (closure or false alarm).
"""

import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(
    os.path.abspath(__file__)))))
sys.path.insert(0, ROOT)

from bench.harness import (  # noqa: E402
    StructuralDiffPolicy,
    _md_heading_tree,
    neutral_heading_tree,
    score_normalized_text_md,
    score_normalized_text_neutral,
)

MD_BINARY = os.environ.get("MD_BINARY", os.path.join(ROOT, "target/release/md"))


def stage_a_direct_extractor_divergence() -> bool:
    """Two scorers return different heading trees on identical input.

    `_md_heading_tree` returns rendered plaintext (markdown formatting
    stripped). `neutral_heading_tree` returns the raw markdown source.
    Realistic technical-Markdown headings (inline code for command names,
    bold/italic for emphasis, links to docs) hit this divergence.

    Returns True if divergence is live (failure observable), False if inert.
    """
    sample = (
        "# The `md tasks` command\n"
        "\n"
        "Body.\n"
        "\n"
        "## **Important** notes\n"
        "\n"
        "More body.\n"
        "\n"
        "### See [docs](https://example.com)\n"
        "\n"
        "Final.\n"
    )
    md = _md_heading_tree(sample, MD_BINARY)
    neutral = neutral_heading_tree(sample)
    print("=== Stage A: direct extractor divergence ===")
    print(f"  md      = {md}")
    print(f"  neutral = {neutral}")
    if md == neutral:
        print("  [INERT] both extractors agree (closure or false alarm)")
        return False
    print("  [LIVE] extractors disagree on inline-markdown rendering")
    return True


def stage_b_scorer_divergence_false_positive() -> bool:
    """End-to-end: md scorer accepts agent answer that differs from expected
    only in inline-markdown formatting; neutral correctly reports MISMATCH.

    Realistic flow: a doc-maintenance task where the agent rewrites a
    heading and accidentally introduces (or strips) emphasis. The md
    scorer renders both to identical plaintext and reports OK; the neutral
    scorer compares raw source and detects the difference.
    """
    expected = (
        "# Configuration\n"
        "\n"
        "Settings live here.\n"
    )
    actual = (
        "# **Configuration**\n"
        "\n"
        "Settings live here.\n"
    )

    policy = StructuralDiffPolicy(
        kind="normalized_text",
        normalize_line_endings=True,
        ignore_trailing_whitespace=True,
        compare_frontmatter_json=False,
        compare_heading_tree=True,
        compare_block_order=False,
        compare_link_destinations=False,
        compare_block_text=False,
    )

    md_report: list[str] = []
    neutral_report: list[str] = []
    ok_md = score_normalized_text_md(policy, actual, expected, MD_BINARY, md_report)
    ok_neutral = score_normalized_text_neutral(policy, actual, expected, neutral_report)

    print()
    print("=== Stage B: end-to-end SCORER DIVERGENCE on compare_heading_tree ===")
    for line in md_report:
        print(f"  md:      {line}")
    for line in neutral_report:
        print(f"  neutral: {line}")
    print(f"  ok_md={ok_md}  ok_neutral={ok_neutral}")

    # Live failing trace = md says PASS while neutral correctly says FAIL
    # (BenchResult.correct = ok_md, so wrong answer aggregates as benchmark PASS)
    if ok_md and not ok_neutral:
        print("  [LIVE] md scorer accepts wrong answer as PASS (false-POSITIVE)")
        return True
    if ok_md == ok_neutral:
        print("  [INERT] scorers agree (closure or false alarm)")
        return False
    print(f"  [UNEXPECTED] divergence shape: md={ok_md} neutral={ok_neutral}")
    return False


def main() -> int:
    a_live = stage_a_direct_extractor_divergence()
    b_live = stage_b_scorer_divergence_false_positive()
    if a_live or b_live:
        print("\nRESULT: F8-6 is LIVE (exit 1)")
        return 1
    print("\nRESULT: F8-6 is inert (exit 0)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
