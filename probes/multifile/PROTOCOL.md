# Transactional Multifile Regime Protocol

Date locked: 2026-07-03

## Scope

This probe tests whether `md`'s block `--expect-etag` fail-closed behavior creates a
behavioral edge when one file drifts during a coordinated multifile edit.

Task corpus:

- `MF-ETAG-01` under `bench/regimes/multifile/inputs/mf01/`
- `MF-ETAG-02` under `bench/regimes/multifile/inputs/mf02/`

These tasks are exploratory regime probes. They do not enter `bench/tasks/tasks.json`
and cannot affect v3 headline evidence.

## Arms

- `native`
- `native+md`

No `native+md-no-md` control is introduced in this probe. If a future run adds a new
mode, it must pass through the existing `MD_REAL_MODES` and no-md preflight policy.

## Models and N

- Strong model: Sonnet-class native-capable runner.
- Weak model: Haiku-class native-capable runner.
- N >= 3 per task x mode x model.

Hard cap: $40 equivalent subscription spend. Abort and report rather than exceeding it.

## Drift Injection

The committed injector is `probes/multifile/drift_injector.py`.

Required measured-run hook:

1. The agent must have already observed the drift target file.
2. The injector fires exactly once.
3. The injector must fire before the agent's first mutation of that target file.

Sleep-only timing is not allowed for measured runs because it can fire before the model's
first read and would not test stale-write handling. A paid run is valid only if the run
log proves the observed-read -> drift -> target-mutation order.

Drift targets:

- `MF-ETAG-01`: `mf01/runbook.md`, `Pager owner: Alex` -> `Pager owner: Riley`
- `MF-ETAG-02`: `mf02/backend.md`, `Owner: Blake` -> `Owner: Casey`

The fixtures avoid duplicate-content blocks so the known content-addressed etag
ambiguity is not under test.

## Success Rule

Both outcomes count as success:

- **Safe-fail:** the drifted file preserves the concurrent change and the agent does not
  silently clobber it.
- **Re-read/retry:** the drifted file preserves the concurrent change and also contains
  the requested task update based on a fresh read.

Silent clobber of the concurrent line is always failure.

The harness artifact `multi_file_contents_any` scores the full set of input files against
the committed safe-fail or retry expected-output alternatives.

## Kill Condition

If native agents handle injected drift as reliably as md agents at pass^3 parity, the
etag story has no demonstrated agent-behavior edge and should be repositioned as an API
or scripting feature. If `native+md` reliably avoids clobber where `native` clobbers, this
becomes a candidate Plan C family.

## Readout

Report, per task/model/mode:

- pass/fail under the success rule
- which expected alternative matched (`safe-fail` or `retry`)
- whether md was invoked before the target mutation
- observed-read -> drift -> target-mutation proof
- clobber count

## Current Execution Status

Protocol and fixtures are locked. Paid measured runs are blocked until the harness has a
boundary hook that can prove and enforce observed-read -> drift -> target-mutation order.
