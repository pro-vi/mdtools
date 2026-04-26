# Bench Ledger

Concise human-memory surface for the frontier loop. Weaker evidence than typed
artifacts under `bench/runs/`. OPEN findings gate claim-expansion; they are
cleared only after a typed artifact confirms the finding, not by prose alone.

## OPEN findings

### P3 — `bytes_output` is not cross-executor comparable
- **Status:** OPEN (filed 2026-04-26 iter 4; P2 hardening backlog severity)
- **Axis:** oracle trustworthiness (cross-executor normalization)
- **Anchor:** the iteration-4 quiet-signal-checkpoint PI smoke (`bench/runs/checkpoint-pi-T1-mdtools-gpt5.4mini-2026-04-26/`) recorded `bytes_output=5,975,843` for T1 mdtools on gpt-5.4-mini. The pre-fix Qwen oai-loop holdout bundle records `bytes_output=679` for T20 mdtools — three orders of magnitude smaller. Root cause: `bench/harness.py:1229` measures pi-json `bytes_output` as `len(raw_stdout.encode())`, which captures the entire `pi --mode json` JSONL stream (per-token deltas, audit envelopes, session-meta events), whereas the oai-loop path captures assistant message content. The two values are not comparable for the same axis.
- **Effect:** does not gate the current acceptance metric (`pass_rate`), so this is P2-severity hardening backlog, not a P0 / P1 claim block. Cross-executor comparisons that include `bytes_output` (or any derived metric) must be flagged as non-comparable until normalized.
- **Closure plan:** either (a) record a separate `bytes_assistant_content` field for pi-json by parsing assistant text from the audit stream, or (b) document in `bench/RESULTS.md` that `bytes_output` is executor-local and not cross-executor comparable. (b) is cheaper and aligns with the spec's existing posture ("Cross-executor results are not apples-to-apples without explicit normalization").

## FIXED_PENDING_CONFIRMATION

_(none — see CLOSED for the 2026-04-26 review pass that promoted F1/F2/F3/F3-a/L1)_

## CLOSED

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
