# Bench Ledger

Concise human-memory surface for the frontier loop. Weaker evidence than typed
artifacts under `bench/runs/`. OPEN findings gate claim-expansion; they are
cleared only after a typed artifact confirms the finding, not by prose alone.

## OPEN findings

_(none — F4 promoted to CLOSED on 2026-04-26 iter 31 via closure-discipline review pass; see "Confirmation review pass (2026-04-26 iter 31)" and "F4 closure: schema-aware json_envelope actual selection (2026-04-26 iter 30)" CLOSED entries below, plus the archived F4 finding body under "## CLOSED" with Status field updated to CLOSED.)_

## CLOSED

### Quiet-signal checkpoint discharge (2026-04-26 iter 33)

Per the spec's "After 3 consecutive iterations with the cheap channel
green, no new failing trace, and no new finding added to the findings /
ledger surface, run the expensive outer channel" rule, iter 33 ran the
expensive outer channel. The quiet-signal counter was at 3 after iter 32
(iters 30 / 31 / 32 were all quiet — F4 closure work and its ratification /
typed-test promotion). Cheap channel re-verified green before and after:
`cargo test -q` all suites pass (32 + 37 + 16 + 0); `python3 -m unittest
bench.test_command_policy bench.test_oai_loop bench.test_pi_audit
bench.test_harness_json bench.test_harness_run_artifacts
bench.test_harness_task_split bench.test_analyze_inputs
bench.test_report_inputs` reports "Ran 79 tests in 1.569s … OK";
`python3 bench/harness.py --md-binary target/release/md` dry-run reports
"All tasks pass dual scorer" on all 24 tasks.

- **Bundle:** `bench/runs/checkpoint-pi-T11-mdtools-gpt5.4mini-2026-04-26/` —
  ninth PI runner bundle. Single task (T11, search-split, GFM-task
  counting per phase heading). Single mode (mdtools). Single run. Model
  `openai-codex/gpt-5.4-mini` at `thinking_level=minimal`, recorded per-
  result and per-run on the metadata bundle. `run.json` line 20 carries
  `holdout_version: 1` — the **fifth** durable bundle in `bench/runs/`
  carrying iter-17's stamp (after iter-18 T2, iter-21 T21, iter-25 T9,
  iter-29 T16).
- **Verdict:** T11 mdtools dual-scorer **PASS** in 8.93s with 2 tool
  calls (`./md outline <file> --json` returning 1,487 bytes,
  `./md tasks <file> --json` returning 6,639 bytes), 0 mutations,
  `requeried=false`, `policy_violations=0`, `bytes_observation=8,126`,
  `bytes_output=834,336` (PI streaming overhead, see P3 cross-executor
  rule in `bench/RESULTS.md`). `diff_report: json_canonical: OK`.
  Pi-audit log at `logs/T11_mdtools_1777227478/pi-audit.jsonl` preserves
  6 events (`model_change`, `thinking_level_change`, then two
  `tool_call`/`tool_result` pairs), parses cleanly via
  `bench/pi_audit_adapter.summarize_pi_audit_events` with
  `PiAuditCounters(tool_calls=2, tool_results=2, tool_errors=0,
  bytes_observation=8126, blocked=0, policy_violations=0, mutations=0,
  requeried=False, model='openai-codex/gpt-5.4-mini',
  thinking_level='minimal', bash_commands=[…])`.
- **F4 closure trail corroboration (forward-pointing observation, no
  historical edit; framing tightened post-second-opinion review on
  iter 33):** the iter-33 T11 trace provides fresh expensive-channel
  regression evidence that the post-iter-30
  `select_json_envelope_actual` selector at `bench/harness.py:1481`
  selects the agent's matching text answer over schema-disjoint JSON
  tool envelopes on a NEW task trajectory — distinct from iter 32's
  typed replay of the original T16 failure bundle. Replaying the
  bundle through `parse_pi_json_output` +
  `select_json_envelope_actual` shows: tool 0 has top keys
  `['entries', 'file', 'schema_version']`, tool 1 has top keys
  `['results', 'schema_version']`, the agent's text answer has top
  keys `['phases', 'totals']`, and expected has top keys
  `['phases', 'totals']` — both tool outputs are JSON-parseable but
  their schemas disjoint with expected, so the post-iter-30 selector
  skips both and returns the text answer, which matches expected
  bit-exact.
  - **Counterfactual executed (not inspection-derived):** running the
    pre-iter-30 selector logic from `git show 7b36502:bench/harness.py`
    lines 1404-1429 (the `for tool_out in reversed(all_tool_outputs)`
    loop that returns the first non-empty parseable JSON dict/list)
    against this same iter-33 T11 trajectory selects tool 1 with keys
    `['results', 'schema_version']` and `score_task` returns
    `md PASS=False, neutral PASS=False` with
    `json_canonical: MISMATCH at line 2: expected "phases": [` /
    `actual: "results": [` — bit-identical in mismatch shape to
    iter-29 T16's `expected: "files": [` / `actual: "results": [`.
    The counterfactual is mechanically pinned, not "structurally
    indicated by inspection."
  - **Closure attribution:** F4 closure remains anchored by iter 30
    (selector + 8 synthetic tests), iter 31 (typed-artifact replay
    of the original T16 bundle), and iter 32 (typed cheap-channel
    bundle-replay assertion). Iter 33 does **not** re-close F4 and
    does **not** open a new finding — it is supporting regression
    evidence that the fix generalizes beyond the original T16 bundle
    to a fresh agent trajectory under PI on a different task. The
    iter-29 T16 trajectory came via a recovery path (`find` → `ls` →
    `./md blocks <dir>` failure → `./md tasks <files> --json`); the
    iter-33 T11 trajectory came via natural extraction (`./md outline
    --json` + `./md tasks --json`). Different upstream trajectory
    shapes producing the same F4 hazard preconditions strengthens the
    "fix generalizes" claim without re-litigating the closure itself.
  - **Attribution modesty (intervention shape):** improves
    observability and anchor justification (the F4 fix has additional
    real-trajectory evidence beyond the original T16 bundle); does
    NOT improve product, oracle semantics, or specification (no code
    change to `bench/harness.py`, no new scorer rule, no spec
    revision). Quiet-signal value: valid expensive-channel sample
    with failure-class relevance — not just quota compliance.
- **Comparability framing:** this is the first cell for (PI runner,
  gpt-5.4-mini, mdtools, minimal thinking, T11, runs_per_task=1,
  holdout_version=1). It is **NOT** a reconfirmation of any prior
  holdout bundle — T11 is in the search split, not holdout. It is
  **NOT** a mode/model comparison versus any prior bundle: no OAI
  same-task mdtools cell exists for T11 (verified by scanning
  `bench/runs/*/results.json` for `task_id == "T11"`; only the
  `default-corpus-dry-run-2026-04-20` cell exists, which is a
  zero-elapsed dry-run with `model=None`, not a comparable real run).
  T11 is therefore not yet eligible for the `bench/RESULTS.md:54`
  cross-executor table — same status as iter-18 T2 and iter-21 T21
  documented at `bench/RESULTS.md:65-67` — but the iter-33 bundle
  could be cashed out into the cross-executor section as a tenth
  bundle reference paragraph following the iter-19 / iter-23 pattern,
  if a future iteration chooses that as its frontier anchor.
- **What this discharges:** the spec's quiet-signal-checkpoint rule
  by introducing fresh typed signal via the expensive channel. It does
  **NOT** discharge any product or oracle claim — those still require
  their own attribution probes and apples-to-apples comparisons.
- **Same-family-rule discharge:** iter 30 was oracle-trustworthiness
  (closing F4 via schema-aware `select_json_envelope_actual`), iter 31
  was closure-discipline ratification paired with `bench/RESULTS.md:72`
  substantive downgrade, iter 32 was oracle-trustworthiness (typed-test
  promotion of iter 30 / iter 31 prose-only replay claim to the
  `F4ClosureBundleReplayTests` cheap-channel assertion). Iter 33 is
  **intervention-diversity** (expensive outer channel run + new durable
  PI bundle), shifting axis cleanly from iter 32. The forced expensive-
  or-halt mandate at iter 33 (per the spec's 3-consecutive-quiet rule)
  is its own escape clause for the same-family rule, parallel in shape
  to iter 25's discharge of iter-22 / -23 / -24 same-family pressure,
  iter 14's discharge of iter-11 / -12 / -13 same-family pressure, and
  iter 29's discharge of iter-26 / -27 / -28 same-family pressure.
- **Closure-discipline status:** **CLOSED** at authoring time per the
  iter-4 / -7 / -10 / -14 / -18 / -21 / -25 / -29 quiet-signal-discharge
  pattern (no FIXED_PENDING_CONFIRMATION promotion needed because
  there is no fix here — the bundle is the deliverable). A future
  review pass should ratify by re-reading every data point in this
  entry against `results.json`, `run.json`, `pi-audit.jsonl`, and the
  persisted `agent_output.txt`; in particular, the F4-closure-trail
  corroboration claim above (top-key disjoint between tool outputs
  and expected; matching top keys in agent text answer; counterfactual
  pre-iter-30 selector returning the schema-mismatching `results`
  envelope and FAILing dual-scorer with `json_canonical: MISMATCH at
  line 2`) is reproducible by running the inline-documented selector
  replay recipe against the bundle, including the
  `git show 7b36502:bench/harness.py` excerpt for the pre-iter-30
  loop body.
- **What this does NOT do:** does not promote any product anchor
  (`bench/probes/anchor-validation/` still does not exist). Does not
  bump `holdout_version` (still 1; T11 is search-side, no holdout-
  side artifact change). Does not edit any harness production code
  (only ledger and a new bundle directory under `bench/runs/`). Does
  not extend the cross-executor table (no OAI T11 mdtools counterpart
  exists). Does not modify any historical ledger entry inline (per
  iter-15 / -22 / -24 / -26 / -27 / -28 / -30 / -31 / -32 discipline).
  Does not edit any published-narrative file (`bench/RESULTS.md`,
  `README.md`, `CLAUDE.md`, `bench/retracted_2026-04-24/README.md`,
  `specs/**`). Does not amend any pass-rate claim. Does not extend
  `bench/probes/`, `bench/search/candidates/`, or any other
  not-yet-existing T7 directory. Does not re-raise F4 — the new bundle
  is independent confirmation that the iter-30 fix holds on a fresh
  agent trajectory, not a re-opening of the closed finding.

### F4 closure trail extension: bundle-replay typed cheap-channel assertion (2026-04-26 iter 32)

Promoted iter 30's and iter 31's REPL-recorded replay claim — that
the iter-29 T16 PI bundle's durable `agent_output.txt`, when re-run
through `parse_pi_json_output` + `select_json_envelope_actual` +
`score_task`, scores `md=PASS neutral=PASS` with `json_canonical: OK`
under the post-iter-30 schema-aware selector — to a typed cheap-channel
assertion via a new `F4ClosureBundleReplayTests` class with one test in
`bench/test_harness_json.py`. The test loads the bundle's agent_output
from disk, asserts the expected trace shape (4 tool outputs, 1 text
output), runs the post-iter-30 extraction + scoring pipeline, and
asserts dual-scorer PASS plus `json_canonical: OK` in the report.
Implicitly ratifies iter 31 by extending the F4 closure trail with a
stronger typed artifact rather than re-raising the finding.

- **Disturbed axis:** oracle trustworthiness — the iter-30 / iter-31
  prose replay claim was the strongest evidence on file that the
  named F4 failure class actually moved on the original failing
  trace, but it lived only in ledger prose. Promoting it to a
  cheap-channel test makes future scorer-extraction regressions
  affecting the iter-29-shaped cell pattern visible immediately,
  rather than waiting for an expensive PI re-run on T16.
- **Frontier anchor:** the iter-31 CLOSED entry's "Replay verification
  of iter-30's typed-artifact attribution probe" bullet, which
  records the replay outcome but anchors it only in prose; the
  spec's "test coverage" admissibility under hardening (zero OPEN
  findings, P0 / P1 not active, but the same allowance applies a
  fortiori); and the recurring pattern of promoting prose claims to
  typed cheap-channel assertions established by iter 16 (procedural
  → mechanical-at-runtime), iter 17 (spec requirement → bundle
  stamp), iter 28 (iter-27 corpus-vacuous-path prose → typed test).
  The 8 existing `JsonEnvelopeActualSelectionTests` exercise the
  selector on synthetic inputs; the new test exercises it on the
  durable real-world failing trace that motivated F4 in the first
  place.
- **Change shape:**
  - Added `from pathlib import Path`, `load_tasks`, `score_task`, and
    `bench.pi_runner.parse_pi_json_output` imports at the top of
    `bench/test_harness_json.py`.
  - Added `F4ClosureBundleReplayTests` class with one test method,
    `test_iter_29_t16_bundle_replays_to_dual_scorer_pass`, immediately
    after `JsonEnvelopeActualSelectionTests` and before `if __name__
    == "__main__"`. The test resolves the bundle path relative to
    `Path(__file__).resolve().parents[1]` so it runs from the repo
    root regardless of caller cwd, and calls `self.skipTest(...)` if
    the bundle is absent (graceful fork-compat parallel to
    `check_holdout_integrity`'s missing-config skip at iter 16).
  - The test asserts:
    1. `parse_pi_json_output(raw)` returns a trace with `tool_calls=4`
       and one `text_output` (matches the iter-29 / iter-30 / iter-31
       prose).
    2. `select_json_envelope_actual` against T16's expected JSON
       returns the agent's text answer (top keys
       `["files", "total_pending"]`), bit-equal as a parsed JSON
       object to the expected output.
    3. `score_task(task, actual, expected, md_binary="md")` reports
       `md=True`, `neutral=True`, with `"json_canonical: OK"` in the
       returned report (the json_canonical scorer does not invoke
       the md binary, so an unresolved binary name is harmless).
- **Tests added (1 new):** `bench/test_harness_json.py`'s
  `F4ClosureBundleReplayTests.test_iter_29_t16_bundle_replays_to_dual_scorer_pass`.
  Total python unittest count across the eight spec-named modules
  rose 78 → **79** (`Ran 79 tests in 1.861s ... OK`).
- **Cheap channel:** green before and after.
  - `cargo test -q` all suites pass: 32 + 37 + 16 + 0.
  - `python3 -m unittest bench.test_command_policy bench.test_oai_loop
    bench.test_pi_audit bench.test_harness_json
    bench.test_harness_run_artifacts bench.test_harness_task_split
    bench.test_analyze_inputs bench.test_report_inputs` reports
    "Ran 79 tests in 1.861s … OK".
  - `python3 bench/harness.py --md-binary target/release/md` dry-run
    reports "All tasks pass dual scorer" on all 24 tasks (T16 in the
    dry-run still feeds expected directly as actual; the new test
    independently exercises the post-iter-30 selector path on the
    real iter-29 trace).
- **Closure-discipline ratification of iter 31 (implicit, by typed
  extension rather than prose re-read):** the new test passing
  end-to-end requires every iter-31 typed-artifact reference to be
  correct.
  - `select_json_envelope_actual` callable from `bench.harness` ✓
    (test imports it; located at `bench/harness.py:1481`).
  - `parse_pi_json_output` returns a `PiJsonTrace` whose
    `tool_outputs` and `text_outputs` lists feed the selector ✓
    (located at `bench/pi_runner.py:73`).
  - The iter-29 bundle's `agent_output.txt` exists and contains the
    JSONL stream iter-31 described ✓ (path
    `bench/runs/checkpoint-pi-T16-mdtools-gpt5.4mini-2026-04-26/logs/T16_mdtools_1777224275/agent_output.txt`,
    the same file iter-31 named in its replay-verification bullet).
  - The trace shape iter-31 described (4 tool outputs, 1 text output)
    is asserted directly ✓.
  - The selector returns the text answer with top keys
    `["files", "total_pending"]` matching `bench/expected/t16_count.json`
    ✓ (asserted via `json.loads` equality).
  - `score_task` returns `md=True neutral=True` with
    `"json_canonical: OK"` in the report ✓ (asserted directly).
  - F4 is not re-raised (no new failure trace surfaced).
- **Comparability framing:** this iteration does **not** modify any
  prior bundle in `bench/runs/` (the iter-29 T16 bundle's
  `agent_output.txt` is read but not written), does **not** bump
  `holdout_version` (still 1), does **not** introduce a new product
  surface or change the agent's action space, does **not** add a new
  CLI command, does **not** re-score any prior bundle (the iter-29
  bundle's recorded `correct=False` remains the bundle's frozen
  truth; the new test verifies the post-iter-30 *selector + scorer
  pipeline* against the bundle's stored agent outputs without
  modifying the bundle), does **not** edit any historical ledger
  entry inline (per iter-15 / iter-22 / iter-24 / iter-26 / iter-27 /
  iter-28 / iter-30 / iter-31 no-silent-edit discipline), does
  **not** extend the cross-executor same-task table in
  `bench/RESULTS.md` (the iter-29 T16 row remains absent for the
  same reason iter 31 named: the bundle's frozen `correct=False` is
  preserved as durable truth), does **not** change `README.md`,
  `CLAUDE.md`, `specs/**`, `bench/tasks/`, `bench/holdout/`, or
  `bench/expected/`. The only external file edits are: (i)
  `bench/test_harness_json.py` (3 added imports plus one new test
  class with one test method), and (ii) this ledger entry plus the
  (after iter 31) → (after iter 32) status block update.
