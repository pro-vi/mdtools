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

(none)

## Closed in T8

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

### Iter 7 (2026-04-26): Close F8-3 — string-aware depth scanner on extract_last_json

**Substantive move:** item 2 (close finding by hardening existing
surface with typed artifact pinning the fix). The `{`/`}` and `[`/`]`
candidate-enumeration passes in `bench/harness.py` `extract_last_json`
now skip characters between unescaped `"` boundaries, so brace and
bracket characters inside JSON string values no longer falsely close
a candidate. Pre-fix, `}` inside a heading.text value caused the
`{`/`}` pass to record a truncated candidate that failed json.loads
and reset start = -1, so the wrapping envelope was never enumerated.
Post-fix, the scanner walks the actual JSON structure, the wrapping
envelope is enumerated, and the F8-2 highest-end-position rule
selects it over the inner entries array.

**Attribution probe:** rerun
`bench/probes/F8-3-brace-in-string-value/probe.py` → exit 0 on both
stages (direct extractor + harness path through
`select_json_envelope_actual` text-output branch). Pre-fix exit was
1 (live failing trace). Probe directory verdict.txt now captures
filed and closure verdicts side-by-side; README.md status flipped
to CLOSED.

**Pinned by:** two tests in `bench/test_harness_json.py` —
`test_extract_last_json_honors_string_boundaries_in_depth_scan` (the
F8-3 trace as a typed test) and
`test_extract_last_json_handles_escaped_quotes_in_string_value`
(non-regression for `\"` escape sequences inside string values
alongside non-string brace characters).

**Cheap channel:** green pre- and post-change. 108 python unittest
tests pass (was 106; +2 new); cargo unit tests pass; harness dry-run
all 24 tasks PASS dual scorer.

**Axis served:** surface-hardening cadence (F8-3 closed within 1
iteration of filing — same canonical file→close shape as F8-1 and
F8-2). All three layers of the json_envelope path are now hardened:
candidate enumeration (F8-3, this iter), preference rule (F8-2), and
shape discrimination across both branches (F8-1 + F8-2). After
closing F8-3, OPEN finding count is back to zero, so halt condition
#2 (hardening exhaustion) is conditionally live. Iter 8 must produce
a fresh failing trace, run an auto-research generator pass, or
operate the probe channel with a new variant — otherwise the
quiet-trace counter starts ticking toward halt.

### Iter 6 (2026-04-26): Fresh failing trace — F8-3 extract_last_json brace-in-string blind

**Substantive move:** item 1 (fresh failing trace against existing
surface). Filed F8-3 against `extract_last_json`
(`bench/harness.py:1539-1587`) and the text-output branch of
`select_json_envelope_actual`. The two-pass character-walk depth
scanner counts `}`/`]`/`{`/`[` characters inside JSON string values
as real closers/openers. When a wrapping envelope's heading.text
contains a `}` (extremely common in technical Markdown — e.g.
`Configuration {key: value}`, format strings, shell expansions),
the `{`/`}` pass closes prematurely on a truncated candidate, the
candidate fails `json.loads`, `start` resets, and the actual outer
envelope is never recorded. Only the inner `entries` array (recorded
by the brace-blind `[`/`]` pass on this shape) survives, which the
text-output branch propagates unchecked to `score_structural_json`.

**Probe:** `bench/probes/F8-3-brace-in-string-value/probe.py` exits 1
on both stages (direct extractor + harness text-output branch). The
probe is a standalone script under `bench/probes/`, not a unit test,
so the cheap channel stays green while F8-3 is OPEN.

**Symmetry to prior F8 closures:** F8-1 (CLOSED iter 3) hardened the
tool-output shape match (intersection→subset). F8-2 (CLOSED iter 5)
fixed extract_last_json's preference rule (array→object via
highest-end-position). Both prior closures assumed the wrapping
envelope is always recorded as a candidate; F8-3 falsifies that
assumption — when string values carry brace characters, the wrapping
envelope is never enumerated. Neither prior closure fires on this
shape.

**Axis served:** failing-trace-freshness (counter resets to 0).
Hooked from iter 5's hypothesis-mining heuristic ("where else does
the json_envelope shape-discrimination discipline NOT apply?") —
asking it on extract_last_json's candidate enumeration (rather than
its preference rule, which iter 5 closed) yielded F8-3.

**Cheap channel:** green pre- and post-change. The probe directory is
the only addition; no production code touched.

