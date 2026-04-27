# F8-5 — `_md_block_texts` slices Python `str` with byte offsets, drifts on UTF-8

**Status:** CLOSED, T8 iter 11 (2026-04-26).
**Closure pin:** `bench/harness.py:_md_block_texts` now encodes `content`
to UTF-8 once per call and slices the resulting `bytes` with the
byte offsets emitted by `md blocks --json`, then decodes back to `str`.
Typed tests at `bench/test_harness_json.py` (`MdBlockTextsUtf8Tests`)
pin both the F8-5 stage A trace (byte-exact `_md_block_texts` output on
multi-byte content) and stage B (md scorer agrees with neutral scorer
on a wrong UTF-8 answer). Attribution probe `probe.py` flips exit 1→0.

**Surface:** `bench/harness.py:650-656` `_md_block_texts(content, md_binary)`
returns the list of block texts that `score_normalized_text_md`
(harness.py:567-578) compares under `policy.compare_block_text`. The
function calls `md blocks --json` (which emits spans as **byte**
offsets into the file's UTF-8 encoding) and slices the Python `str`
content directly:

```python
return [content[b["span"]["byte_start"]:b["span"]["byte_end"]].strip()
        for b in data.get("blocks", [])]
```

Python `str` indexing is character-based, not byte-based. Every
multi-byte UTF-8 character in `content` advances the byte counter
faster than the char counter, so the slice for any block following
the first multi-byte char drifts to the right by the cumulative
byte-excess. Drift compounds: leading char(s) of block K are dropped,
trailing char(s) of block K-1 (or inter-block whitespace) leak in,
and the last block's slice may truncate past `len(content)`.

**Severity:** P1 false-POSITIVE on bench tasks scored by
`compare_block_text` whose expected/actual content carries
international or typographic characters in any block before the one
under comparison. The md scorer accepts a wrong answer as PASS when
the structurally significant difference happens to land on a char
that the slicer drift drops in both `actual` and `expected`. The
neutral scorer (`neutral_block_texts` via markdown-it-py token content,
no file slicing) correctly reports MISMATCH, but `BenchResult.correct`
is set from `ok_md` (harness.py ~1437), so the wrong answer aggregates
as a benchmark PASS.

## Hypothesis

`md blocks --json` is correct: comrak emits `byte_start` / `byte_end`
into the UTF-8 byte sequence of the file. The harness incorrectly
treats those byte offsets as char offsets when slicing the Python
`str`. The neutral scorer is correct because `markdown-it-py` carries
each block's text as `tok.content`, not as a slice of the source
string, so it never crosses the byte/char boundary.

## Stages

- **Stage A** — direct `_md_block_texts` call returns drifted text:
  on `# Héllo\n\nA: foo\n\nB: bar\n` the function returns
  `['# Héllo', ': foo', ': bar']` instead of the byte-correct
  `['# Héllo', 'A: foo', 'B: bar']`. The leading `A`/`B` of paragraphs
  one and two are silently dropped because the `é` in the heading
  contributes 1 byte of drift that compounds across every later block.
- **Stage B** — `score_normalized_text_md` declares `block_text [md]:
  OK` on a wrong agent answer because the differing first char of
  the affected block is dropped from BOTH actual and expected slices.
  Setup: `expected = "# café\n\nFOO\n\nbar\n"`, `actual = "...\n\nXar\n"`
  (only `b → X` differs). md scorer says OK, neutral scorer reports
  `block_text [neutral]: MISMATCH at block 2: 'bar' vs 'Xar'`. The
  scorer divergence is recorded by `score_task` but the headline
  `correct` field uses `ok_md` and aggregates the wrong answer as PASS.

## Realism note

Multi-byte UTF-8 characters appear in realistic Markdown
document-maintenance workflows:

- Specs / READMEs that include non-English identifiers, names, or
  quotations (`café`, `naïve`, `résumé`, `§`, `Ω`, em-dash `—`).
- Code samples with international comments or string literals.
- Headings or task descriptions referencing typographic punctuation
  (curly quotes `"…"`, apostrophes `'`, ellipses `…`).
- CJK, Cyrillic, Arabic, or Hebrew document content.

The current bench corpus's `compare_block_text` tasks (T2, T3, T8,
T17, plus two more in `tasks_v1.json`) do not embed multi-byte
characters in expected output, so the trace does not surface on the
live benchmark today; but the realism is high enough that any
auto-research candidate exercising international content or any real
agent task on a non-ASCII document will hit it. The probe is filed
pre-emptively under the same precedent as F8-2, F8-3, F8-4.

## Symmetry to F8-1 / F8-2 / F8-3 / F8-4

F8-1 through F8-4 closed the four-layer json_envelope failure surface
(shape match, preference rule, depth-scanner string boundary,
preprocessor corruption) all on `select_json_envelope_actual` /
`extract_last_json`. F8-5 is fresh because it lives on a *different
existing surface*: `score_normalized_text_md` (the dispatcher target
for `kind == "normalized_text"` tasks), via the `_md_block_texts`
helper. None of the prior fixes touch the byte-vs-char slicer; the
defect is independent of the json_envelope path.

## Closure plan

Replace the char-indexed slice with a byte-indexed slice:

```python
def _md_block_texts(content: str, md_binary: str) -> list[str]:
    out = _md_run(content, md_binary, ["blocks"])
    if not out:
        return []
    data = json.loads(out)
    b = content.encode("utf-8")
    return [
        b[blk["span"]["byte_start"]:blk["span"]["byte_end"]].decode("utf-8").strip()
        for blk in data.get("blocks", [])
    ]
```

Encoding once per call is O(N) and matches what comrak does
internally; the resulting slices are byte-exact regardless of UTF-8
char width. Pin the closure with two unit tests in
`bench/test_harness_json.py` (or a new test module):

1. `test_md_block_texts_honors_utf8_byte_boundaries` — the F8-5 stage
   A trace, asserting `['# Héllo', 'A: foo', 'B: bar']` is returned.
2. `test_score_normalized_text_md_rejects_wrong_answer_with_utf8` —
   the F8-5 stage B trace, asserting `score_normalized_text_md` returns
   FAIL when the agent answer differs from expected at any char (not
   only at chars the broken slicer happens to keep).

The closure attribution probe is
`python3 bench/probes/F8-5-md-block-texts-utf8-byte-vs-char-slice/probe.py`;
expect exit 0 on both stages.

## Files

- `probe.py` — standalone reproduction (exit 1 = live, exit 0 = inert)
- `verdict.txt` — recorded probe verdicts (filed and, post-closure, closure)
