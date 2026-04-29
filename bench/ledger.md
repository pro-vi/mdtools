# Bench Ledger

T8 surface. Concise index of findings on existing mdtools surfaces (scorer, selectors,
JSON stability, harness, telemetry). Weaker evidence than typed artifacts under
`bench/runs/`, `bench/probes/`, `bench/test_*.py`. OPEN findings gate claim-expansion.

Hard budget: ≤500 lines. Overflow archives to `bench/ledger-archive/<YYYY-Qn>.md`
per T8 spec. Most-recent 30 iterations stay inline; older content moves to archive
in a separate iteration step that does not count as substantive progress.

Promotion path for auto-research candidates:
`bench/search/candidates/` → `bench/search/quarantine/` → `bench/search/accepted/`.
Promotion to `bench/holdout/` requires human review and a `holdout_version` bump.

## OPEN findings

_None._

## T10 steady-state

- **T10-14 candidate rejected** — `bench/search/candidates/oncall-escalation-table-summary/` records a realism-approved on-call table-summary candidate. Qwen3.5-27B-4bit results: mdtools PASS, hybrid PASS, unix FAIL, dual scorers agreed in every cell. Unix extracted the right rows and TSV fields but maxed out assembling final JSON under guarded shell quoting/response-schema constraints. Unix-adversary label: `shell-quoting`; not promoted, no HEADLINE movement.
- **T10-13 candidate rejected** — `bench/search/candidates/scoped-runbook-note-relocation/` records a realism-approved runbook note-relocation candidate. Qwen3.5-27B-4bit results: mdtools FAIL, hybrid FAIL, unix PASS, dual scorers agreed in every cell. The tool-enabled modes deleted the source block and inserted the moved note after the Network firewall paragraph, but the inserted note merged into that preceding paragraph due to missing blank-line separation; unix eventually passed with an awk delete-and-reinsert strategy. Unix-adversary label: `unix-win/no-mdtools-advantage`; not promoted, no HEADLINE movement.
- **T10-12 candidate rejected** — `bench/search/candidates/release-notes-heading-summary/` records a realism-approved heading-summary extraction candidate. The generated expected count was corrected before measurement (7 → 8 H3 headings), and explicit heading-like lines were added inside ignored code/blockquote regions. Qwen3.5-27B-4bit results: mdtools PASS, hybrid PASS, unix PASS, dual scorers agreed in every cell. Unix was slower and less stable (462.91s, 17 invalid responses, 1 policy denial) but still passed, so the unix-adversary label is `both-pass/no-gap`; not promoted, no HEADLINE movement.
- **T10-11 candidate rejected** — `bench/search/candidates/project-milestone-checklist/` records the first steady-state auto-research candidate. Realism review passed before measurement. Qwen3.5-27B-4bit results: mdtools PASS, hybrid PASS, unix FAIL, dual scorers agreed in every cell. Unix-adversary label: `shell-quoting`, not AST-structural, because the unix run formed the right awk plan but spent 27 invalid responses failing to emit it as valid JSON. Stored as rejected candidate evidence; not promoted, no HEADLINE gap movement.
- **F10-1 closed** — candidate measurement exposed a harness path bug for tasks outside `bench/inputs/`: `run_agent` copied `bench/search/candidates/<family>/input.md` into a temp subdirectory, while `build_prompt` and file-content scoring still pointed at `workdir/input.md`. Fixed by centralizing the copied-input path mapping and reusing it for prompt and scoring. Pinned by `bench.test_harness_run_artifacts.HarnessRunArtifactTests.test_run_agent_scores_input_file_copied_under_non_inputs_parent`.

## Closed in T8

