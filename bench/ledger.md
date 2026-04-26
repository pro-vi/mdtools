# Bench Ledger

Concise human-memory surface for the frontier loop. Weaker evidence than typed
artifacts under `bench/runs/`. OPEN findings gate claim-expansion; they are
cleared only after a typed artifact confirms the finding, not by prose alone.

## OPEN findings

_(none)_

## FIXED_PENDING_CONFIRMATION

_(none — P3 promoted to CLOSED on 2026-04-26 iter 6 review pass; see "Confirmation review pass (2026-04-26 iter 6)" below.)_

## CLOSED

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

### Halt-condition / quiet-signal status (after iter 18)

After the iter-18 quiet-signal-checkpoint discharge (see
"Quiet-signal checkpoint discharge (2026-04-26 iter 18)" above):

- **OPEN findings count:** 0. Iter 18's expensive-channel run
  surfaced no new defect — the bundle is dual-scorer PASS, the
  pi-audit.jsonl parses cleanly, and the 1 policy_violation is a
  documented behavioral pattern (stdin-piping deny + `--from`
  recovery, well-known from project memory) rather than a defect.
  The zero-OPEN state holds through iters 8, 9, 10, 11, 12, 13, 14,
  15, 16, 17, and 18 — the fourteenth consecutive zero-OPEN review
  round.
- **Quiet-signal counter:** iters 5–6 quiet, iter 7 expensive, iters
  8–9 quiet, iter 10 expensive, iters 11–13 quiet, iter 14 expensive
  (multistep-family coverage extension), iter 15 quiet (ledger-only
  ratification), iter 16 quiet (cheap-channel-only oracle hardening,
  no expensive run), iter 17 quiet (cheap-channel-only oracle
  telemetry stamping, no expensive run), iter 18 expensive
  (content-delivery-family coverage extension + first stamped
  bundle). Counter reset to **0** after iter 18. Iters 19–21
  admissible quiet; iter 22 is the next forced expensive-or-halt
  point per the spec's "3 consecutive iterations with the cheap
  channel green, no new failing trace, and no new finding added"
  rule.
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

### L1 — Loop lacked holdout-immutability guard
- **Status:** CLOSED (2026-04-26 review pass; FIXED 2026-04-26 iter 2)
- **Axis:** oracle trustworthiness (meta)
- **Anchor:** an iteration earlier in this run made a change to `bench/tasks/tasks.json` where the edited task ID was in `bench/holdout/task_ids.json`, then reran holdout and published the new pass rates as confirmation. The loop's iteration protocol did not catch this. External review (2026-04-24) surfaced it.
- **Typed artifact:** `bench/holdout/fingerprints.json` (`holdout_version: 1`) baselines a SHA-256 over each holdout task's canonical JSON entry plus the bytes of every input/expected/support file it references. Harness function `verify_holdout_fingerprints` (in `bench/harness.py:747`) recomputes fingerprints from `bench/tasks/tasks.json` and raises `holdout-immutability breach (...)` on any drift. `bench/test_harness_task_split.py::HoldoutImmutabilityTests` pin the live-vs-baseline match, the manifest shape (`holdout_version` + per-id fingerprints), description drift detection, expected-output bytes drift detection, and fingerprint determinism. The cheap channel mechanically blocks the L1 mistake — silent edits to a holdout task description, scorer settings, or expected output bytes fail the test.
- **Holdout-repair exception path:** legitimate repairs must (1) file a P0 ledger entry, (2) bump `holdout_version` and regenerate `bench/holdout/fingerprints.json`, (3) mark prior holdout results non-comparable in `bench/RESULTS.md`. The fingerprints file is the artifact that carries the version, satisfying the spec's "increment a `holdout_version` field in `bench/holdout/task_ids.json` (or equivalent manifest)" — `task_ids.json` is intentionally left as a flat array since it is also used by non-holdout selection paths.
