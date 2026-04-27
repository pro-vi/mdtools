# F8-6 — `_md_heading_tree` strips inline markdown; `neutral_heading_tree` does not

**Status:** CLOSED (filed T8 iter 12, closed T8 iter 13, 2026-04-26)

**Closure pin pointers:**
- Hardening: `bench/harness.py:51-69` (new `_render_inline_to_plaintext` helper) and `bench/harness.py:72-83` (`neutral_heading_tree` now calls the helper on `tokens[i+1].children`). The helper concatenates `.content` of `text` and `code_inline` children, recurses into `image` children for alt text, and drops markup wrappers (strong/em/link/strikethrough open/close, html_inline, soft/hardbreak).
- Pinned by `bench/test_harness_json.py::NeutralHeadingTreeInlineRenderingTests` — three methods: stage-A direct extractor agreement, stage-B end-to-end scorer agreement, and an image+html_inline non-regression that validates the closure generalizes beyond the filed fixtures.
- Attribution probe rerun: `python3 bench/probes/F8-6-heading-tree-inline-formatting-divergence/probe.py` exits 0 on both stages (was exit 1 pre-fix). See `verdict.txt` for filed/closure verdicts side-by-side.

**Severity:** P1 false-POSITIVE on `compare_heading_tree` tasks where any
heading text contains inline markdown (inline code, emphasis, links). The
md scorer renders both `actual` and `expected` headings to plaintext
(formatting stripped) and reports OK, while `score_normalized_text_neutral`
preserves the raw source and correctly reports MISMATCH. Per
`bench/harness.py:1437` (`BenchResult.correct = ok_md`), a wrong agent
answer aggregates as a benchmark PASS while the SCORER DIVERGENCE warning
is logged but does not gate `correct`.

**Surface:** `bench/harness.py`
- `_md_heading_tree` (line ~635) reads `md outline --json` and pulls
  `entries[].heading.text`. The md binary's outline already renders
  inline markdown to plaintext (backticks, emphasis markers, link markup
  removed).
- `neutral_heading_tree` (line ~51) reads `tokens[i+1].content` from
  markdown-it-py inline tokens, which is the raw markdown source string
  (formatting characters preserved).

**Attribution probe:** `probe.py` exits 1 on both stages.
- Stage A — direct extractor divergence on a single content with three
  heading variants (inline code, bold, link).
- Stage B — end-to-end SCORER DIVERGENCE on a `compare_heading_tree`
  policy where `expected = '# Configuration'` and the agent emits
  `# **Configuration**`. md says OK, neutral says MISMATCH.

**Closure plan:** align the two scorers on a shared rendering of inline
content. Two reasonable shapes:
1. Make `neutral_heading_tree` render inline children to plaintext
   (concatenate text-token children, drop `code_inline`/`em`/`strong`/
   `link` markup) so it matches `_md_heading_tree`.
2. Make `_md_heading_tree` slice the heading line from `content` using
   the byte span and return the raw source (would diverge from the
   md-binary outline contract; rejected).

Option 1 preserves the `md outline` contract and only changes the
neutral scorer's rendering. Pin with two tests in `bench/test_harness_json.py`:
- `test_neutral_heading_tree_renders_inline_to_plaintext` (the F8-6
  trace as a typed test).
- `test_score_normalized_text_md_and_neutral_agree_on_emphasis_diff`
  (end-to-end agreement on the stage B fixture).

Attribution probe = rerun `probe.py`; expect exit 0 on both stages.

**Realism:** technical Markdown headings frequently contain inline code
for command/symbol names (`# The \`md tasks\` command`), emphasis
(`# **Important** notes`), or links (`# See [docs](url)`). The current
benchmark corpus is plain-ASCII without inline formatting, so the trace
is dormant on the live benchmark, but auto-research candidates exercising
realistic technical README/spec content will fire it. Pre-emptive filing
follows the precedent of F8-2..F8-5 (synthetic-but-realistic trace before
the corpus contains it).

**Symmetry to prior F8 closures:** F8-1..F8-4 closed the json_envelope
four-layer cake on `extract_last_json` / `select_json_envelope_actual`.
F8-5 closed `_md_block_texts` byte-vs-char encoding on the
normalized_text dispatcher's `compare_block_text` path. F8-6 lives on
the same dispatcher's *other* branch — `compare_heading_tree` — and is
a representation-mismatch class (md plaintext vs neutral raw source)
rather than an encoding-boundary class.

**Hypothesis hook:** iter 11's learning ("Future surface-mining for
SCORER DIVERGENCE classes should probe specifically for char-vs-byte
alignment between the two scorers, not just 'do they ever disagree'")
generalized to "what other rendering boundaries do the two scorers
treat differently?" The md binary renders inline markdown; markdown-it-py
returns raw source. They disagree by construction on any heading with
inline formatting.