- **Closure-discipline status (iter 32's own meta-status):** this
  iter-32 entry is itself FIXED_PENDING_CONFIRMATION at authoring;
  the next iteration's review pass either explicitly ratifies by
  re-reading the typed-artifact references in this entry (the
  `F4ClosureBundleReplayTests` class definition in
  `bench/test_harness_json.py`, the 79-test cheap-channel run, the
  iter-29 bundle path) or simply does not re-raise F4 and does not
  re-raise the new test's assertions.
- **Same-family-rule discharge:** iter 28 was oracle-trustworthiness
  (typed-test promotion of iter-27's prose claim — `ScorerDispatcher
  BranchTests`), iter 29 was intervention-diversity (forced
  expensive-or-halt: T16 mdtools PI bundle that surfaced **F4 P1
  OPEN**), iter 30 was oracle-trustworthiness (F4 closure via
  schema-aware selector + 8 synthetic tests), iter 31 was
  closure-discipline ratification paired with `bench/RESULTS.md:72`
  downgrade. Iter 32 is **oracle-trustworthiness** (typed-test
  promotion of iter-30 / iter-31 prose replay claim — the
  `F4ClosureBundleReplayTests` class). This is the third
  oracle-trustworthiness move in the iter-28..32 window, but the
  intervening iter-29 expensive run and iter-31 closure-discipline
  ratification (paired with the `bench/RESULTS.md:72` substantive
  edit) break any concentration. The structural precedent is iter
  28's relation to iter 27 (promote prior-iter prose claim about a
  durable property to a typed cheap-channel assertion), iter 17's
  relation to the spec's stamping requirement, and iter 16's
  relation to iter-3's pre-recorded promotion-from-procedural-to-
  mechanical hint. The fresh-failing-trace escape clause
  additionally applies through the recurring "prose-only replay
  claims are weaker than typed-artifact assertions" learning that
  iter 28 explicitly named ("prose claims about typed-artifact
  properties are a structurally weaker class of evidence than
  mechanical cheap-channel assertions").
- **What this does NOT do:** does not promote any new product
  anchor (none declared); does not run the expensive outer channel;
  does not re-score any prior bundle; does not amend any prior
  pass-rate claim; does not invalidate any cross-executor table row
  (the rule and all five existing rows remain valid); does not
  extend the cross-executor table with a new row; does not
  introduce a new finding (the test ratifies an existing CLOSED
  finding rather than opening a new one); does not bump
  `holdout_version`; does not modify `bench/runs/` contents.

### Confirmation review pass (2026-04-26 iter 31)

Discharged the closure-discipline rule for iter 30's F4 fix by
re-reading every typed artifact bit-exact. F4 transitions from
FIXED_PENDING_CONFIRMATION to **CLOSED**. Paired with a substantive
update to `bench/RESULTS.md:72` downgrading the F4 quarantine
paragraph to an F4 closure note, and one forward-pointing citation
correction for an iter-30 line-number typo recurring three times in
the iter-30 entry and the (after iter 30) halt-condition block.

- **Disturbed axis:** closure-discipline procedural ratification
  (oracle trustworthiness for the paired `bench/RESULTS.md`
  downgrade, since the published narrative was carrying an
  out-of-date "P1 OPEN" framing for the now-closed F4).
- **Frontier anchor:** the iter-30 ledger entry's explicit
  invitation under **Closure-discipline status**: "the next
  iteration's review pass to explicitly confirm by re-reading the
  iter-30 typed artifacts (the new helper functions, the test
  class, the iter-29 bundle replay output) and the modified call
  site against this entry". Per spec, the **FIXED ≠ CLOSED** rule
  requires either explicit ratification by typed-artifact re-read
  or a non-re-raise pass; iter 31 chose explicit ratification.
- **Bit-exact verifications (every iter-30 claim re-read):**
  - `_json_top_keys` is at `bench/harness.py:1457-1466`
    (`def _json_top_keys(parsed) -> set[str]:` confirmed at line
    1457; body returns `set(parsed.keys())` for dicts,
    `set(parsed[0].keys())` for non-empty list-of-dicts, empty set
    otherwise — matches the iter-30 description).
  - `_expected_json_top_keys` is at `bench/harness.py:1469-1478`
    (`def _expected_json_top_keys(expected_str: str) -> set[str] | None:`
    at line 1469; returns `None` on `JSONDecodeError` / `TypeError`,
    or `None` when the parsed value has no observable key shape
    (empty key set), else the key set — matches the iter-30
    description).
  - `select_json_envelope_actual` is at `bench/harness.py:1481-1526`
    (`def select_json_envelope_actual(...)` at line 1481; walks
    `reversed(all_tool_outputs)` preferring shape-matching outputs;
    on no match, falls through to `reversed(all_text_outputs)`;
    final fallback chain is `fallback_tool_actual` (most-recent
    non-shape-matching tool output) then `extract_last_json(stdout)`
    — matches the iter-30 description).
  - `extract_last_json` is at `bench/harness.py:1529` (still
    immediately after `select_json_envelope_actual`).
  - The `expected_artifact == "json_envelope"` branch at
    `bench/harness.py:1404-1408` is now a single 5-line `elif` that
    calls `select_json_envelope_actual(all_tool_outputs,
    all_text_outputs, stdout, expected_content)` — replacing the
    pre-iter-30 inline 26-line loop at lines 1404-1429 (matches the
    iter-30 description that the inline loop was replaced with a
    single call).
  - `bench/test_harness_json.py:135-248` defines
    `JsonEnvelopeActualSelectionTests` with exactly **8** tests
    (`test_t16_shape_text_wins_over_intermediate_md_tasks_envelope`,
    `test_t9_shape_jq_projected_tool_output_wins`,
    `test_t1_shape_matching_tool_output_wins_over_text_noise`,
    `test_t22_top_level_list_tool_output_falls_through_to_fallback`,
    `test_text_only_answer_works`,
    `test_no_shape_match_falls_back_to_most_recent_tool_output`,
    `test_unknown_expected_shape_preserves_first_tool_output_rule`,
    `test_empty_outputs_falls_back_to_extract_last_json_of_stdout`).
    `select_json_envelope_actual` is imported at line 11 of the
    test module. All 8 tests pass under `python3 -m unittest -v
    bench.test_harness_json` alongside the 6 pre-existing tests
    in the module.
  - Total python unittest count across the eight spec-named modules
    is **78** (`Ran 78 tests in 1.650s ... OK`), matching the
    iter-30 claim of "70 → 78 (+8 from
    `JsonEnvelopeActualSelectionTests`)".
  - `cargo test -q` reports `37 passed` and `16 passed` and `0
    passed` on the three reported test runs (matching the iter-30
    "all suites pass" claim).
  - `python3 bench/harness.py --md-binary target/release/md`
    dry-run reports "All tasks pass dual scorer" on all 24 tasks;
    T1 / T5 / T9 / T11 / T16 / T19 / T21 / T22 (the eight
    `expected_artifact == "json_envelope"` tasks) all
    `md=PASS neutral=PASS`.
- **Replay verification of iter-30's typed-artifact attribution
  probe:** parsed
  `bench/runs/checkpoint-pi-T16-mdtools-gpt5.4mini-2026-04-26/logs/T16_mdtools_1777224275/agent_output.txt`
  via `bench/pi_runner.parse_pi_json_output`; the `PiJsonTrace`
  carries 4 tool outputs and 1 text output. Tool outputs, in
  index-order: `[0]` is a 166-byte `[bench-guard] denied …` string
  (non-JSON); `[1]` is a 146-byte guard-deny string (non-JSON);
  `[2]` is the 57-byte `Is a directory (os error 21) …` from the
  agent's first `./md blocks <vault>` mis-call (non-JSON); `[3]`
  is the `./md tasks <files> --json` envelope, a JSON dict with
  top keys `["results", "schema_version"]`. The single text
  output is the agent's correct answer
  `{"total_pending":9,"files":[{"file":"backend.md","pending":3},
  {"file":"devops.md","pending":4},{"file":"frontend.md","pending":2}]}`
  (131 bytes), parsed top keys `["files", "total_pending"]`.
  Calling
  `select_json_envelope_actual(trace.tool_outputs,
  trace.text_outputs, trace.stdout, expected_content)` against
  T16's `bench/expected/t16_count.json` (top keys `["files",
  "total_pending"]`) returns the agent's text answer. `score_task`
  on the returned actual reports
  **`md=True neutral=True`** with `report = "json_canonical: OK"`.
  The iter-29 `correct=False` would now be `correct=True` under
  the new selector — exactly matching iter-30's claim. This is
  the spec's required attribution probe: the named failure class
  (F4 false-negative on json_envelope text-answer cells) actually
  moved.
- **Forward-pointing observation — iter-30 line citation typo:**
  the iter-30 entry contains three references to `bench/harness.py`
  line **1478** for the new helpers / `select_json_envelope_actual`
  helper, but line 1478 in the current file is the closing line of
  `_expected_json_top_keys`, not the location of any helper or of
  `extract_last_json`. The actual line positions at iter-31
  read time are: `_json_top_keys` at 1457,
  `_expected_json_top_keys` at 1469, `select_json_envelope_actual`
  at 1481, `extract_last_json` at 1529. The three iter-30 textual
  references are: (a) the F4 closure entry's "Change shape" first
  bullet, "Added three module-level helpers in `bench/harness.py`
  immediately before `extract_last_json` at line 1478"; (b) the
  (after iter 30) Halt-condition / quiet-signal status block
  preamble, "`select_json_envelope_actual` helper at
  `bench/harness.py:1478`"; (c) the same block's iter-30 quiet-
  signal counter bullet, repeating "`select_json_envelope_actual`
  helper at `bench/harness.py:1478`". The closest match for the
  selector's actual position is **1481** (off by 3); the iter-30
  author likely either pre-finalization-counted or transcribed
  from a draft. Per the iter-15 / iter-22 / iter-24 / iter-26 /
  iter-27 / iter-28 / iter-30 no-silent-edit discipline, the
  iter-30 entry body is preserved unchanged; the correction lives
  here. This is the same defect class as iter-22's "line 18" →
  "line 20" typo (clerical line-citation error in fresh prose,
  not upstream-edit-induced drift).
- **Substantive paired change:** updated `bench/RESULTS.md:72` —
  replaced the "F4 quarantine note (2026-04-26 iter 29)" paragraph
  with an "F4 closure note (2026-04-26 iter 30 + iter 31
  ratification)" paragraph that (i) cites the iter-30 closure
  entry and the iter-31 ratification, (ii) downgrades the T9 row's
  "F4-suspect" labeling (the T9 PI cell's `correct=true` is no
  longer F4-contaminated since the new selector preserves the
  jq-projection PASS shape via the `_json_top_keys` overlap rule),
  (iii) preserves the historical context that the iter-29 T16 PI
  bundle's recorded `correct=false` is the bundle's frozen truth
  (iter 30 explicitly does not retroactively re-score per its
  comparability framing), and (iv) keeps the T16 row out of the
  cross-executor table because adding a row that pairs the FAIL
  PI cell with a PASS OAI cell still mis-attributes the difference
  to executor / model when the actual post-iter-30 cause would be
  the bundle's frozen pre-fix scorer verdict (a future iteration
  may produce a fresh T16 PI bundle scored under the new selector
  to enable that row). The same paragraph also replaces the stale
  `bench/harness.py:1404-1429` reference (the pre-iter-30 inline
  loop range) with a pointer to the new
  `select_json_envelope_actual` helper and the 8 unit tests in
  `bench/test_harness_json.py:JsonEnvelopeActualSelectionTests`.
- **Cheap channel:** green before and after.
  - `cargo test -q`: 37 + 16 + 0 + (rest of integration suites)
    all pass.
  - `python3 -m unittest bench.test_command_policy bench.test_oai_loop
    bench.test_pi_audit bench.test_harness_json
    bench.test_harness_run_artifacts bench.test_harness_task_split
    bench.test_analyze_inputs bench.test_report_inputs` reports
    `Ran 78 tests in 1.650s ... OK`.
  - `python3 bench/harness.py --md-binary target/release/md`
    dry-run reports "All tasks pass dual scorer" on all 24 tasks.
- **Closure transition:** F4 moves from FIXED_PENDING_CONFIRMATION
  to **CLOSED**. Mechanically: the "## FIXED_PENDING_CONFIRMATION"
  section header was removed (it had only the F4 entry); the F4
  finding body is archived under "## CLOSED" alongside F1 / F2 /
  F3 / F3-a / L1 / P3 with the **Status** field updated from
  `FIXED_PENDING_CONFIRMATION` to `CLOSED (2026-04-26 iter 31
  review pass; FIXED 2026-04-26 iter 30 via closure plan option
  (a); filed 2026-04-26 iter 29; P1 metric-quarantine severity)`;
  the "## OPEN findings" parenthetical now points to the iter-31
  CLOSED entry and the iter-30 closure entry. The iter-30 entry's
  "## FIXED_PENDING_CONFIRMATION"-section reference and the
  (after iter 30) halt-condition block's "transitioned OPEN →
  FIXED_PENDING_CONFIRMATION" wording are preserved unchanged per
  the no-silent-edit discipline (the (after iter 31) block records
  the further transition).
- **Comparability framing:** this iteration does **not** modify
  any prior bundle in `bench/runs/`, does **not** bump
  `holdout_version` (still 1; F4 was P1, not P0), does **not**
  introduce a new product surface or change the agent's action
  space, does **not** re-score any prior bundle (the iter-29 T16
  bundle's recorded `correct=False` remains the bundle's frozen
  truth; the iter-31 replay through the new selector is a
  cheap-channel verification of the fix, not a new bundle), does
  **not** edit any historical ledger entry inline, does **not**
  extend the cross-executor same-task table in `bench/RESULTS.md`,
  does **not** change `README.md` / `CLAUDE.md` / `specs/**` /
  `bench/tasks/` / `bench/holdout/` / `bench/expected/`. The only
  external file edits are: (i) ledger transitions described above,
  (ii) `bench/RESULTS.md:72` paragraph replacement (no other lines
  in `RESULTS.md` change), and (iii) a downstream paragraph at
  `bench/RESULTS.md:74` is left unchanged (the P3 future-ratchet
  note remains relevant).
- **Closure-discipline status (iter 31's own meta-status):** this
  iter-31 entry is itself FIXED_PENDING_CONFIRMATION at authoring;
  the next iteration's review pass either explicitly ratifies by
  re-reading the iter-31 typed-artifact references (the four
  function lines at `bench/harness.py:1457`/`1469`/`1481`/`1529`,
  the test class at `bench/test_harness_json.py:135`, the
  78-test-count cheap-channel run, the iter-29 T16 bundle replay
  output) and the `bench/RESULTS.md:72` paragraph against this
  entry, or simply does not re-raise F4 (which would happen if
  iter 32 simply moves to another axis without reopening the
  json_envelope-text-answer defect).
- **Same-family-rule discharge:** iter 28 was oracle-trustworthiness
  (typed-test promotion), iter 29 was intervention-diversity
  (forced expensive-or-halt: T16 bundle that surfaced F4), iter
  30 was oracle-trustworthiness (F4 closure via schema-aware
  selector). Iter 31 is **closure-discipline ratification**
  (procedural axis) paired with **specification coherence**
  (the `bench/RESULTS.md:72` downgrade). Closure-discipline
  ratification iterations are structurally distinct from
  oracle-trustworthiness iterations (precedent: iters 3, 6, 12,
  13, 15, 20, 22, 24, 27 — each ratifies a prior iteration's
  typed-artifact claims without authoring a new fix). The
  fresh-failing-trace escape clause additionally applies: the
  iter-30 line-1478 typo is the trace, surfaced by independent
  re-verification of the iter-30 entry's line citations against
  the live file. Per spec, this is admissible same-axis follow-up
  to an oracle-trustworthiness chain because the trace is real
  and is captured forward-pointing in this entry rather than
  silently corrected.
- **What this does NOT do:** does not promote any new product
  anchor (none declared); does not run the expensive outer
  channel (the iter-31 replay is a typed-artifact re-read, not
  an expensive run); does not re-score any prior bundle; does
  not amend any prior pass-rate claim; does not invalidate any
  cross-executor table row (the rule and all five existing rows
  remain valid); does not extend the cross-executor table with
  a new row; does not introduce a new finding (F4 transition is
  closure of an existing finding, not a new one); does not bump
  `holdout_version`.

### F4 closure: schema-aware json_envelope actual selection (2026-04-26 iter 30)

Discharged the F4 P1 OPEN finding from iter 29 by implementing closure
plan option (a) — schema-aware tool-output preference. The
`expected_artifact == "json_envelope"` branch of the scorer at
`bench/harness.py:1404-1429` no longer captures the first non-empty
parseable JSON tool output unconditionally; instead it walks tool
outputs, prefers those whose parsed top-level shape's keys overlap
with the expected JSON's top-level shape, and only falls through to
text outputs (and then to non-shape-matching tool outputs as a final
fallback) when no shape-matching tool output exists. Iter-29's T16
mdtools FAIL is recovered: replay through the new selector against
`bench/runs/checkpoint-pi-T16-mdtools-gpt5.4mini-2026-04-26/logs/T16_mdtools_1777224275/agent_output.txt`
(parsed via `bench/pi_runner.parse_pi_json_output`) selects the
agent's text answer (top keys `["files", "total_pending"]`) over the
intermediate `./md tasks <files> --json` envelope (top keys
`["results", "schema_version"]`) and `score_task` returns
`md=PASS neutral=PASS` with `json_canonical: OK`.

- **Disturbed axis:** oracle trustworthiness — F4 was a typed scorer
  false-negative on the `json_envelope` + `json_canonical=true` cell
  shape (T9 / T11 / T16 / T19) when the agent answers in assistant
  text rather than via a projecting tool call (e.g. `jq`).
- **Frontier anchor:** the **F4 P1 OPEN finding** filed in
  `bench/ledger.md` at iter 29, anchored to
  `bench/runs/checkpoint-pi-T16-mdtools-gpt5.4mini-2026-04-26/`'s
  `correct=False` and the `diff_report` showing
  `expected: "files": [` vs `actual: "results": [`. Per spec, OPEN
  findings are valid frontier anchors; per the spec's
  "what counts as hardening while a P0 or P1 is open" list, harness
  bug fixes to existing scorer code are admissible (this change
  touches only `bench/harness.py`'s scorer extraction layer; no new
  CLOSED CLI command, no new edit-plan schema, no scorer-coupled
  product behavior, no change to the agent's action space).
- **Change shape:**
  - Added three module-level helpers in `bench/harness.py` immediately
    before `extract_last_json` at line 1478:
    - `_json_top_keys(parsed)` returns the top-level key set for a
      parsed JSON value (a dict's keys, or the first-element keys for
      a non-empty list of dicts; empty set otherwise).
    - `_expected_json_top_keys(expected_str)` returns the top-level
      key set of the expected JSON, or `None` when no observable
      shape is available (the caller then reverts to the pre-F4
      "first non-empty JSON wins" rule).
    - `select_json_envelope_actual(all_tool_outputs, all_text_outputs,
      stdout, expected_str)` is the new entry point that the scorer
      branch calls. It walks `reversed(all_tool_outputs)` preferring
      shape-matching outputs; if none match, falls through to text
      outputs (extracting JSON via `extract_last_json`); if neither
      matches, returns the most-recent non-shape-matching tool output
      (preserving the pre-F4 fallback) or finally
      `extract_last_json(stdout)`.
  - Replaced the inline 26-line loop at the `json_envelope` branch
    of `run_agent` with a single call to
    `select_json_envelope_actual(all_tool_outputs, all_text_outputs,
    stdout, expected_content)`. The behavioral change versus the
    pre-iter-30 inline code is exactly one rule: when a parseable
    tool output's top-level keys disjoint with the expected JSON's
    top-level keys, the loop continues searching rather than
    returning. All other paths (empty parses, non-list/non-dict,
    text fallback when no tool parses, final stdout fallback) match
    the prior behavior bit-exact.
  - Exported `select_json_envelope_actual` (no leading underscore)
    from `bench.harness` so the test module can pin its behavior in
    isolation. The two key-set helpers remain underscore-prefixed
    (internal).
- **Tests added (8 new):**
  Added `bench/test_harness_json.py:JsonEnvelopeActualSelectionTests`
  with 8 tests covering the F4 closure scenarios:
  - `test_t16_shape_text_wins_over_intermediate_md_tasks_envelope` —
    iter-29 trace shape: `{"schema_version":..., "results":[...]}`
    tool output + correct `{"total_pending":..., "files":[...]}` text
    → text wins.
  - `test_t9_shape_jq_projected_tool_output_wins` — iter-25 trace
    shape: raw `md tasks` envelope + jq-projected list-of-dicts tool
    output → jq projection wins (top-element keys overlap).
  - `test_t1_shape_matching_tool_output_wins_over_text_noise` — T1
    case: `md outline --json` output already matches expected shape
    even with text noise present.
  - `test_t22_top_level_list_tool_output_falls_through_to_fallback` —
    F3-closure case: top-level list tool output with no key overlap
    against expected dict, no text answer → list surfaces via
    fallback so F3-closure scorer can wrap it.
  - `test_text_only_answer_works` — agent with no JSON tool outputs
    but a text answer.
  - `test_no_shape_match_falls_back_to_most_recent_tool_output` —
    pre-F4 fallback: no shape-matching tool, no text candidate, the
    most-recent (reversed) tool output is returned so a meaningful
    MISMATCH can still be reported.
  - `test_unknown_expected_shape_preserves_first_tool_output_rule` —
    expected has no observable key shape (list of strings) → revert
    to pre-F4 "first non-empty JSON wins" behavior.
  - `test_empty_outputs_falls_back_to_extract_last_json_of_stdout` —
    final fallback path.
  Total python unittest count across the 8 spec-named modules rose
  from 70 (post-iter-28) to **78**.
- **Cheap channel:** green before and after.
  - `cargo test -q` all suites pass: 37, 16, 0, plus the rest of the
    integration suites (matching the "282 integration tests" claim).
  - `python3 -m unittest bench.test_command_policy bench.test_oai_loop
    bench.test_pi_audit bench.test_harness_json
    bench.test_harness_run_artifacts bench.test_harness_task_split
    bench.test_analyze_inputs bench.test_report_inputs` reports
    "Ran 78 tests in 1.860s … OK" (was 70 before iter 30; +8 from
    `JsonEnvelopeActualSelectionTests`).
  - `python3 bench/harness.py --md-binary target/release/md` dry-run
    reports "All tasks pass dual scorer" on all 24 tasks; T1, T5,
    T9, T11, T16, T19, T21, T22 (the eight `expected_artifact ==
    "json_envelope"` tasks) all `md=PASS neutral=PASS` after the
    selector change. The dry-run feeds `expected_content` directly
    as `actual` (not the new selector path), so this verifies the
    underlying `score_task` continues to work; the new selector is
    separately exercised by the cheap-channel unit tests.
- **Replay verification (typed-artifact attribution probe):** the
  iter-29 T16 PI bundle's `agent_output.txt` was parsed via
  `bench/pi_runner.parse_pi_json_output` (the PI runner's JSONL
  format, not the Claude CLI JSON-array format that
  `parse_agent_output` expects). The trace contains 4 tool outputs
  (3 non-JSON: 2 guard denials and 1 directory error from
  `./md blocks <vault>`; 1 JSON: the `./md tasks --json` envelope
  with top keys `["results", "schema_version"]`) and 1 text output
  (the agent's correct answer with top keys `["files",
  "total_pending"]`). With the iter-30 `select_json_envelope_actual`
  helper, the selector skips the `["results", "schema_version"]`
  envelope (intersection with expected `["files", "total_pending"]`
  is empty), falls through to the text output, and `score_task`
  reports `md=PASS neutral=PASS` with `json_canonical: OK`. The
  iter-29 `correct=False` would now be `correct=True`. This is the
  attribution probe the spec requires for product-class
  interventions: the named failure class (F4 — false-negative on
  json_envelope text-answer cells) actually moved, not just a
  pass-rate observation.
- **Comparability framing:** this change does **not** modify any
  prior bundle in `bench/runs/` (those record what their original
  scorer pass produced; iter 30 does not retroactively re-score).
  It does **not** bump `holdout_version` (still 1; F4 was P1, not
  P0 — no holdout-side artifact was edited; the F4 closure plan
  explicitly noted this in the iter-29 OPEN entry under "What F4
  does NOT block"). It does **not** introduce a new product
  surface, change the agent's action space, or add a new CLI
  command. It does **not** edit any historical ledger entry inline
  (per iter-15 / iter-22 / iter-24 / iter-26 / iter-27 / iter-28
  discipline) — the iter-29 F4 OPEN entry's body is preserved
  verbatim, only its "## OPEN findings" sectional placement and
  Status field were updated to reflect the FIXED_PENDING_CONFIRMATION
  transition. It does **not** invalidate any prior search-set
  observation or extend the cross-executor table (F4 closure
  enables a future iteration to add a T16 row, but the iter-29
  bundle's recorded `correct=False` remains the bundle's frozen
  truth; future iterations would either re-run the cell under the
  new scorer or note the F4-closure-replay PASS as a separate
  cell).
- **What this does NOT do:** does not promote any new product
  anchor (none declared). Does not run the expensive outer
  channel — F4 closure only required cheap-channel verification
  (replay against the durable iter-29 bundle is itself a typed
  artifact, not an expensive run). Does not extend
  `bench/RESULTS.md`'s cross-executor table (the F4 quarantine
  paragraph there still applies until the closure is independently
  ratified at iter 31; the F4-suspect labeling on the T9 row may
  be downgraded then). Does not amend any prior pass-rate claim.
  Does not edit `README.md`, `CLAUDE.md`, `specs/**`, any task
  file under `bench/tasks/` or `bench/holdout/`, or any expected
  output file under `bench/expected/`. Does not add a new scorer
  hint to the per-task `scorer` policy (closure plan option (c)
  was rejected in favor of option (a) per the iter-29 entry's
  pre-commitment language: "option (a) is the leading candidate
  because it generalizes (no per-task config) and degrades
  gracefully (current behavior is the final fallback)").
- **Closure-discipline status:** FIXED_PENDING_CONFIRMATION at
  authoring. Per the FIXED ≠ CLOSED rule, closure requires either
  the next iteration's review pass to explicitly confirm by
  re-reading the iter-30 typed artifacts (the new helper functions,
  the test class, the iter-29 bundle replay output) and the
  modified call site against this entry, or the next pass to not
  re-raise F4 (which would happen if iter 31 simply moves to
  another axis without reopening the json_envelope-text-answer
  defect). At the next-iteration review pass, F4's body should
  move from "## FIXED_PENDING_CONFIRMATION" to a nested CLOSED
  reference within this entry, and the "## OPEN findings" note
  about F4 should be archived to "## CLOSED" historical context.
- **Same-family-rule discharge:** iter 28 was oracle-trustworthiness
  (typed-test promotion of iter-27's prose claim — ScorerDispatcher
  BranchTests), iter 29 was intervention-diversity (forced
  expensive-or-halt: T16 mdtools PI bundle that surfaced F4). Iter
  30 is **oracle-trustworthiness** (closing F4 via schema-aware
  selector). This is two oracle-trustworthiness moves with one
  intervention-diversity move between them; not a same-family
  concentration because iter 29 broke the chain. The
  fresh-failing-trace escape clause additionally applies: F4 itself
  is the trace, filed in iter 29 with a typed bundle pointer
  (`checkpoint-pi-T16-mdtools-gpt5.4mini-2026-04-26/results.json`'s
  `correct=False` and `diff_report`). Per the spec's "Required
  probes for 'transactional multi-edit' candidates" — N/A here, this
  is a scorer fix not a product anchor; per the spec's "Attribution
  requirement (new in T7)" the named failure class (F4) moved,
  verified by replay through the new selector returning PASS on the
  iter-29 bundle's actual outputs.



### Quiet-signal checkpoint discharge (2026-04-26 iter 29)

Per the spec's "After 3 consecutive iterations with the cheap channel
green, no new failing trace, and no new finding added to the findings /
ledger surface, run the expensive outer channel" rule, iter 29 ran the
expensive outer channel. The quiet-signal counter was at 3 after iter 28
(iters 26 / 27 / 28 were all quiet). Cheap channel re-verified green
before and after: `cargo test -q` all suites pass; `python3 -m unittest
bench.test_command_policy bench.test_oai_loop bench.test_pi_audit
bench.test_harness_json bench.test_harness_run_artifacts
bench.test_harness_task_split bench.test_analyze_inputs
bench.test_report_inputs` reports "Ran 70 tests in 1.829s … OK";
`python3 bench/harness.py --md-binary target/release/md` dry-run
reports "All tasks pass dual scorer" on all 24 tasks.

- **Bundle:** `bench/runs/checkpoint-pi-T16-mdtools-gpt5.4mini-2026-04-26/` —
  eighth PI runner bundle. Single task (T16, search-split, multi-file
  GFM-task counting). Single mode (mdtools). Single run. Model
  `openai-codex/gpt-5.4-mini` at `thinking_level=minimal`, recorded per-
  result and per-run on the metadata bundle. `run.json` line 20 carries
  `holdout_version: 1` — the **fourth** durable bundle in `bench/runs/`
  carrying iter-17's stamp (after iter-18 T2, iter-21 T21, iter-25 T9).
- **Verdict:** T16 mdtools dual-scorer **FAIL** in 18.71s with 4 tool
  calls (`find … -name '*.md'`, `ls -1 <vault>`, `./md blocks <vault>`
  → `tool_error: Is a directory`, `./md tasks <files> --json` → 4061
  bytes), 0 mutations, `requeried=false`, `policy_violations=2`,
  `bytes_observation=4430`, `bytes_output=1,900,731` (PI streaming
  overhead, see P3 cross-executor rule in `bench/RESULTS.md`).
  `diff_report` records two distinct artifacts: a `runner_error`
  surfaced from the `./md blocks <directory>` mis-call (the agent
  passed a directory where md expects a single `.md` file; md returned
  a non-zero exit with a clear "Is a directory" message and the agent
  recovered with `./md tasks <files> --json` on the next call) and a
  `json_canonical: MISMATCH at line 2` showing
  `expected: "files": [` vs `actual: "results": [`. Pi-audit log at
  `logs/T16_mdtools_1777224275/pi-audit.jsonl` preserves 10 events
  (`model_change`, `thinking_level_change`, then four
  `tool_call`/`tool_result` pairs with one `tool_error` for the third
  call), parses cleanly via `bench/pi_audit_adapter.summarize_pi_audit_events`.
- **Comparability framing:** this is the first cell for (PI runner,
  gpt-5.4-mini, mdtools, minimal thinking, T16, runs_per_task=1,
  holdout_version=1). It is **NOT** a reconfirmation of any prior
  holdout bundle — T16 is in the search split, not holdout. It is
  **NOT** a mode/model comparison versus the existing OAI bundles for
  T16 (`search-mdtools-extraction-{Hermes-4-70B-4bit,Qwen3.5-122B-A10B-4bit,
  Qwen3.5-27B-4bit,magnum-v4-123b-4bit}-2026-04-21/`): all four cross
  the executor and model normalization axes versus this bundle, so
  any pass-rate observation is search-set, not a comparison. Cross-
  executor downstream pairing (PI gpt-5.4-mini vs OAI Qwen3.5-122B-A10B-4bit)
  is **not** eligible for the `bench/RESULTS.md:54` cross-executor table
  in this state — F4 (filed below) means the PI cell's `correct=False`
  is contaminated by the scorer false-negative; the OAI cell's
  `correct=True` is genuine (Qwen used 6 tool calls, likely projecting
  in-tool the way iter-25 T9 did with jq), so a row that pairs them
  would mis-attribute the difference to executor/model when the actual
  cause is the F4 scorer pattern. Tabling is deferred until F4 closes.
  Per second-opinion review on iter 29, an **F4 quarantine note**
  paragraph has been added to the cross-executor section of
  `bench/RESULTS.md` (immediately after the existing **Rule:**
  paragraph) explicitly flagging that the T9 row in the published
  table sits in the F4-affected scorer cell shape — its behavior-axis
  values (`tool_calls`, `mutations`, `bytes_output`,
  `bytes_observation`) remain valid (those measure agent activity, not
  scorer verdicts), but the `correct` / `correct_neutral` channel for
  any comparison in this scorer cell shape (T9 / T11 / T16 / T19) is
  **F4-suspect** until F4 closes. This is the affected-claim-audit
  move (versus the affected-task-audit move) the second opinion called
  out as load-bearing for keeping iter-29 shipping discipline above
  status-theater.
- **What this discharges:** the spec's quiet-signal-checkpoint rule
  by introducing fresh typed signal via the expensive channel. It does
  **NOT** discharge any product or oracle claim — those still require
  their own attribution probes and apples-to-apples comparisons.
- **What it surfaced:** the new **F4 P1 OPEN finding** above (json_envelope
  scorer prefers any JSON-parseable tool output over the agent's matching
  text answer). This is concrete fresh signal — i.e. the iteration's
  expensive run did not return null information. F4 is the first OPEN
  finding filed since iter-2 closed L1; the 24-round zero-OPEN streak
  ends at iter 29.
- **Cross-finding observation (forward-pointing, no historical edit):** the
  T16 trace also exhibits a per-model behavioral pattern worth recording
  but **not** worth filing as a separate finding: gpt-5.4-mini at minimal
  thinking attempted `./md blocks <directory>` as the third tool call,
  which `md` correctly rejects (md commands operate on files, not
  directories). The agent did not bail on the error and instead pivoted
  to `./md tasks <files> --json` on the fourth call, exiting with the
  correct text answer. This is a per-model-and-prompt-shape observation
  parallel to iter-14's gpt-5.4-mini-emits-2.5×-tool-calls observation
  on T18, recorded as data not as a defect: md's directory rejection
  is correct UX (clear error message, non-zero exit), and the agent's
  recovery worked. No finding is opened by this trace.
  **Affordance-trap nuance (added 2026-04-26 iter 29 post-second-opinion
  refinement):** the directory mis-call is not pure noise. F4's symptom
  shape depends on the recovery path leaving a tempting non-matching
  JSON tool output behind — i.e. `./md tasks <files> --json` was
  *reached* because `./md blocks <dir>` failed and the agent pivoted
  to the per-file `tasks --json` over the multi-file vault. The
  scorer-extraction rule then captured that recovery-path output. So
  although the dominant cause of FAIL is the F4 scorer rule (not the
  affordance confusion), the trajectory shows that vault-shaped tasks
  exposing structural primitives that don't accept directories can
  produce recovery paths whose tool outputs systematically mis-match
  the expected schema. Recorded forward-pointing as cofactor evidence
  for the future iteration that closes F4 — synthetic regression
  fixtures should explicitly include this shape (wrong-schema JSON
  tool output present + correct final JSON in assistant text).
- **Same-family-rule discharge:** iter 26 was specification coherence
  (cross-executor table fifth-row publication of iter-25 T9), iter 27
  was closure-discipline (ledger-only ratification of iter 26 +
  forward-pointing code-path-routing correction in iter-25 prose), iter
  28 was oracle-trustworthiness hardening (typed-test promotion of
  iter-27's prose claim). Iter 29 is **intervention-diversity**
  (expensive outer channel run + new durable PI bundle), shifting axis
  cleanly from iter 28. The forced expensive-or-halt mandate at iter 29
  (per the spec's 3-consecutive-quiet rule) is its own escape clause
  for the same-family rule, parallel in shape to iter 25's discharge of
  iter-22 / -23 / -24 same-family pressure and iter 14's discharge of
  iter-11 / -12 / -13 same-family pressure.
- **Closure-discipline status:** **CLOSED** at authoring time per the
  iter-4 / -7 / -10 / -14 / -18 / -21 / -25 quiet-signal-discharge
  pattern (no FIXED_PENDING_CONFIRMATION promotion needed because
  there is no fix here — the bundle is the deliverable; F4 is a
  separately-filed OPEN finding that future iterations close on their
  own ratchet). A future review pass should ratify by re-reading every
  data point in this entry against `results.json`, `run.json`,
  `pi-audit.jsonl`, and the persisted `agent_output.txt`.
- **What this does NOT do:** does not promote any product anchor
  (`bench/probes/anchor-validation/` still does not exist). Does not
  bump `holdout_version` (still 1; T16 is search-side, no holdout-
  side artifact change). Does not edit any harness production code
  (only ledger and a new bundle directory under `bench/runs/`). Does
  not extend the cross-executor table (deferred until F4 closes).
  Does not modify any historical ledger entry inline (per
  iter-15 / -22 / -24 / -26 / -27 / -28 discipline). Does not edit any
  published-narrative file (`bench/RESULTS.md`, `README.md`,
  `CLAUDE.md`, `bench/retracted_2026-04-24/README.md`, `specs/**`).
  Does not amend any pass-rate claim. Does not extend `bench/probes/`,
  `bench/search/candidates/`, or any other not-yet-existing T7 directory.
  Does not implement the F4 closure plan (deferred to a future iteration
  that explicitly chooses among options (a) / (b) / (c) above).

### Scorer-dispatcher branch coverage assertion (2026-04-26 iter 28)

Promote iter-27's forward-pointing observation about the corpus-vacuous
`bench/harness.py:378` else-arm (`ok = a.strip() == e.strip()`) from
ledger prose to a mechanical cheap-channel assertion. The iter-27 entry
verified by manual re-reading that every task in `bench/tasks/tasks.json`
routes to one of four explicit `score_task` dispatcher branches at lines
340 (raw_bytes), 363 (structural + json_canonical), 367 (structural +
json_envelope), or 371 (normalized_text); iter 28 makes that property
typed by adding a test class that mirrors the dispatcher's predicate
order and asserts no corpus task reaches the else-arm.

- **Disturbed axis:** oracle trustworthiness — typed-artifact gap. Per
  the iteration-protocol step-7 cheap-channel-then-expensive sequencing,
  properties asserted only in ledger prose are weaker evidence than
  properties asserted by harness tests. The "no task reaches line 378"
  property is a real invariant of the corpus×dispatcher interaction:
  if a future task is added with `scorer.kind="structural"` and neither
  `json_canonical` truthy nor `expected_artifact="json_envelope"`, the
  scorer would silently route through an unvalidated string-equality
  fallback that no scorer-correctness audit has ever exercised. The
  cheap channel does not currently surface that drift.
- **Frontier anchor:** *fresh failing trace surfaced by iter 27 +
  missing evaluator artifact (mechanical assertion of corpus×dispatcher
  coverage)*. iter-27's "no task in the current corpus reaches line
  378's else-arm" is the trace; the missing artifact is a typed test
  that fires if that property is ever violated. Same shape as iter
  17's "the four pre-iter-17 PI bundles all lack the `holdout_version`
  field that the spec explicitly requires" — a property that lives in
  the typed-artifact graph and is therefore promotable from prose to
  a mechanical assertion. Per the spec's allowed-during-P0/P1
  hardening list, "harness assertions" are admissible; this change is
  exactly that.
- **Change shape:**
  - Added `bench/test_harness_task_split.py:ScorerDispatcherBranchTests`
    (2 tests). The class carries a `_classify(task)` helper that
    mirrors the dispatcher's predicate order at `bench/harness.py:340–378`
    exactly: raw_bytes early-return first; among `kind=="structural"`
    the json_canonical branch wins over the json_envelope branch.
    `_classify` returns one of `raw_bytes` / `json_canonical` /
    `structural_json` / `normalized_text` / `else_fallback`.
    - `test_no_corpus_task_reaches_else_fallback`: iterates every
      task in `bench/tasks/tasks.json`, asserts none classifies as
      `else_fallback`. The error message names the offending task's
      `id` / `kind` / `expected_artifact` / `json_canonical` and
      points at `bench/harness.py:378`, so any future drift fires
      with a self-describing failure.
    - `test_corpus_exercises_all_four_explicit_branches`: asserts
      each of the four explicit branches is hit by at least one
      corpus task — sanity that no branch becomes dead-code without
      the test catching it. Currently raw_bytes (T10/T12/T13/T14/T15
      /T17/T18/T20/T23/T24, 10 tasks), json_canonical (T9/T11/T16/T19,
      4 tasks), structural_json (T1/T5/T21/T22, 4 tasks), and
      normalized_text (T2/T3/T4/T6/T7/T8, 6 tasks) all fire.
  - Added `BenchTask` to the existing `bench.harness` import list at
    `bench/test_harness_task_split.py:8` so the helper's type hint is
    importable. No production code touched.
- **Tests added (2 new):**
  test count for `bench/test_harness_task_split.py` rose from 11 to 13;
  total python unittest count across the 8 spec-named modules rose from
  68 (post-iter-17) to **70**. Subprocess-verified the negative case
  by constructing a synthetic `BenchTask` with
  `scorer.kind="structural"`, `json_canonical=False`, and
  `expected_artifact="file_contents"` (the corpus-impossible
  combination): `_classify` returns `else_fallback` as expected, so
  the assertion would fire on a real corpus drift of that shape.
- **Cheap channel:** green before and after.
  - `cargo test -q` all suites pass: non-zero counts 36, 13, 37, 32,
    37, 12, 7, 24, 32, 37, 16 across the 11 lib + integration suites
    (~280 total, matching the "282 integration tests" claim in
    `CLAUDE.md`).
  - `python3 -m unittest bench.test_command_policy bench.test_oai_loop
    bench.test_pi_audit bench.test_harness_json
    bench.test_harness_run_artifacts bench.test_harness_task_split
    bench.test_analyze_inputs bench.test_report_inputs` reports
    "Ran 70 tests in 1.623s … OK" (was 68 before iter 28; +2 from
    `ScorerDispatcherBranchTests`).
  - `python3 bench/harness.py --md-binary target/release/md` dry-run
    reports "All tasks pass dual scorer" on all 24 tasks
    (`md=PASS neutral=PASS` for T1–T24, with `json_canonical`,
    `frontmatter_json`, and `link_destinations` scorer branches all
    OK on the relevant tasks).
- **Comparability framing:** this is a typed-test-from-prose
  promotion. It does **not** run the expensive outer channel. It does
  **not** bump `holdout_version` (still 1; no holdout repair
  occurred). It does **not** modify any bundle, any harness production
  code (only test code added; no edit to `bench/harness.py`), any
  scorer, or any task description. It does **not** introduce a new
  product surface or change the agent's action space. It does **not**
  amend any pass-rate claim or extend the cross-executor table. It
  does **not** edit any historical ledger entry inline (per
  iter-15 / iter-22 / iter-24 / iter-26 / iter-27 discipline). It
  does **not** edit any published-narrative file (`bench/RESULTS.md`,
  `README.md`, `CLAUDE.md`, `bench/retracted_2026-04-24/README.md`,
  `specs/**`). The only files modified are
  `bench/test_harness_task_split.py` (one-line import addition + new
  test class at the bottom) and `bench/ledger.md` itself (this
  iter-28 entry plus an updated halt-condition / quiet-signal status
  block).
- **Closure-discipline status:** FIXED_PENDING_CONFIRMATION at
  authoring time. A future review pass should ratify by re-reading
  the new `ScorerDispatcherBranchTests` class, re-running
  `python3 -m unittest bench.test_harness_task_split.ScorerDispatcherBranchTests
  -v` to confirm the 2 tests pass on the live corpus, and re-checking
  the dispatcher predicate order at `bench/harness.py:340-378`
  against the `_classify` helper. Per iter-15's labeling discipline,
  this is **frontier expansion** in the evaluator-trustworthiness
  sense: a previously prose-only invariant is now mechanical, so
  future corpus drift surfaces in the cheap channel rather than only
  via manual re-reading.
- **Same-family-rule discharge:** iter 25 was intervention-diversity
  (T9 PI expensive run), iter 26 was specification coherence
  (cross-executor table fifth-row publication), iter 27 was
  closure-discipline (ledger-only ratification of iter 26 + one
  forward-pointing code-path-routing correction). Iter 28 is
  oracle-trustworthiness hardening (typed-test promotion of iter 27's
  prose claim), shifting axis cleanly from iter 27. Parallel in shape
  to iter 17's "promote spec requirement to mechanical assertion"
  move (also after a chain of cash-out / ratification iterations) and
  to iter 16's "promote procedural protection to runtime mechanical"
  move. The fresh-failing-trace escape clause additionally applies
  via iter-27's specific corpus-vacuous-path observation; the trace
  is durable (the iter-25 line-378 prose claim is preserved in the
  ledger per iter-15 discipline, and the corpus-vacuous property
  is the underlying invariant).
- **What this does NOT do:** does not promote any product anchor
  (`bench/probes/anchor-validation/` still does not exist, no
  candidate primitive validated). Does not bump `holdout_version`
  (still 1). Does not run the expensive outer channel. Does not edit
  any historical ledger entry inline. Does not amend any pass-rate
  claim. Does not modify any bundle, any harness production code, any
  scorer, or any task description. Does not extend the cross-executor
  table. Does not touch any holdout artifact. Does not extend
  `bench/probes/`, `bench/search/candidates/`, or any other
  not-yet-existing T7 directory. Does not edit `bench/harness.py`
  itself; only test code is added (a `bench/test_harness_*.py` file).
- **Forward observation (no historical edit):** the symmetric question
  raised by adding this test is "should the line-378 fallback be
  pruned, since it is corpus-vacuous?" The conservative answer is no:
  the fallback is defensive code that catches future task shapes the
  dispatcher has not been extended to handle, and pruning it would
  exchange a string-equality fallback for a `KeyError`-shaped crash
  on the offending task. With this iter-28 test in place, the cheap
  channel surfaces the drift before any expensive run hits the
  fallback, so the defensive code's cost (vacuous lines) is bounded
  by the test's assurance. Recording this rationale forward-pointing
  rather than acting on it.

### Confirmation review pass (2026-04-26 iter 27)

Closure-discipline review of iter-26's substantive `bench/RESULTS.md`
fifth-row publication (the additive T9 row in the cross-executor
same-task table, parallel in shape to iter-19's T18 row addition and
iter-11's original 3-row publication). Parallels iter 24's review of
iter 23, iter 22's review of iter 21, iter 20's review of iter 19, and
iter 12/13's reviews of iter 11/12: typed-artifact claims in the
iter-26 entry are checked against the underlying bundle files, the
published-narrative additions at `bench/RESULTS.md:56` / `:62` /
`:66` / `:68`, and the cross-executor ratio computations. The
verification surfaces a fresh failing trace inside the **iter-25**
entry's own prose (a code-path claim that iter 26's ratification
verified abstractly but did not specifically dispute), parallel in
shape to iter 26's own forward-pointing scan-methodology correction
of iter 25.

Cheap channel green at review time: `cargo test -q` all suites pass
(non-zero counts: 36, 13, 37, 32, 37, 12, 7, 24, 32, 37, 16 across the
13 lib + integration test binaries — total ~280, matching the
"282 integration tests" claim in `CLAUDE.md`); 68 python unittests OK
across the 8 spec-named modules; `bench/harness.py --md-binary
target/release/md` dry-run all 24 tasks PASS dual-scorer (`md=PASS
neutral=PASS` for T1–T24, with `json_canonical: OK` on T9 and the
other relevant scorer branches OK on their tasks).

What was checked:

- **Cross-executor table values, all five rows** — re-read each row
  against the underlying bundle `results.json` files (T1, T7, T9, T22,
  T18 — both PI and OAI sides, 10 cells total). Every `bytes_output`,
  `bytes_observation`, `tool_calls`, and `mutations` value in
  `bench/RESULTS.md:60–64` reproduces bit-exact. Computed ratios
  reproduce: T1 `5,975,843 / 2,702 = 2,212.0` → ~2,212×;
  T7 `1,172,040 / 699 = 1,676.7` → ~1,677×;
  T9 `3,177,953 / 2,169 = 1,465.2` → ~1,465×;
  T22 `671,515 / 488 = 1,375.6` → ~1,376×;
  T18 `844,124 / 812 = 1,039.6` → ~1,040×.
  All match the table's published ratios.
- **iter-26 T9 cell data points** —
  `bench/runs/checkpoint-pi-T9-mdtools-gpt5.4mini-2026-04-26/results.json`
  re-read: `task_id=T9`, `mode=mdtools`, `correct=true`,
  `correct_neutral=true`, `tool_calls=2`, `mutations=0`,
  `bytes_observation=6675`, `bytes_output=3177953`,
  `elapsed_seconds=14.39`, `diff_report="json_canonical: OK"`,
  `model="openai-codex/gpt-5.4-mini"`, `thinking_level="minimal"`,
  `requeried=false`, `policy_violations=0`. All iter-26 typed-artifact
  citations reproduce bit-exact.
- **iter-26 T9 OAI cell data points** —
  `bench/runs/search-mdtools-extraction-Qwen3.5-122B-A10B-4bit-2026-04-21/results.json`
  re-read: T9 mdtools cell carries `bytes_output=2169`,
  `bytes_observation=7100`, `tool_calls=2`, `mutations=0`,
  `correct=true`, `correct_neutral=true`,
  `model="Qwen3.5-122B-A10B-4bit"`. The bundle's results.json array
  contains T1, T9, T16 mdtools cells (verified by enumerating
  `task_id` values), confirming iter-26's "the same OAI bundle file
  carries both T1 and T9 cells in its `results.json` array" claim.
- **`bytes_observation` delta computation for T9** —
  `|6675 - 7100| = 425`; `425 / 7100 = 5.99%` → ~6%. iter-26's "the
  T9 row tightens this to ~6%" claim reproduces. The bytes_observation
  rule's matched-tool-call-count subset is now: T1 ~7%, T7 ~19%,
  T9 ~6% (the smallest of the three).
- **iter-26 T9 run.json line-20 stamp** — `holdout_version: 1` confirmed
  on line 20 of
  `bench/runs/checkpoint-pi-T9-mdtools-gpt5.4mini-2026-04-26/run.json`.
  Cross-checked against T2 (iter 18 — first stamp) and T21
  (iter 21 — second stamp): both also carry `holdout_version: 1` on
  line 20. T9 is the **third** durable bundle with the iter-17 stamp;
  the first four PI bundles (T1 iter 4, T22 iter 7, T7 iter 10, T18
  iter 14) have `holdout_version: None` because they predate iter 17's
  stamping wiring. iter-26's "third durable bundle" claim reproduces.
- **iter-26 pi-audit.jsonl event histogram** —
  `bench/runs/checkpoint-pi-T9-mdtools-gpt5.4mini-2026-04-26/logs/T9_mdtools_1777221491/pi-audit.jsonl`
  has exactly 6 lines. Event-type histogram via `event` field:
  `{model_change: 1, thinking_level_change: 1, tool_call: 2, tool_result: 2}`.
  The two `tool_call` events carry
  `input.command="./md tasks <temp> --status pending --json"` and
  `input.command="./md tasks <temp> --status pending --json | jq '[.results[0].tasks[] | {loc, nearest_heading, summary_text}]'"`.
  The two `tool_result` events have `outputBytes=4686` and `outputBytes=1989`,
  summing to `bytes_observation=6675`. iter-26's claim reproduces.
- **iter-26 PiAuditCounters output** — re-ran
  `bench.pi_audit_adapter.summarize_pi_audit_events` against the live
  events. Output: `tool_calls=2, tool_results=2, tool_errors=0,
  bytes_observation=6675, blocked=0, policy_violations=0, mutations=0,
  requeried=False, model='openai-codex/gpt-5.4-mini',
  thinking_level='minimal', bash_commands=[<two commands>]`. iter-26's
  claim reproduces bit-exact.
- **iter-26 published-narrative line-position citations** — re-read
  `bench/RESULTS.md:54` (executor description with `:1282` / `:1318`
  bytes_output citations), `:56` (caption "(2026-04-26 iters 11, 19,
  26)" + "Five `mdtools` cells"), `:58` (table header), `:60–64`
  (table body, T1/T7/T9/T22/T18 rows), `:66` (commentary "all five
  pairs" + "T1, T7, and T9" + "(the T9 row tightens this to ~6%)"),
  `:68` (PI/OAI bundle pointers list with iter-25 T9 entry, T2 / T21
  not-yet-eligible parentheticals preserved). All line positions cited
  by iter 26 are accurate.
- **iter-26 corpus-vacuous-path verification** — re-ran the corpus
  scan: no task in `bench/tasks/tasks.json` has
  `compare_block_order=true` OR `compare_block_text=true` under
  `kind=structural`; every task with those flags uses
  `kind=normalized_text` (T2, T3, T4, T6, T7, T8). iter-25's forward-
  pointing observation about the iter-21 framing being corpus-vacuous,
  carried into iter-26's ratification, reproduces.
- **iter-26 forward-pointing OAI-T9 scan correction** — re-ran
  `grep -l '"task_id": "T9"' bench/runs/*/results.json`: returns 11
  bundles (the iter-25 PI checkpoint, the dry-run, and 9 search-* /
  search-mdtools-* / search-hybrid-* / search-unix-* OAI bundles
  including the four `search-mdtools-extraction-*` bundles that
  iter-26 specifically named). The OAI T9 mdtools cell at
  `search-mdtools-extraction-Qwen3.5-122B-A10B-4bit-2026-04-21/`
  carries `task_id=T9`, `mode=mdtools`, predating iter 25 by five
  days (file mtime 2026-04-21 per the directory name suffix).
  iter-26's correction stands: iter 25's directory-name scan missed
  the multi-task aggregation pattern; iter 26's `grep -l '"task_id":
  "TX"' bench/runs/*/results.json` is the correct discovery query.

Forward-pointing correction surfaced during the verification (per
iter-15 / iter-22 / iter-24 / iter-26 "never silently edit historical
entries" discipline; correction recorded here, no historical edits to
iter 25 or iter 26):

- **Iter-25's `bench/harness.py:378` routing claim for T18 was
  incorrect, and iter-25 made a corpus-vacuous-code-path mis-
  identification of the same shape iter-25 itself caught in iter-21's
  prose.** In the "What this exercises that prior PI cells did not"
  section of the iter-25 entry, iter 25 wrote "and the raw_bytes
  branch via the `else` arm at `bench/harness.py:378` (T18)" while
  enumerating the three scorer functions covered by the six prior PI
  bundles. The actual routing for T18 (`scorer.kind="raw_bytes"`,
  `expected_artifact="file_contents"`) is through the explicit
  early-return branch at `bench/harness.py:340–352`
  (`if policy.kind == "raw_bytes": ... return ok, ok, "\n".join(report)`),
  not through line 378's else-arm catch-all
  (`ok = a.strip() == e.strip()`). Verified by re-reading
  `bench/harness.py:340–352` and `:378–379` and tracing all 24 tasks
  through the dispatcher: every task has
  `scorer.kind ∈ {raw_bytes, structural, normalized_text}` and routes
  to one of the four explicit branches (lines 340 / 363 / 367 / 371).
  No task in the current corpus reaches line 378's else-arm — it is
  corpus-vacuous in `bench/tasks/tasks.json`. This is the same
  "corpus-vacuous code path" defect class that iter 25 itself
  surfaced about iter-21's "compare_block_order/compare_block_text in
  isolation under kind=structural" framing (corrected forward-pointing
  in iter 25's own entry); iter 25 then made the same kind of error
  in the very next paragraph of its own prose. iter-26's ratification
  used the more abstract format "T18 (raw_bytes / file_contents,
  iter 14)" without naming a routing line, so iter 26 did not
  specifically verify or dispute the iter-25 line-378 claim. iter-25
  also wrote "Routing through `bench/harness.py:355`'s scorer
  dispatcher" — line 355 is in fact `a, e = actual, expected` (start
  of the post-raw_bytes-branch normalization), not the dispatcher
  function declaration; the function `score_task` is declared at
  line 329. Both citations were imprecise from inception.