- **F8-8** — `neutral_block_texts` (bench/harness.py) collection-type branch over-normalized the five non-{hr, heading, html_block, code_block, fence} block tokens (paragraph_open, bullet_list_open, ordered_list_open, blockquote_open, table_open) by collecting only inline content, dropping list markers, blockquote prefixes, table separators, and nesting indentation that `_md_block_texts` preserves via byte slicing. P2 SCORER DIVERGENCE — md gated `correct` correctly today but the cross-check would mask any future md-side regression mirroring neutral's leniency. Filed T8 iter 16, CLOSED post-loop. Hardening: extended F8-7's `tok.map` line-slice to all five collection-type tokens, with defensive fallback to the inline-collection path when `tok.map is None`. Pinned by `bench/test_harness_json.py::NeutralBlockTextsCollectionFidelityTests` (4 tests covering list markers, blockquote/table fidelity, end-to-end blockquote→paragraph flattening, end-to-end nested→flat list flattening). Attribution probe rerun: `bench/probes/F8-8-neutral-block-texts-collection-over-normalization/probe.py` exit 0 = inert on all four stages.
- **F8-7** — `neutral_block_texts` (bench/harness.py:86) over-normalized hr style (hardcoded `"---"` for any token type) and dropped the heading-level marker prefix (`# `, `## `) and setext underline, while `_md_block_texts` preserved both via byte slicing. P2 SCORER DIVERGENCE on the cross-check direction — md correctly gated `BenchResult.correct = ok_md` on both classes, but the independent neutral cross-check was structurally broken and would have masked any future md-side regression mirroring neutral's leniency. Filed T8 iter 14, CLOSED T8 iter 15. Hardening: source-fidelity slicing of hr and heading tokens via `tok.map` line ranges, mirroring `_md_block_texts`'s byte-slice contract; defensive fallback to prior behavior when `tok.map is None`. Pinned by `bench/test_harness_json.py::NeutralBlockTextsSourceFidelityTests` (4 tests covering hr style, atx+setext heading prefix, end-to-end SCORER DIVERGENCE on dropped heading, and end-to-end SCORER DIVERGENCE on hr style swap). Attribution probe rerun: `bench/probes/F8-7-neutral-block-texts-over-normalization/probe.py` exit 0 = inert on all three stages.
- **F8-6** — `_md_heading_tree` (bench/harness.py) returned `md outline --json`'s rendered plaintext (inline markdown stripped) while `neutral_heading_tree` returned markdown-it-py inline `tokens[i+1].content` (raw markdown source preserved). On `compare_heading_tree` tasks where any heading text contains inline formatting, the two scorers disagreed by construction. P1 false-POSITIVE: per `harness.py:1437` (`BenchResult.correct = ok_md`), the wrong answer aggregated as a benchmark PASS while SCORER DIVERGENCE was logged but did not gate `correct`. Filed T8 iter 12, CLOSED T8 iter 13. Hardening: new `_render_inline_to_plaintext` helper concatenates text+code_inline content, recurses into image children for alt text, and drops markup wrappers (strong/em/link/strikethrough open/close, html_inline, breaks); `neutral_heading_tree` calls it on `tokens[i+1].children`. Pinned by `bench/test_harness_json.py::NeutralHeadingTreeInlineRenderingTests::test_neutral_heading_tree_renders_inline_to_plaintext` + `::test_score_normalized_text_md_and_neutral_agree_on_emphasis_diff` + `::test_neutral_heading_tree_handles_image_alt_and_html_inline`. Attribution probe rerun: `bench/probes/F8-6-heading-tree-inline-formatting-divergence/probe.py` exit 0 = inert.
- **F8-5** — `_md_block_texts` (bench/harness.py:650-656) sliced the Python `str` `content` by the **byte** offsets that `md blocks --json` emits, drifting by the cumulative byte-excess from every preceding multi-byte UTF-8 char. P1 false-POSITIVE on `compare_block_text` tasks where international/typographic chars in any earlier block cause the leading char of the differing later block to drop from BOTH actual and expected slices — the md scorer accepted wrong answers as PASS while `score_normalized_text_neutral` correctly reported MISMATCH. Filed T8 iter 10, CLOSED T8 iter 11. Hardening: encode `content` to UTF-8 bytes once per call, slice the bytes, decode each slice. Pinned by `bench/test_harness_json.py::MdBlockTextsUtf8Tests::test_md_block_texts_honors_utf8_byte_boundaries` + `::test_score_normalized_text_md_rejects_wrong_answer_with_utf8`. Attribution probe rerun: `bench/probes/F8-5-md-block-texts-utf8-byte-vs-char-slice/probe.py` exit 0 = inert.
- **F8-4** — `extract_last_json`'s fence-strip regex preprocessor at `bench/harness.py:1560` blindly stripped backtick-triplet markers anywhere in agent text, including inside JSON string values, silently corrupting heading.text / body content before scoring. P1 false-NEGATIVE on json_envelope tasks where the wrapping envelope embeds a backtick triplet in any string value. Filed T8 iter 8, CLOSED T8 iter 9. Hardening: dropped the regex preprocessor entirely; the F8-3 string-aware depth scanner consumes the agent text directly and finds the JSON region inside ` ```json ` fences without preprocessing (backticks aren't structural JSON characters). Pinned by `bench/test_harness_json.py::test_extract_last_json_preserves_backticks_in_string_value` + `::test_extract_last_json_handles_fenced_json_via_depth_scanner`. Attribution probe rerun: `bench/probes/F8-4-fence-regex-strips-string-content/probe.py` exit 0 = inert.
- **F8-3** — `extract_last_json` depth scanner counted brace/bracket characters inside JSON string values as real openers/closers, so a `}` inside a heading.text value caused the wrapping envelope to never be enumerated as a candidate. P1 false-NEGATIVE on json_envelope tasks. Filed T8 iter 6, CLOSED T8 iter 7. String-aware scanner at `bench/harness.py` `extract_last_json` skips content between unescaped `"` boundaries in both passes; pinned by `bench/test_harness_json.py::test_extract_last_json_honors_string_boundaries_in_depth_scan` + `::test_extract_last_json_handles_escaped_quotes_in_string_value`. Attribution probe rerun: `bench/probes/F8-3-brace-in-string-value/probe.py` exit 0 = inert.
- **F8-2** — `extract_last_json` preferred a nested array over the wrapping object, causing a false-NEGATIVE on json_envelope tasks where the agent's text output embeds the answer envelope (`{…,"entries":[…]}`) within prose. P1. Filed T8 iter 4, CLOSED T8 iter 5. Highest-end-position rule at `bench/harness.py` `extract_last_json`; pinned by `bench/test_harness_json.py::test_extract_last_json_prefers_wrapping_envelope_over_nested_array` + `::test_extract_last_json_prefers_latest_sibling_when_no_containment`. Attribution probe rerun: `bench/probes/F8-2-extract-prefers-nested-array/probe.py` exit 0 = inert.
- **F8-1** — `select_json_envelope_actual` accepted `schema_version`-only overlap as a shape match. P1. Filed T8 iter 2, CLOSED T8 iter 3. Subset check at `bench/harness.py:1510`; pinned by `bench/test_harness_json.py::test_schema_version_only_overlap_rejected` + `::test_subset_check_preserves_extra_keys_on_actual`. Attribution probe rerun: `bench/probes/F8-1-schema-version-overlap/probe.py` exit 0 = inert.

