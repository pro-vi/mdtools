---
title: clean-ablation validity gate — soft md-stub baseline + correctness-parity gate
type: fix
status: active
date: 2026-05-29
origin: follow-up /second-opinion (GPT-Pro) on the md-attribution gate — "partial nod, still broken"
---

# Clean-ablation validity gate — make `hybrid-no-md` a CLEAN counterfactual

## Why

The follow-up consult ("partial nod, still broken") found: the `hybrid-no-md`
baseline is a POISONED counterfactual — it advertises md but HARD-DENIES it
(guard kills the shell), so hybrid can beat hybrid-no-md because the baseline got
confused/derailed, NOT because md is genuinely better than unix. The loop can
satisfy `CLOSES` by making the baseline WORSE. Worst concrete bug: `baseline_ok`
checks COST only, not CORRECTNESS — `unix 90%/100, hybrid 90%/100,
hybrid-no-md 80%/105` closes today (fake correctness "lift" over a sabotaged
baseline).

## Architecture Decision

**Approach:** replace the hard-deny baseline with a **soft canonical md-stub**,
and redesign `baseline_ok` into a **clean-ablation validity gate** (correctness
parity + ≤1-probe + cost parity).

`hybrid-no-md` mode: md is ON the PATH but is a **stub** that (1) appends to a
probe-counter log, (2) prints a canonical line to stderr — "md is unavailable in
this ablation mode; use standard unix tools", (3) `exit 1`. The agent reads one
line and falls back like a competent unix user. No hard-kill.