- **Discipline-correct preservation of iter-25 / iter-26 entries
  remains intact.** The iter-25 entry's prose at the "Routing
  through ... scorer dispatcher" / "raw_bytes branch via the `else`
  arm at line 378" sentences is preserved unchanged per iter-15
  discipline. The forward-pointing correction lives in this iter-27
  entry as a cross-referenced observation. Future readers of the
  iter-25 line-378 sentence should consult this iter-27 forward-
  pointer.

Rationale for forward-pointing only:

Per iter 15 (second-opinion-ratified at 0.9 confidence), the iter-25
and iter-26 entry text is preserved unchanged. The forward-pointing
correction below names the iter-25 prose claim and explains why it is
inaccurate, with no edits to historical iter-25 / iter-26 prose.
Future readers of the iter-25 routing claim should consult this
iter-27 forward-pointer.

Structural lesson — and why this is a real fresh failing trace:

The "corpus-vacuous code path" defect class previously surfaced in
iter 25 (about iter 21's framing) is now seen to be self-similar at
the meta-level: the iteration that surfaces the defect can itself
make the same kind of error in unrelated prose. iter-25 caught
iter-21's `compare_block_order/compare_block_text in isolation under
kind=structural` framing as corpus-vacuous, then in the very next
paragraph mis-identified line 378 as T18's routing path when line
378 is itself corpus-vacuous. The correction discipline is:
ratifications should specifically check any "task X routes through
code path Y" claim against the live dispatcher, not just against the
abstract scorer-branch label. iter-26's abstract "T18 (raw_bytes /
file_contents, iter 14)" wording is correct; iter-25's specific
"line 378 else arm" wording is not. Recording the lesson here, not
as a new finding (no measurement, scorer, or product surface is
affected; only iter-25 prose accuracy about the dispatch routing).

Verdict — iter-26's substantive `bench/RESULTS.md` fifth-row
publication and cross-executor table extension ratified bit-exact.
The closure-discipline rule's "next pass not re-raising the finding"
criterion is satisfied for the publication half. iter-26's ratification
of iter-25's typed-artifact claims (results.json, run.json,
pi-audit.jsonl, adapter counters, the seven-PI-bundle scorer-branch
inventory at the abstract level, the corpus-vacuous-path verification,
and the forward-pointing scan-methodology correction) all reproduce.
The fresh failing trace is in iter-25's specific code-path prose
(line 378 / line 355 citations) which iter 26 did not repeat — so
the trace is contained at iter-25's authoring time and does not
propagate forward in iter 26. No new finding opened, no holdout
artifact touched, no published-narrative file edited.

- **Frontier anchor (review pass):** *closure-discipline rule applied
  to substantive published-narrative edit* + *fresh failing trace in
  iter-25's specific code-path prose surfaced during the routing
  spot-check*. iter 26 made specific typed-artifact claims (the
  cross-executor table values for T9 + the four ratios for the prior
  rows + the corpus-vacuous-path verification + the OAI scan
  methodology correction) that needed independent verification.
  iter 27 discharges this by reading typed artifacts (results.json
  for both T9 sides, run.json line 20, pi-audit.jsonl event count
  and shape, adapter PiAuditCounters, the corpus task scan, the
  bench/harness.py dispatcher routing trace) rather than narrative.
  Same shape as iter 24 / iter 22 (closure-discipline ledger-only
  ratification + forward-pointing citation correction).
- **Same-family-rule discharge:** iter 23 was specification coherence
  (substantive RESULTS.md:67 publication paired with closure-discipline
  ratification of iter 22), iter 24 was closure-discipline (ledger-only
  ratification of iter 23 paired with one forward-pointing citation
  accuracy correction), iter 25 was intervention-diversity (expensive
  outer channel run + new durable PI bundle on a previously-uncovered
  scorer cell shape), iter 26 was specification coherence (additive
  measurement publication of the iter-25 T9 PI bundle as a fifth row
  in the cross-executor table, paired with closure-discipline
  ratification of iter 25). iter 27 returns to a ledger-only
  closure-discipline ratification entry parallel to iter 24 / iter 22 /
  iter 15. The fresh-failing-trace escape clause additionally applies:
  the iter-25 line-378 / line-355 prose-accuracy defect is the fourth
  "ratification surfaces a code-path / citation-accuracy defect" in
  this gnhf run (after iter 13 / iter 20 line-drift in RESULTS.md,
  iter 22 typos in iter-21 ledger prose, iter 24 citation accuracy of
  iter-22's forward-pointers, iter 26 scan methodology error in
  iter-25 prose; this iter-27 trace is in iter-25's code-path-routing
  prose). The chain (intervention-diversity → spec-coherence →
  closure-discipline) is alternating, not concentrated; the
  "ledger-only changes do not break concentration" caveat does not
  block this iteration because there is no concentration to break.
- **Comparability framing:** the ratification is a ledger-only
  verification with one forward-pointing code-path-routing correction
  surfaced from re-reading the bench/harness.py dispatcher against the
  iter-25 prose. It does not change any data, ratio, rule conclusion,
  pass rate, bundle, scorer, or holdout artifact. It does not bump
  `holdout_version` (still 1). It does not run the expensive outer
  channel. It does not edit any published-narrative file
  (`bench/RESULTS.md`, `README.md`, `CLAUDE.md`,
  `bench/retracted_2026-04-24/README.md`, `specs/**`). The only file
  modified is `bench/ledger.md` itself, with two additions: this
  iter-27 entry and an updated halt-condition / quiet-signal status
  block.
- **Closure-discipline status:** this is a non-finding ledger-only
  ratification entry, parallel to iter 22 / iter 24. Per iter-15's
  labeling discipline, this is **not frontier expansion** in the
  sense of evaluator repair, product change, or measurement
  publication. It procedurally ratifies iter 26's substantive
  publication and records one forward-pointing code-path-routing
  correction in iter-25's prose. The 23rd consecutive zero-OPEN
  review round holds.
- **What this does NOT do:** does not promote any product anchor
  (`bench/probes/anchor-validation/` still does not exist, no
  candidate primitive validated). Does not bump `holdout_version`
  (still 1). Does not run the expensive outer channel. Does not edit
  any historical ledger entry inline. Does not amend any pass-rate
  claim. Does not modify any bundle, scorer, harness, or test code.
  Does not extend the cross-executor table. Does not touch any
  holdout artifact. Does not extend `bench/probes/`,
  `bench/search/candidates/`, or any other not-yet-existing T7
  directory. The only sanctioned artifact change is the new ledger
  entry plus the halt-condition / quiet-signal status block update.

### Cross-executor same-task measurement extension (2026-04-26 iter 26)

Specification-coherence move: extending the cross-executor same-task
table at `bench/RESULTS.md:58` from **four** rows to **five** rows by
adding a T9 row pairing the iter-25 PI T9 mdtools bundle with the
pre-existing OAI Qwen3.5-122B-A10B-4bit T9 mdtools cell from
`bench/runs/search-mdtools-extraction-Qwen3.5-122B-A10B-4bit-2026-04-21/`,
paired with closure-discipline ratification of iter 25's typed claims.
Parallel in shape to iter 19's extension of the table from 3 to 4 rows
(cashing out the iter-14 T18 PI bundle 5 iterations later) and iter
11's original 3-row publication. Cheap channel green before and after.

**Disturbance:** specification coherence — published-narrative ↔
bundle-existence drift. The iter-25 T9 PI bundle exists durably under
`bench/runs/checkpoint-pi-T9-mdtools-gpt5.4mini-2026-04-26/` since
iter 25 and a same-task OAI mdtools cell on `Qwen3.5-122B-A10B-4bit`
already existed in
`bench/runs/search-mdtools-extraction-Qwen3.5-122B-A10B-4bit-2026-04-21/`
(verified by reading the bundle's `results.json` and confirming
`task_id=T9`, `mode=mdtools`, `correct=true`, `tool_calls=2`,
`bytes_output=2169`, `bytes_observation=7100`). The cross-executor
table at `bench/RESULTS.md:58` carried only 4 rows (T1, T7, T22, T18)
without the T9 row, even though all data needed for the row was
durably preserved. Same shape as iter 19's pre-edit state for T18
(iter-14 T18 PI bundle uncited in the table).

**Anchor:** *missing evaluator artifact — durable summary for a
newly-run comparison*. Same anchor wording as iter 11 and iter 19;
the intervention is the additive-measurement-publication shape (a
new table row, not the corrective reference removal of iters 8/9/13/20).
The forcing function for choosing this anchor is iter-11's learning
#1 ("Future expensive-channel runs should be examined for downstream
pairing potential, not just cell-coverage credit") combined with the
iter-25 entry's explicit Closure-discipline status invitation for a
future review pass to ratify the bundle's typed claims. Both
preconditions converged at iter 26: the iter-25 expensive run's T9
bundle has a clean OAI same-task pair on `Qwen3.5-122B-A10B-4bit`
(the same model as rows 1–3 of the table — cleaner than iter 19's
T18 row, which had to substitute `Qwen3.5-27B-4bit` because no `-122B`
multistep cell exists), and the closure-discipline ratification of
iter 25 is independently invited.

**Change shape:** four targeted edits to `bench/RESULTS.md`:

1. Caption updated from "(2026-04-26 iters 11, 19)" to "(2026-04-26
   iters 11, 19, 26)" and "Four `mdtools` cells" to "Five `mdtools`
   cells" (line 56).
2. New table row added between T7 and T22 (sorted by ratio, descending):
   `| T9 | 2 / 0 | 2 / 0 | 3,177,953 | 2,169 | ~1,465× | 6,675 | 7,100 |`
   (line 62).
3. Commentary paragraph (line 66) updated from "all four pairs" to
   "all five pairs" and the bytes_observation rule extended from "T1
   and T7" to "T1, T7, and T9 where both executors produced the same
   tool-call count (the T9 row tightens this to ~6%)".
4. PI bundle pointers list (line 68) extended with the iter-25
   bundle path; OAI bundle pointers list (line 68) updated to clarify
   that the existing
   `search-mdtools-extraction-Qwen3.5-122B-A10B-4bit-2026-04-21/`
   bundle file carries both T1 and T9 cells in its `results.json`
   array (verified — that bundle's `results.json` contains T1, T9,
   T16 mdtools cells). The iter-18 T2 and iter-21 T21 not-yet-eligible
   sentences are preserved unchanged.

No edits to ledger lines outside this entry, to historical bundle
artifacts, or to other published-narrative files. No code or test
changes.

**Data points (iter-26 additions, source: typed artifacts):**

- `bench/runs/checkpoint-pi-T9-mdtools-gpt5.4mini-2026-04-26/results.json`:
  `task_id=T9`, `mode=mdtools`, `correct=true`,
  `correct_neutral=true`, `tool_calls=2`, `mutations=0`,
  `bytes_observation=6675`, `bytes_output=3177953`,
  `elapsed_seconds=14.39`, `diff_report="json_canonical: OK"`,
  `model="openai-codex/gpt-5.4-mini"`, `thinking_level="minimal"`.
  ✓ (re-confirmed bit-exact at iter-26 review time).
- `bench/runs/search-mdtools-extraction-Qwen3.5-122B-A10B-4bit-2026-04-21/results.json`
  T9 mdtools cell: `task_id=T9`, `mode=mdtools`, `correct=true`,
  `correct_neutral=true`, `tool_calls=2`, `mutations=0`,
  `bytes_observation=7100`, `bytes_output=2169`,
  `elapsed_seconds=20.69`, `diff_report="json_canonical: OK"`,
  `model="Qwen3.5-122B-A10B-4bit"`. ✓.
- Cross-executor `bytes_output` ratio: 3,177,953 / 2,169 = **~1,465×**.
  Sits inside the existing 10³ envelope (existing rows: 2,212× /
  1,677× / 1,376× / 1,040×); the T9 row is the third-largest of five.
- Cross-executor `bytes_observation` delta: 6,675 (PI) vs 7,100 (OAI)
  = **~6%** difference, both at `tool_calls=2`. Tightens the
  "within ~20% on matched tool-call count" sub-rule to the smallest
  observed delta in the table (T1=7%, T7=19%, T9=6%).
- The iter-25 T9 PI bundle is **the third durable bundle in
  `bench/runs/`** carrying iter-17's `holdout_version: 1` stamp on
  `run.json` (after iter-18 T2 first, iter-21 T21 second; verified
  via `bench/runs/checkpoint-pi-T9-mdtools-gpt5.4mini-2026-04-26/run.json`
  line 20).

**Closure-discipline ratification of iter 25 (paired):**

- `bench/runs/checkpoint-pi-T9-mdtools-gpt5.4mini-2026-04-26/results.json`
  re-read bit-exact: every field iter 25 cited (`tool_calls=2`,
  `mutations=0`, `bytes_observation=6,675`, `bytes_output=3,177,953`,
  `elapsed_seconds=14.39`, `diff_report="json_canonical: OK"`,
  `requeried=false`, `correct=true`, `correct_neutral=true`,
  `model="openai-codex/gpt-5.4-mini"`, `thinking_level="minimal"`)
  reproduces.
- `bench/runs/checkpoint-pi-T9-mdtools-gpt5.4mini-2026-04-26/run.json`
  re-read: `holdout_version: 1` confirmed on line 20; all 15 other
  metadata keys present; `runner=pi-json`, `executor=guarded`,
  `runs_per_task=1`. Aggregates match the per-result data.
- `bench/runs/checkpoint-pi-T9-mdtools-gpt5.4mini-2026-04-26/logs/T9_mdtools_1777221491/pi-audit.jsonl`
  re-read: 6 events parse cleanly via `bench.pi_audit_adapter.summarize_pi_audit_events`;
  histogram is `{model_change: 1, thinking_level_change: 1, tool_call: 2, tool_result: 2}`;
  the two `tool_call` commands are `./md tasks ... --status pending --json`
  and `./md tasks ... --status pending --json | jq '[.results[0].tasks[] | {loc, nearest_heading, summary_text}]'`;
  the two `tool_result` events have `outputBytes=4686` and `outputBytes=1989`
  summing to `bytes_observation=6675` per the iter-25 entry.
- `bench.pi_audit_adapter.summarize_pi_audit_events` re-run on the
  events: `PiAuditCounters(tool_calls=2, tool_results=2, tool_errors=0,
  bytes_observation=6675, blocked=0, policy_violations=0, mutations=0,
  requeried=False, model='openai-codex/gpt-5.4-mini',
  thinking_level='minimal', bash_commands=[<two commands>])` — exactly
  matches the iter-25 entry's adapter-counter claim.
- Seven-PI-bundle scorer-branch inventory verified: T1 (heading_tree,
  iter 4), T22 (link_destinations, iter 7), T7 (block_text +
  block_order + heading_tree via normalized_text, iter 10), T18
  (raw_bytes / file_contents, iter 14), T2 (block_text + block_order
  via normalized_text, iter 18), T21 (frontmatter_json, iter 21),
  T9 (json_canonical, iter 25). The iter-25 entry's claim that T9 is
  the first PI cell exercising `score_json_canonical` reproduces.
- The iter-25 forward-pointing observation about iter-21's
  "compare_block_order/compare_block_text in isolation" framing
  being corpus-vacuous is independently verified: scanning
  `bench/tasks/tasks.json` for `"compare_block_order": true` returns
  the tasks T2 / T7 (both `kind=normalized_text`, not `structural`),
  and `"compare_block_text": true` returns the same set; no task in
  the corpus has either flag set under `kind=structural`. The
  iter-25 framing-correction prose reproduces.

Forward-pointing correction (no historical edits per iter-15
discipline): iter 25's "Comparability framing" claim that "no
`search-mdtools-extraction-*` OAI bundle exists for T9 in this repo;
scanned `bench/runs/` for T9-named cells, found only the present
iter-25 PI bundle — verified by listing `bench/runs/` and excluding
the seven PI checkpoint dirs" was a fresh failing trace surfaced by
this iteration. The OAI T9 mdtools cell **does exist** at
`bench/runs/search-mdtools-extraction-Qwen3.5-122B-A10B-4bit-2026-04-21/results.json`
(verified: that bundle's `results.json` array contains T1 / T9 / T16
mdtools cells). The iter-25 scan methodology error was scanning
**directory names** for "T9" rather than scanning **`results.json`
contents** for `task_id == "T9"`; the multi-task search-mdtools
bundles aggregate multiple tasks under one directory keyed by model
+ family, so a directory-name scan misses task cells that live inside
multi-task bundles. The correct discovery query is
`grep -l '"task_id": "T9"' bench/runs/*/results.json`, which returns
all bundles carrying a T9 cell (including the four
`search-mdtools-extraction-*` OAI bundles for the four search-pilot
models). This is parallel in shape to iter 12's argparse `--executor`
typo and iter 13 / iter 20's line-number-drift fresh traces — a
methodology error in the prior iteration's prose surfaced by the
ratification step itself, with the correction recorded forward-
pointing per iter-15 discipline. Iter 26's table extension is the
substantive remedy; the underlying OAI T9 cell has been queryable
since 2026-04-21 (well before iter 25), so the cash-out was
admissible at iter 25 commit time but missed because of the scan
error.

**Cheap channel status:** green before and after.
`cargo test -q` all suites pass (24+32+37+16+0 + the doctest 7).
`python3 -m unittest` 68 tests OK across the 8 spec-named modules.
`python3 bench/harness.py --md-binary target/release/md` dry-run
reports "All tasks pass dual scorer" on all 24 tasks with
`json_canonical: OK` on T9.

**Comparability framing:** This is NOT a holdout reconfirmation
(T9 is search-side, holdout split is T4/T14/T20/T22/T23/T24). It is
NOT a cross-model comparison (the table caption explicitly notes
model is confounded — PI uses `openai-codex/gpt-5.4-mini`, OAI uses
`Qwen3.5-122B-A10B-4bit`, with T18 row using `Qwen3.5-27B-4bit`
exception). It is NOT a comparison vs the iter-25 ratification status
(iter 25 was a fresh-signal expensive run; this is the downstream
specification-coherence cash-out). It does NOT change any PI bundle,
any OAI bundle, any holdout artifact, any scorer, or any
`holdout_version` (still 1). It does NOT promote any product anchor
(`bench/probes/anchor-validation/` still does not exist). It does
NOT amend any historical ledger entry inline (per iter-15 discipline).

**Closure-discipline status:** FIXED_PENDING_CONFIRMATION at
authoring time. A future review pass should re-read every typed
artifact cited above (the new `bench/RESULTS.md` row and pointers
against the bundle `results.json` files, the closure-discipline
ratification claims against the bundle log files, the
seven-PI-bundle inventory against `bench/runs/`), per the iter-12
/ iter-13 / iter-15 / iter-20 / iter-22 / iter-24 ratification
pattern.

**Same-family-rule discharge:** iter 23 was specification coherence
(substantive RESULTS.md:67 publication + closure-discipline
ratification of iter 22), iter 24 was closure-discipline (ledger-only
ratification of iter 23 + forward-pointing citation accuracy
correction), iter 25 was intervention-diversity (expensive outer
channel run + new durable PI bundle). Iter 26 is specification
coherence (additive measurement publication of the iter-25 T9 PI
bundle as a fifth row in the cross-executor table) **paired with
closure-discipline ratification of iter 25** — same shape as iter
19's pairing of the T18 row addition with closure-discipline
ratification of iter 18. The iter-25 expensive run reset the
quiet-signal counter and broke any prior concentration; iter 26's
single specification-coherence move is independently admissible.
Additionally, the fresh-failing-trace escape clause applies via
the iter-25 OAI-scan methodology error surfaced during the
ratification step (parallel in shape to iter 12's argparse typo,
iter 13 / iter 20's line-drift, and iter 22 / iter 24's
ledger-citation-accuracy errors). Parallel in shape to iter 19
relative to iter 18 (both extend the cross-executor table by one
row immediately after an expensive-channel PI bundle).

**What this does NOT do:** does not promote any product anchor
(`bench/probes/anchor-validation/` still does not exist, no
candidate primitive validated). Does not bump `holdout_version`
(still 1). Does not run the expensive outer channel. Does not edit
any historical ledger entry inline. Does not amend any pass-rate
claim. Does not modify any bundle, scorer, harness, or test code.
Does not extend the cross-executor table for the iter-18 T2 or
iter-21 T21 PI bundles (no OAI same-task `mdtools` cell exists for
either; the not-yet-eligible sentences in
`bench/RESULTS.md:68` remain unchanged). Does not touch any
holdout artifact.

### Quiet-signal checkpoint discharge (2026-04-26 iter 25)

Discharged the spec's mandated quiet-signal checkpoint after iter 22 /
iter 23 / iter 24 each incremented the counter (1 → 2 → 3) without an
expensive run. Iter 25 was the next forced expensive-or-halt point per
the spec's "3 consecutive iterations with the cheap channel green, no
new failing trace, and no new finding added" rule, parallel in
structural position to iter 4 (after iters 1–3), iter 7 (after iters
5–6), iter 10 (after iters 8–9 same-family spec-coherence), iter 14
(after iters 11–13), iter 18 (after iters 15–17), and iter 21 (after
iters 19–20).

**Frontier anchor:** missing evaluator artifact — a comparable-harness-
axis cell coverage extension. PI runner coverage now spans a seventh
distinct scorer-function path: the `score_json_canonical` function at
`bench/harness.py:400`, dispatched at `bench/harness.py:363` when
`policy.kind == "structural"` AND `policy.json_canonical` is truthy
(canonical-JSON equality with optional `json_required_keys` projection
at `bench/harness.py:391`). All six prior PI bundles routed through
either `score_structural_json` (T1 / T22 / T21 — `kind=structural`
with one `compare_*` flag set and `json_canonical` falsy / unset),
`score_normalized_text_*` (T7 / T2), or the raw_bytes branch (T18) —
*none* exercised `score_json_canonical`. Parallel in shape to
iter-10's anchor framing for T7 (mutation + normalized_text + re-query
coverage extension) and iter-21's for T21 (frontmatter_json scorer-
branch coverage extension).

**Bundle citation:** `bench/runs/checkpoint-pi-T9-mdtools-gpt5.4mini-2026-04-26/`
— seventh PI runner bundle in this repo and the first cell exercising
the `score_json_canonical` scorer function (with `json_required_keys`
set on T9: `["loc", "nearest_heading", "summary_text"]`) end-to-end
through the PI executor. T9 mdtools dual-scorer PASS
in 14.39s (`md=PASS neutral=PASS`, `diff_report: json_canonical: OK`),
with 2 tool calls (`./md tasks ... --status pending --json` followed by
the same command piped to a `jq` projector for the `loc` /
`nearest_heading` / `summary_text` keys), 0 mutations, `requeried=false`,
`bytes_observation=6,675`, `bytes_output=3,177,953`. `pi-audit.jsonl`
preserves 6 events (`model_change` + `thinking_level_change` + 2 ×
`tool_call` + 2 × `tool_result`) that parse cleanly via
`bench.pi_audit_adapter.summarize_pi_audit_events` (counters: tool_calls
2, tool_results 2, tool_errors 0, blocked 0, mutations 0,
policy_violations 0, requeried False, bytes_observation 6675, model
`openai-codex/gpt-5.4-mini`, thinking_level minimal,
bash_commands matches the two issued commands). The bundle's `run.json`
line 20 carries `holdout_version: 1` — the third durable bundle in
`bench/runs/` carrying the iter-17 stamp (after iter-18 T2 and iter-21
T21).

**Normalization-axis disclosure:** PI runner; `model =
openai-codex/gpt-5.4-mini`; `mode = mdtools`; `executor = guarded`;
`thinking_level = minimal`; `runs_per_task = 1`; `holdout_version = 1`;
T9 is on the search side (holdout split is T4/T14/T20/T22/T23/T24), so
no holdout-repair path is implicated.

**Comparability framing:** This is NOT a holdout reconfirmation
(T9 is search-side and the cheapest-named-probe of a Qwen3.5-122B-A10B-4bit
holdout reconfirmation remains environment-blocked per iter 7). It is
NOT a comparison vs prior PI bundles (different task, different scorer
function). It is NOT a comparison vs OAI bundles for T9 (no
`search-mdtools-extraction-*` OAI bundle exists for T9 in this repo;
scanned `bench/runs/` for T9-named cells, found only the present iter-25
PI bundle — verified by listing `bench/runs/` and excluding the seven
PI checkpoint dirs). It is the first PI cell to exercise (a) the
`score_json_canonical` scorer function at `bench/harness.py:400`,
(b) the `_project_keys` projection helper at `bench/harness.py:391`
via the `json_required_keys` policy field, and (c) the mdtools-mode-
with-jq-projection pattern on a structural-shape task (distinct from
T7's mdtools-mode mutation-then-jq-free re-query and T21's mdtools-
mode `--json` direct-extract).

**What this exercises that prior PI cells did not:** Routing through
`bench/harness.py:355`'s scorer dispatcher, the six prior PI bundles
cover three distinct scorer functions: `score_structural_json` (T1
heading_tree, T22 link_destinations, T21 frontmatter_json),
`score_normalized_text_md` + `score_normalized_text_neutral` (T7
block_text+block_order+heading_tree, T2 block_text+block_order), and
the raw_bytes branch via the `else` arm at `bench/harness.py:378`
(T18). T9 adds a fourth scorer function — `score_json_canonical` at
`bench/harness.py:400` — which the dispatcher selects when
`kind=structural` AND `json_canonical` is truthy (line 363, ahead of
the `score_structural_json` branch at line 367). It also adds the
first PI demonstration of the canonical mdtools+jq projection pattern
on a benchmark task (the agent first runs `./md tasks --status pending
--json` to inspect, then re-issues with `| jq '[.results[0].tasks[] |
{loc, nearest_heading, summary_text}]'` to project — a pure-extraction
analog to T7's mutate-then-re-query pattern).

