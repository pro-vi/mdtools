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

Measured-run model substitution locked before data collection (2026-07-04): this
probe requires `pi-json` for the live drift hook, and the current Pi registry does
not expose Claude Haiku/Sonnet models. The measured pair is therefore:

- Strong: `openai-codex/gpt-5.4` with `--thinking minimal`
- Weak/local: `omlx/Qwen3.6-35B-A3B-8bit` with no `--thinking` flag

Interpret the readout as a strong/weak native-capable Pi-runner pair, not as a
Claude-vs-Claude replication.

## Drift Injection

The committed injector is `probes/multifile/drift_injector.py`.
The shared drift target spec is `probes/multifile/drift_specs.json`.

Required measured-run hook:

1. The agent must have already observed the drift target file.
2. The injector fires exactly once.
3. The injector must fire before the agent's first mutation of that target file.

Sleep-only timing is not allowed for measured runs because it can fire before the model's
first read and would not test stale-write handling. A paid run is valid only if the run
log proves the observed-read -> drift -> target-mutation order.

Locked hook implementation for measured runs:

- Runner: `pi-json` only, because Pi extension hooks expose live native `read`/`edit`/`write`
  and `bash` tool lifecycle events. `claude-cli` remains invalid for this probe because its
  transcript is only available after the subprocess exits.
- Extension: `probes/multifile/pi_drift_audit_extension/index.ts`, selected with
  `BENCH_PI_AUDIT_EXTENSION` for measured runs.
- Trigger: after the first successful target-file query result (`read` target, or a bash
  query such as `cat`/`md block` mentioning the target), the extension applies the
  deterministic one-shot drift immediately and writes `multifile_drift_observed_read` and
  `multifile_drift_fired` events to `PI_AUDIT_LOG`.
- Proof gate: `bench/harness.py` requires `multifile_drift_config`,
  `multifile_drift_observed_read`, `multifile_drift_fired`, and a later
  `multifile_drift_target_mutation` with `afterObservedRead=true` and `afterDrift=true`.
  Rows without this proof are scored failure even if the final files happen to match an
  expected alternative.

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

The committed readout script is `probes/multifile/summarize_results.py`.

## Current Execution Status

Protocol, fixtures, and the live Pi drift hook are locked. Paid measured runs must use the
hook and pass the proof gate above; rows lacking proof are invalid failures, not evidence
for either arm.
