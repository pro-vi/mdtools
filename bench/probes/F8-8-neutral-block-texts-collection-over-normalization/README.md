# F8-8 — `neutral_block_texts` over-normalizes paragraph/list/blockquote/table inline-collection paths

**Status:** OPEN (filed T8 iter 16)

**Severity:** P2 (SCORER DIVERGENCE on cross-check direction; binary `correct` flag preserved by md gating today, but neutral cross-check is structurally broken on collection-type blocks).

**Surface:** `bench/harness.py:120-144` — the inline-collection branch of `neutral_block_texts` that dispatches all non-{hr, heading, html_block, code_block, fence} block types.

**Class:** Symmetric mirror of F8-7 on the remaining `neutral_block_texts`
token-types branch. F8-7 (closed iter 15) added `tok.map` line slicing for
hr and heading blocks — the rest of the dispatch (paragraph_open,
bullet_list_open, ordered_list_open, blockquote_open, table_open) still
uses the inline-collection path that strips list markers (`- `, `1. `),
blockquote prefixes (`> `), table separators (`|`, `---`), and indentation
that distinguishes nesting levels. `_md_block_texts` byte-slices the
source and preserves all of the above; the two extractors diverge on every
collection-type block.

**Probe:** `probe.py` exits 1 on all four stages.

- **Stage A** — Direct extractor divergence on a 6-block sample (bullet list, ordered list, blockquote, table, nested list, paragraph). 5 of 6 blocks disagree; only the plain paragraph survives.
- **Stage B** — End-to-end SCORER DIVERGENCE on `compare_block_text` where the agent swapped `- foo`/`- bar`/`- baz` for `1. foo`/`2. bar`/`3. baz`. md correctly says MISMATCH (`- ` vs `1. ` is a different byte sequence); neutral falsely says OK.
- **Stage C** — End-to-end SCORER DIVERGENCE on a blockquote→paragraph flattening. md correctly says MISMATCH; neutral falsely says OK.
- **Stage D** — End-to-end SCORER DIVERGENCE on a nested→flat list flattening. md correctly catches the indentation loss (which changes commonmark nesting semantics); neutral falsely says OK.

**Closure plan:** mirror F8-7's source-fidelity-via-`tok.map` shape on the four collection-type branches. For `paragraph_open`, `bullet_list_open`, `ordered_list_open`, `blockquote_open`, `table_open`, when `tok.map is not None`, line-slice the source via `lines[start:end]` (rstripping the per-block trailing whitespace) and append the joined result. Keep the existing inline-collection path as a defensive fallback when `tok.map is None`. Pin with four typed tests in `bench/test_harness_json.py` covering: stage-A direct extractor equality on each of the 4 collection types, stage-B end-to-end scorer agreement on list-type swap, stage-C end-to-end agreement on blockquote→paragraph, stage-D end-to-end agreement on nested→flat list.

**Attribution probe:** rerun `python3 bench/probes/F8-8-neutral-block-texts-collection-over-normalization/probe.py`. Closure flips exit 1 → 0 on all four stages.

**Realism justification:** Technical Markdown (READMEs, design docs, runbooks, task lists) routinely contains lists, blockquotes, and tables. Agent-driven doc maintenance failure modes that flatten or restructure these blocks are extremely realistic — list-style swap (`-` ↔ `*` ↔ `1.`), blockquote dropped, indentation lost on copy/paste — and would all silently pass the neutral scorer today. F8-7 closed the hr/heading equivalent class, so this filing closes the symmetric gap on the remaining branch.

**Hooked from:** iter 15's learning footnote — "neither the inline-collection paths in neutral_block_texts on paragraph/list/blockquote/table types nor extract_final_text/score_structural_json policy normalization have been mined." Cheap surface-mining of that hint produced this trace without invoking the auto-research generator.
