# F8-7 — `neutral_block_texts` over-normalizes hr style and drops heading prefix

**Status:** CLOSED (filed T8 iter 14, closed T8 iter 15, 2026-04-26)

**Closure pin:**
- `bench/harness.py` `neutral_block_texts` now source-slices hr and
  heading tokens via `tok.map` line ranges, mirroring `_md_block_texts`'s
  byte-slice contract. hr style (`---`/`***`/`___`) and heading marker
  prefix (`# `/`## `) or setext underline (`====`) are preserved.
- `bench/test_harness_json.py` `NeutralBlockTextsSourceFidelityTests`
  pins all three stages with four typed tests (hr style, heading prefix,
  end-to-end SCORER DIVERGENCE on dropped heading and on hr style swap).
- Attribution probe rerun → exit 1 → 0 on Stages A, B, and C; see
  `verdict.txt` for side-by-side filed/closure verdicts.

**Severity:** P2 SCORER DIVERGENCE on the cross-check direction. The harness's
`BenchResult.correct = ok_md` (`bench/harness.py:1456`) means the md scorer
gates the binary correct flag, and md correctly catches both classes of
divergence today, so benchmark binary correctness is preserved on a wrong
agent answer. The fileable defect is that `score_normalized_text_neutral`
is the *independent* cross-check on the `normalized_text` dispatcher; over-
normalization defeats that purpose and would mask any future md-side
regression that mirrored neutral's leniency. Mirror of F8-6: where F8-6
had md as the over-lenient side on heading inline formatting (P1 because
md gates), F8-7 has neutral as the over-lenient side on block-level
structural representation (P2 because neutral does not gate today).

**Surface:** `bench/harness.py`
- `_md_block_texts` (line ~669) byte-slices the raw `content` against
  `md blocks --json` byte spans, then `.strip()`s each slice. Source
  fidelity is preserved: hr blocks return the actual marker (`---`/`***`/
  `___`), heading blocks include the level marker prefix or setext
  underline.
- `neutral_block_texts` (line ~86) walks markdown-it-py tokens and
  collects only inline `.content` for paragraph/heading/list/blockquote/
  table parents. For hr tokens it appends a hardcoded `"---"` regardless
  of source. For headings it appends only the inline content (no `# `
  prefix, no `=====` underline).

**Two distinct over-normalizations on the same surface:**

1. **HR style hardcoded.** `neutral_block_texts` line ~99: `texts.append("---")`
   for any `tok.type == "hr"`. Every hr (`---`, `***`, `___`, `- - -`)
   normalizes to `"---"`. md preserves the actual source via byte slicing.
   markdown-it-py exposes `tok.markup` (e.g. `"----"`, `"****"`, `"____"`)
   and `tok.map` (line range), either of which would let neutral
   reconstruct a higher-fidelity representation, but the current code
   ignores both.
2. **Heading marker dropped.** When neutral encounters a `heading_open`
   token, it descends into the heading's tokens and collects only the
   `inline.content` of the heading text. The level marker (`# `, `## `,
   etc.) and setext underline (`=====`) are not added back. md's byte
   slicing preserves the full heading line as it appears in source.

**Attribution probe:** `probe.py` exits 1 on all three stages.
- Stage A — direct extractor divergence on a single content carrying
  three hr styles (`---`/`***`/`___`) and two heading levels (h1/h2).
  md returns `['# Hello', '---', '## Subsection', '***', 'foo', '___', 'bar']`.
  neutral returns `['Hello', '---', 'Subsection', '---', 'foo', '---', 'bar']`.
  4 of 7 blocks disagree.
- Stage B — end-to-end SCORER DIVERGENCE on `compare_block_text` where
  `expected = '# Configuration\n\nSettings live here.\n'` and the agent
  emits `'Configuration\n\nSettings live here.\n'` (heading dropped to
  paragraph). md says MISMATCH (catches `# Configuration` vs `Configuration`),
  neutral says OK (both inline contents are `Configuration`).
- Stage C — end-to-end SCORER DIVERGENCE on `compare_block_text` where
  `expected` uses `---` for an hr and the agent emits `***`. md says
  MISMATCH (`---` vs `***`), neutral says OK (both hardcoded `---`).

**Why P2 not P1:** The harness binds `BenchResult.correct` to `ok_md`
only. md correctly gates both Stage B and Stage C as MISMATCH, so the
binary correct flag accurately rejects the wrong agent answer. The
SCORER DIVERGENCE warning is logged in `report` but does not promote
neutral's false OK to a benchmark PASS. Contrast F8-5 (md was over-
lenient via UTF-8 byte/char drift; neutral correctly caught it but
binary correct flag had already false-passed) — that direction is P1.
F8-7 is the cross-check-trustworthiness direction.

**Closure plan:** align `neutral_block_texts` to `_md_block_texts`'s
source-fidelity contract. Two reasonable shapes:
1. **Line-slice via `tok.map`.** For hr tokens and heading tokens with
   non-`None` `tok.map`, slice the relevant lines from `content` and
   use that as the block text (with `.strip()`, matching md's behavior).
   Most faithful, mirrors md byte slicing.
2. **Reconstruct from token state.** For hr, fall through to `tok.markup`
   and use that (loses count but distinguishes char class). For headings,
   prepend `'#' * level + ' '` before the inline content (matches ATX
   form; setext headings would still differ but at least the level marker
   is present).

Pin with three tests in `bench/test_harness_json.py`:
- `test_neutral_block_texts_preserves_hr_style` (Stage A subset on hr).
- `test_neutral_block_texts_preserves_heading_prefix` (Stage A subset on
  headings).
- `test_score_normalized_text_md_and_neutral_agree_on_dropped_heading`
  (Stage B end-to-end agreement after closure).

Attribution probe = rerun `probe.py`; expect exit 0 on all three stages.

**Realism:** Technical Markdown frequently uses different hr styles in
the same document tree (`---` is canonical but `***` is common in older
specs and `___` appears in some style guides). Heading-vs-paragraph
swaps are a common doc-maintenance failure mode (an agent over-edits
and accidentally collapses a heading to a paragraph or vice versa). The
current benchmark corpus uses only `---` for hr and stable heading
structure, so this trace is dormant on the live benchmark, following
the F8-2..F8-6 pre-emptive-filing precedent.

**Hypothesis hook:** iter 13's learning footnote ("iter 14's quiet-trace
counter is at 0; if surface-mining no longer produces a fresh trace,
iter 14 may need to invoke the auto-research generator or reach for a
different surface — extract_final_text, score_structural_json policy
normalization, or the scorer-dispatcher branch coverage map") suggested
extending the divergence search beyond `compare_heading_tree` (F8-6) to
`compare_block_text`'s OTHER half — block_texts — which the F8-6 closure
left untouched. Asking "where else does the md/neutral representation
asymmetry persist?" surfaced the hr hardcoding (line ~99) and the
heading-prefix drop on the same function, both untouched by F8-6.

**Symmetry to prior F8 closures:**
- F8-1..F8-4 closed the json_envelope four-layer cake on
  `extract_last_json` / `select_json_envelope_actual`.
- F8-5 closed `_md_block_texts` UTF-8 byte/char drift on the
  `compare_block_text` md branch (md was the buggy gating scorer; P1).
- F8-6 closed `_md_heading_tree` vs `neutral_heading_tree` inline-markdown
  rendering divergence on the `compare_heading_tree` branch (md was the
  over-lenient gating scorer; P1).
- F8-7 lives on the `compare_block_text` neutral branch — same dispatcher
  as F8-5/F8-6 but the cross-check defect direction (neutral over-lenient,
  md correct; P2).