**Rationale:** the hard-kill was the poison. A soft stub makes hybrid-no-md a
competent unix fallback → cost-flail mostly vanishes AND correctness parity is
achievable, which is the load-bearing validity check. The stub writes its OWN
probe counter → runner-agnostic (works for claude-cli, whose Bash bypasses the
bench guard so tool_mix/guard-log can't count md attempts).

**Trade-off:** no precise token-subtraction of the one probe — rely on the soft
stub being cheap + the parity tolerances. Parity thresholds (10pp correctness,
1.2× cost) are judgment calls → expose as params + surface near-threshold cells.

**Composition (model-shape):** `baseline_ok` goes scalar (cost-ratio) →
3-member validity bundle (`correctness_parity ∧ probe≤1 ∧ cost_parity`); verdict
renders which member failed: `SUSPECT:baseline-flails(correctness|probes|cost)`.
Preserve: ignore md → hybrid≈hybrid-no-md → `OPEN:no-lift`; over-push md → probes
>1 / baseline flails → `SUSPECT`; md genuinely helps → clean baseline, hybrid beats it.

## Implementation Units

### U1 — soft canonical md-stub + probe count
- **Goal:** hybrid-no-md runs a stub md (canonical message, exit 1, probe count); no hard-kill.
- **Dependencies:** None.
- **Files:**
  - Modify `bench/command_policy.py`: `allowed_commands_for_mode("hybrid-no-md")` → `UNIX_TOOLS + ["md"]` (guard allows md so it isn't hard-killed); `create_restricted_shell_env` (~67-105) writes a stub script for md when `mode == "hybrid-no-md"` instead of symlinking `md_binary_path`; add `env["BENCH_MD_PROBE_LOG"] = str(workdir/".md-probe.log")`.
  - Modify `bench/harness.py`: after the run, read the probe log → `BenchResult.md_probe_count: int = 0` (new field); populate in `run_agent`. (build_prompt hybrid-no-md→HYBRID_DOCS already done.)
- **Approach:** the stub (shell):
  `printf '%s\n' "$BASH_COMMAND" >> "$BENCH_MD_PROBE_LOG"; echo "md is unavailable in this ablation mode; use standard unix tools (grep/sed/awk/...)." >&2; exit 1`
  (or a small written script). md_probe_count = number of lines in the probe log.
- **Patterns to follow:** the symlink loop in `create_restricted_shell_env`; the guard log read in `run_agent` (~1433).
- **Test scenarios:**
  - *Happy:* a hybrid-no-md cell where the agent runs `md outline` → exit 1, canonical stderr, probe log has 1 line → `md_probe_count == 1`; run completes (no kill).
  - *Edge:* agent never calls md → probe log absent/empty → `md_probe_count == 0`.
  - *Edge:* agent calls md 3× → `md_probe_count == 3`.
  - *Integration:* unix/mdtools/hybrid modes unchanged (md stub only for hybrid-no-md).
- **Verification:** hybrid-no-md no longer hard-kills on md; `md_probe_count` recorded per run, runner-agnostic.

### U2 — clean-ablation validity in `attribution_verdict`
- **Goal:** baseline must be a CLEAN counterfactual; a sabotaged baseline → SUSPECT, never CLOSES.
- **Dependencies:** U1 (records carry md_probe_count; fixture-testable now).
- **Files:** Modify `bench/agg_util.py` (`attribution_verdict` baseline validity; add `_probe_count(records, mode)`); Test `bench/test_agg_util.py`.
- **Approach:** `baseline_ok = correctness_parity AND probe_ok AND cost_parity`, where
  `correctness_parity = nomd_pass ≥ unix_pass − parity_tol` (default 0.10);
  `probe_ok = median(md_probe_count over hybrid-no-md runs) ≤ 1`;
  `cost_parity = (bu n==0) or (no-md ≤ unix × (1+baseline_tol))` (kept as secondary).
  On `not baseline_ok` → `SUSPECT:baseline-flails(<failing member>)`.
  Cost-lift closure additionally requires `lf.n ≥ min_overlap` (default 2) AND `probe_ok`; else cost-lift invalid → fall back to correctness-lift / `OPEN:insufficient-evidence`.
- **Patterns to follow:** the existing `attribution_verdict` (this is its v2); `_pass_rate`, `intersection_cost`.
- **Test scenarios (RED for the two exploits the consult named):**
  - *(a) Correctness-poisoning:* unix 90%/100, hybrid 90%/100, hybrid-no-md 80%/105 → `SUSPECT:baseline-flails(correctness)`, NOT CLOSES.
  - *(b) >1 probe:* clean cost/pass but median md_probe_count = 3 in hybrid-no-md → `SUSPECT:baseline-flails(probes)`.
  - *(c) Clean baseline + real lift:* hybrid-no-md ≈ unix on pass+cost, probe ≤ 1, hybrid cheaper than hybrid-no-md → `CLOSES`.
  - *Regression:* neutering (hybrid≈hybrid-no-md) still → `OPEN:no-lift`; tie-acceptable still CLOSES on Pareto; loses-unix still `OPEN:loses-unix`.
  - *Edge:* tiny overlap (lf.n < min_overlap) + no correctness lift → `OPEN:insufficient-evidence`.
- **Verification:** the correctness-poisoning + multi-probe exploits cannot CLOSE; clean lift still can.

### U3 — report surfaces probe count + failing validity member
- **Goal:** the verdict row shows md_probe_count + which `SUSPECT` sub-reason.
- **Dependencies:** U2.
- **Files:** Modify `bench/report.py` (`render_cost_slice` — add `probes=` and the typed SUSPECT reason; flag near-threshold parity).
- **Test scenarios:** *Happy:* a 4-mode fixture renders probe count + SUSPECT(correctness) reason.
- **Verification:** a human/loop sees WHY a cell is SUSPECT (correctness vs probes vs cost).

### U4 — loop docs (clean-ablation gate)
- **Goal:** the loop's gate = clean-ablation (correctness parity + ≤1 probe), not just cost.
- **Dependencies:** U2, U3.
- **Files:** Modify `loop/ACCEPTANCE.md` + `loop/PROMPT.md` (baseline validity = correctness parity + ≤1 canonical probe; "make the baseline worse" no longer closes a cell; the canonical-stub note; oracle-drift guard gains "don't sabotage the ablation baseline").
- **Test scenarios:** `Test expectation: none — docs`.
- **Verification:** a loop session cannot close a structural cell by degrading hybrid-no-md.

## Scope Boundaries
- bench/ python + loop/ docs ONLY. **No Rust md (src/)**, no task/scorer/threshold changes.
- This is v2 of the attribution gate (BENCH_V2_ATTRIBUTION.md) — extends, doesn't replace, U1-U4 there.

### Deferred to Follow-Up Work
- Precise per-probe token-cost subtraction (rely on soft-stub cheapness for now).
- Tail-aware per-task paired cost (still deferred from the prior round).

## System-Wide Impact
- **Interaction graph:** new stub flows through `create_restricted_shell_env` → workdir → `run_agent` → `BenchResult.md_probe_count` → `attribution_verdict`. Additive field; no existing field changes meaning.
- **Unchanged invariants:** unix/mdtools/hybrid modes, pass_rate, dual-scoring, the bench-v2 cost slice, and the existing attribution verdicts (neutering/loses-unix/tie) all still hold.

## Risks & Dependencies
| Risk | Mitigation |
|---|---|
| Soft stub still leaves residual flail cost | correctness-parity is now primary; cost_parity secondary; report surfaces near-threshold |
| Parity thresholds (10pp / 1.2×) arbitrary | params + surfaced in report so loop sees near-threshold cells |
| Stub probe-log missing for some runner | stub writes unconditionally on each invocation; md_probe_count defaults 0 |
| min_overlap too strict on small categories | default 2; falls back to correctness-lift / insufficient-evidence, never a silent close |