## Archived findings (T7, iter 1–67)

T7 closure trail archived to `bench/ledger-archive/2026-Q2.md` on 2026-04-26 at T8
iter 1. All entries below were CLOSED in T7; they are pointers, not active state.

### Named findings (definitions in archive)

- **F1** — Search-split pilots lack holdout confirmation (partial). CLOSED 2026-04-26. Holdout bundles exist; durability claim retracted. → archive
- **F2** — Legacy N=3 snapshot overlaps the current holdout set. CLOSED 2026-04-26. `bench/RESULTS.md` carries split-disclosure header. → archive
- **F3** — T22 structural scorer rejects list-shape JSON. CLOSED 2026-04-26. `score_structural_json` normalizes top-level array under `compare_link_destinations`-only policy. → archive
- **F3-a** — `raw_bytes` scorer now honors EOF whitespace. CLOSED 2026-04-26. `harness.py:347-348` `.rstrip()` on whole normalized string. → archive
- **F4** — `json_envelope` scorer prefers JSON-parseable tool output over agent text. CLOSED 2026-04-26 iter 31. `select_json_envelope_actual` (harness.py:1481-1526) does shape-aware tool-output preference. → archive
- **L1** — Loop lacked holdout-immutability guard. CLOSED 2026-04-26. `bench/holdout/fingerprints.json` (`holdout_version: 1`) baselines SHA-256; `verify_holdout_fingerprints` raises on drift; `HoldoutImmutabilityTests` pin it. → archive
- **P3** — `bytes_output` is not cross-executor comparable. CLOSED 2026-04-26 iter 6. `bench/RESULTS.md` documents executor-locality rule and comparable-fields list. → archive