**Closure plan:** next iteration may close F8-3 by replacing the
character-walk depth scanner with a string-aware scanner that skips
characters between unescaped `"` boundaries when counting depth. Pin
with a unit test in `bench/test_harness_json.py` mirroring the
probe's stage-A shape. Attribution probe = rerun `probe.py`; expect
exit 0 on both stages.

### Iter 5 (2026-04-26): Close F8-2 — highest-end-position rule on extract_last_json

**Substantive move:** item 2 (close finding by hardening existing surface
with typed artifact pinning the fix). Replaced the legacy "prefer last
array, then last object" rule in `bench/harness.py` `extract_last_json`
with "prefer the candidate whose source-span end position is highest."
Highest-end-position subsumes both intended behaviors with a single
comparator: when one candidate's span contains another (F8-2's nested
entries array inside its own envelope), the container has the greater
end; when candidates are non-overlapping siblings (independent JSON
documents in agent text), the later one has the greater end and is
the agent's final answer.

**Attribution probe:** rerun
`bench/probes/F8-2-extract-prefers-nested-array/probe.py` → exit 0 on
both stages (direct extractor + harness path through
`select_json_envelope_actual`). Pre-fix exit was 1 (live failing
trace). Probe directory verdict.txt now captures both verdicts
side-by-side; README.md status flipped to CLOSED.

**Pinned by:** two tests in `bench/test_harness_json.py` —
`test_extract_last_json_prefers_wrapping_envelope_over_nested_array`
(the F8-2 trace as a typed test) and
`test_extract_last_json_prefers_latest_sibling_when_no_containment`
(non-regression for the highest-end-position rule under the sibling
case). Both pass.

**Cheap channel:** green pre- and post-change. 106 python unittest
tests pass (was 104; +2 new); 116 cargo unit tests pass; harness
dry-run all 24 tasks PASS dual scorer.

**Axis served:** surface-hardening cadence (F8-2 closed within 1
iteration of filing — same canonical file→close shape as F8-1). Both
text-output (F8-2) and tool-output (F8-1) branches of the
json_envelope selector path are now shape-aware. After closing F8-2,
the OPEN finding count is back to zero, so halt condition #2
(hardening exhaustion) is conditionally live again. The next
iteration must produce a fresh failing trace, run an auto-research
generator pass, or operate the probe channel with a new variant —
otherwise the quiet-trace counter starts ticking toward halt.

### Iter 4 (2026-04-26): Fresh failing trace — F8-2 extract_last_json prefers nested array

**Substantive move:** item 1 (fresh failing trace against existing surface).
Filed F8-2 against `extract_last_json` (`bench/harness.py:1539-1584`) and the
text-output branch of `select_json_envelope_actual` (`harness.py:1525-1532`).
Symmetric to F8-1 but on the text-output side: the extractor's
unconditional "prefer arrays over objects" rule returns a nested
`entries`/`results`/`links` array when the agent's wrapping envelope is
embedded in prose, and the text-output branch passes the candidate
through without a shape check, so the array reaches
`score_structural_json` and triggers the "expected top-level JSON
object" early-out FAIL.

**Probe:** `bench/probes/F8-2-extract-prefers-nested-array/probe.py`
exits 1 on both stages (direct extractor + harness path through
`select_json_envelope_actual` on a text-only transcript). The probe is
a standalone script, not a unit test, so the cheap channel stays green
while F8-2 is OPEN — same staging as F8-1.

**Axis served:** failing-trace-freshness (counter resets to 0). Hooked
from iter 3's learning that "the next iteration must produce a fresh
failing trace, run an auto-research generator pass, or operate the
probe channel with a new variant" — this iteration produces the fresh
trace by mining the F4/F8-1 surface for its symmetric text-output gap.

**Cheap channel:** green pre- and post-change. 53 cargo unit tests +
104 python unittest tests pass; harness dry-run unaffected (no
production code changed). The probe directory is the only addition.

**Closure plan:** next iteration may close F8-2 by hardening
`extract_last_json` so a wrapping object is preferred over a nested
array (e.g. by tracking source-span containment, or by dropping the
unconditional array-preference rule and preferring the outermost
top-level candidate by source order). Pin with a unit test in
`bench/test_harness_json.py` using the same `WRAPPING_ENVELOPE` shape.
Attribution probe = rerun `probe.py`; expect exit 0 on both stages.

### Iter 3 (2026-04-26): Close F8-1 — subset check on json_envelope shape match