**Forward-pointing observation about iter-21's "compare_block_order /
compare_block_text in isolation" framing (and iter-24's repetition of
it):** Iter 21's learning #2 stated "only `compare_block_order` in
isolation and `compare_block_text` in isolation are not yet exercised
through PI (T2 and T7 use them via normalized_text but no PI bundle
uses them via structural+block_text without other branches)." Iter 24's
halt-condition block carried that framing forward as the named cheapest
probe ("PI runner extending to compare_block_order/compare_block_text
isolation cells (T9/T16/T19)"). Inspection of the live corpus
(`bench/tasks/tasks.json`) shows that *no task in the corpus has
`compare_block_order=true` in isolation under `kind=structural`*, and
*no task has `compare_block_text=true` in isolation under
`kind=structural`*: every task with `compare_block_order=true` or
`compare_block_text=true` uses `kind=normalized_text`, not `structural`.
The "in isolation under structural" code path in
`score_structural_json` is therefore corpus-vacuous — it cannot be
exercised by any current task. The actual uncovered cell shape that
T9/T11/T16/T19 share is `kind=structural` + `json_canonical=true` +
*all* `compare_*` flags false (with optional `json_required_keys` set
on T9), which is what iter-25's T9 bundle exercises. Recorded
forward-pointing in iter-25 per iter-15 / iter-22 / iter-24 discipline
(no silent edits to historical iter-21 / iter-24 prose); the iter-21
typo class is "framing of a corpus property that doesn't actually hold,"
distinct from iter-22's clerical-typo class and iter-13/iter-20's
line-drift class. The iter-24 named-cheapest-probe candidate cells
(T9 / T16 / T19) were correct as cells; only the gap-naming was wrong.

**Behavioral observation (per-model data, not finding):** gpt-5.4-mini
at minimal thinking issued 2 tool calls on T9 mdtools — no
`./md tasks --json` re-query for verification, just inspect-then-project.
On T1 (iter 4) gpt-5.4-mini also used 1 tool call (extraction); on T22
(iter 7) 1 call; on T21 (iter 21) 1 call; on T9 (iter 25) 2 calls
(inspect + project). Only mutation tasks (T7 iter 10, T18 iter 14, T2
iter 18) trigger the verification re-query pattern in this model. T9 is
the first PI extraction cell where the agent issues 2 calls (project
re-issue), refining the iter-10 learning that "the verification
re-query is emitted spontaneously" — for *mutation* tasks. Pure
extraction with downstream projection is a 2-call pattern (read +
project), not a 3-call pattern (read + mutate + verify-read).

**Cheap channel status:** green before and after the run.
`cargo test -q` all four suites pass (32+37+16+0). `python3 -m unittest`
68 tests OK across the 8 spec-named modules. `python3 bench/harness.py
--md-binary target/release/md` dry-run reports "All tasks pass dual
scorer" on all 24 tasks. The expensive run did not change any code; it
produced one new directory under `bench/runs/`.

**Closure-discipline status:** FIXED_PENDING_CONFIRMATION at authoring
time. A future review pass should re-read every typed-artifact claim in
this entry against the bundle (results.json fields, run.json
metadata-key completeness, pi-audit.jsonl event count and shapes,
adapter counter outputs, the seven-PI-bundle scorer-branch inventory,
the corpus-vacuous-path verification), per the iter-15 / iter-22 / iter-24
ratification pattern; bit-exact reproduction of every claim with no
re-raise of a finding promotes this entry to CLOSED.

**Same-family-rule discharge:** iter 22 was closure-discipline (ledger-
only with forward-pointing typo corrections), iter 23 was specification
coherence (substantive RESULTS.md:67 publication + closure-discipline
ratification of iter 22), iter 24 was closure-discipline (ledger-only
ratification of iter 23 + forward-pointing citation accuracy correction).
Iter 25 is intervention-diversity (expensive outer channel run + new
durable bundle), shifting axis cleanly. The quiet-signal rule's
expensive-or-halt mandate is its own escape clause for the same-family
rule when the iteration is forced to act, parallel to iter 4 / iter 14 /
iter 18 / iter 21.

**What this does NOT do:** This entry does not promote any new product
surface, does not alter any scorer or harness behavior, does not touch
the holdout split, does not run holdout reconfirmation (Qwen3.5
environment still blocked), does not establish any cross-executor
comparison (no OAI T9 mdtools bundle exists for pairing), does not
discharge any product-anchor justification (no candidate primitive is
validated by a passing T9 cell), and does not promote any
FIXED_PENDING_CONFIRMATION entry to CLOSED beyond its own closure-
discipline status. It only discharges the quiet-signal rule by
introducing fresh typed signal on a previously-uncovered scorer cell
shape.

### Confirmation review pass (2026-04-26 iter 24)

Closure-discipline review of iter-23's substantive `bench/RESULTS.md:67`
sixth-bundle publication (the additive sentence citing the iter-21 T21
PI bundle, parallel in shape to iter-19's relation to the iter-18 T2
bundle). Parallels iter 22's review of iter 21 (also a substantive
ratification + forward-pointing correction iteration), iter 13's review
of iter 12 (line-drift fix), and iter 20's review of iter 19 (line-drift
fix): typed-artifact claims in the iter-23 entry are checked against
the underlying bundle files, the published-narrative addition at
RESULTS.md:67, and the OAI bundle inventory. Differs from iter 15's
ratification-only-no-fresh-trace shape in that the verification surfaces
a fresh failing trace inside the iter-22 entry's own citation prose
(carried forward into iter 23's verification claim about it). Cheap
channel green at review time (`cargo test --quiet` all suites pass:
24+32+37+16+0+0; 68 python unittests OK across the 8 spec-named
modules; `harness.py --md-binary` dry-run all 24 tasks PASS
dual-scorer).

What was checked:

- **`bench/RESULTS.md:67` sixth-bundle sentence** — re-read. The
  appended sentence reads "The sixth PI bundle,
  `bench/runs/checkpoint-pi-T21-mdtools-gpt5.4mini-2026-04-26/` (iter
  21 — first `compare_frontmatter_json` scorer-branch PI cell, and the
  second durable bundle carrying iter-17's `holdout_version: 1` stamp
  on `run.json`), is similarly not yet eligible for this table because
  no OAI same-task `mdtools` cell exists on file." Matches iter-23's
  claim bit-exact.
- **Sixth-bundle inventory** — confirmed by enumerating bundle paths
  in chronological order: T1 (iter 4), T22 (iter 7), T7 (iter 10),
  T18 (iter 14), T2 (iter 18), T21 (iter 21) = 6 bundles. The
  iter-23 sentence's "sixth" claim reproduces.
- **First `compare_frontmatter_json` scorer-branch claim** — the
  iter-21 / iter-22 corpus uniqueness check (only T21 sets this flag)
  remains valid; the iter-23 sentence's "first compare_frontmatter_json
  scorer-branch PI cell" claim reproduces.
- **Second durable bundle with `holdout_version: 1` stamp** —
  re-verified: `bench/runs/checkpoint-pi-T21-mdtools-gpt5.4mini-2026-04-26/run.json`
  line 20 reads `"holdout_version": 1`, and
  `bench/runs/checkpoint-pi-T2-mdtools-gpt5.4mini-2026-04-26/run.json`
  line 20 reads `"holdout_version": 1`. T21 is the second stamped
  bundle after T2 (first), matching the iter-23 sentence.
- **`bench/harness.py:1282` and `:1318` bytes_output** — re-read in
  the current file: `bytes_output = len(raw_stdout.encode())` on both
  lines, immediately following `raw_stdout = result.stdout` in the
  pi-json branch (line 1282) and the oai-loop branch (line 1318). The
  iter-20 line-citation fix and iter-22's closure-discipline ratification
  of it both still hold; iter-23's forward-carry of that ratification
  reproduces.
- **`pi-audit.jsonl` event count and shape** —
  `bench/runs/checkpoint-pi-T21-mdtools-gpt5.4mini-2026-04-26/logs/T21_mdtools_1777219293/pi-audit.jsonl`
  has exactly 4 lines. Event-type histogram:
  `{model_change: 1, thinking_level_change: 1, tool_call: 1, tool_result: 1}`.
  The single `tool_call` event carries
  `input.command="./md frontmatter <input> --json"` (full temp-dir
  path elided). The single `tool_result` event has `outputBytes=565`
  matching `bytes_observation=565`. Matches iter-23's bullet bit-exact.
- **OAI T21 `mdtools` scan** — re-ran the scan
  iter-23 cited (search-mdtools-* / search-hybrid-* /
  holdout-mdtools-* / holdout-hybrid-* `results.json` for
  `task_id=T21`): 22 bundles scanned, 0 T21 hits. Confirms
  iter-23's "no OAI same-task `mdtools` cell on file" claim.

Forward-pointing correction surfaced during the verification (per
iter-15 "never silently edit historical entries" discipline; correction
recorded here, no historical edits to iter 22 or iter 23):

- **Iter-22's `bench/ledger.md:54` and `:61–62` citations were never
  accurate.** Iter 22's forward-pointing correction bullets cite
  "`bench/ledger.md:54`" (for the iter-21 entry's "iter 22's
  link_destinations" → should-be "iter 7's" typo) and
  "`bench/ledger.md:61–62`" (for the iter-21 entry's "`run.json` line
  18" → should-be "line 20" typo). At iter-22 commit time
  (`d4547d3`), `bench/ledger.md:54` was inside iter-22's own
  "What was checked" body (specifically, an `Aggregates section`
  remark at offset 5 from the iter-22 entry header), and the iter-21
  typos were actually at lines 230 and 237 — verified via
  `git show d4547d3:bench/ledger.md | grep -n "iter 22's link_destinations\|run.json.*line 18"`.
  The miscitation was wrong from inception; iter 22's intended
  pointer was lines 230 and 237 (offset 37 and 44 from the iter-21
  entry header at line 193 in the iter-22-commit-time file).
- **Iter-23's verification claim about lines 54 and 61 was internally
  inconsistent at iter-23 commit time.** Iter-23 wrote: "re-verified
  that lines 54 and 61 of the ledger still carry the original iter-21
  prose with the typos preserved." At iter-23 commit time
  (`30563dc`), iter 23 had added 148 lines above iter 22, pushing
  the iter-21 typos to ~378/385 — and lines 54 and 61 were now
  inside iter-23's own "Change shape" bullet body, not iter-21.
  Verified via `git show 30563dc:bench/ledger.md | sed -n '50,65p'`
  showing iter-23's "appending a parallel-shape sixth-bundle
  sentence" at line 54 of that commit's file. Currently (post
  iter-23) lines 54 and 61 of the live ledger file are even further
  inside iter-23's body, the iter-21 typos are at lines 379 and
  386.
- **Discipline-correct preservation of iter-21 typos remains
  intact.** The fresh failing trace is in the citation accuracy of
  iter-22 / iter-23, not in the typos themselves: iter-21's "iter
  22's link_destinations coverage" prose is at line 379 and "`run.json`
  line 18 reads" prose is at line 386 (verified via
  `grep -n` on the live file), unchanged from iter-22/iter-23
  commits as the iter-15 discipline requires.

Rationale for forward-pointing only:

Per iter 15 (second-opinion-ratified discipline at 0.9 confidence),
the iter-22 and iter-23 entry text is preserved unchanged. Both
correction observations live in this iter-24 entry as
cross-referenced observations. Future readers of the iter-22
forward-pointer should consult this iter-24 forward-pointer when
reading iter-22's `bench/ledger.md:54` and `:61–62` citations.

Structural lesson — and why this is a real fresh failing trace, not
ratification theater:

Ledger line citations drift faster than file citations because every
new ledger entry pushed atop CLOSED shifts all lines below by the
new entry's length. File citations into source code (e.g.
`bench/harness.py:1282`) drift only when intervening edits touch
that file (~53 lines over five iterations for iter-20's
RESULTS.md:54 citation; that drift is still the smallest unit of
fresh signal in those iterations). Ledger citations drift on every
single new CLOSED entry (148 lines from iter 22 → iter 23 alone).
The iter-22 / iter-23 citation defect surfaced here is the
foreseeable consequence: any ledger citation by absolute line
number is fragile. Future ledger citations should anchor by entry
header + content pattern (e.g. "in the iter-21 entry's Anchor
bullet, the phrase 'iter 22's link_destinations'") rather than by
absolute line number. This iter-24 entry follows that pattern in
its own correction prose. Recording the lesson here, not as a new
finding (no measurement, scorer, or product surface is affected;
only ledger citation accuracy).

Verdict — iter-23's substantive `bench/RESULTS.md:67` sixth-bundle
publication ratified, the closure-discipline rule's "next pass not
re-raising the finding" criterion is satisfied for the publication
half. Iter-23's coverage of iter-22's typed-artifact citations
(results.json, run.json, pi-audit.jsonl, harness.py:1282/1318)
ratified bit-exact; only the iter-22 line-number citation accuracy
is the fresh failing trace, which propagated forward into iter-23's
verification claim and is corrected here forward-pointing. No new
finding opened, no holdout artifact touched, no published-narrative
file edited.

- **Frontier anchor (review pass):** *closure-discipline rule
  applied to substantive published-narrative edit* + *fresh failing
  trace in the iter-22 / iter-23 citation accuracy chain*. Iter 23
  made specific typed-artifact claims (the RESULTS.md:67 sentence
  content, the sixth-bundle inventory, the first-frontmatter_json
  claim, the second-stamped-bundle claim, the OAI scan emptiness,
  the iter-22 ratification carry-forward) that needed independent
  verification. Iter 24 discharges this by reading typed artifacts
  (results.json + run.json + pi-audit.jsonl + RESULTS.md:67 +
  bench/harness.py:1282/1318 + the OAI bundle scan + git-history
  spot-checks at d4547d3 and 30563dc) rather than narrative. Same
  shape as iter 22's relation to iter 21 — both are ratifications
  of a prior iteration's bundle/publication paired with a
  forward-pointing citation correction.
- **Same-family-rule discharge:** iter 22 was closure-discipline
  (ledger-only with citation corrections), iter 23 was specification
  coherence (substantive RESULTS.md:67 publication paired with
  closure-discipline ratification of iter 22). Iter 24 returns to a
  ledger-only closure-discipline ratification entry parallel to
  iter 22, but the fresh-failing-trace escape clause additionally
  applies: the iter-22 / iter-23 citation-accuracy defect is the
  third instance of this exact "ratification surfaces a citation
  defect" shape (iter 12 argparse typo, iter 13 / iter 20 line-drift
  in RESULTS.md, iter 22 typos in iter-21 ledger prose, iter 24
  citation accuracy of iter-22's forward-pointers). The "ledger-only
  changes do not break concentration" caveat does not block this
  iteration because there is no concentration to break: iter 22 was
  ledger-only-with-corrections, iter 23 was substantive published
  narrative + ratification, iter 24 is ledger-only-with-corrections
  again. The chain (closure-discipline → spec-coherence →
  closure-discipline) is alternating, not concentrated.
- **Comparability framing:** the ratification is a ledger-only
  verification with one forward-pointing line-citation correction
  surfaced from git-history spot-checks. It does not change any
  data, ratio, rule conclusion, pass rate, bundle, scorer, or
  holdout artifact. It does not bump `holdout_version` (still 1).
  It does not run the expensive outer channel. It does not edit
  any published-narrative file (`bench/RESULTS.md`, `README.md`,
  `CLAUDE.md`, `bench/retracted_2026-04-24/README.md`, `specs/**`).
  The only file modified is `bench/ledger.md` itself, with two
  additions: this iter-24 entry and an updated halt-condition /
  quiet-signal status block.
- **Closure-discipline status:** this is a non-finding ledger-only
  ratification entry, parallel to iter 22. Per iter-15's labeling
  discipline, this is **not frontier expansion** in the sense of
  evaluator repair, product change, or measurement publication. It
  procedurally ratifies iter 23's substantive publication and
  records one forward-pointing citation accuracy correction
  spanning iter 22 and iter 23. The 20th consecutive zero-OPEN
  review round holds.
- **What this does NOT do:** does not promote any product anchor
  (`bench/probes/anchor-validation/` still does not exist, no
  candidate primitive validated). Does not bump `holdout_version`
  (still 1). Does not run the expensive outer channel. Does not
  edit any historical ledger entry inline. Does not amend any
  pass-rate claim. Does not modify any bundle or any
  published-narrative file. Does not touch any holdout artifact.
  Does not extend the cross-executor same-task table (no new
  same-task pair was made available by iter 23).

### Specification coherence — iter-21 T21 PI bundle reference extension (2026-04-26 iter 23)

Specification-coherence move: extending the published narrative at
`bench/RESULTS.md:67` to cite the iter-21 T21 mdtools PI bundle as the
**sixth** PI bundle in `bench/runs/`, parallel in shape to iter-19's
publication of the iter-18 T2 PI bundle as the fifth. Surfaced during
post-iter-22 routine reading of the published narrative: the cross-
executor "PI bundle pointers" enumeration listed only T1 (iter 4), T22
(iter 7), T7 (iter 10), T18 (iter 14), and T2 (iter 18, called out as
the fifth bundle in a separate sentence) — but not the iter-21 T21
bundle, even though the bundle exists durably under
`bench/runs/checkpoint-pi-T21-mdtools-gpt5.4mini-2026-04-26/` since
iter 21 and is materially relevant to the same-task table's coverage
gap framing. Cheap channel green before and after.

- **Disturbance:** specification coherence — published-narrative ↔
  bundle-existence drift. A reader of `bench/RESULTS.md` could not
  determine from the published prose that the iter-21 T21 PI bundle
  exists, even though the bundle is durably preserved under
  `bench/runs/`. Same shape as iter 19's pre-edit state: after iter 18
  produced the T2 PI bundle, RESULTS.md needed an explicit "fifth PI
  bundle… not yet eligible for this table" sentence (because no OAI
  T2 same-task `mdtools` cell existed). Iter 23 mirrors that pattern
  for the sixth PI bundle (T21).
- **Anchor:** *missing evaluator artifact — durable summary for a
  newly-run comparison*. Same anchor wording as iter 11 and iter 19;
  the intervention is the additive-measurement-publication shape (not
  corrective reference removal — iters 8/9/13/20 — and not a new table
  row, because no OAI T21 mdtools cell exists on file). The forcing
  function for choosing this anchor is the iter-21 entry's own
  "no cross-executor pair extension to the iter-19 same-task table is
  yet possible — content-delivery T2 is the same gap class" framing,
  which only became actionable at iter 23 because iter 22 was a
  ledger-only ratification iteration that did not touch published
  narrative.
- **Change shape:** one targeted edit to the trailing sentence of
  `bench/RESULTS.md:67`, appending a parallel-shape sixth-bundle
  sentence after the existing fifth-bundle (T2) sentence. The new
  sentence cites the iter-21 T21 PI bundle path, names it the
  **first** `compare_frontmatter_json` scorer-branch PI cell, names
  it the **second** durable bundle in `bench/runs/` carrying iter-17's
  `holdout_version: 1` stamp on `run.json` (after T2's first), and
  records the same not-yet-eligible-for-the-table caveat (no OAI
  same-task `mdtools` cell exists). All other RESULTS.md prose and
  table data unchanged. No edits to ledger lines outside this entry,
  to historical bundle artifacts, or to other published-narrative
  files.
- **Data points (iter-23 additions, source: typed artifacts):**
  - `bench/runs/checkpoint-pi-T21-mdtools-gpt5.4mini-2026-04-26/results.json`:
    `task_id=T21`, `mode=mdtools`, `correct=true`,
    `correct_neutral=true`, `tool_calls=1`, `mutations=0`,
    `bytes_observation=565`, `bytes_output=801,952`,
    `elapsed_seconds=6.7`, `diff_report="frontmatter_json: OK"`,
    `model="openai-codex/gpt-5.4-mini"`,
    `thinking_level="minimal"`. ✓ (re-confirmed by iter-22's
    bit-exact ratification on the same fields).
  - `bench/runs/checkpoint-pi-T21-mdtools-gpt5.4mini-2026-04-26/run.json`
    line 20: `"holdout_version": 1` — second durable bundle carrying
    the iter-17 stamp (after iter-18's T2 bundle, also on line 20).
  - Cross-executor T21 mdtools coverage gap on the OAI side: scanning
    `bench/runs/search-mdtools-*/results.json`,
    `bench/runs/holdout-mdtools-*/results.json`,
    `bench/runs/search-hybrid-*/results.json`,
    `bench/runs/holdout-hybrid-*/results.json` for any `task_id=T21`
    `mode=mdtools` row returns empty — no OAI same-task `mdtools`
    cell on file (also confirmed by iter-21's own pre-iteration
    scan).
- **Closure-discipline ratification of iter 22 paired with the
  publication:** the iter-22 entry's typed-artifact verifications
  re-checked through reading the same bundle files iter 22 cited:
  - `results.json` field bit-exactness: re-confirmed (the values in
    the iter-22 entry match the live bundle bit-exact).
  - `run.json` `holdout_version: 1` on line 20 in the T21 bundle and
    in the T2 bundle: re-confirmed (both bundles carry the field at
    line 20, matching iter-22's "second durable stamped bundle"
    claim).
  - `pi-audit.jsonl` 4-event histogram (`model_change=1`,
    `thinking_level_change=1`, `tool_call=1`, `tool_result=1`):
    re-confirmed by re-reading
    `logs/T21_mdtools_1777219293/pi-audit.jsonl`.
  - `bench/harness.py:1282` and `:1318` carrying
    `bytes_output = len(raw_stdout.encode())`: re-confirmed (both
    lines bit-exact in the current file). Iter-20's line-drift fix
    and iter-22's confirmation both still hold.
  - Forward-pointing corrections in iter 22 (iter-21 entry's
    "iter 22's link_destinations" → should be "iter 7's"; "line 18"
    → should be "line 20"): not silently amended in the iter-21
    entry per iter-15 discipline; re-verified that lines 54 and 61
    of the ledger still carry the original iter-21 prose with the
    typos preserved, and the iter-22 forward-pointer is the
    consumer-visible correction. No new defect surfaced during the
    iter-22 ratification.
- **Cheap channel:** green before and after (cargo: 24+32+37+16+0+0
  across integration suites; python: 68 tests OK across the 8
  spec-named modules; `harness.py --md-binary` dry-run all 24 tasks
  PASS dual-scorer).
- **Comparability framing:** the iter-23 published-narrative edit is
  an additive forward-pointing observation, not a measurement
  publication. The cross-executor same-task table at lines 58–63
  remains 4 rows (T1, T7, T22, T18) unchanged — no new row is added
  because no OAI T21 mdtools cell exists. The new sentence does not
  amend any pass-rate claim, does not change any cited number, does
  not bump `holdout_version` (still 1), and does not alter any data
  in any bundle. The sentence's role is to keep the published
  inventory of PI bundles aligned with the durable inventory in
  `bench/runs/`. The "no comparable OAI same-task `mdtools` cell on
  file" caveat is the same shape as the T2 caveat in the existing
  prose; future iterations could close the gap by producing OAI T2
  and OAI T21 mdtools cells (Qwen3.5-122B-A10B-4bit or another
  reachable model) — these would extend the same-task table to 5 or
  6 rows but are not iter 23's scope.
- **Same-family-rule discharge:** iter 21 was intervention-diversity
  (T21 PI bundle expensive run), iter 22 was closure-discipline
  ratification (ledger-only). Iter 23 is specification coherence
  (additive measurement publication, parallel to iter 19). This is
  the first specification-coherence move since iter 20 (line-drift
  fix), with two iterations between (iter 21 expensive, iter 22
  ledger-only ratification) — well clear of any concentration. The
  fresh-failing-trace escape clause additionally applies: the
  iter-21 T21 PI bundle has been sitting in `bench/runs/` since iter
  21 uncited in the cross-executor section, and the iter-21 entry's
  own "content-delivery T2 is the same gap class" framing was the
  pre-recorded forcing function that became actionable when iter 23
  surfaced the gap during routine reading. Parallel to iter 19's
  cashing out of iter-14's T18 PI bundle 5 iterations later (iters
  15, 16, 17, 18, 19); iter 23 cashes out iter-21's T21 PI bundle
  2 iterations later (iters 22, 23).
- **Closure-discipline status:** this is a substantive
  published-narrative edit authored by iter 23, parallel to iter 11
  and iter 19. Per the FIXED ≠ CLOSED rule, the entry is
  FIXED_PENDING_CONFIRMATION-shaped; a future review pass should
  ratify by re-reading the cited bundles' results.json and run.json
  files against the new sentence and confirming the "no OAI T21
  mdtools cell exists" claim by scanning the OAI bundles. No
  retroactive edits to historical bundles or to iter 19's original
  fifth-bundle sentence; iter 23's entry forward-points to the
  appended sixth-bundle sentence it authored.
- **What this does NOT do:** does not promote any product anchor
  (`bench/probes/anchor-validation/` still does not exist, no
  candidate primitive validated). Does not bump `holdout_version`
  (still 1). Does not run the expensive outer channel (the addition
  is entirely from existing typed artifacts, all already preserved).
  Does not retroactively modify any existing bundle or any
  historical ledger entry. Does not amend any pass-rate claim,
  any ratio, any rule conclusion, or any historical citation. Does
  not extend the cross-executor same-task table (no new same-task
  pair was made available since iter 19). No new finding opened, no
  holdout artifact touched.

### Confirmation review pass (2026-04-26 iter 22)

Closure-discipline review of iter-21's `comparable-harness-axis cell
coverage extension` (T21 mdtools PI bundle, frontmatter_json scorer
branch). Parallels iter 15's review of iter-14's quiet-signal-checkpoint
discharge (also a non-finding bundle introduction): typed-artifact
claims in the iter-21 entry are checked against the underlying bundle
files. Differs from iter 15 in that two fresh failing traces surfaced
during verification — both are citation errors **inside the iter-21
ledger entry itself**, recorded forward-pointing here per iter-15's
"do not silently edit historical entries" discipline. Cheap channel
green at review time (`cargo test --quiet` all suites pass:
24+32+37+16+0+0; 68 python unittests OK across the 8 spec-named
modules; `harness.py --md-binary` dry-run all 24 tasks PASS
dual-scorer).

What was checked:

- **Bundle metrics in `results.json`** —
  `bench/runs/checkpoint-pi-T21-mdtools-gpt5.4mini-2026-04-26/results.json`
  re-read. Every iter-21-published number matches bit-exact:
  `task_id=T21`, `mode=mdtools`, `correct=true`, `correct_neutral=true`,
  `elapsed_seconds=6.7`, `tool_calls=1`, `turns=2`, `mutations=0`,
  `requeried=false`, `bytes_observation=565`, `bytes_output=801952`,
  `policy_violations=0`, `invalid_responses=0`,
  `unique_invalid_responses=0`, `diff_report="frontmatter_json: OK"`,
  `runner_error=null`, `model="openai-codex/gpt-5.4-mini"`,
  `thinking_level="minimal"`.
- **Run metadata in `run.json`** — re-read. Confirms
  `runner=pi-json`, `executor=guarded`, `model=openai-codex/gpt-5.4-mini`,
  `thinking_level=minimal`, `runs_per_task=1`, `modes=["mdtools"]`,
  `selected_task_ids=["T21"]`, `holdout_version=1`. The
  `holdout_version` field is on **line 20** of the file (not line 18 as
  the iter-21 entry claims; see correction below). Aggregates section
  reproduces the same numbers as the per-result entry.
- **Pi-audit JSONL event count and shape** —
  `logs/T21_mdtools_1777219293/pi-audit.jsonl` has exactly 4 lines.
  Event-type histogram: `{model_change: 1, thinking_level_change: 1,
  tool_call: 1, tool_result: 1}`. Matches the iter-21 entry's "4
  events: `model_change`, `thinking_level_change`, `tool_call`,
  `tool_result`" claim exactly.
- **Tool-call command** — the single `tool_call` event carries
  `input.command="./md frontmatter
  /var/folders/sw/.../t21_frontmatter.md --json"`. Matches iter-21's
  "(`./md frontmatter <input> --json`)" enumeration with `<input>`
  abstracting the temp-dir path. The single `tool_result` event has
  `outputBytes=565` matching `bytes_observation=565` exactly.
- **Adapter summary** —
  `bench.pi_audit_adapter.summarize_pi_audit_events` invoked on the
  parsed event list returns `PiAuditCounters(tool_calls=1,
  tool_results=1, tool_errors=0, bytes_observation=565, mutations=0,
  requeried=False, policy_violations=0,
  model='openai-codex/gpt-5.4-mini', thinking_level='minimal')`. Every
  reported counter matches the iter-21 entry's enumeration.
- **`compare_frontmatter_json` corpus uniqueness** — re-verified by
  scanning `bench/tasks/tasks.json` for tasks where
  `scorer.compare_frontmatter_json is True`: result `['T21']`. T21 is
  the only task with that flag set; the iter-21 claim "first PI bundle
  exercising the `compare_frontmatter_json` scorer branch" reproduces.
- **Iter-21's implicit ratification of iter 20** —
  `bench/harness.py:1282` and `:1318` re-read in the current file:
  both lines carry `bytes_output = len(raw_stdout.encode())` bit-exact.
  Iter-20's `RESULTS.md:54` line-citation correction (1229/1265 →
  1282/1318) remains valid; iter-21's implicit ratification holds
  through iter-22 verification.
- **Second durable stamped bundle claim** — re-verified that
  `bench/runs/checkpoint-pi-T2-mdtools-gpt5.4mini-2026-04-26/run.json`
  has `holdout_version: 1` on line 20 (the iter-18 first stamped
  bundle), and `bench/runs/checkpoint-pi-T21-mdtools-gpt5.4mini-2026-04-26/run.json`
  has `holdout_version: 1` on line 20 (the iter-21 second stamped
  bundle). The iter-21 claim "second durable bundle in `bench/runs/`
  carrying the iter-17 stamp on a real benchmark cell (after iter
  18's T2 bundle)" reproduces.

Forward-pointing corrections to iter-21 ledger entry (per iter-15
"never silently edit historical entries" discipline; cross-references
preserved instead of inline edits):

- **At `bench/ledger.md:54`** — the iter-21 entry text reads
  "parallel to iter 14's first raw_bytes coverage and **iter 22's**
  link_destinations coverage." This is a clerical typo: the
  T22-link_destinations PI bundle was introduced in **iter 7**, not
  iter 22. Iter 22 is the current iteration (this confirmation review
  pass) and produced no link_destinations bundle. The phrase should
  read "iter 7's link_destinations coverage" (matching the established
  pattern "iter X's Y coverage" at line 53). Verified by reading the
  iter-7 ledger entry "Quiet-signal checkpoint discharge (2026-04-26
  iter 7)" which describes the T22-mdtools PI bundle exercising the
  post-F3 `compare_link_destinations` envelope normalization.
- **At `bench/ledger.md:61–62`** — the iter-21 entry text reads
  "`run.json` line **18** reads `\"holdout_version\": 1`". This is an
  off-by-two error: in
  `bench/runs/checkpoint-pi-T21-mdtools-gpt5.4mini-2026-04-26/run.json`
  the `holdout_version` field is on **line 20**, not line 18. Line 18
  reads `"thinking_level": "minimal"`. A reader following the iter-21
  citation would land on the wrong field. The iter-17, iter-18, and
  iter-19 ledger entries (lines 362, 460, 539) all correctly cite
  "line 20" for the same field on the same shape of run.json.
  Verified by `grep -n "holdout_version"` against the bundle file
  and the iter-21 commit (`6a81800`), confirming line 20 was the
  position both at commit time and currently.

Rationale for forward-pointing only:

Per iter 15 ("Editing historical ledger entries silently is incorrect
amendment discipline; recording corrections in the current iteration's
entry is the correct pattern" — second-opinion ratified at 0.9
confidence), the iter-21 entry text is preserved unchanged. Both
corrections live in this iter-22 entry as cross-referenced
observations. Future readers of the iter-21 entry should consult this
iter-22 forward-pointer when reading lines 54 and 61–62.

Verdict — iter-21 comparable-harness-axis cell coverage extension
ratified, but with two ledger-internal citation corrections recorded
forward-pointing. The bundle data itself (`results.json`, `run.json`,
`pi-audit.jsonl`, `guard.log`, `task_ids.json`) reproduces bit-exact
against every claim in the iter-21 entry; only the cite-line numbers
within the iter-21 narrative prose contain typos. The closure-discipline
rule's "next pass not re-raising the finding" criterion is satisfied
for the bundle-introduction half; the citation typos are the kind of
fresh failing trace that previous iterations (iter 12, iter 13, iter
20) surfaced during ratification, here recorded as forward-pointing
observations rather than direct edits because the trace is in the
ledger (auxiliary memory) rather than the published narrative
(consumer-facing). No new finding opened, no holdout artifact touched,
no published-narrative file edited.

- **Frontier anchor (review pass):** *closure-discipline rule applied
  to bundle introduction* + *fresh failing traces in iter-21 entry*.
  Iter 21 made specific typed-artifact claims (event count, tool-call
  command, adapter counters, scorer-branch uniqueness, second-stamped-
  bundle assertion, line citations to the bundle) that needed
  independent verification. Iter 22 discharges this by reading typed
  artifacts (results.json + run.json + pi-audit.jsonl +
  bench/tasks/tasks.json + adapter output + bench/harness.py:1282,1318
  + iter-7 ledger entry) rather than narrative.
