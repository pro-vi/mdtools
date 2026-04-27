#!/usr/bin/env python3
"""F8-5 attribution probe: `_md_block_texts` slices content with byte
offsets but Python `str` indexing is character-based, so multi-byte
UTF-8 chars cause cross-block drift.

`_md_block_texts(content, md_binary)` (bench/harness.py:650-656) calls
`md blocks --json` which returns spans as **byte** offsets into the
file's UTF-8 encoding, then slices `content` (a Python `str`) by those
offsets:

    return [content[b["span"]["byte_start"]:b["span"]["byte_end"]].strip()
            for b in data.get("blocks", [])]

Python `str` indices count characters, not bytes. When `content`
contains multi-byte UTF-8 characters (e.g. `é`, `§`, `Ω`, CJK), the
char-position N is past the byte-position N for every block following
the first multi-byte char. Each block's slice drifts toward the right
by the cumulative byte-excess. Drift compounds across blocks: the
slice for block K starts after the K-th multi-byte char's offset
excess and may leak chars from block K+1 (or truncate past
`len(content)` for the last block).

`score_normalized_text_md` (harness.py:567-578) uses `_md_block_texts`
under `policy.compare_block_text`, so any task with that policy on
content carrying international/typographic characters scores against
drifted text. The neutral scorer (`neutral_block_texts` via
`markdown-it-py`) operates on token content directly, not file slices,
and is unaffected.

Run: `python3 bench/probes/F8-5-md-block-texts-utf8-byte-vs-char-slice/probe.py`
Exit 0 = inert (closure landed). Exit 1 = live failing trace.
"""
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2].parent
sys.path.insert(0, str(REPO_ROOT / "bench"))

from harness import (  # noqa: E402
    StructuralDiffPolicy,
    _md_block_texts,
    neutral_block_texts,
    score_normalized_text_md,
    score_normalized_text_neutral,
)

MD_BIN = str(REPO_ROOT / "target" / "release" / "md")


def _byte_correct_block_texts(content: str) -> list[str]:
    """Reference implementation: slice the UTF-8 bytes, not the Python str."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".md", delete=False) as f:
        f.write(content)
        f.flush()
        r = subprocess.run(
            [MD_BIN, "blocks", f.name, "--json"],
            capture_output=True, text=True, timeout=10,
        )
    data = json.loads(r.stdout)
    b = content.encode("utf-8")
    return [
        b[blk["span"]["byte_start"]:blk["span"]["byte_end"]].decode("utf-8").strip()
        for blk in data.get("blocks", [])
    ]


def stage_a_direct_slicer_drift() -> int:
    """Stage A: `_md_block_texts` returns text whose first chars belong
    to the previous block (or are missing entirely on the last block)
    when content has multi-byte UTF-8 characters in any preceding block.

    Pre-fix: every multi-byte char in content adds 1+ byte of drift.
    Slice for block K starts (drift) chars further into `content` than
    intended, so the leading char(s) of block K's actual text are
    omitted and trailing char(s) of block K-1 (or whitespace between
    blocks) leak into the slice. After `.strip()`, the result is wrong
    by at least the leading-char count.

    Post-fix: the slicer honors UTF-8 byte boundaries — either by
    encoding to bytes once and indexing the bytes, or by precomputing
    a byte→char index map and using char offsets to slice `content`.
    """
    content = "# Héllo\n\nA: foo\n\nB: bar\n"
    actual = _md_block_texts(content, MD_BIN)
    expected = _byte_correct_block_texts(content)

    if actual == expected:
        print("[OK stage A] _md_block_texts honors UTF-8 byte boundaries")
        print(f"  block_texts: {actual}")
        return 0

    print("[FAIL stage A] _md_block_texts drifts on UTF-8 content")
    print(f"  content: {content!r}")
    print(f"  byte-correct (expected): {expected}")
    print(f"  actual (drifted):        {actual}")
    return 1


def stage_b_scorer_false_positive() -> int:
    """Stage B: SCORER DIVERGENCE — the md scorer returns OK on a wrong
    agent answer because the slicer drift drops the differing first
    char of the affected block in BOTH actual and expected. The neutral
    scorer correctly reports MISMATCH.

    Setup: a heading with one multi-byte `é` creates a 1-char drift.
    The first char of every later block is dropped from the md scorer's
    block_text comparison. If the agent's wrong answer differs from
    expected only in that first char, the broken slicer washes the
    difference out and the md scorer accepts the wrong answer as
    correct (BenchResult.correct = True via ok_md).

    Pre-fix: ok_md=True (false-POSITIVE), ok_neutral=False, harness
    emits a "⚠ SCORER DIVERGENCE" warning in score_task but the
    headline `correct` field uses ok_md. The agent answer Xar (wrong)
    is reported as PASS in run aggregations.

    Post-fix: ok_md=False (correct), agreeing with the neutral scorer.
    """
    expected = "# café\n\nFOO\n\nbar\n"
    actual = "# café\n\nFOO\n\nXar\n"  # b → X in last block (only difference)

    policy = StructuralDiffPolicy(
        kind="normalized_text",
        normalize_line_endings=True,
        ignore_trailing_whitespace=True,
        compare_frontmatter_json=False,
        compare_heading_tree=True,
        compare_block_order=False,
        compare_link_destinations=False,
        compare_block_text=True,
    )
    report_md: list[str] = []
    ok_md = score_normalized_text_md(policy, actual, expected, MD_BIN, report_md)
    report_n: list[str] = []
    ok_n = score_normalized_text_neutral(policy, actual, expected, report_n)

    if not ok_md and not ok_n:
        print("[OK stage B] both scorers reject the wrong answer (no divergence)")
        return 0

    if ok_md and not ok_n:
        print("[FAIL stage B] md scorer accepts WRONG answer as PASS (false-POSITIVE)")
        print(f"  expected: {expected!r}")
        print(f"  actual:   {actual!r}")
        print(f"  ok_md={ok_md}  ok_neutral={ok_n}  ← divergence")
        for line in report_md:
            print(f"  [md]      {line}")
        for line in report_n:
            print(f"  [neutral] {line}")
        return 1

    print(f"[FAIL stage B] unexpected scorer state ok_md={ok_md} ok_neutral={ok_n}")
    return 1


def main() -> int:
    rc_a = stage_a_direct_slicer_drift()
    print()
    rc_b = stage_b_scorer_false_positive()
    print()
    if rc_a == 0 and rc_b == 0:
        print("[F8-5 inert] _md_block_texts honors UTF-8 byte boundaries")
        return 0
    print("[F8-5 live] _md_block_texts uses char-indexed slicing on byte offsets")
    return 1


if __name__ == "__main__":
    sys.exit(main())