**Substantive move:** item 2 (close finding by hardening existing surface
with typed artifact pinning the fix). Replaced the F4 intersection check
(`_json_top_keys(parsed) & expected_top_keys`) at
`bench/harness.py:1481-1526` with a subset check
(`expected_top_keys.issubset(_json_top_keys(parsed))`). Subset requires
every discriminating key from the expected shape to be present in the
candidate, which rejects schema_version-only overlap and surfaces the
correct envelope.

**Attribution probe:** rerun
`bench/probes/F8-1-schema-version-overlap/probe.py` → exit 0 (inert).
Pre-fix exit was 1 (live failing trace). Probe directory verdict.txt now
captures both the filed and closure verdicts side-by-side.

**Pinned by:** two tests in `bench/test_harness_json.py` —
`test_schema_version_only_overlap_rejected` (the F8-1 trace) and
`test_subset_check_preserves_extra_keys_on_actual` (regression-protection
that an envelope with extra fields still matches). Both pass.

**Cheap channel:** green. 104 python unittest tests pass; 116 cargo unit
tests pass; harness dry-run all 24 tasks PASS dual scorer.

**Axis served:** surface-hardening cadence (F8-1 closed within 1
iteration of filing, satisfies the disturbance-sign threshold of ≤2
iterations). Failing-trace-freshness counter resets on the typed
artifact (`bench/test_harness_json.py` adds two new tests directly
attributable to F8-1). All OPEN findings closed → halt condition #2
(hardening exhaustion) is conditionally live: the next iteration must
either produce a fresh failing trace, run an auto-research generator
pass, or move on a candidate, or the loop will halt by quiet-trace
counter at iter 6.

### Iter 2 (2026-04-26): Fresh failing trace — F8-1 scorer false-negative

**Substantive move:** item 1 (fresh failing trace against existing surface).
Filed F8-1 against `select_json_envelope_actual` (harness.py:1481-1526). The
F4 closure (iter 31) uses `_json_top_keys(parsed) & expected_top_keys`
(intersection non-empty) as the shape-match gate. Because every `mdtools.v1`
envelope shares the top-level key `schema_version`, the gate accepts *any*
mdtools envelope when expected is *any* mdtools envelope — no discrimination
across mdtools commands. Probe at `bench/probes/F8-1-schema-version-overlap/`
reproduces the false-negative on a synthetic two-tool-call transcript
(`md outline --json` then `md tasks --json`); reverse iteration picks the
tasks envelope, downstream structural comparison fails, agent's correct
answer in `all_tool_outputs[0]` is skipped. Exit code 1 = live.

**Axis served:** failing-trace-freshness (counter resets to 0). The
hypothesis was hooked from iter 1's learning ("md tasks --json envelope is
the same shape that triggered F4… potential failing-trace surface"); this
iteration converts the hypothesis into a typed probe artifact.

**Cheap channel:** green pre- and post-change. 102 python unittest tests pass;
85 cargo unit tests pass; harness dry-run all 24 tasks PASS dual scorer. The
probe is a standalone script under `bench/probes/`, not part of the unit-test
discovery, so it does not break the cheap channel while the finding is OPEN.

**Closure plan:** next iteration may close F8-1 by replacing the intersection
check with a subset check (`expected_top_keys.issubset(_json_top_keys(parsed))`)
plus a unit test in `bench/test_harness_json.py` pinning the discriminating-key
requirement. Attribution probe = rerun `probe.py`; expect exit 0.

### Iter 1 (2026-04-26): Ledger archive + first telemetry contract artifact

**Substantive move:** item 5 (telemetry on existing command). Added
`bench/telemetry/tasks.md` as the first telemetry contract artifact, recording
the contract shape for `md tasks` (the most-used command per re-query pattern).
The contract is documentation-only — no `src/` hook implemented, per T8 spec
("Implementing the actual telemetry hook in `src/` is admissible only if paired
with a finding or candidate that benefits from it").

**Mechanical step:** archived all 67 T7 iterations + 7 named findings to
`bench/ledger-archive/2026-Q2.md` (15,658 → ~80 lines for the active ledger).
Per spec, the archive operation does not count as the iteration's substantive move.

**Cheap channel:** green pre- and post-change. 16 cargo unit tests + 102 python
unittest tests pass; `harness.py --md-binary` dry-run all 24 tasks PASS dual scorer.

**Axis served:** failing-trace-freshness counter is at 0 (T8 just started). No
disturbance yet. Surface-hardening cadence and auto-research realism axes are
also at 0. The telemetry contract creates the missing structural artifact
directory under `bench/telemetry/` so future findings on `md tasks` selector or
output stability can reference a recording shape without first having to invent
the contract.