- **Same-family discharge:** iters 19–20 were two consecutive
  spec-coherence iterations (publication, line-drift fix +
  ratification); iter 21 broke the chain with an expensive-channel
  frontmatter_json-coverage bundle. Iter 22 returns to a ledger-only
  ratification entry — the same-family rule's concentration was
  already broken by iter 21, so a single ratification entry is
  admissible. Differs from iter 15 (no fresh trace) in that iter 22
  surfaces two citation typos (parallel to iter 12's argparse-typo
  and iter 13/20's line-drift corrections), but the corrections are
  recorded forward-pointing rather than as direct edits because they
  are in the ledger (auxiliary memory), not in published narrative.
- **Comparability framing:** the ratification is a ledger-only
  verification with two ledger-internal forward-pointing corrections.
  It does not change any data, ratio, rule conclusion, pass rate,
  bundle, scorer, or holdout artifact. It does not bump
  `holdout_version` (still 1). It does not run the expensive outer
  channel. It does not edit any published-narrative file
  (`bench/RESULTS.md`, `README.md`, `CLAUDE.md`,
  `bench/retracted_2026-04-24/README.md`, `specs/**`). The only file
  modified is `bench/ledger.md` itself, with two additions: this
  iter-22 entry and an updated halt-condition / quiet-signal status
  block.
- **Closure-discipline status:** this is a non-finding ledger-only
  ratification entry, parallel to iter 15. Per iter-15's labeling
  discipline, this is **not frontier expansion** in the sense of
  evaluator repair, product change, or measurement publication. It
  procedurally ratifies iter 21's typed-artifact claims and records
  two forward-pointing citation corrections. The 18th consecutive
  zero-OPEN review round holds.
- **What this does NOT do:** does not promote any product anchor
  (`bench/probes/anchor-validation/` still does not exist, no
  candidate primitive validated). Does not bump `holdout_version`
  (still 1). Does not run the expensive outer channel. Does not edit
  any historical ledger entry inline. Does not amend any pass-rate
  claim. Does not modify any bundle or any published-narrative file.
  Does not touch any holdout artifact. Does not extend the
  cross-executor same-task table (no new same-task pair was made
  available by iter-21 — T21 has no oai-loop counterpart on file,
  per iter-21's own framing).

### Comparable-harness-axis cell coverage extension (2026-04-26 iter 21)

Iter 21 broke the iters 19–20 specification-coherence chain by running
the expensive outer channel on a previously-uncovered scorer branch:
T21 mdtools through the PI runner. This is the **sixth** PI runner
bundle in `bench/runs/` and the **first** cell that exercises the
`compare_frontmatter_json` structural scorer branch end-to-end through
the PI executor. Parallel in structural position to iter 10
(proactive intervention shift before the forced-expensive point) and
parallel in shape to iter 4 / 7 / 10 / 14 / 18 (PI bundle introductions
that extend executor coverage to a new family or scorer branch).

- **Disturbance:** intervention diversity — drifting toward
  spec-coherence concentration after iters 19 and 20 both did
  specification-coherence work (iter 19 additive measurement
  publication, iter 20 corrective line-number drift fix paired with
  iter-19 ratification). A third consecutive same-axis move at iter 21
  would extend the chain to clear concentration; the same-family rule
  required either an axis shift, a fresh failing trace, or halt with
  `stop-and-summarize`. Iter 21's pre-iteration verification swept all
  `bench/harness.py:LINE` references in published narrative
  (`bench/RESULTS.md`, `README.md`, `CLAUDE.md`, `specs/**`,
  `bench/retracted_2026-04-24/README.md`) and all
  `bench/oai_loop.py` / `bench/pi_audit_adapter.py` references in the
  ledger — every citation reproduces bit-exact against the current
  code. No fresh failing trace surfaced. With no fresh trace and a
  cheap, anchored intervention available, the axis shift is the
  cleanest discharge.
- **Anchor:** missing evaluator artifact — *first PI bundle exercising
  the `compare_frontmatter_json` scorer branch end-to-end*. T21 is the
  only task in the live 24-task corpus that uses
  `compare_frontmatter_json: true`; before iter 21, the PI runner had
  exercised four distinct scorer branches across five bundles
  (T1: structural+heading_tree; T7/T2: normalized_text;
  T18: raw_bytes/file_contents; T22: structural+link_destinations).
  Iter 21's bundle adds the fifth distinct scorer-branch coverage
  cell (`structural+frontmatter_json`), parallel to iter 14's first
  raw_bytes coverage and iter 22's link_destinations coverage. (Halt
  was defensible too, but premature given the available cheap,
  anchored intervention.)
- **Bundle:** `bench/runs/checkpoint-pi-T21-mdtools-gpt5.4mini-2026-04-26/` —
  Single task (T21, search-split, frontmatter-extraction). Single mode
  (mdtools). Single run. Model `openai-codex/gpt-5.4-mini` at
  `thinking_level=minimal`, recorded per-result and per-run on the
  metadata bundle. `run.json` line 18 reads
  `"holdout_version": 1` — the second durable bundle in `bench/runs/`
  carrying the iter-17 stamp on a real benchmark cell (after iter
  18's T2 bundle).
- **Verdict:** T21 mdtools dual-scorer PASS in 6.7s with 1 tool call
  (`./md frontmatter <input> --json`), 0 mutations, 0
  policy_violations, `requeried=false`, `bytes_observation=565`,
  `bytes_output=801,952` (PI streaming overhead, see P3 cross-executor
  rule in `bench/RESULTS.md`),
  `diff_report: frontmatter_json: OK`. Pi-audit log preserved at
  `logs/T21_mdtools_1777219293/pi-audit.jsonl` (4 events:
  `model_change`, `thinking_level_change`, `tool_call`,
  `tool_result`), parses cleanly via
  `bench/pi_audit_adapter.summarize_pi_audit_events` with
  `tool_calls=1`, `tool_results=1`, `tool_errors=0`,
  `bytes_observation=565`, `mutations=0`, `requeried=False`,
  `policy_violations=0`, `model='openai-codex/gpt-5.4-mini'`,
  `thinking_level='minimal'` — all bit-exact against
  `results.json`.
- **Comparability framing:** this is the first cell for (PI runner,
  gpt-5.4-mini, mdtools, minimal thinking, T21, runs_per_task=1,
  task-set version: live `bench/tasks/tasks.json` with
  `holdout_version=1` from `bench/holdout/fingerprints.json`). It is
  **NOT** a holdout reconfirmation (T21 is search-split, not
  holdout) and **NOT** a comparison against the iter-4 T1, iter-7
  T22, iter-10 T7, iter-14 T18, or iter-18 T2 bundles — same
  executor / model / mode / thinking / runs-per-task across all six,
  but different tasks and different scorer-branch slices, so any
  pass-rate-aggregation across cells would be a search-set
  observation, not a comparison. Likewise it is **NOT** an
  apples-to-apples comparison to any oai-loop T21 bundle (none
  exists on file; verified by scanning `search-mdtools-*`,
  `search-hybrid-*`, `holdout-mdtools-*`, `holdout-hybrid-*` for
  any T21 result row), so no cross-executor pair extension to the
  iter-19 same-task table is yet possible — content-delivery T2 is
  the same gap class.
- **What this exercises:** for the first time in `bench/runs/`, the
  PI runner pipeline (harness pi-json branch → `pi --mode json` →
  audit extension at `~/.pi/agent/extensions/audit/index.ts` →
  `bench/pi_audit_adapter.summarize_pi_audit_events`) is verified
  end-to-end on a `compare_frontmatter_json` structural scorer
  cell. The single-tool-call shape matches T1 and T22 (zero-mutation
  extraction) but the scorer branch is unique. The `md frontmatter
  --json` command surface is also exercised end-to-end through PI
  for the first time — prior PI bundles used `outline`, `links`,
  `tasks`, `set-task`, `delete-section`, `blocks`, `insert-block`,
  `cat`.
- **Implicit ratification of iter 20:** per the closure-discipline
  rule (FIXED ≠ CLOSED) "or the next pass not re-raising the
  finding," iter 21's pre-iteration verification re-read
  `bench/harness.py:1282` and `:1318` (iter-20's corrected
  citations) and confirmed both lines carry
  `bytes_output = len(raw_stdout.encode())` bit-exact:
  - Line 1282 sits in the pi-json runner branch immediately after
    `raw_stdout = result.stdout` inside the `try:` block following
    `subprocess.run([…pi --mode json…])`.
  - Line 1318 sits in the oai-loop branch (the `else:` after the
    pi-json branch's TimeoutExpired/parsed_output sequence)
    immediately after `raw_stdout = result.stdout` inside the
    `try:` block following `subprocess.run([…agent_cmd…])`.
  No re-raise. Iter 20's
  `FIXED_PENDING_CONFIRMATION`-shaped substantive
  published-narrative edit at `bench/RESULTS.md:54` is now
  ratified. Parallel to iter 4's implicit ratification of iters 1–3
  via expensive run, and iter 14's implicit ratification of iter 13
  via expensive run.
- **What this discharges:** intervention-diversity drift, parallel
  to iter 10. The spec-coherence axis was the most recent move
  (iters 19, 20); iter 21 cleanly shifts to intervention-diversity /
  failure-legibility. Iter 22 is now newly admissible as quiet
  under the reset counter; the next forced expensive-or-halt is
  iter 24 unless other rules fire.
- **What it surfaced:** no new defect. The PI pipeline produced
  fresh typed signal that exercised the frontmatter_json scorer
  branch cleanly. This is a "no new finding" expensive run,
  admissible as fresh signal because the bundle is on a different
  (task, scorer-branch) cell than iters 4 / 7 / 10 / 14 / 18, and
  the audit log + scorer outputs are durably persisted as a
  queryable bundle.
- **Cheap channel:** green before and after (`cargo test --quiet`
  all suites pass: 24 + 32 + 37 + 16 + 0 + 0 across integration-test
  binaries; 68 python unittests OK across the 8 spec-named modules;
  `harness.py --md-binary` dry-run all 24 tasks PASS dual-scorer).
- **Closure-discipline status:** this is a non-finding bundle
  introduction, not a substantive code change, parallel to iter 4 /
  7 / 10 / 14 / 18 entries. Per iter-15's labeling discipline, this
  is **not** frontier expansion in the sense of evaluator repair or
  product change — it is durable PI executor coverage that fills a
  scorer-branch gap. The iter-21 entry is forward-pointing only;
  no historical entry is silently amended.
- **Same-family-rule discharge:** iter 19 was specification
  coherence (additive measurement publication), iter 20 was
  specification coherence (corrective line-drift fix paired with
  iter-19 ratification), iter 21 is intervention-diversity
  (expensive channel introducing fresh signal on a previously-
  uncovered scorer branch). The shift from spec-coherence
  substantive narrative edits to intervention-diversity is itself
  the discharge — parallel to iter 4 (after iters 1–3 oracle), iter
  10 (after iters 8–9 spec-coherence), iter 14 (after iters 11–13
  spec-coherence). The forcing function is the iter-19/20 chain
  approaching concentration without a fresh trace; the cheapest
  axis-shift available is the T21 PI run.
- **What this does NOT do:** does not promote any product anchor
  (`bench/probes/anchor-validation/` still does not exist, no
  candidate primitive validated). Does not bump `holdout_version`
  (still 1). Does not modify any prior bundle. Does not amend any
  pass-rate claim. Does not retroactively edit any historical
  ledger entry.

### Confirmation review pass (2026-04-26 iter 20)

Closure-discipline review of iter-19's cross-executor measurement
extension paired with a corrective line-number drift fix surfaced
during the verification: `bench/RESULTS.md:54` cited
`bench/harness.py:1229` (pi-json) and `:1265` (oai-loop) for the
`bytes_output = len(raw_stdout.encode())` lines, but those lines are
now at 1282 and 1318 — a 53-line forward drift introduced by iter-16's
`check_holdout_integrity` wrapper (~30 new lines at 747-775 + main()
wiring at 1597-1599) and iter-17's `read_holdout_version` helper
(~17 new lines at 778-794 + `build_run_metadata` modifications). A
reader following the published citation `bench/harness.py:1229` would
land on `workdir=workdir_path,` (an oai_loop kwarg) instead of the
bytes_output computation; `:1265` would land on
`session_dir=pi_session_dir,` (a pi-json kwarg). Same shape as iter
13's stale `bench/harness.py:339-341` → `347-348` fix at RESULTS.md:152
for the F3-a rstrip body, with the same drift-after-upstream-edits
origin pattern.

- **Disturbance:** specification coherence — line-number drift in
  published-narrative typed-artifact pointer. The pointer was added at
  iter 5 (commit `b10d8b8`) when the lines were genuinely at 1229 and
  1265; the iter-16/17 oracle hardening work added code above them
  without any line-citation update. This is structurally identical to
  iter 13's `RESULTS.md:152` drift caused by edits before line 339-341.
- **Anchor:** *fresh failing trace* — the citations point to wrong
  lines in the current `bench/harness.py`. A reader following the
  published references lands on unrelated kwargs, not the bytes_output
  computation the prose describes. Verified by reading the actual lines
  at the cited offsets and the actual `bytes_output = len(raw_stdout.encode())`
  body in the current file.
- **Change shape:** one targeted edit to `bench/RESULTS.md:54`,
  replacing `bench/harness.py:1229` with `:1282` (pi-json) and `:1265`
  with `:1318` (oai-loop). All other prose unchanged.
- **Drift origin (verified by git history):**
  - iter 5 commit `b10d8b8`: `bench/harness.py` lines 1229 (pi-json)
    and 1265 (oai-loop) carried `bytes_output = len(raw_stdout.encode())`,
    matching the published citation.
  - iter 16 commit `75115c7`: added `check_holdout_integrity` wrapper
    at `bench/harness.py:747-775` (28 new lines) plus 3-line
    `parser.error(...)` block at `:1597-1599`. Cumulative drift on
    bytes_output lines: ~31 lines forward.
  - iter 17 commit `7b36502`: added `read_holdout_version` at
    `:778-794` (17 new lines) plus modifications to `build_run_metadata`
    signature/body and three call-site `holdout_version=...` arguments
    in main() at lines around 1648, 1714, 1773. Cumulative drift on
    bytes_output lines: ~53 lines forward (1229 → 1282; 1265 → 1318).
- **Closure-discipline ratification of iter 19 paired with the
  fix:** independent re-reading of every iter-19 typed-artifact claim
  against the cited bundles:
  - All four published table rows reproduce bit-exact from the
    bundles' `results.json` files: T1 (PI 1/0, OAI 1/0,
    bytes_output 5,975,843 vs 2,702, bytes_observation 2,266 vs
    2,436 — ratio 2,211.64× → published ~2,212×); T7 (PI 3/1, OAI
    3/1, 1,172,040 vs 699, 16,219 vs 13,671 — 1,676.74× →
    ~1,677×); T22 (PI 1/0, OAI 2/0, 671,515 vs 488, 514 vs 1,036 —
    1,376.06× → ~1,376×); T18 (PI 10/2, OAI 5/2, 844,124 vs 812,
    14,858 vs 6,015 — 1,039.56× → ~1,040×). ✓
  - PI bundle pointers list correctly enumerates iters 4, 7, 10, 14
    in the table and cites iter 18 separately as fifth PI bundle. ✓
  - OAI bundle pointers list correctly enumerates the four cells
    used (extraction-122B for T1, mutation-122B for T7,
    holdout-122B for T22, multistep-27B for T18). ✓
  - Iter 19's claim that no comparable OAI same-task `mdtools` cell
    exists for T2: verified by scanning all `search-mdtools-*` and
    `holdout-mdtools-*` and `search-hybrid-*` and `holdout-hybrid-*`
    bundles for any T2 result row — none exist. ✓
  - Iter 19's data points (`bytes_output=844,124` and
    `bytes_observation=14,858` for PI T18; `bytes_output=812` and
    `bytes_observation=6,015` for OAI T18) reproduce bit-exact from
    `bench/runs/checkpoint-pi-T18-mdtools-gpt5.4mini-2026-04-26/results.json`
    and `bench/runs/search-mdtools-multistep-Qwen3.5-27B-4bit-2026-04-21/results.json`. ✓
  All claims reproduce bit-exact. The drift fix surfaced during
  cross-checking the published prose against the actual file
  surface — no claim in iter 19's data needed correction; only the
  iter-5-era pointer citations.
- **Cheap channel:** green before and after (cargo: 32+37+16+0 across
  integration suites; python: 68 tests OK across the 8 spec-named
  modules; `harness.py --md-binary` dry-run all 24 tasks PASS
  dual-scorer).
- **Comparability framing:** this is a corrective specification
  coherence change. It does **not** change any data, any pass rate,
  any executor behavior, any bundle, or any scorer. It does **not**
  bump `holdout_version` (still 1). It does **not** run the expensive
  outer channel. It does **not** introduce or modify any product
  surface. The only edit is the line-number citation in the published
  narrative. Per the spec's "telemetry-only instrumentation" /
  "harness assertions" / "specification coherence" allowances, the
  change is squarely within the admissible work envelope. The
  closure-discipline ratification half discharges iter 19's invitation
  to "ratify by re-reading the cited bundles' results.json files
  against the new table row and the updated bundle-pointer list" —
  every datum reproduces bit-exact.
- **Same-family-rule discharge:** iter 19 was specification coherence
  (additive measurement publication); iter 20 is also specification
  coherence (corrective reference fix). Two same-axis moves in a row
  is borderline, but iter 20 cites a fresh failing trace — the
  drifted line-number citations at RESULTS.md:54 are a real defect a
  reader following the published reference would hit, not a cosmetic
  refresh. Per the same-family rule's "cite a fresh failing trace,
  external finding, or blocked claim" escape clause, the trace makes
  the same-axis move admissible. Structurally identical to iter 13's
  pairing of line-drift fix with iter-12 ratification, which itself
  paired typo fix with iter-11 ratification.
- **Closure-discipline status:** this is a substantive
  published-narrative edit authored by iter 20, parallel to iter 13's
  RESULTS.md:152 fix. Per the FIXED ≠ CLOSED rule, the entry is
  `FIXED_PENDING_CONFIRMATION`-shaped; a future review pass should
  ratify by re-reading `bench/harness.py:1282` and `:1318` against
  this entry's claims and verifying both lines carry
  `bytes_output = len(raw_stdout.encode())`.
- **What this does NOT do:** does not promote any product anchor
  (`bench/probes/anchor-validation/` still does not exist, no
  candidate primitive validated). Does not bump `holdout_version`
  (still 1). Does not run the expensive outer channel. Does not
  amend any pass-rate claim. Does not retroactively modify any
  bundle or any historical ledger entry. Does not touch any holdout
  artifact.

### Cross-executor same-task measurement extension (2026-04-26 iter 19)

Iter 19 cashed out iter 14's PI T18 multistep bundle by extending the
iter-11 cross-executor same-task validation table in `bench/RESULTS.md`
from 3 rows to 4, and updated the PI bundle pointers list to enumerate
iter 14 (T18) and iter 18 (T2 — noted as having no comparable OAI
same-task `mdtools` cell on file). Parallel in shape to iter 11: both
iterations cash out a previously-uncited PI bundle's downstream pairing
potential into the published narrative.

- **Disturbance:** specification coherence — `bench/RESULTS.md`
  cross-executor section listed only 3 same-task pairs (T1, T7, T22)
  and 3 PI bundle pointers (iters 4, 7, 10), but the repo now contains
  5 PI bundles (iters 4, 7, 10, 14, 18). The iter-14 T18 multistep
  bundle pairs with two pre-existing 2026-04-21 OAI multistep T18
  bundles (Hermes-4-70B-4bit, Qwen3.5-27B-4bit), forming an unincorporated
  fourth same-task cross-executor pair. Iter 11's learning #1 explicitly
  invited "Future expensive-channel runs should be examined for
  downstream pairing potential, not just cell-coverage credit"; iter 14
  produced its bundle but did not extend the published table, and iters
  15–18 did not either. Iter 18's T2 has no comparable OAI same-task
  cell so it is not eligible for the table, but it should still be
  cited as the fifth PI bundle and the first durable bundle carrying
  iter-17's `holdout_version: 1` stamp.
- **Anchor:** missing evaluator artifact — *durable summary for a
  newly-run comparison*. Same anchor wording as iter 11; the
  intervention is the same shape (additive measurement publication,
  not corrective reference removal). The forcing function for choosing
  this anchor is iter 11's pre-recorded learning #1 invitation, which
  only became actionable after iter 14's T18 PI bundle landed and
  stayed unincorporated through iters 15–18.
- **Change shape:** one targeted edit to `bench/RESULTS.md` (the
  "Same-task validation" block at lines 56–66):
  - Caption updated from "(2026-04-26 iter 11)" to "(2026-04-26 iters
    11, 19)" and "Three" to "Four"; model caveat extended to note
    T18's OAI cell uses Qwen3.5-27B-4bit (because no `-122B` multistep
    cell exists).
  - One row appended to the table: T18 with PI `10 / 2` vs OAI `5 / 2`
    (tool-calls / mutations), PI `bytes_output=844,124` vs OAI `812`
    (~1,040× ratio), PI `bytes_observation=14,858` vs OAI `6,015`.
  - Commentary updated from "across all three pairs" to "across all
    four pairs"; the bytes_observation scaling clause extended to note
    the T18 PI cell issued 2× as many reads as the T18 OAI cell with
    bytes_observation ~2.47× larger correspondingly.
  - PI bundle pointers list extended to include iter 14 (T18); OAI
    bundle pointers list extended to include
    `bench/runs/search-mdtools-multistep-Qwen3.5-27B-4bit-2026-04-21/`
    (T18).
  - Iter-18 T2 PI bundle cited explicitly as the fifth PI bundle, with
    the published note that no comparable OAI same-task `mdtools` cell
    exists so it is not yet eligible for the table; T2 is the first
    durable bundle in `bench/runs/` carrying iter-17's
    `holdout_version: 1` stamp on `run.json`.
- **Data points (iter-19 additions, source: typed artifacts):**
  - T18 mdtools PI (iter 14): `tool_calls=10`, `mutations=2`,
    `bytes_output=844,124`, `bytes_observation=14,858` from
    `bench/runs/checkpoint-pi-T18-mdtools-gpt5.4mini-2026-04-26/results.json`.
  - T18 mdtools OAI Qwen3.5-27B-4bit: `tool_calls=5`, `mutations=2`,
    `bytes_output=812`, `bytes_observation=6,015` from
    `bench/runs/search-mdtools-multistep-Qwen3.5-27B-4bit-2026-04-21/results.json`.
  - Ratios: bytes_output 844,124 / 812 = 1,039.56 → ~1,040×;
    bytes_observation 14,858 / 6,015 = 2.470 → ~2.47×. Both consistent
    with iter 11's published rule (1,000–4,000× bytes_output ratio
    range; bytes_observation scales with tool-call count when it
    differs).
- **Closure-discipline ratification of iter 18 paired with the
  publication:** independent re-reading of every iter-18 typed-artifact
  claim against
  `bench/runs/checkpoint-pi-T2-mdtools-gpt5.4mini-2026-04-26/`:
  - `results.json`: `correct=true`, `correct_neutral=true`,
    `elapsed_seconds=17.72`, `tool_calls=4`, `mutations=1`,
    `policy_violations=1`, `requeried=true`, `bytes_observation=732`,
    `bytes_output=1,811,504`. ✓
  - `run.json`: line 20 reads `"holdout_version": 1` alongside the
    existing 15 metadata keys (the first durable bundle in
    `bench/runs/` carrying the iter-17 stamp). ✓
  - `logs/T2_mdtools_1777217027/pi-audit.jsonl`: 10 events parse
    cleanly via `bench.pi_audit_adapter.summarize_pi_audit_events` —
    1 `model_change` + 1 `thinking_level_change` + 4 `tool_call` + 4
    `tool_result`. ✓
  - `logs/T2_mdtools_1777217027/guard.log`: 5 events via
    `bench.command_policy.load_guard_events` — 1 deny `printf`, 4
    allows (allow `md ./md blocks`, allow `cat` heredoc, allow `md
    ./md insert-block --from`, allow `md ./md blocks`). ✓
  All claims reproduce bit-exact. No new defect surfaced during
  verification. The cross-counter measurement note in iter 18's entry
  on the adapter `policy_violations` field stands as written
  (informational-only); per iter 15 learning #4 and the iter-15
  second-opinion ratification, no silent amendment is authored to
  historical entries.
- **Cheap channel:** green before and after (cargo: 32+37+16+0 across
  integration suites; python: 68 tests OK across the 8 spec-named
  modules; `harness.py --md-binary` dry-run all 24 tasks PASS
  dual-scorer).
- **Comparability framing:** the iter-19 table extension preserves the
  iter-11 caveats — model-confounded across each pair, `correct` not
  aggregated, behavioral consistencies reported as observations
  rather than comparisons. The new T18 row introduces additional
  model variance (Qwen3.5-27B-4bit OAI for T18, vs Qwen3.5-122B-A10B-4bit
  for the other three rows), explicitly disclosed in the caption. The
  iter-19 row strengthens iter 11's executor-locality finding by
  widening the empirical base to 4 task families (extraction,
  mutation, link extraction, multi-step) without weakening the rule
  (~1,040× still well within the 10³–10⁴ envelope from iter 11).
- **Same-family-rule discharge:** iter 16 was oracle hardening
  (runtime guard), iter 17 was oracle hardening (holdout_version
  stamping), iter 18 was the expensive channel (T2 PI bundle). Iter
  19's specification-coherence move (additive measurement
  publication) is not same-family with any of those — the
  specification-coherence axis was last touched at iter 13
  (line-number drift correction), so this is a fresh axis from the
  6-iteration perspective. The fresh-failing-trace escape clause
  additionally applies: the iter-14 T18 PI bundle has been sitting in
  the repo since iter 14 uncited in the cross-executor section, and
  iter 11's learning #1 was the pre-recorded forcing function.
- **Closure-discipline status:** this is a substantive
  published-narrative edit authored by iter 19, parallel to iter 11.
  Per the FIXED ≠ CLOSED rule, the entry is
  FIXED_PENDING_CONFIRMATION-shaped; a future review pass should
  ratify by re-reading the cited bundles' results.json files against
  the new table row and the updated bundle-pointer list. No
  retroactive edits to historical bundles or to iter 11's original
  entry; iter 19's entry forward-points to the change it authored.
- **What this does NOT do:** does not promote any product anchor
  (`bench/probes/anchor-validation/` still does not exist, no
  candidate primitive validated). Does not bump `holdout_version`
  (still 1). Does not run the expensive outer channel (the additions
  are entirely from existing typed artifacts). Does not retroactively
  modify any existing bundle or any historical ledger entry. Does
  not amend any pass-rate claim — the table reports per-cell
  behavioral measurements (`bytes_output`, `bytes_observation`,
  `tool_calls`, `mutations`), not pass rates. No new finding opened,
  no holdout artifact touched.

### Quiet-signal checkpoint discharge (2026-04-26 iter 18)

Per the spec's "After 3 consecutive iterations with the cheap channel
green, no new failing trace, and no new finding added" rule flagged at
the end of iteration 17, iter 18 ran the expensive outer channel to
introduce fresh typed signal — and incidentally cashed out iter-17's
holdout-version stamping work as the first durable typed artifact in
`bench/runs/` that carries the new field. Cheap channel re-verified
green before and after the run.

- **Disturbed axis:** intervention diversity / failure legibility —
  the quiet-signal counter at 3 after iter 17 forced an
  expensive-channel run independently of any fresh failing trace.
- **Frontier anchor:** *missing evaluator artifact — first PI bundle
  carrying `holdout_version: 1` in `bench/runs/`*. The four pre-iter-17
  PI bundles (T1, T7, T18, T22) deliberately remain unstamped (iter
  17's "Does not modify any prior bundle" carve-out), and the iter-17
  end-to-end proof was a `/tmp` dry-run not preserved as a checkpoint.
  Iter 18's run produces the first durable typed artifact under
  `bench/runs/` that carries the spec-mandated stamp on a real
  benchmark cell. Parallel to iter 4 (first PI bundle), iter 7 (first
  holdout PI bundle), iter 10 (first mutation PI bundle), iter 14
  (first multistep PI bundle).
- **Bundle:** `bench/runs/checkpoint-pi-T2-mdtools-gpt5.4mini-2026-04-26/`
  — fifth PI runner bundle in this repo and the first
  content-delivery-family + `--from` + agent-recovery-from-policy-deny
  cell. T2 mdtools dual-scorer PASS in 17.72s with 4 tool calls, 1
  mutation, 1 policy_violation, requeried=true,
  `bytes_observation=732`, `bytes_output=1,811,504`. `diff_report:
  block_order [md]: OK; block_text [md]: OK; block_text [neutral]: OK`.
  pi-audit.jsonl preserves 10 events (model_change +
  thinking_level_change + 4×tool_call + 4×tool_result), parses
  cleanly via `bench/pi_audit_adapter.summarize_pi_audit_events`.
  guard.log records 5 entries: 1 deny on the first attempt's `printf`
  prefix (stdin pipe), 4 allows including the recovered
  `--from /tmp/new_section.md` invocation.
- **End-to-end proof of iter-17 stamping on a durable bundle:** the
  bundle's `run.json` line 20 reads `"holdout_version": 1` alongside
  the existing 15 metadata keys, exactly as iter-17's
  `build_run_metadata` change wires it. This is the first durable
  bundle-side typed evidence that iter-17's stamping survives
  end-to-end through `harness.py main()` → `read_holdout_version()` →
  `build_run_metadata` → `_write_atomic` to disk on a real benchmark
  invocation (not just the iter-17 `/tmp` dry-run). Pre-iter-17
  bundles continue to lack the field, intentionally.
- **Cheap channel:** green before and after (cargo: 32+37+16+0 across
  integration suites; python: 68 tests OK across the 8 spec-named
  modules; `harness.py --md-binary` dry-run: all 24 tasks PASS
  dual-scorer).
- **Comparability framing:** this is the first cell for (PI runner,
  gpt-5.4-mini, mdtools, minimal thinking, T2, runs_per_task=1,
  holdout_version=1, task-set version: live `bench/tasks/tasks.json`).
  - **NOT** a holdout reconfirmation: T2 is search-split, not
    holdout. The iter-7 T22 bundle remains the only holdout PI cell
    in the repo.
  - **NOT** a comparison versus prior PI bundles (iter-4 T1 / iter-7
    T22 / iter-10 T7 / iter-14 T18) — different task family,
    different scorer-shape exercise; cross-task aggregate is a
    search-set observation, not a comparison.
  - **NOT** a comparison versus existing OAI-loop content-delivery
    cells (e.g. the search bundles on Hermes-4-70B-4bit /
    magnum-v4-123b-4bit / Qwen3.5-122B-A10B-4bit / Qwen3.5-27B-4bit
    from 2026-04-21) — different executor and different model, both
    of which cross spec-required normalization axes.
  - **NOT** a product or anchor-validation claim. The benchmark
    family ('content delivery') is now exercised in PI for the
    first time, but no candidate primitive is being validated by
    a passing T2 cell. `bench/probes/anchor-validation/` still
    does not exist; no Route A or Route B justification is on file.
- **What this exercises that prior PI bundles did not:**
  - First content-delivery-family cell on PI (T2/T3/T8/T17 family
    per CLAUDE.md). Previous PI families: extraction (T1, T22),
    targeted-mutation (T7), multistep (T18).
  - First exercise of `insert-block --after N --from PATH` through
    the PI executor — distinct from prior PI mutations
    (T7 set-task, T18 delete-section + set-task).
  - First end-to-end demonstration through PI of the published
    `--from PATH` recovery pattern after a stdin-pipe policy
    deny — i.e. the documented stdin-piping weakness fires and
    the agent recovers via the documented `--from` workaround.
  - First durable PI bundle under `bench/runs/` carrying the
    iter-17 `holdout_version: 1` stamp on `run.json`.
- **Behavioral observation (per-model data, not a finding):**
  gpt-5.4-mini at minimal thinking on T2 mdtools first attempted
  `printf '## v2.5\n\nHotfix release for auth regression.\n' | ./md
  insert-block ... -i --after 2`. The guard policy denied the
  `printf` prefix (recorded in guard.log as `deny printf`). The agent
  recovered on the next turn with `cat > /tmp/new_section.md
  <<'EOF' ... EOF; ./md insert-block ... -i --after 2 --from
  /tmp/new_section.md`, which the guard allowed and the harness
  scored as a successful mutation. The verification re-query
  (`./md blocks ...`) followed, satisfying `requeried=true`. The
  spec language "Policy violations, retries, and observation
  volume are part of the behavioral story, not incidental noise"
  applies — the deny+recovery is recorded as part of this cell's
  behavioral story rather than as a defect.
- **Cross-counter measurement note (informational, not a finding):**
  `results.json:policy_violations=1` (harness's bash-command-level
  guard-deny counter from `bench_guarded_executor`, see
  `bench/oai_loop.py` and `bench/harness.py:1229,1265`) and
  `bench/pi_audit_adapter.py:summarize_pi_audit_events` would also
  return `policy_violations=1` if invoked with the parsed `guard.log`
  events (`bench/pi_audit_adapter.py:103-106`). The harness's value
  is the authoritative cross-executor-comparable counter on
  `BenchResult.policy_violations`; the adapter's audit-event-only
  `decision == "block"` path tracks PI's audit-hook layer, which has
  no `block` event for this run because the `printf|md` was a
  single PI tool_call observed as a unit at the audit layer and
  denied later at the bash-command-level guard (one layer below).
  Both fields measure what they document; the difference is the
  measurement layer, not a defect. No claim depends on this; no
  finding is filed.
- **Closure-discipline status:** parallel to iter-4 / iter-7 /
  iter-10 / iter-14 entries — this is a non-finding bundle
  introduction, not a substantive code change, and is not marked
  FIXED_PENDING_CONFIRMATION. A future review pass may ratify by
  re-reading the bundle's `run.json` (verifying line 20 carries
  `holdout_version: 1`), `results.json`, and the 10-event
  pi-audit.jsonl.
- **What this does NOT do:** does not promote any product anchor
  (`bench/probes/anchor-validation/` still does not exist, no
  candidate primitive validated). Does not bump `holdout_version`
  (still 1). Does not amend any published claim. Does not add any
  new typed artifact under `bench/probes/`. Does not retroactively
  modify any pre-iter-17 bundle.

### Holdout-version bundle stamping (2026-04-26 iter 17)

