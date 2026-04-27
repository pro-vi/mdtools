#!/usr/bin/env python3
"""
F8-8 attribution probe — `neutral_block_texts` over-normalizes the four
non-heading/non-hr block container types (paragraph, bullet_list,
ordered_list, blockquote, table) by collecting only `inline.content` from
their child tokens and dropping the source markers (`- `, `1. `, `> `, `| `,
indentation) that `_md_block_texts` preserves via byte slicing.

This is the symmetric mirror of F8-7's hr/heading slice (closed iter 15) on
the remaining `neutral_block_texts` token-types branch (harness.py:120-144).
F8-7's tok.map line-slice fix only landed on the hr/heading paths; the rest
of the dispatch (`paragraph_open`, `bullet_list_open`, `ordered_list_open`,
`blockquote_open`, `table_open`) still uses the inline-collection path that
strips list markers, blockquote prefixes, and table separators.

Stage A — direct extractor divergence on a 6-block sample exercising
bullet list, ordered list, blockquote, table, nested list, and a
paragraph-vs-blockquote contrast. `_md_block_texts` byte-slices the source;
`neutral_block_texts` returns inline-only content. The two extractors
disagree on every collection-type block in the sample.

Stage B — end-to-end SCORER DIVERGENCE on `compare_block_text` where the
agent has swapped a bullet list for an ordered list (`- foo`/`- bar`/`- baz`
→ `1. foo`/`2. bar`/`3. baz`). md correctly catches MISMATCH (the `- ` vs
`1. ` markers are different bytes); neutral falsely reports OK because both
lists collapse to inline-only `foo\\nbar\\nbaz`.

Stage C — end-to-end SCORER DIVERGENCE on `compare_block_text` where the
agent has flattened a blockquote into a paragraph (`> Important note\\n>
Don't forget.\\n` → `Important note\\nDon't forget.\\n`). md correctly says
MISMATCH; neutral falsely says OK because both blocks collapse to inline-
only `Important note\\nDon't forget.`.

Stage D — end-to-end SCORER DIVERGENCE on a nested-list flattening
(`- top\\n  - inner\\n  - inner2\\n` → `- top\\n- inner\\n- inner2\\n`).
md correctly catches the indentation loss (which changes nesting structure
in commonmark semantics); neutral falsely says OK because both list shapes
collapse to inline-only `top\\ninner\\ninner2`.

Severity: SCORER DIVERGENCE on the cross-check direction (P2 mirror of
F8-7). md gates `BenchResult.correct = ok_md` and correctly catches all
four classes today, so the binary correct flag is preserved on a wrong
agent answer. But the neutral scorer's role is to provide an independent
cross-check; over-normalization defeats that purpose and would mask any
future md-side regression that mirrored neutral's leniency on list/
blockquote/table source fidelity.

Run:
    python3 bench/probes/F8-8-neutral-block-texts-collection-over-normalization/probe.py

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
    """Two scorers return different block-text lists on identical mixed input.

    `_md_block_texts` byte-slices the raw source: list blocks preserve
    `- `/`1. ` markers and indentation, blockquotes preserve `> ` prefix,
    tables preserve `| ... |` and `---` separators. `neutral_block_texts`
    collects only inline content from child tokens, dropping all source
    markers.

    Returns True if divergence is live (failure observable), False if inert.
    """
    sample = (
        "- bullet a\n"
        "- bullet b\n"
        "\n"
        "1. ordered a\n"
        "2. ordered b\n"
        "\n"
        "> quoted line\n"
        "\n"
        "| col1 | col2 |\n"
        "| --- | --- |\n"
        "| a | b |\n"
        "\n"
        "- top\n"
        "  - inner\n"
        "\n"
        "plain paragraph\n"
    )
    md = _md_block_texts(sample, MD_BINARY)
    neutral = neutral_block_texts(sample)
    print("=== Stage A: direct extractor divergence on collection blocks ===")
    print(f"  md      = {md}")
    print(f"  neutral = {neutral}")
    if md == neutral:
        print("  [INERT] both extractors agree (closure or false alarm)")
        return False

    diffs = []
    for i, (m, n) in enumerate(zip(md, neutral)):
        if m != n:
            diffs.append(f"block[{i}]: md={m!r} vs neutral={n!r}")
    if len(md) != len(neutral):
        diffs.append(f"length mismatch: md={len(md)} vs neutral={len(neutral)}")
    print(f"  [LIVE] extractors disagree on {len(diffs)} blocks:")
    for d in diffs:
        print(f"    {d}")
    return True


def stage_b_scorer_divergence_list_type_swap() -> bool:
    """End-to-end: agent swaps bullet list (`- foo`) for ordered list
    (`1. foo`). md scorer correctly says MISMATCH (the marker bytes are
    different); neutral scorer says OK because both lists collapse to
    inline-only `foo\\nbar\\nbaz`.
    """
    expected = (
        "# Items\n"
        "\n"
        "- foo\n"
        "- bar\n"
        "- baz\n"
    )
    actual = (
        "# Items\n"
        "\n"
        "1. foo\n"
        "2. bar\n"
        "3. baz\n"
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
    print("=== Stage B: end-to-end SCORER DIVERGENCE on list-type swap ===")
    for line in md_report:
        print(f"  md:      {line}")
    for line in neutral_report:
        print(f"  neutral: {line}")
    print(f"  ok_md={ok_md}  ok_neutral={ok_neutral}")

    if ok_md != ok_neutral:
        if not ok_md and ok_neutral:
            print("  [LIVE] neutral falsely accepts bullet→ordered list swap")
        else:
            print(f"  [LIVE] unexpected divergence shape: md={ok_md} neutral={ok_neutral}")
        return True
    print("  [INERT] scorers agree (closure or false alarm)")
    return False


def stage_c_scorer_divergence_blockquote_flattened() -> bool:
    """End-to-end: agent flattens a blockquote (`> Important note...`) into
    a plain paragraph. md scorer correctly says MISMATCH; neutral scorer
    says OK because both blocks collapse to inline-only.
    """
    expected = (
        "# Notice\n"
        "\n"
        "> Important note\n"
        "> Do not forget.\n"
    )
    actual = (
        "# Notice\n"
        "\n"
        "Important note\n"
        "Do not forget.\n"
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
    print("=== Stage C: end-to-end SCORER DIVERGENCE on blockquote→paragraph ===")
    for line in md_report:
        print(f"  md:      {line}")
    for line in neutral_report:
        print(f"  neutral: {line}")
    print(f"  ok_md={ok_md}  ok_neutral={ok_neutral}")

    if ok_md != ok_neutral:
        if not ok_md and ok_neutral:
            print("  [LIVE] neutral falsely accepts blockquote→paragraph flattening")
        else:
            print(f"  [LIVE] unexpected divergence shape: md={ok_md} neutral={ok_neutral}")
        return True
    print("  [INERT] scorers agree (closure or false alarm)")
    return False


def stage_d_scorer_divergence_nested_list_flattened() -> bool:
    """End-to-end: agent flattens a nested list (indented sub-items) into a
    flat list. md scorer correctly says MISMATCH (indentation bytes differ
    and changes commonmark nesting semantics); neutral scorer says OK
    because both list shapes collapse to inline-only.
    """
    expected = (
        "- top\n"
        "  - inner\n"
        "  - inner2\n"
    )
    actual = (
        "- top\n"
        "- inner\n"
        "- inner2\n"
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
    print("=== Stage D: end-to-end SCORER DIVERGENCE on nested→flat list ===")
    for line in md_report:
        print(f"  md:      {line}")
    for line in neutral_report:
        print(f"  neutral: {line}")
    print(f"  ok_md={ok_md}  ok_neutral={ok_neutral}")

    if ok_md != ok_neutral:
        if not ok_md and ok_neutral:
            print("  [LIVE] neutral falsely accepts nested→flat list flattening")
        else:
            print(f"  [LIVE] unexpected divergence shape: md={ok_md} neutral={ok_neutral}")
        return True
    print("  [INERT] scorers agree (closure or false alarm)")
    return False


def main() -> int:
    a_live = stage_a_direct_extractor_divergence()
    b_live = stage_b_scorer_divergence_list_type_swap()
    c_live = stage_c_scorer_divergence_blockquote_flattened()
    d_live = stage_d_scorer_divergence_nested_list_flattened()
    if a_live or b_live or c_live or d_live:
        print("\nRESULT: F8-8 is LIVE (exit 1)")
        return 1
    print("\nRESULT: F8-8 is inert (exit 0)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