### T7 closure-trail iterations (archived, narrative-only)

T7 ran 67 iterations producing closure trails for the seven named findings above
plus narrative status checkpoints. The narrative entries are recorded once in the
archive; they do not reset T8's failing-trace-freshness counter.

Iteration index pointer (all → `bench/ledger-archive/2026-Q2.md`):

- iter 1–6: F1/F2/F3/F3-a baseline closure work; iter-6 cheap-channel snapshot.
- iter 7–11: P3 cross-executor field disclosure; first PI runner bundles (T1/T22).
- iter 12–15: Confirmation-review-pass cycles; cheap-channel re-verification.
- iter 16: L1 mechanical-guard runtime promotion (`check_holdout_integrity` no longer threads `--tasks-path`).
- iter 17: Holdout-version bundle stamping landed.
- iter 18–19: Cross-executor same-task measurement extensions.
- iter 20–22: Confirmation review passes + iter 21 comparable-harness-axis cell coverage.
- iter 23–25: T21 PI bundle reference extension; quiet-signal discharges.
- iter 26: Cross-executor same-task measurement extension (publication trail).
- iter 27–28: Confirmation review + scorer-dispatcher branch coverage assertion.
- iter 29–32: F4 filing → schema-aware json_envelope closure → bundle-replay typed test → confirmation review.
- iter 33–35: T11 PI bundle, quiet-signal discharge, pre-iter-30 selector counterfactual typed test.
- iter 36–39: Confirmation review + T19 PI bundle + quiet-signal + iter-37 counterfactual typed test.
- iter 40–43: Confirmation review + T10 PI bundle + quiet-signal + canonical re-query typed test.
- iter 44–47: Confirmation review + T15 PI bundle + quiet-signal + parallel-mutation FAIL pattern typed test.
- iter 48–51: Confirmation review + T12 PI bundle + quiet-signal + batch-mutation moat at scale typed test.
- iter 52–56: Confirmation review + T1 hybrid PI bundle + quiet-signal + spec coherence + T1 hybrid mode baseline typed test.
- iter 57–59: Quiet-signal + T1 unix PI bundle + spec coherence + T1 unix mode baseline typed test.
- iter 60–63: Confirmation review + T7 hybrid PI bundle + quiet-signal + spec coherence + T7 hybrid mode baseline typed test.
- iter 64–67: Confirmation review + T7 unix PI bundle + quiet-signal + spec coherence + T7 unix mode baseline typed test.
- 10 "Halt-condition / quiet-signal status" blocks for iters 58–67 (drift narrative; carried no fresh failing trace).

## T8 iterations

### Post-loop (2026-04-26): Close F8-8 — extend tok.map line-slice to collection-type tokens

Item 2 (close finding via hardening), authored manually outside the loop after the 5h loop-launch wall clock expired with F8-8 OPEN. Extended F8-7's `tok.map`-line-slice path in `neutral_block_texts` to the five collection-type tokens (paragraph_open, bullet_list_open, ordered_list_open, blockquote_open, table_open) with the same `tok.map is None` defensive fallback to the prior inline-collection path. Pinned by `bench/test_harness_json.py::NeutralBlockTextsCollectionFidelityTests` (4 tests: list markers, blockquote/table fidelity, end-to-end blockquote→paragraph flattening, end-to-end nested→flat list flattening). Attribution probe `bench/probes/F8-8-neutral-block-texts-collection-over-normalization/probe.py` exit 1 → 0 on all four stages. Cheap channel: 123 python unittests + cargo unit tests + harness dry-run all 24 tasks PASS dual scorer.

### Iter 16 (2026-04-26): Fresh failing trace — F8-8 neutral_block_texts collection-type over-normalization

