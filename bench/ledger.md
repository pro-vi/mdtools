# Bench Ledger

Concise human-memory surface for the frontier loop. Weaker evidence than typed
artifacts under `bench/runs/`. OPEN findings gate claim-expansion; they are
cleared only after a typed artifact confirms the finding, not by prose alone.

## OPEN findings

_(none)_

## FIXED_PENDING_CONFIRMATION

_(none — P3 promoted to CLOSED on 2026-04-26 iter 6 review pass; see "Confirmation review pass (2026-04-26 iter 6)" below.)_

## CLOSED

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

### Halt-condition / quiet-signal status (after iter 10)

After the iter-10 expensive-channel run (see "Comparable-harness-axis cell
coverage extension (2026-04-26 iter 10)" below):

- **OPEN findings count:** 0. Iter 10 introduced fresh typed signal — a
  previously-uncovered PI runner cell on the targeted-mutation family —
  without surfacing a new defect. The iter-7 status block noted "third
  consecutive review round"; iters 8, 9, and 10 each preserved the
  zero-OPEN state without re-raising any cleared finding, so by iter-7's
  counting convention the count is now in its sixth consecutive
  zero-OPEN round.
- **Quiet-signal counter:** iters 5–6 quiet, iter 7 expensive, iters 8–9
  quiet (specification-coherence cleanups on RESULTS.md and the retracted
  README), iter 10 expensive. Counter resets to 0 after iter 10. Iters
  11–13 admissible as quiet iterations; iter 14 would otherwise re-trigger
  the spec's "After 3 consecutive iterations …" rule absent earlier signal.
- **Iter-10 same-family-rule discharge:** iters 8 and 9 were both
  specification-coherence cleanups, so iter 10 was constrained by the
  same-family admissibility rule from a third spec-cleanup absent a fresh
  failing trace (none surfaced — full grep across `README.md`, `CLAUDE.md`,
  `specs/**`, `bench/RESULTS.md`, `bench/retracted_2026-04-24/README.md`,
  `bench/ledger.md` confirms zero remaining stale F3 / L1 / pending-fix
  references). Iter 10 shifted intervention to the comparable-harness-axis
  frontier anchor instead of halting, because a real cell gap was
  available at low cost (PI runner had been exercised only on extraction
  tasks T1 and T22; mutation- and multistep-family cells were absent from
  the PI executor's coverage).
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

### L1 — Loop lacked holdout-immutability guard
- **Status:** CLOSED (2026-04-26 review pass; FIXED 2026-04-26 iter 2)
- **Axis:** oracle trustworthiness (meta)
- **Anchor:** an iteration earlier in this run made a change to `bench/tasks/tasks.json` where the edited task ID was in `bench/holdout/task_ids.json`, then reran holdout and published the new pass rates as confirmation. The loop's iteration protocol did not catch this. External review (2026-04-24) surfaced it.
- **Typed artifact:** `bench/holdout/fingerprints.json` (`holdout_version: 1`) baselines a SHA-256 over each holdout task's canonical JSON entry plus the bytes of every input/expected/support file it references. Harness function `verify_holdout_fingerprints` (in `bench/harness.py:747`) recomputes fingerprints from `bench/tasks/tasks.json` and raises `holdout-immutability breach (...)` on any drift. `bench/test_harness_task_split.py::HoldoutImmutabilityTests` pin the live-vs-baseline match, the manifest shape (`holdout_version` + per-id fingerprints), description drift detection, expected-output bytes drift detection, and fingerprint determinism. The cheap channel mechanically blocks the L1 mistake — silent edits to a holdout task description, scorer settings, or expected output bytes fail the test.
- **Holdout-repair exception path:** legitimate repairs must (1) file a P0 ledger entry, (2) bump `holdout_version` and regenerate `bench/holdout/fingerprints.json`, (3) mark prior holdout results non-comparable in `bench/RESULTS.md`. The fingerprints file is the artifact that carries the version, satisfying the spec's "increment a `holdout_version` field in `bench/holdout/task_ids.json` (or equivalent manifest)" — `task_ids.json` is intentionally left as a flat array since it is also used by non-holdout selection paths.