The frontier-loop spec's holdout-repair exception path requires bumped
versions to be *stamped onto subsequent run bundles* so future cross-version
comparability is mechanical, not inferred from bundle dates. Iter 16 wired
the runtime drift guard but explicitly recorded ("Does not bump
`holdout_version`. ... Does not modify any prior bundle.") that the
companion stamping work — labelling each new bundle with the holdout_version
under which it was produced — was not addressed. Iter 17 closes that
companion gap by threading `holdout_version` from the live fingerprints
manifest through `build_run_metadata` to the run.json bundle on every
harness-issued bundle.

- **Disturbed axis:** oracle trustworthiness — explicit unmet spec
  requirement. The spec's holdout-repair exception path step 2 reads
  "increment a `holdout_version` field in `bench/holdout/task_ids.json`
  (or equivalent manifest) **and stamp the new version onto subsequent
  run bundles**." The first half (manifest version + drift guard) landed
  in iter 2 (L1 closure) and iter 16 (runtime promotion); the second
  half (bundle stamping) was not in code. All four PI bundles in the
  repo (T1, T7, T18, T22) were produced under `holdout_version=1` but
  carry no `holdout_version` field on their run.json — a future
  `holdout_version=2` bump would silently render those bundles
  non-comparable with no mechanical marker on the typed artifact.
- **Frontier anchor:** *missing evaluator artifact — comparability stamp
  on run bundle metadata*. The spec language "stamp the new version
  onto subsequent run bundles" is unambiguous and unmet in code. Per
  the spec's allowed-during-P0/P1 hardening list, "telemetry-only
  instrumentation" and "harness assertions" are admissible; this change
  is the former (it records a fact, does not change behavior). Same
  axis as iter 16 (oracle trustworthiness) but a distinct artifact
  (per-bundle metadata vs the runtime-guard mechanism); the
  same-family-rule's *fresh failing trace* clause applies — the
  pre-iter-17 PI bundles are themselves the unstamped instances.
- **Change shape:**
  - Added `read_holdout_version(fingerprints_path=...)` at
    `bench/harness.py:778-794` — returns the integer `holdout_version`
    from the fingerprints manifest, or None for missing/malformed files
    (graceful skip for forks without holdout configuration, mirroring
    iter-16's `check_holdout_integrity` skip behavior).
  - Added an optional `holdout_version: int | None = None` parameter
    to `build_run_metadata` at `bench/harness.py:889` and a corresponding
    `"holdout_version": holdout_version` field in the returned dict at
    `bench/harness.py:931`. Default None means existing callers and
    tests are backward-compatible (the field is null when not provided).
  - Wired all three `build_run_metadata` call sites in `main()` to
    pass the version: dry-run at `bench/harness.py:1648`, partial
    incremental writes at `bench/harness.py:1714`, and final write at
    `bench/harness.py:1773`. The version is read once at startup
    (`bench/harness.py:1600`) immediately after `check_holdout_integrity`
    so the I/O cost is one extra small file read per harness invocation.
- **Tests added (6 new):**
  `bench/test_harness_task_split.py:HoldoutVersionStampTests` (3 tests):
  (a) `read_holdout_version()` returns 1 for the live repo,
  (b) returns None when fingerprints.json is missing,
  (c) returns None for malformed manifest (empty JSON object).
  `bench/test_harness_run_artifacts.py:HoldoutVersionMetadataTests`
  (3 tests): (a) metadata includes `holdout_version=1` when explicitly
  passed, (b) field is present and None when not passed (backward
  compat for existing callers), (c) future version bumps (e.g. v2)
  propagate cleanly through `build_run_metadata`. Test count rose from
  62 to 68 across the 8 spec-named modules.
- **End-to-end proof of mechanical stamping:** invoked
  `python3 bench/harness.py --md-binary target/release/md --results-dir /tmp/iter17-dryrun-bundle`
  on the live repo. The resulting `run.json` includes the new key
  `holdout_version: 1` alongside the existing 15 metadata keys. The
  runtime drift guard (iter 16) still fires with exit code 2 on tampered
  `tasks.json` — the breach message
  `holdout-immutability breach (holdout_version=1): T22: fingerprint drift
  in fields ['task_json_sha256']` reproduces bit-exact post-iter-17.
- **Cheap channel:** green before and after (cargo: 32+37+16+0 across
  integration suites; python: 68 tests OK across the 8 spec-named
  modules — was 62 before iter 17, +6 from new `HoldoutVersionStampTests`
  (3 tests) and `HoldoutVersionMetadataTests` (3 tests);
  `harness.py --md-binary` dry-run: all 24 tasks PASS dual-scorer).
- **Comparability framing:** this is a telemetry-only stamping change.
  It does **not** bump `holdout_version` (still 1; no holdout repair
  occurred), does **not** modify any pre-iter-17 bundle (existing
  `bench/runs/...` artifacts are unchanged and still lack the field —
  intentionally, since they pre-date this change and stamping them
  retroactively would itself be a holdout-repair-shaped artifact edit),
  does **not** change the agent's action space, does **not** introduce
  a new product surface, does **not** change any scorer, and does
  **not** affect any pass rate. It is an additive ratchet on the
  oracle: any new bundle produced from this point forward carries the
  version under which it was produced, so the first holdout repair
  that bumps to v2 will leave a clean cross-version record on all
  subsequent typed artifacts. Per the spec's "telemetry-only
  instrumentation" allowance, the change is squarely within the
  admissible work envelope.
- **Closure-discipline status:** this is a substantive fix authored by
  iter 17, parallel to iter 16's harness-assertion fix. Per the
  FIXED ≠ CLOSED rule, the entry is `FIXED_PENDING_CONFIRMATION`-shaped;
  a future review pass should ratify by re-reading
  `bench/harness.py` lines 778-794, 889, 931, 1600, 1648, 1714, 1773
  and the two new test classes, and by re-running the harness with
  `--results-dir` to verify the field appears on a fresh run.json.
  Like iter 16, this is filed as a non-finding harness-instrumentation
  improvement rather than a finding (no defect uncovered; the change
  closes a documented gap from iter-16's "Does not modify any prior
  bundle" carve-out + the spec's explicit stamping requirement).
- **Same-family-rule discharge:** iter 16 was oracle hardening
  (runtime-guard mechanical promotion); iter 17 is also oracle
  hardening but on a different artifact (per-bundle metadata vs the
  runtime-guard surface). Two consecutive oracle-axis substantive code
  changes is borderline same-family. The fresh-failing-trace escape
  clause applies: the four pre-iter-17 PI bundles (T1, T7, T18, T22)
  all lack the `holdout_version` field that the spec explicitly
  requires; this is the same shape as iter 13's line-number-drift
  trace (a published instruction that doesn't match the code). The
  trace is durable (the unstamped bundles are still in the repo) and
  the fix is the smallest reversible change that closes the gap.
- **What this does NOT do:** does not promote any product anchor
  (`bench/probes/anchor-validation/` still does not exist, no
  candidate primitive validated). Does not run the expensive outer
  channel. Does not bump `holdout_version` (the manifest version is
  still 1 and remains the single authoritative version for the live
  holdout). Does not amend any published claim. Does not retroactively
  modify any pre-iter-17 bundle's run.json. Does not invoke the
  holdout-repair exception path (the holdout is not being repaired —
  its fingerprints, descriptions, and expected outputs are untouched,
  only the bundle-side recording mechanism is added).

### L1 mechanical-guard runtime promotion (2026-04-26 iter 16)

The L1 closure (iter 2) landed `verify_holdout_fingerprints` as a function
plus a cheap-channel unit test, but the iter-3 review-pass learning #3
and the iter-6 ledger entry both explicitly recorded that the function
was *not* invoked by the harness at runtime — protection was procedural
(cheap channel before expensive channel), not mechanical at the runtime
boundary. Iter 16 closes that recorded gap by wiring a runtime invocation
of the guard into `bench/harness.py`'s `main()`, gating any benchmark
invocation on holdout integrity rather than relying on the unit-test
pathway having been run first.

- **Disturbed axis:** oracle trustworthiness — recorded harness-assertion
  gap from iter-3 learning #3 ("Adding a runtime invocation is a viable
  additive ratchet") and iter-6 finding ("verify_holdout_fingerprints
  is correctly defensive against four drift classes (...) but is not
  auto-invoked by the harness when --task-ids-path bench/holdout/task_ids.json
  is selected. The protection is therefore procedural ... not mechanical
  at the runtime boundary").
- **Frontier anchor:** *missing evaluator artifact — harness assertion*.
  The spec's "What counts as 'hardening'" list explicitly names
  "harness assertions" as allowed work, and the iter-15 entry's
  "Iters 16-17 should expect to do frontier work" framing required
  iter 16 to author frontier-shifting work rather than another
  ratification entry. The mechanical-guard promotion is the smallest
  pre-recorded available frontier move.
- **Change shape:** added `check_holdout_integrity(...)` wrapper at
  `bench/harness.py:747-775` that returns `None` on clean state /
  missing-files (skipped silently for forks without holdout
  configuration) or the breach message string on drift. Wired into
  `main()` at `bench/harness.py:1597-1599` immediately after
  `load_tasks(args.tasks_path)` and before any task selection or
  scoring, surfacing failure via `parser.error(...)` (exits with code 2
  and the self-describing breach message).
- **Tests added (3 new):**
  `bench/test_harness_task_split.py:HarnessRuntimeHoldoutGuardTests` —
  (a) clean-repo invocation returns `None`,
  (b) tampered-tasks.json invocation returns the breach message
  containing `holdout-immutability breach` and the drifted task ID
  (T22), (c) missing holdout files (no `task_ids.json`, no
  `fingerprints.json`) returns `None` so forks without holdout
  configuration are not blocked. Test count rose from 59 to 62 across
  the 8 spec-named modules.
- **Subprocess-level proof of mechanical fire:** invoked
  `python3 bench/harness.py --md-binary target/release/md --tasks-path /tmp/tasks-tampered.json`
  with a copy of `bench/tasks/tasks.json` whose T22 description was
  edited (sneak-edit). The harness exited with code 2 and stderr line
  `harness.py: error: holdout-immutability breach (holdout_version=1):
  T22: fingerprint drift in fields ['task_json_sha256'] — follow the
  holdout-repair exception path before reporting holdout results.`
  No tasks were scored. The mechanical guard fired before any agent
  invocation could begin.
- **Cheap channel:** green before and after (cargo: 32+37+16+0 across
  integration suites; python: 62 tests OK across the 8 spec-named
  modules — was 59 before iter 16, +3 from new
  `HarnessRuntimeHoldoutGuardTests`; `harness.py --md-binary`
  dry-run: all 24 tasks PASS dual-scorer).
- **Comparability framing:** this is a harness-assertion hardening
  change. It does **not** change the agent's action space, does not
  introduce a new product surface, does not touch any holdout
  artifact (the holdout ID list, fingerprints manifest, expected
  outputs, and tasks corpus are unchanged), does not change any
  scorer, and does not affect any pass rate. It is an additive
  ratchet on the oracle: future harness invocations now fail-fast on
  holdout drift rather than relying on the unit test pathway. Per the
  spec's "hardening allowed during P0/P1" rule (currently no P0/P1
  open, so this rule is permissive), and the spec's "telemetry-only
  instrumentation" / "harness assertions" allowance, the change is
  squarely within the admissible work envelope.
- **Closure-discipline status:** this is a substantive fix authored by
  iter 16, not a ratification. Per the FIXED ≠ CLOSED rule, the entry
  is `FIXED_PENDING_CONFIRMATION`-shaped; a future review pass should
  ratify by re-reading `bench/harness.py` lines 747-775 and 1597-1599
  + `bench/test_harness_task_split.py:HarnessRuntimeHoldoutGuardTests`
  against this entry. Per L1 spec language ("the cheapest reachable
  probe"), the new test class now subsumes the role of "the unit test
  pathway pinning runtime guarding." Since the change is in CLOSED
  ledger position above (not OPEN or FIXED_PENDING_CONFIRMATION), it
  is filed as a non-finding harness-assertion improvement rather than
  a finding; the closure-discipline rule's "FIXED_PENDING_CONFIRMATION"
  ledger position applies to OPEN findings, not to substantive
  hardening additions, parallel to iter-1's F3 fix structure.
- **What this does NOT do:** does not promote any product anchor
  (`bench/probes/anchor-validation/` still does not exist, no
  candidate primitive validated). Does not run the expensive outer
  channel. Does not bump `holdout_version`. Does not amend any
  published claim. Does not modify any prior bundle. The
  `holdout-repair exception path` is not invoked because the holdout
  is *not* being repaired — its fingerprints are untouched, only the
  guard's runtime invocation surface is added.

### Confirmation review pass (2026-04-26 iter 15)

Closure-discipline review of iter-14's quiet-signal-checkpoint discharge
applied to a bundle introduction (not a substantive fix). Parallels iter
12's review of iter-11's measurement publication and iter 13's review
of iter-12's typo fix, but in a "no substantive fix to pair" shape: the
typed-artifact claims in iter-14's discharge entry were checked against
the underlying bundle, no drift was found, no fresh failing trace was
surfaced. Cheap channel green at review time (`cargo test -q` all suites
pass, 59 python unittests OK across the 8 spec-named modules,
`harness.py --md-binary` dry-run all 24 tasks PASS dual-scorer).

What was checked:

- **Bundle metrics in `results.json`** —
  `bench/runs/checkpoint-pi-T18-mdtools-gpt5.4mini-2026-04-26/results.json`
  re-read. Every iter-14-published number matches bit-exact: `task_id=T18`,
  `mode=mdtools`, `correct=true`, `correct_neutral=true`,
  `elapsed_seconds=11.03`, `tool_calls=10`, `turns=6`, `mutations=2`,
  `requeried=true`, `bytes_observation=14858`, `bytes_output=844124`,
  `policy_violations=0`, `invalid_responses=0`,
  `unique_invalid_responses=0`, `diff_report=""`, `runner_error=null`,
  `model=openai-codex/gpt-5.4-mini`, `thinking_level=minimal`.
- **Run metadata in `run.json`** — re-read. Confirms
  `runner=pi-json`, `executor=guarded`, `model=openai-codex/gpt-5.4-mini`,
  `thinking_level=minimal`, `runs_per_task=1`, `modes=["mdtools"]`,
  `selected_task_ids=["T18"]`. Aggregates section reproduces the same
  numbers as the per-result entry.
- **Pi-audit JSONL event count and shape** —
  `logs/T18_mdtools_1777214592/pi-audit.jsonl` has exactly 22 lines.
  Event-type histogram: `{model_change: 1, thinking_level_change: 1,
  tool_call: 10, tool_result: 10}`. Matches the iter-14 entry's claim
  "22 events: `model_change`, `thinking_level_change`, plus 10 ×
  `tool_call` + 10 × `tool_result`" exactly.
- **Tool-call sequence** — the 10 `tool_call` events (in order)
  carry these `input.command` strings (with the temp-dir prefix
  abstracted): (1) `./md outline … --json`, (2) `./md blocks … --json`,
  (3) `./md tasks … --json`, (4) `./md delete-section "Draft Notes"
  … -i`, (5) `./md outline … --json`, (6) `./md blocks … --json`,
  (7) `./md tasks … --json`, (8) `./md set-task 4.1 … -i --status
  done`, (9) `./md tasks … --json`, (10) `cat …`. Counts: outline×2,
  blocks×2, tasks×3, delete-section×1, set-task×1, cat×1 = 10. Matches
  the iter-14 entry's "(`md outline --json` × 2, `md blocks --json` × 2,
  `md tasks --json` × 3, `md delete-section "Draft Notes" -i`,
  `md set-task 4.1 -i --status done`, final `cat`)" enumeration.
- **All three structural commands re-queried after delete-section** —
  events 5/6/7 confirm `outline`, `blocks`, and `tasks` were each
  re-queried after the `delete-section` mutation at event 4, before the
  second mutation at event 8. Matches iter-14 "All three structural
  commands (`outline`, `blocks`, `tasks`) are re-queried after the
  deletion" claim.
- **Adapter summary** —
  `bench.pi_audit_adapter.summarize_pi_audit_events` invoked on the
  parsed event list returns
  `PiAuditCounters(tool_calls=10, tool_results=10, tool_errors=0,
  bytes_observation=14858, blocked=0, policy_violations=0, mutations=2,
  requeried=True, model='openai-codex/gpt-5.4-mini',
  thinking_level='minimal', bash_commands=[…])`. Every reported counter
  matches the iter-14 entry's enumeration (`tool_calls=10`,
  `tool_errors=0`, `mutations=2`, `policy_violations=0`, `blocked=0`,
  `requeried=True`, `bytes_observation=14,858`); the
  `bytes_observation` from the adapter matches the `results.json` field
  exactly.
- **Cross-model behavioral observation** — iter-14 claimed
  "gpt-5.4-mini at minimal thinking emits ~2.5× as many tool calls on
  T18 as Hermes-4-70B-4bit (10 vs 4) — both pass dual-scorer."
  Independently verified from
  `bench/runs/search-mdtools-multistep-Hermes-4-70B-4bit-2026-04-21/results.json`
  (T18 row: `tool_calls=4`, `mutations=3`, `requeried=True`,
  `correct=true`) and
  `bench/runs/search-mdtools-multistep-Qwen3.5-27B-4bit-2026-04-21/results.json`
  (T18 row: `tool_calls=5`, `mutations=2`, `requeried=True`,
  `correct=true`). Ratio 10/4 = 2.5 (matches "~2.5×"). The Qwen3.5-27B
  cell at 5 tool calls also confirms the qualitative claim that
  gpt-5.4-mini's read pattern is more thorough than either small-model
  baseline.

Verdict — iter-14 quiet-signal-checkpoint discharge ratified. All
typed-artifact claims in the iter-14 entry reproduce bit-exact against
the bundle's `results.json`, `run.json`, and
`logs/T18_mdtools_1777214592/pi-audit.jsonl` files. The closure-discipline
rule's "next pass not re-raising the finding" criterion is satisfied for
the iter-14 introduction. (No FIXED_PENDING_CONFIRMATION ledger entry
needed promotion — iter 14's bundle introduction was a non-finding-
producing expensive run, not filed as a finding.) No new finding opened,
no holdout artifact touched.

**No frontier expansion explicitly labeled.** This iteration is audit
ratification of an existing bundle, not measurement publication, not
scorer change, not new product surface, not claim expansion, not
holdout-artifact touch. It hardens trust in iter-14's evidence
substrate (the typed-artifact claims now have an independently-verified
durable record) but does not move the benchmark frontier. Treating this
as audit work rather than improvement work is the honest framing: a
mature T7 loop should refuse ratification-only iterations unless they
close a finding, amend a claim, or promote a new reusable invariant —
this iteration does the first (procedurally ratifies iter-14's claims)
without the second or third. Iters 16–17 should expect to do frontier
work or trigger the iter-18 expensive-or-halt rule.

- **Frontier anchor (review pass):** *closure-discipline rule applied
  to bundle introduction* — iter 14 made specific typed-artifact claims
  (event count, tool-call breakdown, adapter counters, cross-model ratio)
  that needed independent verification. Iter 15 discharges this by
  reading typed artifacts (results.json + run.json + pi-audit.jsonl +
  adapter output) rather than narrative.
- **Same-family discharge:** iters 11–13 were three consecutive
  spec-coherence iterations (publication, typo fix + ratification,
  line-number fix + ratification); iter 14 broke the chain with an
  expensive-channel multistep-family bundle. Iter 15 returns to a
  ledger-only ratification entry — the same-family rule's concentration
  was already broken by iter 14, so a single ratification entry
  (without the substantive-fix pairing iters 12–13 carried, because no
  fresh failing trace surfaced) is admissible without invoking the
  fresh-trace clause. Iter 12 set the precedent for "ratify by reading
  typed artifacts"; iter 15 applies the same shape to iter-14.
- **Comparability framing:** the ratification is a ledger-only
  verification. It does not change any data, ratio, rule conclusion,
  pass rate, or holdout artifact. It cites already-extant typed
  artifacts to confirm iter-14's already-published claims. Per the
  "What this exercises" subsection in the iter-14 entry, the
  observation that T18 exercises the `file_contents` `expected_artifact`
  branch is correct (T18 has `expected_artifact=file_contents` in
  `bench/tasks/tasks.json`); the more precise underlying claim is that
  T18 specifically uses `scorer.kind=raw_bytes` (whereas T7 also has
  `expected_artifact=file_contents` but uses `scorer.kind=normalized_text`).
  This is a refinement worth recording as iter-15 review-pass observation
  but not a defect: the iter-14 narrative parenthetical is approximately
  correct and the bundle's actual behavior is unchanged. No edit applied.

### Quiet-signal checkpoint discharge (2026-04-26 iter 14)

Per the spec's "After 3 consecutive iterations …" rule, the
quiet-signal counter reached 3 at the end of iter 13 (iters 11, 12, 13
all quiet). Iter 14 ran the expensive outer channel to introduce fresh
typed signal: the **first** PI runner bundle for the **multistep**
task family. Prior PI coverage was extraction (T1 iter 4, T22 iter 7)
and mutation (T7 iter 10); multistep had zero PI cells. Cheap channel
re-verified green before and after the run.

- **Bundle:** `bench/runs/checkpoint-pi-T18-mdtools-gpt5.4mini-2026-04-26/` —
  fourth PI runner bundle in `bench/runs/` and the **first** cell that
  exercises (a) the multistep task family, (b) the `delete-section`
  command via PI, and (c) the canonical "structural mutation → full
  re-query of `outline` + `blocks` + `tasks` → second mutation → verify"
  pattern that the multistep family is designed to test. Single task
  (T18, search-split, multistep family). Single mode (mdtools). Single
  run. Model `openai-codex/gpt-5.4-mini` at `thinking_level=minimal`,
  recorded per-result and per-run on the metadata bundle.
- **Verdict:** T18 mdtools dual-scorer PASS in 11.03s with 10 tool
  calls (`md outline --json` × 2, `md blocks --json` × 2,
  `md tasks --json` × 3, `md delete-section "Draft Notes" -i`,
  `md set-task 4.1 -i --status done`, final `cat`), 2 mutations,
  `requery_rate=1.0`, `bytes_observation=14,858`,
  `bytes_output=844,124` (PI streaming overhead, see P3 cross-executor
  rule in `bench/RESULTS.md`), empty `diff_report`. Pi-audit log
  preserved at `logs/T18_mdtools_1777214592/pi-audit.jsonl` (22 events:
  `model_change`, `thinking_level_change`, plus 10 × `tool_call` +
  10 × `tool_result`), parses cleanly via
  `bench/pi_audit_adapter.summarize_pi_audit_events` returning
  `tool_calls=10`, `tool_errors=0`, `mutations=2`, `policy_violations=0`,
  `blocked=0`, `requeried=True`, `bytes_observation=14,858` (matches
  `results.json` exactly).
- **Comparability framing:** this is the first cell for (PI runner,
  gpt-5.4-mini, mdtools, minimal thinking, T18, runs_per_task=1, task-set
  version: live `bench/tasks/tasks.json` with `holdout_version=1` from
  `bench/holdout/fingerprints.json`). It is **NOT** a holdout
  reconfirmation (T18 is search-split, not holdout) and **NOT** an
  apples-to-apples comparison against the iter-4 T1, iter-7 T22, or
  iter-10 T7 bundles — same executor / model / mode / thinking /
  runs-per-task across all four PI cells, but different tasks and
  different scorer expectations (T18 is `file_contents`-artifact, not
  `json_envelope` or `normalized_text`), so any pass-rate aggregation
  across cells would be a search-set observation, not a comparison.
  Likewise it is **NOT** an apples-to-apples comparison to the
  pre-existing OAI-loop multistep bundles (`search-mdtools-multistep-Hermes-4-70B-4bit-2026-04-21/`
  and `search-mdtools-multistep-Qwen3.5-27B-4bit-2026-04-21/`) — both
  the executor axis and the model axis differ. Tool-call count
  divergence (10 PI vs 4 Hermes / 5 Qwen) is a **per-model behavioral
  signal**, not an executor or product comparison.
- **What this exercises:** for the first time in `bench/runs/`, the PI
  runner pipeline (harness pi-json branch → `pi --mode json` → audit
  extension at `~/.pi/agent/extensions/audit/index.ts` →
  `bench/pi_audit_adapter.summarize_pi_audit_events`) is verified
  end-to-end on (a) a multistep-family task with two structurally
  different mutations (`delete-section` then `set-task`) on the same
  file, (b) the `file_contents` scorer artifact (the `expected_artifact`
  branch that compares against an expected output file rather than a
  JSON envelope or normalized text), and (c) the canonical drift-handling
  pattern where `delete-section` shifts block indices and the agent
  must re-query the structural surface before the second mutation can
  be addressed. All three structural commands (`outline`, `blocks`,
  `tasks`) are re-queried after the deletion — the gpt-5.4-mini agent
  spontaneously emits the most thorough re-query pattern observed on
  this task across the available models (Hermes-4-70B and Qwen3.5-27B
  re-queried fewer commands).
- **What this discharges:** the spec's quiet-signal-checkpoint rule
  (3 consecutive quiet iterations 11–13). It does **not** discharge any
  product or oracle claim — those still require their own attribution
  probes and apples-to-apples comparisons. It does **not** validate any
  candidate primitive's failure-class fit; the bundle is *evaluator
  coverage* (extending the comparable-harness-axis frontier anchor),
  not anchor justification.
- **What it surfaced:** no new defect. The PI pipeline produced fresh
  typed signal that exercised the multistep + drift-handling pattern
  cleanly. This is a "no new finding" expensive run — admissible as
  fresh signal because the bundle is on a previously-uncovered (task,
  family, scorer-branch) cell and the audit log + scorer outputs are
  durably persisted as a queryable bundle. A behavioral observation
  worth recording but **not** filing as a finding: gpt-5.4-mini at
  minimal thinking emits ~2.5× as many tool calls on T18 as
  Hermes-4-70B-4bit (10 vs 4) — both pass dual-scorer, but the
  read-pattern shape differs materially across models. This is
  per-model behavioral data, not a product or scorer issue.
- **Cheap channel:** green before and after (`cargo test -q` all suites
  pass, 59 python unittests OK across the 8 spec-named modules,
  `harness.py --md-binary` dry-run all 24 tasks PASS dual-scorer).

### Confirmation review pass (2026-04-26 iter 13)

Continuation of the published-narrative-vs-typed-artifact cross-check
pattern iter 12 began. Iter 13 swept `bench/harness.py:LINE` references
across all published-narrative files and surfaced one stale
line-number reference. Cheap channel green at review time and again
after the edit (`cargo test -q` all suites pass, 59 python unittests OK
across the 8 spec-named modules, `harness.py --md-binary` dry-run all
24 tasks PASS dual-scorer).

What was checked:

- **All `harness.py:LINE` references in published-narrative files** — full
  grep across `bench/RESULTS.md`, `bench/retracted_2026-04-24/README.md`,
  `README.md`, `CLAUDE.md`, and `specs/**`. Three references found, all
  in `bench/RESULTS.md`:
  - `bench/RESULTS.md:54` → `bench/harness.py:1229` and `:1265` for the
    pi-json and oai-loop `bytes_output = len(raw_stdout.encode())` call
    sites. Verified — both lines match the current code (confirmed via
    `grep -n "bytes_output = len" bench/harness.py`).
  - `bench/RESULTS.md:151` → `bench/harness.py:443-537` for the F3
    `score_structural_json` envelope-normalization span. Verified — the
    current `def score_structural_json` runs from line 443 to its
    closing `return ok_md, ok_neutral` on line 537. Matches the ledger
    F3 entry's typed-artifact pointer.
  - `bench/RESULTS.md:152` → `bench/harness.py:339-341` for the F3-a
    rstrip fix. **Stale.** Current rstrip body lines are at 347-348
    (confirmed: `git blame -L 347,348 bench/harness.py` returns commit
    `03af07d0` from 2026-04-24, which is the F3-a FIXED commit).
    Lines 339-341 in the current file are blank + `if policy.kind ==
    "raw_bytes":` + `# Normalize before raw comparison if requested` —
    none of which is the rstrip fix. The ledger F3-a CLOSED entry
    (lines 656–660) and the iter-3 F3-a re-verification (line 586)
    both correctly cite `bench/harness.py:347-348`.
- **Drift origin** — `git show 03af07d:bench/harness.py | sed -n
  '339,341p'` confirms that at the time `bench/RESULTS.md:152` was
  authored (2026-04-24, same commit that introduced the F3-a fix), the
  rstrip body was at lines 339-341. Subsequent edits (most prominently
  iter 1's F3 fix at `score_structural_json`, which sits *after* the
  raw_bytes branch in the file) shifted the F3-a rstrip body 8 lines
  down to 347-348. The ledger entries written 2026-04-26 cite the new
  correct position; only the published-narrative table was not updated
  at the same time.
- **No other `harness.py` line numbers are stale.** `bench/RESULTS.md:54`
  and `:151` reproduce against the current code bit-exact. Sibling
  published-narrative files (`bench/retracted_2026-04-24/README.md`,
  `README.md`, `CLAUDE.md`, `specs/**`) carry zero `harness.py:LINE`
  citations, so there is no parallel sweep work to do on those files.

What was fixed:

- **Stale line-number reference at `bench/RESULTS.md:152`** — replaced
  `bench/harness.py:339-341` with `bench/harness.py:347-348` to match
  the ledger F3-a CLOSED entry's typed-artifact pointer and the actual
  position of the rstrip body lines in the current file. The narrative
  description ("rstrip the whole string after per-line rstrip") and
  the reproducibility-rule conclusion are unchanged. No data, ratios,
  or pass rates touched.
- **Frontier anchor (substantive fix):** *fresh failing trace* — a
  reader following the cited line range lands on
  `if policy.kind == "raw_bytes":` plus a comment, not on the rstrip
  fix the narrative describes. This is the same shape of fresh failing
  trace iter 12 cited for `--executor pi-json`: a published-narrative
  pointer that does not reproduce against the underlying code. The
  same-family rule's "cite a fresh failing trace, external finding, or
  blocked claim" escape clause is therefore satisfied.
- **Frontier anchor (review pass):** the iter-9 learning #1 invitation
  to "grep for stale references across all published-narrative files
  before declaring the sweep complete" — extended from status keywords
  (F3 / L1 / pending-fix) to file:line citations, which iters 8–12
  swept for status but did not systematically validate for line-number
  drift. Iter 13 closes the line-number subset of that sweep.
- **Comparability framing:** the line-number fix is a published-narrative
  edit that does not change the data, the ratios, the rule conclusion,
  or any pass rate. It does not touch any holdout artifact
  (`bench/RESULTS.md` is published narrative). No new finding opened.
- **Iter-12 typo-fix ratification (paired):** the iter-12 substantive
  fix (`--executor pi-json` → `--runner pi-json` at
  `bench/RESULTS.md:54`) was re-verified during this pass. Current
  `bench/RESULTS.md:54` reads "selected via `--runner pi-json` on
  `bench/harness.py`; `--executor` is a separate flag controlling
  guarded vs legacy shell execution." Matches the iter-12 ledger
  entry's intended diff. `bench/harness.py:1498` defines `--runner`
  with choices `['claude-cli', 'oai-loop', 'pi-json']` and
  `bench/harness.py:1513` defines `--executor` with choices
  `['guarded', 'legacy']` — both confirmed via
  `grep -n "add_argument.*runner\\|add_argument.*executor"`. The
  iter-12 typo fix is hereby treated as having survived a confirmation
  pass; the closure-discipline rule's "next pass not re-raising the
  finding" criterion is satisfied. (No FIXED_PENDING_CONFIRMATION
  ledger entry needed promotion — iter 12's substantive fix was a
  non-finding-producing edit, not filed as a finding.)

### Confirmation review pass (2026-04-26 iter 12)

Closure-discipline review of iter-11's measurement-publication, paired
with one small substantive fix to a published-narrative typo iter 12
surfaced during the verification pass. Cheap channel green at review
time and again after the edit (`cargo test -q` all suites pass, 59
python unittests OK across the 8 spec-named modules,
`harness.py --md-binary` dry-run all 24 tasks PASS dual-scorer).

What was checked:

- **Iter-11 measurement-publication** — re-read the
  "Same-task validation (2026-04-26 iter 11)" block in
  `bench/RESULTS.md:56-66` against the six bundle `results.json` and
  `run.json` files cited as data sources. Every published number
  matches bit-exact:
  - T1 PI `bytes_output=5,975,843`, `bytes_observation=2,266`,
    `tool_calls=1`, `mut=0`, `correct=True/True`, `elapsed=9.83s`
    from `bench/runs/checkpoint-pi-T1-mdtools-gpt5.4mini-2026-04-26/results.json`.
  - T7 PI `bytes_output=1,172,040`, `bytes_observation=16,219`,
    `tool_calls=3`, `mut=1`, `correct=True/True`, `elapsed=16.45s`
    from `bench/runs/checkpoint-pi-T7-mdtools-gpt5.4mini-2026-04-26/results.json`.
  - T22 PI `bytes_output=671,515`, `bytes_observation=514`,
    `tool_calls=1`, `mut=0`, `correct=True/True`, `elapsed=9.63s`
    from `bench/runs/checkpoint-pi-T22-mdtools-gpt5.4mini-2026-04-26/results.json`.
  - T1 OAI `bytes_output=2,702`, `bytes_observation=2,436`,
    `tool_calls=1`, `mut=0`, `correct=True/True`, `elapsed=23.53s`
    from `bench/runs/search-mdtools-extraction-Qwen3.5-122B-A10B-4bit-2026-04-21/results.json`.
  - T7 OAI `bytes_output=699`, `bytes_observation=13,671`,
    `tool_calls=3`, `mut=1`, `correct=True/True`, `elapsed=39.68s`
    from `bench/runs/search-mdtools-mutation-Qwen3.5-122B-A10B-4bit-2026-04-21/results.json`.
  - T22 OAI `bytes_output=488`, `bytes_observation=1,036`,
    `tool_calls=2`, `mut=0`, `correct=False/False`, `elapsed=21.67s`
    from `bench/runs/holdout-mdtools-Qwen3.5-122B-A10B-4bit-2026-04-24/results.json`
    (the `correct=False/False` confirms the published "T22 OAI cell
    predates the F3 fix" caveat is honest).
  Ratios: 5,975,843 / 2,702 ≈ 2,212.0 (published ~2,212×); 1,172,040
  / 699 ≈ 1,676.7 (published ~1,677×); 671,515 / 488 ≈ 1,375.7
  (published ~1,376×). All three match. Bytes_observation deltas:
  T1 (2,266-2,436)/2,436 = -7.0% (published -7%); T7
  (16,219-13,671)/13,671 = +18.6% (published +19%). Both match.
- **Normalization-axis claim** — verified against `run.json` for each
  of the 6 bundles. All six share `modes=['mdtools']`,
  `executor=guarded`, `runs_per_task=1`. The three PI bundles share
  `runner=pi-json`, `model=openai-codex/gpt-5.4-mini`,
  `thinking_level=minimal`. The three OAI bundles share
  `runner=oai-loop`, `model=Qwen3.5-122B-A10B-4bit`,
  `thinking_level=None`. The model confound caveat in the published
  block is therefore the *only* differing axis material to the
  published claim. (Thinking-level differs nominally —
  `minimal` vs `None` — but `None` on the older OAI runs reflects the
  absence of a thinking-level concept, not a different setting; the
  claim is about executor stdout shape, not thinking-level.)
- **Verdict — iter-11 measurement-publication ratified.** All published
  numbers, ratios, and bytes_observation deltas reproduce bit-exact
  against the underlying bundles. The iter-11 invitation to "treat
  this measurement-publication as `FIXED_PENDING_CONFIRMATION`-
  equivalent and ratify it by re-reading the table against the cited
  results.json files" is hereby discharged.

What was fixed:

- **Published reproducibility typo at `bench/RESULTS.md:54`** — the
  iter-11 publication added the parenthetical "selected via
  `--executor pi-json`" describing how to invoke the PI runner. The
  actual harness CLI flag is `--runner pi-json` (defined at
  `bench/harness.py:1498` with choices
  `['claude-cli', 'oai-loop', 'pi-json']`); `--executor` is a separate
  flag (defined at `bench/harness.py:1513` with choices
  `['guarded', 'legacy']`) controlling guarded-vs-legacy shell
  execution. The published instruction is therefore non-reproducible —
  argparse would reject `--executor pi-json` with "invalid choice".
  Replaced with "selected via `--runner pi-json` on
  `bench/harness.py`; `--executor` is a separate flag controlling
  guarded vs legacy shell execution" to make the disambiguation
  explicit and self-contained for readers. Cross-checked: README.md:220
  correctly uses `--executor legacy`, and pi_runner.py:46 correctly
  uses `--runner pi-json`; the typo is isolated to RESULTS.md:54.
- **Frontier anchor (substantive fix):** *fresh failing trace* — the
  published narrative contradicts harness CLI flag definitions in a
  way an external reader following the document would hit immediately.
  This is the same-family rule's "fresh failing trace" escape clause,
  so a same-axis (specification coherence) move after iter 11 is
  admissible.
- **Frontier anchor (review pass):** *closure-discipline rule + iter-11
  invitation* — iter 11 explicitly invited a future review pass to
  ratify the measurement-publication; iter 12 discharges that
  invitation by reading typed artifacts (results.json + run.json)
  rather than narrative.
- **Comparability framing:** the typo fix is a published-narrative
  edit that does not change the data, the ratios, the ratio-rule
  conclusion, or any pass rate. It does not touch any holdout artifact
  (`bench/RESULTS.md` is published narrative). No new finding opened.

### Confirmation review pass (2026-04-26 iter 6)

Explicit closure-discipline review of the single FIXED_PENDING_CONFIRMATION
entry remaining after iter 5 (P3). Verified against typed artifacts, not
against prose. Cheap channel was green at review time (cargo test all
suites passing — see iter-6 cheap-channel block below — plus 59 python
unittests across the 8 spec-named modules and `harness.py --md-binary`
dry-run all 24 tasks PASS dual-scorer).

What was checked, per finding:

- **P3** — re-read `bench/RESULTS.md:52-60` ("Cross-executor comparability
  (PI runner vs OAI loop)" section). Section names the executor-locality
  rule for `bytes_output`, cites both reference data points (PI T1
  `bytes_output=5,975,843` from
  `bench/runs/checkpoint-pi-T1-mdtools-gpt5.4mini-2026-04-26/results.json`
  and oai-loop T20 `bytes_output=679` from
  `bench/runs/holdout-mdtools-Qwen3.5-122B-A10B-4bit-2026-04-24/results.json`),
  identifies both harness call sites (`bench/harness.py:1229` for pi-json
  and `:1265` for oai-loop, both confirmed as
  `bytes_output = len(raw_stdout.encode())`), enumerates the
  cross-executor-comparable fields, and flags the future
  `bytes_assistant_content` ratchet without committing to it.
  Independently verified that `bytes_observation` is genuinely shared
  across executors: `bench/oai_loop.py:212` increments it from observation
  content size, and `bench/pi_audit_adapter.py:86,89` increment it from
  audit event `outputBytes` — both branches parse tool-result content
  rather than raw stdout, so the published comparability claim holds.
  **Verdict: closed.**

### Iter-6 cheap-channel snapshot

For audit traceability of the closure-review pass:

- `cargo test -q`: all suites pass (32, 37, 16, 0, 0, 0, 36, 13, 37, 32,
  37, 12, 7, 24, 32, 37, 16, 0 across the integration-test binaries).
- `python3 -m unittest bench.test_command_policy bench.test_oai_loop
  bench.test_pi_audit bench.test_harness_json bench.test_harness_run_artifacts
  bench.test_harness_task_split bench.test_analyze_inputs
  bench.test_report_inputs`: 59 tests, OK.
- `python3 bench/harness.py --md-binary target/release/md`: all 24 tasks
  pass dual scorer (`md=PASS neutral=PASS` for T1–T24, with
  `json_canonical`, `frontmatter_json`, and `link_destinations` scorer
  branches all OK on the relevant tasks).

### Halt-condition / quiet-signal status (after iter 33)

After iter 33's discharge of the spec's quiet-signal-checkpoint by
producing the ninth PI runner bundle (T11 mdtools dual-scorer PASS in
8.93s on `openai-codex/gpt-5.4-mini` at minimal thinking), the fifth
durable bundle in `bench/runs/` carrying iter-17's `holdout_version: 1`
stamp, and corroborating the F4 closure trail with fresh expensive-
channel regression evidence on a new task trajectory plus a mechanically-
pinned pre-iter-30 selector counterfactual showing dual-scorer FAIL on
the same trajectory — see "Quiet-signal checkpoint discharge (2026-04-26
iter 33)" CLOSED entry above:

- **OPEN findings count:** **0**. The zero-OPEN streak that began at
  iter 30 now stands at count **4** (iter 30 + iter 31 + iter 32 + iter 33).
  The "no OPEN findings for 2 consecutive review rounds" halt
  condition remains met on this counter, but per spec it is one of
  several halt conditions — the quiet-signal counter and homeostasis
  balance also gate halt; see below. F4 was not re-raised by iter
  33's expensive-channel extension; the new T11 PI bundle's PASS
  on a fresh agent trajectory exhibiting the same recovery shape
  (intermediate JSON tool outputs + text answer in assistant turn)
  that originally caused F4 on T16 implicitly ratifies iter 31's
  closure call by independent empirical confirmation.
- **Quiet-signal counter:** iters 5–6 quiet, iter 7 expensive, iters
  8–9 quiet, iter 10 expensive, iters 11–13 quiet, iter 14 expensive
  (multistep-family coverage extension), iter 15 quiet (ledger-only
  ratification), iter 16 quiet (cheap-channel-only oracle hardening),
  iter 17 quiet (cheap-channel-only oracle telemetry stamping), iter
  18 expensive (content-delivery-family coverage extension + first
  stamped bundle), iter 19 quiet (cheap-channel-only
  specification-coherence publication), iter 20 quiet
  (cheap-channel-only closure-discipline ratification + corrective
  line-drift fix), iter 21 expensive (frontmatter_json scorer-branch
  coverage extension; counter reset to 0), iter 22 quiet
  (ledger-only closure-discipline ratification + two forward-pointing
  citation corrections; counter increments to 1), iter 23 quiet
  (cheap-channel-only specification-coherence publication of iter-21
  T21 PI bundle as the sixth PI bundle in the cross-executor section;
  counter increments to 2), iter 24 quiet (ledger-only
  closure-discipline ratification of iter 23 + one forward-pointing
  citation accuracy correction; counter increments to **3**), iter 25
  expensive (T9 mdtools structural-with-no-compare-flags scorer-branch
  coverage extension; counter reset to **0**), iter 26 quiet
  (cheap-channel-only specification-coherence cash-out of iter-25 T9
  PI bundle into the cross-executor table as a fifth row, paired with
  closure-discipline ratification of iter 25; counter increments to
  **1**), iter 27 quiet (ledger-only closure-discipline ratification
  of iter 26 paired with one forward-pointing code-path-routing
  correction in iter-25 prose; counter increments to **2**), iter 28
  quiet (cheap-channel-only oracle-trustworthiness hardening —
  promoting iter-27's corpus-vacuous-path prose claim to a typed
  `ScorerDispatcherBranchTests` cheap-channel assertion, +2 unit
  tests, no expensive run; counter increments to **3**), iter 29
  expensive (T16 mdtools PI runner bundle as the eighth PI bundle,
  fourth durable bundle carrying iter-17's `holdout_version=1`
  stamp, **first PI mdtools FAIL** on disk, surfaced **F4 P1
  OPEN**; counter reset to **0**), iter 30 quiet (cheap-channel-only
  oracle-trustworthiness — F4 closure via schema-aware
  `select_json_envelope_actual` helper at `bench/harness.py:1478`
  plus 8 unit tests in `JsonEnvelopeActualSelectionTests`, no
  expensive run, F4 transitioned OPEN → FIXED_PENDING_CONFIRMATION;
  counter increments to **1**), iter 31 quiet (cheap-channel-only
  closure-discipline review pass — typed-artifact ratification of
  iter 30 + replay-verification of the iter-29 T16 bundle through
  the new selector + paired `bench/RESULTS.md:72` F4-quarantine →
  F4-closure paragraph downgrade + one forward-pointing iter-30
  line-1478 citation typo correction; F4 transitioned
  FIXED_PENDING_CONFIRMATION → CLOSED; counter increments to
  **2**), iter 32 quiet (cheap-channel-only oracle-trustworthiness
  hardening — promoting iter 30 / iter 31 prose-only replay claim
  to a typed cheap-channel assertion via `F4ClosureBundleReplayTests`
  in `bench/test_harness_json.py`, +1 unit test, no expensive run,
  F4 not re-raised; counter increments to **3**), iter 33 expensive
  (T11 mdtools PI runner bundle as the **ninth** PI bundle, **fifth**
  durable bundle carrying iter-17's `holdout_version=1` stamp;
  corroborates the F4 closure trail with regression evidence on a
  fresh task trajectory plus a mechanically-pinned counterfactual
  showing the pre-iter-30 selector would have FAILed dual-scorer on
  the same trajectory; F4 closure remains anchored by iter 30/31/32;
  counter reset to **0**). Iters 34-36 admissible quiet, iter 37
  the next forced expensive-or-halt point unless another expensive
  run between now and then independently introduces fresh signal
  that resets the counter.
  The cheapest reachable expensive probe in this environment remains
  the PI runner via `~/.pi/agent/auth.json` — Qwen3.5-122B-A10B-4bit
  holdout reconfirmation remains environment-blocked (no local LM
  server) per iter 7. After iter 33, the remaining PI-runner-coverage
  gaps are: cross-mode (no PI hybrid or PI unix bundles yet — all
  nine PI bundles are mdtools mode); cross-model (all nine use
  `openai-codex/gpt-5.4-mini` at minimal thinking); cross-task within
  the iter-25-exercised scorer cell shape (T19 still uncovered;
  T11 closed iter 33 via PASS empirical F4 confirmation, T16 stayed
  FAIL frozen at iter 29 as the original F4 trace); T3 / T5 add
  their own scorer-branch combinations; the raw_bytes branch beyond
  T18 could be extended via T10 / T12 / T13 / T15 / T17. Note for
  future named-cheapest-probe entries: avoid the iter-21-style
  framing that names a corpus-vacuous code path; prefer "first PI
  cell to exercise scorer cell shape X (where X is grounded in an
  actual `bench/tasks/tasks.json` task config)" or a task-family /
  cross-mode / cross-model gap.
- **Iter-33 same-family-rule discharge:** iter 30 was oracle-
  trustworthiness (closing F4 via schema-aware
  `select_json_envelope_actual` + 8 synthetic tests), iter 31 was
  closure-discipline ratification paired with `bench/RESULTS.md:72`
  substantive downgrade, iter 32 was oracle-trustworthiness
  (typed-test promotion of iter 30 / iter 31 prose-only replay
  claim via `F4ClosureBundleReplayTests`). Iter 33 is **intervention-
  diversity** (expensive outer channel run + new durable PI bundle),
  shifting axis cleanly from iter 32. The forced expensive-or-halt
  mandate at iter 33 (per the spec's 3-consecutive-quiet rule) is
  its own escape clause for the same-family rule, parallel in shape
  to iter 25's discharge of iter-22 / -23 / -24 same-family pressure,
  iter 14's discharge of iter-11 / -12 / -13 same-family pressure,
  and iter 29's discharge of iter-26 / -27 / -28 same-family pressure.
- **Iter-32 same-family-rule discharge:** iter 28 was oracle-
  trustworthiness (typed-test promotion of iter-27 corpus-vacuous-
  path prose claim), iter 29 was intervention-diversity (forced
  expensive-or-halt: T16 mdtools PI bundle that surfaced **F4 P1
  OPEN**), iter 30 was oracle-trustworthiness (closing F4 via
  schema-aware `select_json_envelope_actual` + 8 synthetic tests),
  iter 31 was closure-discipline ratification paired with
  `bench/RESULTS.md:72` substantive downgrade. Iter 32 is **oracle-
  trustworthiness** (typed-test promotion of iter 30 / iter 31
  prose-only replay claim to a cheap-channel assertion against the
  durable iter-29 T16 bundle's `agent_output.txt` —
  `F4ClosureBundleReplayTests`). This is the third oracle-
  trustworthiness move in the iter-28..32 window, but the
  intervening iter-29 expensive run and iter-31 closure-discipline
  ratification (with substantive `bench/RESULTS.md:72` edit) both
  break the chain. Structural precedent: iter 28's relation to iter
  27 (same shape — promote prior-iter prose claim about a durable
  property to a typed cheap-channel assertion), iter 17's relation
  to the spec stamping requirement, iter 16's relation to iter-3's
  pre-recorded promotion-from-procedural-to-mechanical hint. The
  fresh-failing-trace escape clause additionally applies through
  iter-28's explicit learning that "prose claims about typed-
  artifact properties are a structurally weaker class of evidence
  than mechanical cheap-channel assertions" — the iter-30 / iter-31
  REPL-only replay outputs were exactly that weaker-class evidence,
  and iter 32 promotes them.
- **Iter-31 same-family-rule discharge:** iter 28 was oracle-
  trustworthiness (typed-test promotion), iter 29 was
  intervention-diversity (forced expensive-or-halt: T16 mdtools
  PI bundle that surfaced **F4 P1 OPEN**), iter 30 was oracle-
  trustworthiness (closing F4 via schema-aware
  `select_json_envelope_actual`). Iter 31 is **closure-discipline
  ratification** (procedural axis), structurally distinct from
  oracle-trustworthiness — precedent: iters 3, 6, 12, 13, 15, 20,
  22, 24, 27, each ratifying a prior iteration's typed-artifact
  claims without authoring a new fix. Closure-discipline iters
  paired with substantive corrective work are precedent at iters
  12, 13, 20 (line-citation drift) and iter 22 / 24 / 26 / 27
  (forward-pointing line-typo / scan-methodology / code-path-routing
  corrections). Iter 31 follows the iter-12 / iter-13 / iter-20
  pattern: ratification paired with one fresh-trace correction (the
  iter-30 line-1478 citation typo). The fresh-failing-trace escape
  clause additionally applies — the typo is the trace, surfaced by
  independent re-verification of the iter-30 entry's line citations
  against the live `bench/harness.py` file.
- **Iter-30 same-family-rule discharge:** iter 27 was closure-
  discipline (ledger-only ratification + one forward-pointing
  code-path-routing correction), iter 28 was oracle-trustworthiness
  (typed-test promotion of iter-27's prose claim), iter 29 was
  intervention-diversity (forced expensive-or-halt: T16 mdtools PI
  bundle that surfaced **F4 P1 OPEN**). Iter 30 is **oracle-
  trustworthiness** (closing F4 via schema-aware
  `select_json_envelope_actual`). Two oracle-trustworthiness moves
  (iter 28 and iter 30) bracket one intervention-diversity move
  (iter 29); not a same-family concentration because the
  intervening expensive run broke the chain. Additionally the
  fresh-failing-trace escape clause applies: F4 itself is the
  trace, filed in iter 29 with a typed bundle pointer
  (`checkpoint-pi-T16-mdtools-gpt5.4mini-2026-04-26/results.json`'s
  `correct=False`). The iter-29 ledger entry's pre-recorded
  invitation — "iter 30's first task is to either close F4 or do
  other admissible work; halt cannot fire on this counter until F4
  closes and at least one subsequent quiet round completes" — is
  exactly the kind of forward-pointing forcing function that
  iter-11 learning #1 names, and iter 30 acts on it.
- **Iter-29 same-family-rule discharge:** iter 26 was specification
  coherence (cross-executor table fifth-row publication), iter 27 was
  closure-discipline (ledger-only ratification of iter 26 + one
  forward-pointing code-path-routing correction), iter 28 was oracle-
  trustworthiness (typed-test promotion of iter-27's prose claim).
  Iter 29 is **intervention-diversity** (forced expensive-or-halt
  discharge: T16 mdtools PI bundle), shifting axis cleanly from
  iter 28. The forced expensive-or-halt mandate at iter 29 is its own
  escape clause for the same-family rule, parallel in shape to iter
  25's discharge of iter-22 / -23 / -24 same-family pressure, iter
  18's discharge of iter-16 / -17 same-family pressure, and iter 14's
  discharge of iter-11 / -12 / -13 same-family pressure. The expensive
  run additionally implicitly ratifies iter-28's
  `ScorerDispatcherBranchTests` by not re-raising the cheap-channel
  property (the dry-run on all 24 tasks before and after the expensive
  run still passes; T16 falls into the **json_canonical** branch per
  the iter-28 dispatcher classification at `bench/harness.py:363`,
  alongside T9 / T11 / T19, and the iter-28 negative-case invariant —
  no task reaches line 378 — is preserved). The F4 finding does **not**
  invalidate iter-28's typed-test work: F4 is a defect in the
  `expected_artifact == "json_envelope"` branch's `actual` extraction
  (the harness path that decides what string to feed `score_task`),
  whereas iter-28's tests assert the dispatcher predicate routing
  inside `score_task` itself. The two layers are orthogonal.
- **Iter-28 same-family-rule discharge:** iter 25 was
  intervention-diversity (T9 PI expensive run), iter 26 was
  specification coherence (cross-executor table fifth-row
  publication), iter 27 was closure-discipline (ledger-only
  ratification of iter 26 + one forward-pointing code-path-routing
  correction in iter-25 prose). Iter 28 is oracle-trustworthiness
  hardening (typed-test promotion of iter-27's corpus-vacuous-path
  prose claim into `ScorerDispatcherBranchTests`), shifting axis
  cleanly from iter 27. Parallel in shape to iter 17's "promote spec
  requirement to mechanical assertion" move (also after a chain of
  cash-out / ratification iterations) and to iter 16's "promote
  procedural protection to runtime mechanical" move. The
  fresh-failing-trace escape clause additionally applies via
  iter-27's specific corpus-vacuous-path observation; the trace is
  durable (the iter-25 line-378 prose claim is preserved in the
  ledger per iter-15 discipline, and the corpus-vacuous property is
  the underlying invariant). The "ledger-only changes do not break
  concentration" caveat is not invoked here because iter 28 is not a
  ledger-only iteration — it adds production-graph test code to
  `bench/test_harness_task_split.py`.
- **Iter-27 same-family-rule discharge:** iter 24 was closure-
  discipline (ledger-only ratification of iter 23 + forward-pointing
  citation accuracy correction), iter 25 was intervention-diversity
  (expensive outer channel run + new durable PI bundle), iter 26 was
  specification coherence (additive measurement publication of the
  iter-25 T9 PI bundle as a fifth row in the cross-executor table,
  paired with closure-discipline ratification of iter 25). Iter 27
  is closure-discipline (ledger-only ratification of iter 26's
  substantive RESULTS.md publication + one forward-pointing
  code-path-routing correction in iter-25 prose). Parallel in
  structural position to iter 24 (after iter 23 cash-out + iter 22
  ratification, iter 24 was ledger-only ratification + forward-pointing
  correction), iter 22 (after iter 21 expensive, iter 22 was
  ledger-only ratification + forward-pointing corrections), and iter
  15 (after iter 14 expensive, iter 15 was ledger-only ratification
  with no fresh trace). The fresh-failing-trace escape clause applies:
  the iter-25 line-378 / line-355 prose-accuracy defect is the first
  "code-path-routing accuracy" defect surfaced via ratification in
  this run (prior shapes: line-drift in RESULTS.md at iter 13 / iter
  20, ledger typos at iter 22, ledger citation accuracy at iter 24,
  scan methodology error at iter 26). The chain
  (intervention-diversity → spec-coherence → closure-discipline) is
  alternating, not concentrated; the "ledger-only changes do not
  break concentration" caveat does not block this iteration because
  there is no concentration to break.
- **Iter-26 same-family-rule discharge:** iter 23 was specification
  coherence (substantive RESULTS.md:67 publication + closure-discipline
  ratification of iter 22), iter 24 was closure-discipline (ledger-only
  ratification of iter 23 + forward-pointing citation accuracy
  correction), iter 25 was intervention-diversity (expensive outer
  channel run + new durable PI bundle). Iter 26 is specification
  coherence (additive measurement publication of the iter-25 T9 PI
  bundle as a fifth row in the cross-executor table, paired with
  closure-discipline ratification of iter 25). The iter-25 expensive
  run reset the quiet-signal counter and broke any prior concentration;
  iter 26's single specification-coherence move is admissible without
  invoking a fresh-failing-trace escape clause. Parallel in structural
  position to iter 19 (after iter 18 expensive, iter 19 specification
  coherence cash-out + closure-discipline ratification): both extend
  the cross-executor table by one row immediately after an expensive-
  channel PI bundle, and both pair the table extension with closure-
  discipline ratification of the prior expensive-run iteration.
- **Iter-25 same-family-rule discharge:** iter 22 was closure-
  discipline (ledger-only forward-pointing typo corrections), iter
  23 was specification coherence (substantive RESULTS.md:67
  publication + closure-discipline ratification of iter 22), iter
  24 was closure-discipline again (ledger-only ratification of iter
  23 + forward-pointing citation accuracy correction). Iter 25 is
  intervention-diversity (expensive outer channel run + new durable
  PI bundle on a previously-uncovered scorer cell shape), shifting
  axis cleanly. The quiet-signal rule's expensive-or-halt mandate
  is its own escape clause for the same-family rule when the
  iteration is forced to act, parallel in structural position to
  iter 4 (after iters 1–3 oracle-trustworthiness), iter 14 (after
  iters 11–13 specification coherence), iter 18 (after iters 15–17
  oracle-trustworthiness + observability), and iter 21 (after iters
  19–20 specification coherence). The structural pattern "three
  quiet iterations on adjacent axes followed by a forced expensive
  axis-shift" has now fired in this run at iter 4, iter 14, iter
  18, iter 21, and iter 25 — a stable rhythm where ledger-only and
  cheap-channel-only iterations alternate with intervention-
  diversity expensive runs at the spec's mandated quiet-signal
  rule firing.
- **Iter-24 same-family-rule discharge:** iter 22 was closure-
  discipline (ledger-only with two forward-pointing typo
  corrections), iter 23 was specification coherence (substantive
  RESULTS.md:67 publication paired with closure-discipline
  ratification of iter 22). Iter 24 is closure-discipline again
  (ledger-only ratification of iter 23 paired with one
  forward-pointing citation accuracy correction). The fresh-
  failing-trace escape clause applies cleanly: the iter-22 /
  iter-23 citation accuracy chain is the third instance of
  "ratification surfaces a citation defect" in this gnhf run,
  parallel in shape to iter 13 / iter 20 (line-drift in RESULTS.md)
  and to iter 22 itself (typos in iter-21 ledger prose), but
  located in iter-22's forward-pointer line numbers rather than in
  iter-21's prose. The chain (closure-discipline → spec-coherence
  → closure-discipline) is alternating, not concentrated; the
  "ledger-only changes do not break concentration" caveat does not
  block this iteration. Iter 25 cannot continue this pattern
  without first running the expensive outer channel — the
  quiet-signal counter has reached 3 for the third time in this
  run (after iters 13 and 17 prior).
- **Iter-23 same-family-rule discharge:** iter 21 was
  intervention-diversity (T21 PI bundle expensive run), iter 22 was
  closure-discipline ratification (ledger-only, parallel to iter 15).
  Iter 23 is specification coherence (additive measurement
  publication of the iter-21 T21 PI bundle as the sixth PI bundle in
  the cross-executor section), which is the first specification-
  coherence move since iter 20 — well clear of any concentration with
  iter 21 (intervention-diversity) and iter 22 (closure-discipline)
  between. The fresh-failing-trace escape clause additionally
  applies: the iter-21 T21 PI bundle has been sitting in
  `bench/runs/` since iter 21 uncited in the cross-executor section
  of `bench/RESULTS.md`, and the iter-21 entry's own "content-
  delivery T2 is the same gap class" framing was the pre-recorded
  forcing function that became actionable when iter 23 surfaced the
  gap during routine reading. Parallel in shape to iter 19's cashing
  out of iter-14's T18 PI bundle 5 iterations later.
- **Iter-22 same-family-rule discharge:** iter 19 was specification
  coherence (additive measurement publication), iter 20 was also
  specification coherence (corrective line-drift fix paired with
  iter-19 ratification), iter 21 was intervention-diversity
  (expensive channel frontmatter_json bundle). Iter 22 is a
  ledger-only closure-discipline ratification (parallel to iter 15's
  relation to iter 14): the iter-21 expensive run already broke any
  prior concentration, so iter 22's single ratification entry is
  admissible. Differs from iter 15 (no fresh trace surfaced) in that
  iter 22 surfaces two citation typos in the iter-21 entry —
  parallel in shape to iter 12 (argparse `--executor pi-json` typo
  in iter-11's RESULTS.md edit), iter 13 (line-drift in RESULTS.md:152
  for F3-a rstrip), and iter 20 (line-drift in RESULTS.md:54 for
  bytes_output). Differs from iters 12/13/20 in that the typos are
  in the ledger (auxiliary memory) rather than in published narrative
  (consumer-facing), so the iter-15 "do not silently edit historical
  entries" discipline applies and the corrections are recorded
  forward-pointing rather than as direct edits to the iter-21 entry.
  The "ledger-only changes do not break concentration" caveat does
  not block this iteration because there is no concentration to break
  (iter 21 reset it).
- **Iter-21 same-family-rule discharge:** iter 19 was specification
  coherence (additive measurement publication), iter 20 was also
  specification coherence (corrective line-drift fix paired with
  iter-19 ratification). A third consecutive same-axis move at iter
  21 would extend the chain to clear concentration; the same-family
  rule required either an axis shift, a fresh failing trace, or
  halt. The iter-21 pre-iteration sweep surfaced no fresh failing
  trace (all citations bit-exact). With no fresh trace and a cheap,
  anchored intervention available (T21 PI run extending PI
  scorer-branch coverage to `compare_frontmatter_json`), the axis
  shift to intervention-diversity is the cleanest discharge.
  Parallel to iter 10 (after iters 8–9 spec-coherence), iter 14
  (after iters 11–13 spec-coherence), iter 18 (after iters 16–17
  oracle hardening). The shift is independently justified by the
  missing-evaluator-artifact frontier anchor — T21 is the only task
  in the corpus that exercises `compare_frontmatter_json`, and the
  PI executor had not previously been exercised on that branch.
- **Iter-20 same-family-rule discharge:** iter 19 was specification
  coherence (additive measurement publication); iter 20 is also
  specification coherence (corrective line-number drift fix paired
  with closure-discipline ratification of iter 19). Two same-axis
  moves in a row is borderline, but iter 20 cites a fresh failing
  trace — the drifted `bench/harness.py:1229` and `:1265` citations
  at RESULTS.md:54 point to wrong lines (the actual bytes_output
  computation is at 1282 and 1318 in the current file, after iter 16
  and iter 17 added ~53 lines of code above). Per the same-family
  rule's "cite a fresh failing trace" escape clause, the trace makes
  the same-axis move admissible. Structurally identical to iter 13's
  pairing of line-drift fix with iter-12 ratification (which itself
  paired typo fix with iter-11 ratification). Three line-numberic-or-
  argparse-citation drift fixes paired with ratification passes is
  the same pattern repeating with new triggers (iter 12 typo, iter 13
  rstrip line drift, iter 20 bytes_output line drift), each surfaced
  as a fresh trace by the verification step itself.
- **Iter-19 same-family-rule discharge:** iter 16 was oracle
  hardening (runtime guard), iter 17 was oracle hardening
  (holdout_version stamping), iter 18 was the expensive channel
  (T2 PI bundle). Iter 19 is a specification-coherence move
  (additive measurement publication parallel to iter 11) — not
  same-family with any of iters 16–18. The specification-coherence
  axis was last touched at iter 13 (line-number drift correction),
  so this is a fresh axis from the 6-iteration perspective. The
  fresh-failing-trace escape clause additionally applies: the
  iter-14 T18 PI bundle has been sitting in the repo since iter 14
  uncited in the cross-executor section, and iter 11's learning #1
  ("Future expensive-channel runs should be examined for downstream
  pairing potential") is the pre-recorded forcing function.
- **Iter-18 same-family-rule discharge:** iter 16 was
  oracle-trustworthiness hardening (runtime-guard mechanical
  promotion), iter 17 was also oracle-trustworthiness hardening
  (per-bundle telemetry stamping), iter 18 is an
  intervention-diversity / failure-legibility move (expensive
  outer channel introducing fresh signal). The shift from
  oracle-axis substantive code edits to intervention-diversity is
  itself the discharge — parallel to iter 4 (after iters 1–3
  oracle), iter 7 (after iters 5–6 spec-coherence + review),
  iter 10 (after iters 8–9 spec-coherence), iter 14 (after iters
  11–13 spec-coherence). The forcing function is the spec's
  quiet-signal-counter rule firing at 3 after iter 17, which makes
  the intervention shift independently mandated regardless of the
  same-family rule's evaluation.
- **Iter-17 same-family-rule discharge:** iter 14 was an
  expensive-channel run (intervention diversity), iter 15 was a
  ledger-only closure-discipline ratification (rule explicitly
  excludes ledger-only changes from concentration), iter 16 was an
  oracle-trustworthiness hardening change (runtime-guard mechanical
  promotion), iter 17 is also an oracle-trustworthiness hardening
  change (per-bundle telemetry stamping). Two consecutive oracle-axis
  substantive code changes is borderline same-family; the
  fresh-failing-trace escape clause applies because the four
  pre-iter-17 PI bundles (T1, T7, T18, T22) all lack the
  `holdout_version` field that the spec explicitly requires —
  parallel to iter 13's line-number-drift trace (a published
  instruction that doesn't match the code). The fix is the smallest
  reversible change that closes the documented gap.
- **Iter-16 same-family-rule discharge:** iter 14 was an
  expensive-channel run (intervention diversity), iter 15 was a
  ledger-only closure-discipline ratification (which the rule
  explicitly says does *not* break concentration), and iter 16 is an
  oracle-trustworthiness hardening change with substantive code edits
  + new unit tests. Iter 14 already broke any prior concentration;
  iter 15 left it broken; iter 16's harness-assertion change is the
  first substantive code edit since iter 13's RESULTS.md line-number
  fix (3 iterations earlier) and the first oracle-axis code edit
  since iter 2's L1 closure (14 iterations earlier). No same-family
  concentration to discharge.
- **Iter-15 same-family-rule discharge (preserved):** iter 14 was an
  expensive-channel run (intervention diversity), which broke the
  iters 11–13 spec-coherence concentration. Iter 15 returns to a
  ledger-only ratification entry (parallel to iter 12's review of
  iter 11 and iter 13's review of iter 12) — the spec's "ledger-only
  changes do not break concentration" caveat does not block this
  iteration because there is no concentration to break (iter 14 reset
  it). Iter 15 differs from iters 12 and 13 in that no fresh failing
  trace surfaced during the verification, so no substantive fix is
  paired with the ratification. This is the first "ratification-only,
  no fix paired" iteration in this gnhf run; structurally analogous
  to iters 3 and 6 (closure-discipline review passes that promoted
  FIXED_PENDING_CONFIRMATION findings to CLOSED without authoring a
  new fix), but with iter-14 being a non-finding bundle introduction
  rather than a FIXED finding.
- **Iter-14 same-family-rule discharge (preserved):** iters 11–13 were three
  consecutive specification-coherence iterations (iter 11 measurement
  publication, iter 12 typo fix + closure-discipline pass, iter 13
  line-number drift fix + closure-discipline pass). The same-family
  rule blocked a fourth consecutive same-axis move absent a fresh
  failing trace. Iter 14 cleanly shifts to a different intervention
  shape (`comparable-harness-axis cell coverage` via expensive run on
  previously-uncovered family) without invoking the fresh-trace
  escape clause — the quiet-signal rule mandates expensive-or-halt at
  the 3-quiet boundary, so the intervention is independently required
  by the spec. Parallel to iter 4 (after 3 quiet iters 1–3) and iter
  10 (proactive intervention shift before the counter fired).
- **Iter-13 same-family-rule discharge:** iter 11 was specification
  coherence (additive measurement publication); iter 12 was
  closure-discipline review pass + corrective spec-coherence (typo
  fix); iter 13 is closure-discipline review pass + corrective
  spec-coherence (line-number drift fix). Three same-axis moves in a
  row is genuine concentration, but iter 13 cites a fresh failing
  trace — the published `bench/harness.py:339-341` reference at
  `bench/RESULTS.md:152` does not point to the F3-a rstrip fix in the
  current file (the rstrip body is at 347-348, an 8-line drift since
  the original 2026-04-24 RESULTS.md authoring). A reader following
  the published citation lands on a comment and a branch entry, not
  on the rstrip fix. Per the same-family rule's "cite a fresh failing
  trace" escape clause, the trace makes the same-axis move admissible.
  The iter-13 work is paired with the closure-discipline ratification
  of iter 12 (parallel to iter 12's pairing of typo fix with iter-11
  ratification, and parallel to iter 9's pairing of retracted-README
  spec-cleanup with iter-8 RESULTS.md ratification).
- **Iter-12 same-family-rule discharge (preserved):** iter 11 was
  specification coherence (additive measurement publication). Iter 12
  is also specification coherence (corrective reference fix), citing a
  fresh failing trace — the published `--executor pi-json` instruction
  at `bench/RESULTS.md:54` (added by iter 11) contradicted the harness
  CLI flag definitions at `bench/harness.py:1498` (`--runner pi-json`)
  and `:1513` (`--executor` distinct flag for shell mode). An external
  reader following the published instruction would hit argparse
  rejection. Per the same-family rule's "cite a fresh failing trace,
  external finding, or blocked claim" escape clause, the trace makes
  the same-axis move admissible. The iter-12 work is paired with the
  closure-discipline ratification of iter 11 (parallel to iter 9's
  pairing of retracted-README spec-cleanup with iter-8 RESULTS.md
  ratification).
- **Iter-11 same-family-rule discharge (preserved):** iter 11 published
  the same-task cross-executor measurement that iter 10's bundle made
  possible. This is specification-coherence work, the same axis as
  iters 8 and 9, but the same-family chain was broken by iter 10's
  expensive-channel run, and the move cited a fresh forcing function
  (the iter-5 P3 closure entry's learning #1) that only became
  actionable after iter-10's T7 PI bundle paired with the pre-existing
  T7 OAI-loop bundle.
- **Iter-10 same-family-rule discharge (preserved):** iters 8 and 9 were
  both specification-coherence cleanups, so iter 10 was constrained by
  the same-family admissibility rule from a third spec-cleanup absent a
  fresh failing trace (none surfaced — full grep across `README.md`,
  `CLAUDE.md`, `specs/**`, `bench/RESULTS.md`,
  `bench/retracted_2026-04-24/README.md`, `bench/ledger.md` confirms
  zero remaining stale F3 / L1 / pending-fix references). Iter 10
  shifted intervention to the comparable-harness-axis frontier anchor
  instead of halting, because a real cell gap was available at low cost
  (PI runner had been exercised only on extraction tasks T1 and T22;
  mutation- and multistep-family cells were absent from the PI
  executor's coverage).
- **Iter-7 obligation history (preserved):** iter 7 ran a PI runner smoke
  on the F3-affected holdout task (T22) — the cheapest *reachable* probe
  in this environment, since the previously-named cheapest probe
  (Qwen3.5-122B-A10B-4bit holdout reconfirmation) requires a local LLM
  server that is not running here. Bundle:
  `bench/runs/checkpoint-pi-T22-mdtools-gpt5.4mini-2026-04-26/`. PASS in
  9.63s with 1 `md links … --json` tool call,
  `diff_report: link_destinations: OK`. Iter 10's bundle does **not**
  supersede or compare to iter 7's; they are different cells (different
  task, different family, different scorer branch).
- **Product-anchor admissibility unchanged:** promoting a product anchor
  (`md apply` / `md batch`) is still inadmissible without a Route A or
  Route B justification under `bench/probes/anchor-validation/`, which
  still does not exist. Iter 10's mutation-family bundle is *evaluator
  coverage*, not anchor justification — a passing T7 cell does not
  validate any candidate primitive's failure-class fit, and was not
  framed as such.

### Cross-executor same-task measurement publication (2026-04-26 iter 11)

Iter 11 cashed out iter 10's PI T7 bundle by publishing the first
same-task cross-executor measurement table in `bench/RESULTS.md`. The
table pairs three PI bundles (T1, T7, T22 — the only three PI bundles in
`bench/runs/`) with their pre-existing OAI-loop counterparts and
validates the published P3 cross-executor comparability rule with
measurement, not just code-reading.

- **Disturbance:** specification coherence — the published cross-executor
  section in `bench/RESULTS.md:52-60` made an assertion (`The gap is not
  driven by task or model`) supported only by a *different-task* pair
  (T1 PI vs T20 OAI), which the iter-5 P3 closure entry's learning #1
  flagged as a class of disclosure to avoid. Iter 10's T7 PI bundle was
  the third PI bundle in the repo, completing the third same-task pair
  with an existing OAI-loop counterpart. Without an iter-11-shaped
  publication, the iter-10 bundle would sit unincorporated as a typed
  artifact whose published implication is uncited.
- **Anchor:** missing evaluator artifact — *durable summary for a
  newly-run comparison*. Same anchor wording as iters 8 and 9, but the
  intervention is *additive measurement publication* (citing new same-task
  data) rather than *corrective reference removal* (which iters 8 and 9
  performed). The chain was broken by iter 10's expensive run, so
  same-axis is admissible per the same-family rule's escape clause.
- **Change:** one targeted edit in `bench/RESULTS.md` replacing the
  iter-5-era one-paragraph contrast (T1 PI vs T20 OAI; different cells,
  three orders of magnitude smaller) with a same-task-validation block
  containing (a) a 3-row table of T1 / T7 / T22 mdtools cells across
  executors, (b) explicit acknowledgement of the model confound
  (gpt-5.4-mini PI vs Qwen3.5-122B-A10B-4bit OAI), (c) the
  ratio-of-magnitude observation across all three pairs, and (d) the
  explicit caveat that the T22 OAI cell predates the F3 fix but
  `bytes_output` / `bytes_observation` / `tool_calls` are behavior
  measurements unaffected by F3. The **Rule** paragraph was tightened to
  note that the same-task table corroborates the bytes_observation claim
  with measurement, scaling with tool-call count rather than executor.
- **Data points used:**
  - T1 mdtools: PI (iter 4) `bytes_output=5,975,843`,
    `bytes_observation=2,266`, 1 tool call, 0 mutations vs OAI
    `bytes_output=2,702`, `bytes_observation=2,436`, 1 tool call,
    0 mutations. Ratio: ~2,212×; bytes_observation Δ: −7%.
  - T7 mdtools: PI (iter 10) `bytes_output=1,172,040`,
    `bytes_observation=16,219`, 3 tool calls, 1 mutation vs OAI
    `bytes_output=699`, `bytes_observation=13,671`, 3 tool calls,
    1 mutation. Ratio: ~1,677×; bytes_observation Δ: +19%.
  - T22 mdtools: PI (iter 7) `bytes_output=671,515`,
    `bytes_observation=514`, 1 tool call, 0 mutations vs OAI
    `bytes_output=488`, `bytes_observation=1,036`, 2 tool calls,
    0 mutations. Ratio: ~1,376×; bytes_observation scales with tool-call
    count (OAI cell made an extra read).
- **Cheap channel:** green before and after (`cargo test -q` all suites
  pass, 59 python unittests OK across the 8 spec-named modules,
  `harness.py --md-binary` dry-run all 24 tasks PASS dual-scorer).
- **Comparability framing:** the published table is **NOT** an
  apples-to-apples comparison — it is model-confounded across each pair
  (PI: gpt-5.4-mini at minimal thinking; OAI: Qwen3.5-122B-A10B-4bit).
  The rule under test is about executor stdout shape, not model. Pass
  rates are not aggregated across the table — `correct` is a per-cell
  fact preserved in the underlying results.json files, not republished
  here. The behavioral consistencies the table surfaces (T1 and T7 both
  produce the same tool-call and mutation count across executors) are
  reported as *observations*, not as comparisons; T22's tool-call
  divergence (1 PI vs 2 OAI) is the data point that surfaces the
  bytes_observation scaling rule.
- **Verdict:** specification restored. The published P3 rule is now
  measurement-validated rather than only code-derived. The future
  `bytes_assistant_content` ratchet remains unchanged — option (a) from
  the original P3 entry is still a viable additive ratchet should a
  fresh failing claim or external finding make it necessary, but no
  forcing function exists yet. No new finding opened, no holdout
  artifact touched (`bench/RESULTS.md` is published narrative; the T22
  OAI-loop bundle is in `bench/runs/holdout-mdtools-...` but iter 11
  only *cites* it — it does not modify it). A future review pass may
  treat this measurement-publication as `FIXED_PENDING_CONFIRMATION`-
  equivalent and ratify it by re-reading the table against the cited
  results.json files.

### Comparable-harness-axis cell coverage extension (2026-04-26 iter 10)

Iter 10 broke the iters 8–9 specification-cleanup same-family pattern by
running the expensive outer channel on a previously-uncovered cell: T7
mdtools through the PI runner. This is the **third** PI runner bundle in
`bench/runs/` and the **first** cell that exercises (a) the
targeted-mutation task family, (b) the `normalized_text` scorer branch,
and (c) the canonical re-query pattern (read → mutate → verify) end-to-end
through the PI executor.

- **Disturbance:** intervention diversity — drifting toward
  spec-cleanup concentration after iters 8 and 9 both did
  specification-coherence work. Same-family admissibility forced iter 10
  to either shift to a different intervention shape, cite a fresh failing
  trace, or halt with stop-and-summarize. No fresh failing trace existed
  (full sibling-narrative grep was clean), so the only constructive
  options were intervention shift or halt.
- **Anchor:** missing evaluator artifact — *comparable harness axis*. The
  PI runner had been exercised only on extraction tasks (T1 in iter 4,
  T22 in iter 7); the mutation, multistep, and content-delivery families
  had zero PI cells. Adding a single mutation-family bundle closes the
  largest cell gap in PI executor coverage at low cost. (Halt was
  defensible too, but premature given the available cheap, anchored
  intervention.)
- **Bundle:** `bench/runs/checkpoint-pi-T7-mdtools-gpt5.4mini-2026-04-26/` —
  Single task (T7, search-split, targeted-mutation family). Single mode
  (mdtools). Single run. Model `openai-codex/gpt-5.4-mini` at
  `thinking_level=minimal`, recorded per-result and per-run on the
  metadata bundle.
- **Verdict:** T7 mdtools dual-scorer PASS in 16.45s with 3 tool calls
  (`./md tasks … --json` query, `./md set-task 9.3 -i --status done`
  mutation, `./md tasks … --json --status done` verification re-query),
  1 mutation, `requery_rate=1.0`, `bytes_observation=16,219`,
  `bytes_output=1,172,040` (PI streaming overhead, see P3 cross-executor
  rule in `bench/RESULTS.md`),
  `diff_report: heading_tree [md]: OK / block_order [md]: OK / block_text
  [md]: OK / heading_tree [neutral]: OK / block_text [neutral]: OK`.
  Pi-audit log preserved at
  `logs/T7_mdtools_1777212494/pi-audit.jsonl` (8 events: `model_change`,
  `thinking_level_change`, plus 3 × `tool_call` + 3 × `tool_result`),
  parses cleanly via `bench/pi_audit_adapter.summarize_pi_audit_events`
  with `tool_calls=3`, `mutations=1`, `bytes_observation` populated.
- **Comparability framing:** this is the first cell for (PI runner,
  gpt-5.4-mini, mdtools, minimal thinking, T7, runs_per_task=1, task-set
  version: live `bench/tasks/tasks.json` with `holdout_version=1` from
  `bench/holdout/fingerprints.json`). It is **NOT** a holdout
  reconfirmation (T7 is search-split, not holdout) and **NOT** a
  comparison against the iter-4 T1 bundle or the iter-7 T22 bundle —
  same executor / model / mode / thinking / runs-per-task across all
  three, but different tasks and different scorer branches, so any
  pass-rate-aggregation across cells would be a search-set observation,
  not a comparison. Likewise it is **NOT** an apples-to-apples comparison
  to any oai-loop T7 bundle, because the executor axis differs.
- **What this exercises:** for the first time in `bench/runs/`, the PI
  runner pipeline (harness pi-json branch → `pi --mode json` → audit
  extension at `~/.pi/agent/extensions/audit/index.ts` →
  `bench/pi_audit_adapter.summarize_pi_audit_events`) is verified end-to-end
  on (a) a mutation-family task (`set-task` produces a 1-byte sourcepos
  edit), (b) the `normalized_text` scorer branch (`compare_heading_tree`
  + `compare_block_order` + `compare_block_text` all OK on both `md` and
  neutral scorers), and (c) the full re-query pattern (mutation followed
  by `--status done` verification re-query). All prior PI bundles were
  zero-mutation extraction-only.
- **What this discharges:** intervention-diversity drift. It does **not**
  discharge the spec's quiet-signal-checkpoint rule unconditionally —
  iter 10 was admissible as a quiet iteration per iter-7's forecast, but
  iter 10 explicitly chose the expensive channel over halting because the
  same-family rule blocked another spec-cleanup and the intervention shift
  required a non-spec-cleanup move. Iters 11–13 are now newly admissible
  as quiet iterations under the reset counter.
- **What it surfaced:** no new defect. The PI pipeline produced fresh
  typed signal that exercised mutation + re-query cleanly. This is a
  "no new finding" expensive run, admissible as fresh signal because the
  bundle is on a different (task, family, scorer-branch) cell than iter-4
  T1 or iter-7 T22, and the audit log + scorer outputs are durably
  persisted as a queryable bundle.
- **Cheap channel:** green before and after (`cargo test -q` all suites
  pass, 59 python unittests OK across the 8 spec-named modules,
  `harness.py --md-binary` dry-run all 24 tasks PASS dual-scorer).

### Specification coherence cleanup (2026-04-26 iter 9)

`bench/retracted_2026-04-24/README.md` line 25 carried a stale "See
`bench/ledger.md` F3 for the ongoing scorer-layer fix requirement and L1
for the loop-level learning" reference — same disturbance pattern that
iter 8 swept out of `bench/RESULTS.md`, but on a sibling published
artifact that iter 8 did not touch. F3 has been CLOSED since iter 1
(ratified iter 3 review pass, end-to-end-verified iter 7) and L1 has been
CLOSED since iter 2 (ratified iter 3 review pass). Calling either
"ongoing" contradicts the ledger CLOSED status.

- **Disturbance:** specification coherence — `bench/retracted_2026-04-24/README.md`,
  the canonical pointer for readers landing on the four retracted holdout
  bundles, framed F3 as an ongoing requirement and L1 as a generic
  "learning" without acknowledging the holdout-immutability mechanical
  guard that closes it.
- **Anchor:** same as iter 8 — the spec's "missing evaluator artifact …
  durable summary for a newly-run comparison" frontier anchor, applied to
  the retracted-bundles README. The iter-7 PI bundle is the durable
  end-to-end verification of F3 and the `fingerprints.json` +
  `verify_holdout_fingerprints` pair is the durable mechanical closure of
  L1; neither was cited from the retracted README before this edit.
- **Change:** one targeted edit in `bench/retracted_2026-04-24/README.md`
  replacing "ongoing scorer-layer fix requirement" with "scorer-layer fix
  (CLOSED 2026-04-26; end-to-end-verified through a real frontier model
  on the actual T22 holdout task in `bench/runs/checkpoint-pi-T22-mdtools-gpt5.4mini-2026-04-26/`)"
  and "L1 for the loop-level learning" with "L1 for the loop-level
  learning (CLOSED 2026-04-26 via the holdout-immutability fingerprint
  guard at `bench/holdout/fingerprints.json` and
  `verify_holdout_fingerprints` in `bench/harness.py`)". The "do not cite"
  instruction and the per-bundle invalid-pass-rate listing are
  unchanged — readers are still warned off the retracted bundles, and no
  pass rate is restated.
- **Cheap channel:** green before and after (`cargo test -q` all suites
  pass, 59 python unittests OK across the 8 spec-named modules,
  `harness.py --md-binary` dry-run all 24 tasks PASS dual-scorer with
  `link_destinations: OK` on T22).
- **Iter-8 RESULTS.md cleanup ratified during this pass:** as part of
  this iteration's spec-coherence sweep, the four lines edited in iter 8
  (`bench/RESULTS.md:118, :141, :152, :158`) were re-read against the F3
  CLOSED entry and the iter-7 PI bundle. All four updates remain
  consistent with current ledger state (F3 CLOSED, F3-a CLOSED, iter-7
  PI bundle present and parseable, published 50% Qwen pass rates
  unchanged with the "fresh-Qwen-run-pending-environment" caveat
  preserved). The iter-8 invitation to "treat the RESULTS.md cleanup as
  `FIXED_PENDING_CONFIRMATION`-equivalent and ratify it by re-reading
  the four updated lines against this ledger entry and the iter-7
  bundle" is hereby discharged.
- **Verdict:** specification restored on the retracted README; iter-8
  RESULTS.md cleanup ratified. No new finding opened, no holdout artifact
  touched (`bench/retracted_2026-04-24/README.md` is published narrative
  about retracted-and-quarantined bundles, not a holdout artifact under
  `bench/holdout/`). This completes the broader specification-coherence
  sweep that iter 8 began. A future review pass need not re-ratify
  either edit unless the underlying ledger state changes.

### Specification coherence cleanup (2026-04-26 iter 8)

`bench/RESULTS.md` carried four references describing F3 (T22 structural-array
envelope normalization) as OPEN or scorer-fix-pending: lines 118 ("Local
search-pilot takeaways"), 141 (per-task failure-analysis row for T22), 152
(Qwen mdtools row in current holdout coverage table), and 158 ("What this
confirms honestly" paragraph). All four contradicted the ledger CLOSED
status on F3 (FIXED iter 1, ratified iter 3 review pass, end-to-end-verified
through a real frontier model in iter 7's
`bench/runs/checkpoint-pi-T22-mdtools-gpt5.4mini-2026-04-26/`).

- **Disturbance:** specification coherence — published narrative
  contradicted ledger CLOSED on F3, and the iter-7 PI bundle was a durable
  artifact with no published summary in `bench/RESULTS.md`.
- **Anchor:** the spec's "missing evaluator artifact … durable summary for
  a newly-run comparison" frontier anchor.
- **Change:** four targeted edits in `bench/RESULTS.md` replacing
  "F3 pending" / "OPEN as F3" / "scorer-layer fix is pending" framing
  with CLOSED-status citation plus a pointer to the iter-7 PI bundle.
  Published 50% Qwen holdout pass rates were left unchanged — the
  original Qwen bundles have not been re-scored, and a fresh Qwen run
  is pending environment availability.
- **Cheap channel:** green before and after (`cargo test -q` all suites
  pass, 59 python unittests OK, `harness.py --md-binary` dry-run all 24
  tasks PASS dual-scorer).
- **Verdict:** specification restored. No new finding opened, no holdout
  artifact touched (`bench/RESULTS.md` is published narrative, not a
  holdout artifact under `bench/holdout/`). This is a non-finding
  iter-8 record; a future review pass may treat the
  RESULTS.md cleanup as `FIXED_PENDING_CONFIRMATION`-equivalent and
  ratify it by re-reading the four updated lines against this ledger
  entry and the iter-7 bundle.

### P3 — `bytes_output` is not cross-executor comparable
- **Status:** CLOSED (2026-04-26 iter 6 review pass; FIXED 2026-04-26 iter 5 via closure plan option (b); filed 2026-04-26 iter 4; P2 hardening backlog severity)
- **Axis:** oracle trustworthiness (cross-executor normalization); closure intervention is specification coherence
- **Anchor:** the iteration-4 quiet-signal-checkpoint PI smoke (`bench/runs/checkpoint-pi-T1-mdtools-gpt5.4mini-2026-04-26/`) recorded `bytes_output=5,975,843` for T1 mdtools on gpt-5.4-mini. The pre-fix Qwen oai-loop holdout bundle records `bytes_output=679` for T20 mdtools — three orders of magnitude smaller. Root cause: `bench/harness.py:1229` measures pi-json `bytes_output` as `len(raw_stdout.encode())`, which captures the entire `pi --mode json` JSONL stream (per-token deltas, audit envelopes, session-meta events), whereas the oai-loop path (`bench/harness.py:1265`) captures assistant terminal content.
- **Effect:** does not gate the current acceptance metric (`pass_rate`), so this is P2-severity hardening backlog, not a P0 / P1 claim block. Cross-executor comparisons that include `bytes_output` (or any derived metric) must be flagged as non-comparable until normalized.
- **Typed artifact:** `bench/RESULTS.md` now carries a "Cross-executor comparability (PI runner vs OAI loop)" section that names the executor-locality rule, cites the iteration-4 checkpoint bundle as the supporting datum, identifies the harness call sites for both branches, and enumerates the cross-executor-comparable fields (`correct`, `correct_neutral`, `elapsed_seconds`, `tool_calls`, `mutations`, `policy_violations`, `requeried`, `bytes_observation`). The section also flags the future ratchet (a `bytes_assistant_content` parser over the audit stream) without committing to it.
- **Closure plan executed:** option (b) from the original entry — cheaper documentation in `bench/RESULTS.md` rather than option (a) bytes_assistant_content extraction. Option (a) remains a viable additive ratchet if a fresh failing claim or external finding makes it necessary; the same-family admissibility rule applies.

### Quiet-signal checkpoint discharge (2026-04-26 iter 7)

Per the spec's "After 3 consecutive iterations …" rule flagged at the end
of iteration 6, iteration 7 ran the expensive outer channel to introduce
fresh typed signal before the quiet count would have fired. Cheap channel
re-verified green before and after the run.

- **Bundle:** `bench/runs/checkpoint-pi-T22-mdtools-gpt5.4mini-2026-04-26/` —
  second PI runner bundle in `bench/runs/`. Single task (T22, holdout-split,
  link extraction). Single mode (mdtools). Single run. Model
  `openai-codex/gpt-5.4-mini` at `thinking_level=minimal`, recorded
  per-result and per-run on the metadata bundle.
- **Verdict:** T22 mdtools dual-scorer PASS in 9.63s with 1 tool call
  (`./md links … --json`), `bytes_observation=514`, `bytes_output=671,515`
  (PI streaming overhead, see P3 cross-executor rule in `bench/RESULTS.md`),
  `diff_report: link_destinations: OK`. Pi-audit log preserved at
  `logs/T22_mdtools_1777210835/pi-audit.jsonl` (4 events:
  `model_change`, `thinking_level_change`, `tool_call`, `tool_result`),
  parses cleanly via `bench/pi_audit_adapter.summarize_pi_audit_events`.
- **Comparability framing:** this is the first cell for (PI runner,
  gpt-5.4-mini, mdtools, minimal thinking, T22, runs_per_task=1, task-set
  version: live `bench/tasks/tasks.json` with `holdout_version=1` from
  `bench/holdout/fingerprints.json`). It is **NOT** a reconfirmation of
  the 2026-04-24 Qwen3.5-122B-A10B-4bit oai-loop holdout bundles —
  different executor (PI vs OAI loop) and different model (gpt-5.4-mini vs
  Qwen). It crosses two of the five spec-required normalization axes
  versus those bundles, so any pass-rate comparison would be a search-set
  observation, not a comparison. The same applies versus the iter-4
  T1 bundle: same executor / model / mode / thinking / runs-per-task, but
  different task — no aggregate comparison is implied.
- **What this exercises:** the post-F3 `compare_link_destinations` scorer
  normalization path through a real frontier model on the actual T22
  holdout task. Pre-fix, the same shape of agent output (top-level JSON
  array from `md links --json`) produced
  `T22: json_envelope: MISMATCH expected top-level JSON object (actual=list, expected=dict)`
  in the 2026-04-24 Qwen mdtools bundle. Post-fix, the structural-array
  envelope normalization at `bench/harness.py:443-537` accepts the same
  output shape and the dict-vs-dict link-destination comparison passes.
  This is the first end-to-end (real-model + real-task + real-scorer)
  exercise of the F3 fix; cheap-channel coverage was via dry-run +
  unit tests only.
- **What this discharges:** the spec's quiet-signal-checkpoint rule.
  It does **not** discharge any product or oracle claim — those still
  require their own attribution probes and apples-to-apples comparisons.
- **What it surfaced:** no new defect. The PI pipeline produced fresh
  typed signal that exercised the F3 closure path cleanly. This is a
  "no new finding" expensive run — admissible as fresh signal because
  the run is on a different (task) cell than iter-4 and its outputs are
  durably persisted as a queryable bundle.

### Quiet-signal checkpoint discharge (2026-04-26 iter 4)

Per the spec's "After 3 consecutive iterations …" rule flagged at the end of
iteration 3, iteration 4 ran the expensive outer channel to introduce fresh
typed signal. Cheap channel re-verified green before and after the run.

- **Bundle:** `bench/runs/checkpoint-pi-T1-mdtools-gpt5.4mini-2026-04-26/` —
  first PI runner bundle in `bench/runs/`. Single task (T1, search-split,
  extraction). Single mode (mdtools). Single run. Model
  `openai-codex/gpt-5.4-mini` at `thinking_level=minimal`, recorded
  per-result and per-run on the metadata bundle.
- **Verdict:** T1 mdtools dual-scorer PASS in 9.83s with 1 tool call
  (`./md outline … --json`). Pi-audit log preserved at
  `logs/T1_mdtools_1777209684/pi-audit.jsonl` (4 events:
  `model_change`, `thinking_level_change`, `tool_call`, `tool_result`),
  parses cleanly via `bench/pi_audit_adapter.summarize_pi_audit_events`.
- **Comparability framing:** this is the first cell for (PI runner,
  gpt-5.4-mini, mdtools, minimal thinking, T1, runs_per_task=1,
  task-set version: live `bench/tasks/tasks.json`). It is **NOT** a
  reconfirmation of any prior holdout bundle — different executor, different
  model, different task split (T1 is search-split, not holdout). It is also
  **NOT** a comparison against any other bundle until normalized on the
  five spec-required axes. It establishes a single typed datum that the PI
  runner pipeline (harness ↔ pi --mode json ↔ audit extension ↔ pi_audit_adapter)
  works end-to-end against the live task corpus.
- **What this discharges:** the spec's quiet-signal-checkpoint rule, which
  is satisfied by introducing fresh signal via the expensive channel. It does
  **not** discharge any product or oracle claim — those still require their
  own attribution probes and apples-to-apples comparisons.
- **What it surfaced:** the new P3 finding above (`bytes_output` cross-executor
  incomparability), which is concrete fresh signal — i.e. the iteration's
  expensive run did not return null information.

### Confirmation review pass (2026-04-26)

Explicit closure-discipline review of every FIXED_PENDING_CONFIRMATION entry below.
Verified against typed artifacts, not against prose. Cheap channel was green at
review time (282 cargo tests + 8 unittest modules all passing,
`harness.py --md-binary` dry-run all 24 tasks PASS dual-scorer).

What was checked, per finding:

- **F3** — re-read `bench/harness.py:443-537`. The `only_link_destinations` gate
  excludes every other `compare_*` flag, so the list→`{"links": ...}` wrap fires
  only when `compare_link_destinations` is the sole structural check. Both sides
  (actual and expected) are wrapped, so dict-vs-dict comparison is preserved.
  Dispatcher at `harness.py:367-369` only enters `score_structural_json` when
  `expected_artifact == "json_envelope"` and `json_canonical` is False, so the
  fix does not interact with `score_json_canonical` or `json_required_keys`.
  Corpus survey: only T22 uses `compare_link_destinations` today; the
  policy-shape gating means future link-extraction tasks with the same shape
  inherit the fix without per-task wiring. All four tests in
  `bench/test_harness_json.py::StructuralLinkEnvelopeTests` cover the four
  cases listed in the closure plan. **Verdict: closed.**

- **F3-a** — confirmed `.rstrip()` at `bench/harness.py:347-348` on the
  `raw_bytes` branch's normalized strings. The structural / normalized_text
  branches do not add the trailing `.rstrip()`, but those scorers re-tokenize
  via the AST so trailing-newline difference is already absorbed. **Verdict: closed.**

- **F2** — confirmed `bench/RESULTS.md:1-3` opens with the legacy snapshot
  header and a split-disclosure paragraph naming T4/T14/T20 as now-holdout
  rows and T22–T24 as post-snapshot tasks; readers encounter the caveat
  before the per-task table. **Verdict: closed.**

- **F1** — confirmed both pre-fix holdout bundles still under
  `bench/runs/holdout-{mdtools,hybrid}-Qwen3.5-122B-A10B-4bit-2026-04-24/`
  (the 50% baseline). Confirmed `bench/retracted_2026-04-24/README.md`
  enumerates four invalidated bundles with reason and points readers to the
  pre-fix bundles as the only valid record. The "ability to run holdout"
  closure is satisfied; the "durability" claim remains explicitly retracted.
  **Verdict: closed.** (A future holdout reconfirmation run is search-cycle
  work, not a residual on F1.)

- **L1** — confirmed `bench/holdout/fingerprints.json` carries
  `holdout_version: 1` and a per-task fingerprint covering the canonical task
  JSON entry plus SHA-256 of every input/expected file byte. Confirmed
  `verify_holdout_fingerprints` raises `holdout-immutability breach` on:
  task missing from `tasks.json`, missing baseline entry, fingerprint drift,
  and extra IDs in the baseline manifest. The five tests in
  `HoldoutImmutabilityTests` exercise live-vs-baseline match, manifest shape,
  description-drift detection, expected-output bytes drift detection, and
  determinism. The cheap-channel test
  (`test_live_holdout_matches_recorded_fingerprints`) is the mechanical
  block. **Verdict: closed.**

### Review-pass observation (informational, not P2)

`verify_holdout_fingerprints` is exercised only via the cheap-channel unit
test; the harness does not auto-invoke it when a holdout split is selected at
runtime (e.g. `--task-ids-path bench/holdout/task_ids.json`). The closure
plan was explicitly "cheap channel mechanically blocks", which is satisfied,
and the iteration protocol step 7 ("Run the cheap validator; if it passes,
run the stronger oracle") sequences the test before any expensive run. So
this is a procedural — not mechanical — closure. A future iteration could
add a runtime invocation as an additive ratchet, but only if a fresh failing
trace, external finding, or blocked claim makes the same-axis move
necessary; absent that, the same-family admissibility rule applies.

### Quiet-signal checkpoint status (informational)

Iterations 1, 2, and 3 of this gnhf run all (a) kept the cheap channel green,
(b) added no failing trace, and (c) added no new OPEN finding to the ledger
surface. Per the spec's "After 3 consecutive iterations …" rule, the next
iteration must either run the expensive outer channel to introduce fresh
signal (e.g. a holdout reconfirmation run on Qwen3.5-122B-A10B-4bit now that
F3 is closed) or emit `stop-and-summarize`.

### F1 — Search-split pilots lack holdout confirmation (partial)
- **Status:** CLOSED (2026-04-26 review pass; FIXED 2026-04-24)
- **Axis:** oracle trustworthiness / specification coherence
- **Typed artifacts:** `bench/runs/holdout-{mdtools,hybrid}-Qwen3.5-122B-A10B-4bit-2026-04-24/` (pre-fix, 50% each). Four additional bundles produced during the loop have been moved to `bench/retracted_2026-04-24/` with an invalidation README — see that directory.
- **Substantive outcome:** holdout can now be run, durable bundles exist, and the first real holdout run exposed scorer-surface defects the search split hid. The narrower claim "holdout-confirmed at 100%" has been retracted — current valid holdout result is 50% for the best-in-class search cell, with the F3 scorer defect (now closed) having previously prevented any honest reconfirmation.

### F2 — Legacy N=3 snapshot overlaps the current holdout set
- **Status:** CLOSED (2026-04-26 review pass; FIXED 2026-04-24)
- **Axis:** specification coherence
- **Typed artifact:** `bench/RESULTS.md` opens with a "Legacy N=3 Haiku snapshot" header and a split-disclosure note stating the snapshot predates the search/holdout split, naming T4/T14/T20 as now-holdout rows and T22–T24 as post-snapshot tasks. Readers encounter the caveat before the per-task table.

### F3 — T22 structural scorer rejects list-shape JSON with mode-neutral task description
- **Status:** CLOSED (2026-04-26 review pass; FIXED 2026-04-26 iter 1)
- **Axis:** oracle trustworthiness
- **Typed artifact:** `bench/harness.py:score_structural_json` normalizes a top-level JSON array to `{"links": [...]}` when `compare_link_destinations` is the sole structural check. Mode-neutral; gated on policy shape, not task ID. The pre-fix anchor remains `bench/runs/holdout-{mdtools,hybrid}-Qwen3.5-122B-A10B-4bit-2026-04-24/` which recorded `T22: json_envelope: MISMATCH expected top-level JSON object (actual=list, expected=dict)`. Unit tests at `bench/test_harness_json.py::StructuralLinkEnvelopeTests` cover the list/dict equivalence, mismatched-link rejection, and the multi-check guardrail (top-level list still rejected when other comparisons are required). Harness dry-run all 24 tasks pass dual scorer.
- **Holdout-version note:** treated as a mode-neutral scorer bug fix (precedent: F3-a EOF whitespace). The change is not gated on T22 specifically; it applies to any task with policy shape `compare_link_destinations` only. Pre-fix and post-fix holdout T22 bundles are not apples-to-apples for that one task; future holdout runs are the fresh baseline.

### F3-a — `raw_bytes` scorer now honors EOF whitespace
- **Status:** CLOSED (2026-04-26 review pass; FIXED 2026-04-24)
- **Axis:** oracle trustworthiness
- **Typed artifact:** `bench/harness.py:347-348` — `.rstrip()` added on the whole normalized string after per-line rstrip, so `ignore_trailing_whitespace: true` covers end-of-file whitespace consistent with the option name. Mode-neutral change; external review accepted. Harness dry-run confirms all 24 tasks still pass dual scorer.

### F4 — `json_envelope` scorer prefers any JSON-parseable tool output over the agent's matching text answer

- **Status:** CLOSED (2026-04-26 iter 31 review pass; FIXED 2026-04-26 iter 30 via closure plan option (a) "Schema-aware tool-output preference"; filed 2026-04-26 iter 29; **P1** metric-quarantine severity at filing).
- **Axis:** oracle trustworthiness (scorer false-negative on a specific output-shape pattern)
- **Anchor:** the iter-29 quiet-signal-checkpoint expensive run
  (`bench/runs/checkpoint-pi-T16-mdtools-gpt5.4mini-2026-04-26/`) recorded
  `correct=False`, `correct_neutral=False` for T16 mdtools on
  gpt-5.4-mini at minimal thinking. The agent's final assistant text
  was the correct expected JSON
  `{"total_pending":9,"files":[{"file":"backend.md","pending":3},
  {"file":"devops.md","pending":4},{"file":"frontend.md","pending":2}]}`
  (verified by parsing the persisted Pi session output at
  `logs/T16_mdtools_1777224275/agent_output.txt` and concatenating its
  text deltas). The scorer's `actual` was `./md tasks <files> --json`'s
  raw envelope `{"schema_version":"mdtools.v1","results":[{...}]}` —
  valid JSON, non-empty dict, but the wrong schema for T16 (which
  expects `total_pending` + `files`, not `schema_version` + `results`).
- **Root cause (pre-iter-30):** `bench/harness.py:1404-1429` (the
  `expected_artifact == "json_envelope"` branch of the scorer) walked
  `reversed(all_tool_outputs)` first and captured the first tool output
  that parsed as a non-empty JSON dict/list (`bench/harness.py:1408-1415`),
  only falling through to `all_text_outputs` when **no** tool output
  parsed (lines 1416-1426). When an agent ran an intermediate tool that
  emitted valid-but-different-schema JSON (e.g. `./md tasks --json` →
  `{"schema_version":..., "results":[...]}`) **and** then computed the
  expected-schema answer in their final assistant text rather than via a
  projecting tool call (e.g. `jq '...'`), the scorer captured the
  intermediate envelope instead of the agent's correct answer.
- **Effect (P1 quarantine, lifted at iter-30 closure):** affected
  `json_envelope` + `json_canonical=true` tasks where the agent's
  projection lived in text rather than in a tool call whose stdout
  matched the expected schema. T16 was one such cell. The same scorer
  cell shape (`kind=structural` + `json_canonical=true` +
  `expected_artifact=json_envelope`) covers T9 / T11 / T16 / T19 per
  iter-28's dispatcher matrix; T9 passed in iter-25 because the agent
  used `jq` to project in-tool, so the tool-output-priority rule worked
  in T9's favor. Per the claim-gate, F4 did **not** block holdout
  results (none of T9/T11/T16/T19 are in the holdout split — holdout is
  T4/T14/T20/T22/T23/T24 per iter-2 fingerprints) and did **not** block
  product work on the unaffected slice.
- **Typed artifact (closure):** `select_json_envelope_actual` at
  `bench/harness.py:1481-1526` (with helpers `_json_top_keys` at line
  1457 and `_expected_json_top_keys` at line 1469) walks
  `reversed(all_tool_outputs)` preferring outputs whose parsed
  top-level keys overlap with the expected JSON's top-level keys; on
  no shape-match, falls through to `reversed(all_text_outputs)`; final
  fallback chain is the most-recent non-shape-matching tool output,
  then `extract_last_json(stdout)`. The
  `expected_artifact == "json_envelope"` branch at
  `bench/harness.py:1404-1408` is now a single 5-line `elif` calling
  the helper. `bench/test_harness_json.py:JsonEnvelopeActualSelectionTests`
  pins the new precedence with 8 unit tests (T16 text wins, T9 jq
  projection wins, T1 matching tool wins, T22 list fallback, text-only,
  no-shape-match fallback, unknown expected shape preserves pre-F4
  rule, empty outputs fallback). Replay-verification: parsing the
  iter-29 T16 bundle's `agent_output.txt` via
  `bench/pi_runner.parse_pi_json_output` and running the new selector
  against `bench/expected/t16_count.json` returns the agent's text
  answer, and `score_task` reports **`md=PASS neutral=PASS`** with
  `report = "json_canonical: OK"` — the iter-29 `correct=False` would
  now be `correct=True`.
- **Closure plan executed:** option (a) from the original entry —
  schema-aware tool-output preference. Options (b) (text-output
  equality short-circuit) and (c) (per-task scorer hint) were rejected
  in favor of (a) per the iter-29 entry's pre-commitment language:
  "option (a) is the leading candidate because it generalizes (no
  per-task config) and degrades gracefully (current behavior is the
  final fallback)".
- **What F4 did NOT block:** did not block product anchor promotion
  (none declared); did not bump `holdout_version` (no holdout-side
  artifact change); did not invalidate any prior search-set bundle
  (those bundles record what they recorded; the question is what
  future scoring does with comparable shapes); did not retract any
  CLOSED finding.

### L1 — Loop lacked holdout-immutability guard
- **Status:** CLOSED (2026-04-26 review pass; FIXED 2026-04-26 iter 2)
- **Axis:** oracle trustworthiness (meta)
- **Anchor:** an iteration earlier in this run made a change to `bench/tasks/tasks.json` where the edited task ID was in `bench/holdout/task_ids.json`, then reran holdout and published the new pass rates as confirmation. The loop's iteration protocol did not catch this. External review (2026-04-24) surfaced it.
- **Typed artifact:** `bench/holdout/fingerprints.json` (`holdout_version: 1`) baselines a SHA-256 over each holdout task's canonical JSON entry plus the bytes of every input/expected/support file it references. Harness function `verify_holdout_fingerprints` (in `bench/harness.py:747`) recomputes fingerprints from `bench/tasks/tasks.json` and raises `holdout-immutability breach (...)` on any drift. `bench/test_harness_task_split.py::HoldoutImmutabilityTests` pin the live-vs-baseline match, the manifest shape (`holdout_version` + per-id fingerprints), description drift detection, expected-output bytes drift detection, and fingerprint determinism. The cheap channel mechanically blocks the L1 mistake — silent edits to a holdout task description, scorer settings, or expected output bytes fail the test.
- **Holdout-repair exception path:** legitimate repairs must (1) file a P0 ledger entry, (2) bump `holdout_version` and regenerate `bench/holdout/fingerprints.json`, (3) mark prior holdout results non-comparable in `bench/RESULTS.md`. The fingerprints file is the artifact that carries the version, satisfying the spec's "increment a `holdout_version` field in `bench/holdout/task_ids.json` (or equivalent manifest)" — `task_ids.json` is intentionally left as a flat array since it is also used by non-holdout selection paths.
