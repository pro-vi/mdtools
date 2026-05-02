# T11 Halt Summary

## Final state

- Phase: `steady-state`
- Fixed-anchor baseline: **18 tasks** (stamp from T10-10)
- Fixed-anchor gap: **+38.9pp** (hybrid pass minus unix pass on the fixed 18)
- Current-corpus gap: **+45.0pp** on **20 tasks**
- Corpus composition: `20/26` total, with `6` holdout IDs excluded by policy
- Baseline measurement: unchanged since launch
- Latest run bundle for fixed-anchor recheck: `bench/runs/t11-retest-T6-fixed-anchor-Qwen3.5-27B-4bit-2026-05-02/`

## Halt trigger outcome

- **Fired halt condition:** `#1` gap saturation
- **Trigger signal:** 3 consecutive steady-state iterations (and now a 4th) produced **no fixed-anchor movement and no corpus growth**
- **Consecutive no-movement streak:** `T11-4`, `T11-5`, `T11-6`, `T11-7` (≥ 3, counter reset not previously observed)
- **Reason this is still admissible:** condition was not gated on promotion-attempts; it is now counted across all iterations
- **Cross-model check:** not triggered (fixed-anchor gap did not move by +5pp)
- **Lock-blocked accumulation condition:** not yet triggered (`2` cumulative lock-blocked rejections in candidates, threshold `3`)

## Families in current search set

- **Accepted candidates in `bench/search/` proper:**
  - `server-setup-subsection-relocation` (from T10-16)
  - `error-logging-format-relocation` (from T10-29)
- **Rejected-candidate counts by status (`bench/search/candidates/*/manifest.json`):**
  - `promoted-to-search`: 2
  - `rejected-both-fail-no-gap`: 6
  - `rejected-both-pass-no-gap`: 2
  - `rejected-cross-seed-instability`: 2
  - `rejected-hybrid-fail-no-gap`: 4
  - `rejected-lock-blocked`: 2
  - `rejected-planning`: 1
  - `rejected-planning-mdtools-fail`: 1
  - `rejected-shell-quoting`: 1
  - `rejected-shell-quoting-gap`: 1
  - `rejected-unix-win-no-mdtools-advantage`: 1
- **Current notable `lock-blocked` families:**
  - `certificate-rotation-runbook-relocation`
  - `pager-rotation-review-relocation`
- **Candidate whose status changed after `md move-section` retest:**
  - `readme-infrastructure-setup-relocation` moved from lock-blocked-only failure characterization to `rejected-cross-seed-instability` (mdtools now passes on the seed run, hybrid remains unstable; unix path still failing)

## Telemetry / findings delta this tier

- No new production-code findings were added in T11-8.
- Product surface change tested this tier (`md move-section` admission, already shipped at launch) did not produce fixed-anchor movement.
- The only substantive activity this iteration is evidence consolidation and halt determination.

## Halt disposition and disposition of key conditions

- **Condition #1 (tightened saturation):** Fired; stop-and-summarize action recommended.
- **Condition #9 (equilibrium-as-valid):** Not fired (`current-corpus` has not grown since T11 launch baseline).
- **Condition #6 (3 lock-blocked accumulations):** Not fired.
- **Condition #2 (cross-model divergence):** Not evaluated; fixed-anchor gap did not move enough to demand it.
- **Condition #7 (spec incoherence) / #8 (buildup stall):** Not applicable in this phase.

## Recommendation for next loop

- Ship this result and exit T11 as saturation-bound.
- Route next work as scope expansion outside this loop:
  1. Cross-model expansion using the frozen fixed-anchor set if resource permits.
  2. Lock-lift work (`/code-architect`) only if a new forbidden surface is strongly evidenced by fresh real traces.
  3. New candidate generator / model path experimentation outside the current fixed-anchor lock-in.