Item 1 (fresh failing trace). Filed F8-8 against `neutral_block_texts` (bench/harness.py:120-144): the inline-collection branch dispatching paragraph_open/bullet_list_open/ordered_list_open/blockquote_open/table_open collects only `inline.content` from child tokens, dropping list markers (`- `/`1. `), blockquote prefixes (`> `), table separators (`|`/`---`), and indentation that distinguishes nesting levels. `_md_block_texts` byte-slices the source and preserves all of the above. Probe `bench/probes/F8-8-neutral-block-texts-collection-over-normalization/probe.py` exits 1 on all four stages: Stage A direct extractor divergence (5 of 6 collection-type blocks disagree on a single mixed sample); Stages B/C/D end-to-end SCORER DIVERGENCE on bullet→ordered swap, blockquote→paragraph flattening, and nested→flat list flattening (md=MISMATCH correctly, neutral=OK falsely on each).

Severity P2 (cross-check direction, mirror of F8-7's hr/heading half): md gates `BenchResult.correct` and correctly catches all four traces today, so binary correct flag preserved. Hooked from iter 15's learning ("the not-yet-touched paragraph/list/blockquote/table inline-collection paths in neutral_block_texts" remain unmined); cheap surface-mining yielded an 8th fresh trace (F8-1..F8-8) without invoking the auto-research generator. Closure plan: extend F8-7's `tok.map`-line-slicing fix to the five collection-type branches. Cheap channel green pre-filing: 119 python unittests + 116 cargo unit tests + harness dry-run all 24 corpus tasks PASS dual scorer; F8-7 probe still exit 0 (prior closure intact). Axis: failing-trace-freshness (counter resets to 0).

### Iter 15 (2026-04-26): Close F8-7 — source-fidelity slicing for hr/heading in neutral_block_texts

Item 2 (close finding by hardening). `bench/harness.py` `neutral_block_texts` now line-slices hr and heading tokens via `tok.map` (mirroring `_md_block_texts`'s byte-slice contract) instead of hardcoding `"---"` for hr and dropping the heading marker prefix. The closure preserves source fidelity for hr style (`---`/`***`/`___`) and heading marker (`# `/`## ` for atx, `=====`/`-----` for setext). Pinned by `NeutralBlockTextsSourceFidelityTests` (4 methods: hr style preservation, heading prefix preservation, end-to-end agreement on dropped heading, end-to-end agreement on hr style swap). Attribution probe `bench/probes/F8-7-neutral-block-texts-over-normalization/probe.py` exit 1→0 on Stages A, B, and C. Cheap channel green: 116 cargo unit tests + 119 python unittests + 24 corpus tasks all PASS dual scorer post-fix. Axis: surface-hardening cadence (file iter 14 → close iter 15, canonical 1-iteration pair). OPEN list now `_None._`; quiet-trace counter starts ticking from iter 15.

### Iter 14 (2026-04-26): Fresh failing trace — F8-7 neutral_block_texts over-normalizes hr style + heading prefix

Item 1 (fresh failing trace). Filed F8-7 against `neutral_block_texts` (bench/harness.py:86): the function hardcodes `"---"` for any hr token (regardless of source `---`/`***`/`___`) and drops the heading-level marker prefix on heading blocks (returning only inline content while `_md_block_texts` preserves the full `# Hello` byte slice). Probe `bench/probes/F8-7-neutral-block-texts-over-normalization/probe.py` exits 1 on all three stages: Stage A direct extractor divergence shows 4 of 7 blocks disagree on a single 7-block sample; Stage B end-to-end SCORER DIVERGENCE on `compare_block_text` where actual=`Configuration` (heading dropped) vs expected=`# Configuration` — md correctly says MISMATCH, neutral falsely says OK; Stage C similar but on hr style swap (`---` vs `***`).

Severity P2 (cross-check direction): md gates `BenchResult.correct` and correctly catches both classes, so binary correct flag is preserved on the wrong answer. Mirror of F8-6 (P1, md over-lenient side); F8-7 has neutral as the over-lenient side (P2 because non-gating). Hooked from iter 13's learning ("iter 14 may need to invoke the auto-research generator or reach for a different surface"); cheap surface-mining yielded a seventh fresh trace (F8-1..F8-7) without invoking auto-research. Cheap channel green pre-filing: 53 cargo + 115 python unittest + 24 corpus all PASS. Axis: failing-trace-freshness (counter resets to 0). Closure plan: align `neutral_block_texts` to `_md_block_texts`'s source-fidelity contract via `tok.map` line slicing or `tok.markup` + level-marker reconstruction; pin with three tests in `bench/test_harness_json.py`.

### Iter 13 (2026-04-26): Close F8-6 — render neutral_heading_tree inline children to plaintext

Item 2 (close finding by hardening). Added `_render_inline_to_plaintext` helper in `bench/harness.py` that concatenates `.content` of `text` and `code_inline` children, recurses into `image` children for alt text, and drops markup wrappers (strong/em/link/s open/close, html_inline, soft/hardbreak); `neutral_heading_tree` now invokes it on `tokens[i+1].children` instead of returning `tokens[i+1].content`. Matches `_md_heading_tree`'s md-outline rendered-plaintext contract on inline code, emphasis, links, images, and html — verified by direct equality against `_md_heading_tree` on the stage-A fixture and on an extended image+html_inline non-regression fixture. Pinned by `NeutralHeadingTreeInlineRenderingTests` (3 methods). Attribution probe `bench/probes/F8-6-heading-tree-inline-formatting-divergence/probe.py` exit 1→0 on both stages. Cheap channel green: 53 cargo unit tests + 115 python unittest tests + 24 corpus tasks all PASS dual scorer. Axis: surface-hardening cadence (file iter 12 → close iter 13, canonical 1-iteration pair). OPEN list now `_None._` again — quiet-trace counter starts ticking from iter 13; iter 14 must produce a fresh failing trace, an auto-research candidate, or a fresh probe variant or halt at iter 16.

### Iter 12 (2026-04-26): Fresh failing trace — F8-6 heading_tree md/neutral inline-markdown rendering divergence

**Substantive move:** item 1 (fresh failing trace against existing surface).
Filed F8-6 against `_md_heading_tree` (bench/harness.py:635) and
`neutral_heading_tree` (bench/harness.py:51). The two `compare_heading_tree`
extractors disagree by construction on any heading text containing inline
markdown: `_md_heading_tree` reads `md outline --json`'s already-rendered
plaintext (backticks, emphasis markers, link markup stripped), while
`neutral_heading_tree` returns markdown-it-py's inline `tokens[i+1].content`
(raw markdown source preserved). When `actual` and `expected` differ only by
inline formatting (e.g. agent introduces `**bold**` around a heading word),
the md scorer says heading_tree OK while neutral correctly says MISMATCH;
per `harness.py:1437` (`BenchResult.correct = ok_md`), the wrong answer
aggregates as a benchmark PASS.

**Probe:** `bench/probes/F8-6-heading-tree-inline-formatting-divergence/probe.py`
exits 1 on both stages. Stage A: direct extractor divergence on
`# The \`md tasks\` command`, `## **Important** notes`,
`### See [docs](url)` — md returns `[(1, 'The md tasks command'), …]`,
neutral returns `[(1, 'The \`md tasks\` command'), …]`. Stage B:
end-to-end SCORER DIVERGENCE on `compare_heading_tree` policy where
`expected = '# Configuration'` and `actual = '# **Configuration**'` —
md says OK (false-POSITIVE PASS), neutral correctly MISMATCH.

**Severity & realism:** P1 false-POSITIVE on any `compare_heading_tree`
task when any heading text contains inline formatting (inline code for
command/symbol names, emphasis, links). Current corpus is plain-ASCII
without inline formatting, so the trace is dormant on the live benchmark,
but technical Markdown frequently uses `# The \`md tasks\` command`
patterns; auto-research candidates exercising realistic README/spec
content will fire it. Pre-emptive filing follows the precedent of F8-2..F8-5.

**Surface freshness:** F8-1..F8-4 closed the json_envelope four-layer
cake on `extract_last_json` / `select_json_envelope_actual`. F8-5 closed
`_md_block_texts` byte-vs-char encoding on the `compare_block_text`
branch of the normalized_text dispatcher. F8-6 lives on the same
dispatcher's *other* branch — `compare_heading_tree` — but is a
representation-mismatch class (md plaintext vs neutral raw source)
rather than an encoding-boundary class. Hooked from iter 11's learning
("future surface-mining for SCORER DIVERGENCE classes should probe
specifically for char-vs-byte alignment between the two scorers, not
just 'do they ever disagree'"); generalized "char-vs-byte" to
"any rendering boundary the two scorers treat differently."

**Axis served:** failing-trace-freshness (counter resets to 0).
Auto-research generator still not invoked; cheap surface-mining yielded
a sixth fresh trace (F8-1 → F8-6) across iters 1–12. Hypothesis-mining
heuristic continues to outperform auto-research generator at this
stage of T8.

**Cheap channel:** green pre- and post-change. 32+37+16=85 cargo unit
tests pass; 112 python unittest tests pass; harness dry-run all 24
tasks PASS dual scorer. Probe directory only; no production code touched.

**Closure plan:** render `neutral_heading_tree`'s inline children to
plaintext (concatenate text-token children, drop code_inline/em/strong/
link markup) so it matches `_md_heading_tree`'s md-binary outline
contract. Pin with two tests in `bench/test_harness_json.py`:
(a) `test_neutral_heading_tree_renders_inline_to_plaintext` (the F8-6
trace as a typed test on the stage-A fixture), and (b)
`test_score_normalized_text_md_and_neutral_agree_on_emphasis_diff`
(end-to-end agreement on the stage-B fixture). Attribution probe =
rerun `probe.py`; expect exit 0 on both stages.

### Iter 11 (2026-04-26): Close F8-5 — encode content to bytes before slicing in `_md_block_texts`

Item 2 (close finding by hardening). `_md_block_texts` now encodes `content` to UTF-8 bytes once per call, slices, decodes. Pinned by `bench/test_harness_json.py::MdBlockTextsUtf8Tests` (stage A direct slicer; stage B md scorer rejects wrong UTF-8 answer). Attribution probe `bench/probes/F8-5-md-block-texts-utf8-byte-vs-char-slice/probe.py` exit 1→0. Cheap channel green. Axis: surface-hardening cadence (file→close in 1 iteration).

### Iter 10 (2026-04-26): Fresh failing trace — F8-5 `_md_block_texts` byte-vs-char slice drift on UTF-8

Item 1. Filed F8-5: `_md_block_texts` (harness.py:650-656) sliced Python `str` by byte offsets from `md blocks --json`, drifting on multi-byte UTF-8. Stage A: `# Héllo\n\nA: foo\n\nB: bar\n` returns drifted `['# Héllo', ': foo', ': bar']`. Stage B: SCORER DIVERGENCE — md says OK on `expected='# café\n\nFOO\n\nbar\n'` vs `actual='...\n\nXar\n'`, neutral correctly MISMATCH. P1 false-POSITIVE; current corpus ASCII-only so dormant. Probe: `bench/probes/F8-5-md-block-texts-utf8-byte-vs-char-slice/probe.py`. Hooked from iter 9's learning ("iter 10 must reach for a different surface"). Cheap channel green.

### Iter 9 (2026-04-26): Close F8-4 — drop fence-strip preprocessor on extract_last_json

Item 2. Removed the global ` ```(?:json)?\s*\n? ` regex preprocessor in `extract_last_json` and routed text directly into the F8-3 string-aware depth scanner; backticks aren't structural JSON characters so the brace tracker handles fenced JSON cleanly. Dropping the preprocessor eliminates the silent-corruption bug class entirely rather than papering over the backtick case. Pinned by `test_extract_last_json_preserves_backticks_in_string_value` + `test_extract_last_json_handles_fenced_json_via_depth_scanner`. Attribution probe `bench/probes/F8-4-fence-regex-strips-string-content/probe.py` exit 1→0. Cheap channel green. Axis: surface-hardening cadence.

### Iter 8 (2026-04-26): Fresh failing trace — F8-4 fence-strip regex is string-blind

Item 1. Filed F8-4: `extract_last_json`'s fence-strip preprocessor (`re.sub` for backtick-triplets) ran string-blind, silently mutating heading.text/body content inside JSON string values before scoring. JSON parses (backticks aren't quotes) but content is corrupted. Stage A: triplet inside `"Example: ...python block"` silently removed. Stage B: end-to-end MISMATCH on agent's byte-exact correct answer. Symmetry to F8-3: same string-blind-preprocessor class on a different layer (regex pre-scanner vs depth scanner) and a different character class (backticks vs braces/brackets). Probe: `bench/probes/F8-4-fence-regex-strips-string-content/probe.py`. Cheap channel green.

### Iter 7 (2026-04-26): Close F8-3 — string-aware depth scanner on extract_last_json

Item 2. The `{`/`}` and `[`/`]` candidate-enumeration passes now skip characters between unescaped `"` boundaries with backslash-escape handling. Pinned by `test_extract_last_json_honors_string_boundaries_in_depth_scan` + `test_extract_last_json_handles_escaped_quotes_in_string_value`. Attribution probe `bench/probes/F8-3-brace-in-string-value/probe.py` exit 1→0. Cheap channel green. Axis: surface-hardening cadence (file→close in 1 iteration).

### Iter 6 (2026-04-26): Fresh failing trace — F8-3 extract_last_json brace-in-string blind

Item 1. Filed F8-3: the two-pass character-walk depth scanner counted `}/]/{/[` inside JSON string values as real closers/openers. A `}` inside heading.text caused the wrapping envelope to never be enumerated as a candidate; only the inner array survived via the brace-blind `[`/`]` pass and was propagated unchecked through the text-output branch. Falsifies F8-1/F8-2's implicit assumption that the wrapping envelope is always enumerated. Probe: `bench/probes/F8-3-brace-in-string-value/probe.py`. Cheap channel green.

### Iter 5 (2026-04-26): Close F8-2 — highest-end-position rule on extract_last_json

Item 2. Replaced legacy "prefer last array, then last object" rule with "prefer the candidate whose source-span end position is highest." Highest-end subsumes both intended behaviors: containment forces `outer.end > nested.end`; sibling-recency forces `later.end > earlier.end`. Pinned by `test_extract_last_json_prefers_wrapping_envelope_over_nested_array` + `test_extract_last_json_prefers_latest_sibling_when_no_containment`. Attribution probe `bench/probes/F8-2-extract-prefers-nested-array/probe.py` exit 1→0. Cheap channel green.

### Iter 4 (2026-04-26): Fresh failing trace — F8-2 extract_last_json prefers nested array

Item 1. Filed F8-2: symmetric to F8-1 on the text-output branch. The extractor's "prefer arrays over objects" rule returns a nested entries/results/links array when the wrapping envelope is embedded in prose; the text-output branch passes the candidate through without a shape check, triggering "expected top-level JSON object" early-out FAIL. Probe: `bench/probes/F8-2-extract-prefers-nested-array/probe.py`. Cheap channel green.

### Iter 3 (2026-04-26): Close F8-1 — subset check on json_envelope shape match

Item 2. Replaced F4 intersection check (`_json_top_keys(parsed) & expected_top_keys`) at `harness.py:1481-1526` with subset check (`expected_top_keys.issubset(...)`) — every discriminating key from expected must be present, rejecting schema_version-only overlap. Pinned by `test_schema_version_only_overlap_rejected` + `test_subset_check_preserves_extra_keys_on_actual`. Attribution probe `bench/probes/F8-1-schema-version-overlap/probe.py` exit 1→0. Cheap channel green.

### Iter 2 (2026-04-26): Fresh failing trace — F8-1 scorer false-negative

Item 1. Filed F8-1: F4's intersection-on-top-keys gate accepts schema_version-only overlap as a shape match because every mdtools.v1 envelope shares schema_version. On a synthetic two-tool-call transcript (`md outline --json` then `md tasks --json`), reverse iteration picks the tasks envelope and the agent's correct answer in `all_tool_outputs[0]` is skipped. Hooked from iter 1's learning ("md tasks --json envelope is the same shape that triggered F4"). Probe: `bench/probes/F8-1-schema-version-overlap/probe.py`. Cheap channel green.

### Iter 1 (2026-04-26): Ledger archive + first telemetry contract artifact

Item 5 (telemetry on existing command). Added `bench/telemetry/tasks.md` as the first telemetry contract artifact, recording the contract shape for `md tasks` (command/args/selector_type/input_size_class/input_files/output_type/tasks_emitted/error_class/elapsed_micros) without implementing the src/ hook. Mechanical step: archived all 67 T7 iterations + 7 named findings to `bench/ledger-archive/2026-Q2.md` (15,658 → ~80 lines). Cheap channel green.
