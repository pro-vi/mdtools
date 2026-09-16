# Evaluate CLI workflows with independent grading and immutable attempts

**Date:** 2026-09-16
**Status:** Proposed
**Deciders:** Provi, Codex

## Context

The current five-command CLI differs from the surface used by the historical
benchmark. The retired harness also used treatment-dependent diagnostics and
separate success rules for execution, resume, and reporting. Reusing its scores
would mix tool changes with changes to grading and execution conditions.

A comparison requires the same task semantics, an independent grader, and one
rule for selecting attempts. A successful runner receipt alone cannot establish
that a task was completed under the intended configuration.

## Decision

Recover the fixed-corpus Python execution path, not its whole provider framework.
Compare three CLI workflows: no-md, the pinned historical CLI, and the pinned
current CLI with compact defaults. Keep ordinary utilities equal across them.
Leave Pi tools and native file-tool comparisons outside this experiment.

`grade_submission` judges only the declared final submission and controller-owned
expected values. It cannot invoke or import the treatment. `AttemptResult`
separates execution, grade, usage, and evidence; `derive_trial_disposition` is the
shared selection rule for resume and reporting. A retry is a new attempt, never
an overwritten result or a choice of the best answer.

Configuration, permission, identity, and evidence faults hold campaign admission
and withhold comparisons. Supported wrong answers remain reportable failures.
Unknown usage remains unknown; tokens, estimated dollars, elapsed time, and CLI
output size are different measurements.

Use one native boundary on each Bash invocation, not PATH-only isolation or a
shell-language security parser. Preserve the runner and treatment binaries by
content identity. Offline scripted-provider checks cannot authorize or establish
real-provider compatibility. Paid runs require separate bounded grants.

## Rationale

Existing corpus, staging, and standard-library statistics can be recovered without
restoring Pi adapters, local-provider loops, automatic task generation, or old
report authorities. The independent grader prevents a treatment failure from
becoming its own grading oracle.

Do not add a forced-full agent condition: the permitted underlying executable
would also let an agent bypass a presentation wrapper. `replay_read_pair` instead
compares identical requests with compact/full presentation. That measures output
size, not downstream agent savings. Each selected model is a separate experiment.

## Consequences

Historical scores remain historical; the new protocol requires new trials. The
first native runner is macOS-only, and its OS/profile dependencies must be pinned
and checked. Raw attempts remain private; reports must expose missing evidence
and configuration holds rather than silently dropping failures.

Real-provider calls require an explicit bounded grant tied to the frozen campaign
and its prerequisite evidence. Saved authorization is audit evidence, not new
consent. Verification so far uses a loopback scripted endpoint; real-provider
authentication and billing behavior, and the sealed stdout_text/stdout_and_file
task contracts, remain unverified.

## Revisit Triggers

- A comparison requires another provider, native file tools, or an enforced
  presentation intervention; define that separate boundary before adding it.
- A runner or OS change invalidates the exercised native profile or receipt
  contracts; require a new identity and dependent checks before paid work.
- A real task exposes a grading contract defect; invalidate the affected evidence
  rather than tuning the frozen protocol and retaining its scores.

## References

- Independent grader: `bench/neutral_scorer.py` — `grade_submission`
- Attempt and selection authority: `bench/trial_records.py` — `AttemptResult`, `derive_trial_disposition`
- Persistence and replay: `bench/harness.py` — `AttemptStore`, `replay_read_pair`
- Native boundary: `bench/command_policy.py` — `ClaudeRunner`, `prepare_native_boundary`; `bench/claude_shell.py` — `OwnedShells`
- Execution contract and remaining holds: `bench/CLI_EVAL.md`
- Historical recovery sources: Git commits `c93352002e3855527980dbdba0941c8099143e45` and `c8e081301fe845d1d8b9bd9ac125d1dbf7c86fa7`
