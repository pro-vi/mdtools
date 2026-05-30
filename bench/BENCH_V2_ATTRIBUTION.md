---
title: bench-v2 attribution gate — hybrid-no-md baseline + md-causal-value gate
type: feat
status: active
date: 2026-05-29
origin: /second-opinion (GPT-Pro) on the hybrid-pareto loop design; /architect
---

# Attribution gate — make the loop test "does `md` add value", not "can hybrid avoid losing"

## Why

A `/second-opinion` found the loop goal ("hybrid Pareto-dominates unix") is
gameable: the loop can edit the hybrid prompt to steer the agent AWAY from `md`
(adoption is observed but NOT gated) → hybrid ≈ unix → every cell "wins" while
`md` is never improved. **Prompt-neutering is the dominant strategy** because the
objective doesn't require `md` to be *causally* responsible for the win.

Fix: a counterfactual baseline that isolates `md`'s causal value.

## Architecture Decision

**Approach:** add a 4th bench mode `hybrid-no-md` (same hybrid prompt, md denied)
as the counterfactual baseline, and gate **structural** cells on a 3-baseline
attribution lattice (`unix`, `hybrid`, `hybrid-no-md`) computed in `agg_util`.

`hybrid-no-md` = HYBRID_DOCS prompt (md advertised) + md OFF the bench-bin PATH
(denied by the existing guard). Only md *availability* differs from `hybrid`.

**The gate (per STRUCTURAL tier×category cell — all three required):**
1. **Pareto (product claim):** hybrid pass-rate ≥ unix AND hybrid cost ≤ unix (+5%).
2. **Attribution (md is causal):** hybrid strictly beats `hybrid-no-md` beyond a
   pre-registered margin — correctness (passes tasks no-md fails) OR cost
   (cheaper on the hybrid∩no-md intersection). *Neuter the prompt → hybrid ≈
   hybrid-no-md → lift≈0 → cell stays OPEN.*
3. **Anti-counter-game (diagnostic guard):** hybrid-no-md cost ≈ unix cost
   (generous tol). Blocks the inverse game ("push md so hard the no-md baseline
   flails below unix to fake lift"). If no-md ≫ unix → verdict `SUSPECT`, not closed.

NON-structural categories (Text manipulation, T4/T6) keep the original rule:
hybrid ≥ unix (tie = win), **no lift required**.

**Structural classification (gate on lift):** Extraction, Targeted mutation,
Batch mutation, Multi-step, Content delivery, Safe-fail, Metadata, Table
projection. **Tie-acceptable (no lift):** Text manipulation.

**Rationale:** reuses the mode→tool-policy + guard machinery (U1 ~6 lines) and
the bench-v2 `agg_util` intersection logic (U2); the gate is a pure function →
unit-testable, including the load-bearing neutering-detection test.
**Trade-off:** denied-md attempts add bounded cost (measured via existing
`policy_violations`); guard #3 bounds the confound rather than eliminating it.

**Composition (model-shape change):** one comparison (hybrid vs unix) → a
3-baseline lattice with a typed per-cell **verdict**: `CLOSES` ·
`OPEN:no-lift` · `OPEN:loses-unix` · `OPEN:insufficient-evidence` ·
`SUSPECT:baseline-flails`. Every surface renders the *verdict*, not just a cost.

## Implementation Units

### U1 — `hybrid-no-md` ablation mode
- **Goal:** 4th mode — hybrid prompt, md unavailable.
- **Dependencies:** None.
- **Files:**
  - Modify: `bench/command_policy.py` (`BenchMode` Literal:11; `allowed_commands_for_mode` ~59-64 → `UNIX_TOOLS` for `hybrid-no-md`)
  - Modify: `bench/harness.py` (`BenchMode` Literal ~184; `build_prompt` ~373-379 → `hybrid-no-md` uses `HYBRID_DOCS`; `--mode` choices ~1774)
- **Approach:** `hybrid-no-md` groups with `hybrid` for the prompt (HYBRID_DOCS) but with `unix` for the toolset (md off PATH → guard denies it).
- **Patterns to follow:** existing `hybrid` branches in both files.
- **Test scenarios:**
  - *Happy:* `allowed_commands_for_mode("hybrid-no-md") == UNIX_TOOLS` (no `md`); `build_prompt(...,"hybrid-no-md")` == HYBRID_DOCS content.
  - *Integration:* a `hybrid-no-md` cell where the agent tries `md` → guard denies (`policy_violations ≥ 1`), falls back to unix; run completes.
- **Verification:** a `hybrid-no-md` record exists with md-denials counted and the hybrid prompt in the log.

### U2 — md-lift + attribution verdict in `agg_util`
- **Goal:** pure-function gate `attribution_verdict(records, tier, category) → {pareto, lift, baseline_ok, verdict}`.
- **Dependencies:** U1 (fixture-testable now).
- **Files:**
  - Modify: `bench/agg_util.py` (add `STRUCTURAL_CATEGORIES`; `md_lift(records, tier, cat)` = hybrid vs hybrid-no-md correctness+cost; `attribution_verdict(...)` composing Pareto + lift + baseline guard)
  - Test: `bench/test_agg_util.py` (extend)
- **Approach:** reuse `intersection_cost` for each pairing (hybrid|unix, hybrid|hybrid-no-md, hybrid-no-md|unix). `lift_positive = hybrid_pass > no_md_pass OR hybrid_cost < no_md_cost − margin`. `baseline_ok = no_md_cost ≤ unix_cost × (1+guard_tol)`.
- **Patterns to follow:** `intersection_cost` (agg_util) — same keying/basis.
- **Test scenarios (the integrity gates — most coverage here):**
  - *Neutering detected (LOAD-BEARING):* hybrid records ≈ hybrid-no-md (md unused) → lift≈0 → structural verdict `OPEN:no-lift`.
  - *Genuine md value:* hybrid cheaper/more-correct than hybrid-no-md AND ≥ unix → `CLOSES`.
  - *Loses unix:* hybrid cost > unix → `OPEN:loses-unix` regardless of lift.
  - *Counter-game:* hybrid-no-md ≫ unix → `SUSPECT:baseline-flails`.
  - *Tie-acceptable:* category ∈ Text-manipulation, hybrid ≈ unix, lift 0 → `CLOSES`.
  - *Empty hybrid∩no-md intersection:* lift n=0 → `OPEN:insufficient-evidence` (never silent pass).
- **Verification:** verdict correct across the matrix; structural neuter → never `CLOSES`.

### U3 — report surfaces the attribution verdict
- **Goal:** the cost slice shows, per structural cell, unix · hybrid · hybrid-no-md · lift · **verdict**.
- **Dependencies:** U2.
- **Files:** Modify `bench/report.py` (`render_cost_slice` — add hybrid-no-md column + `verdict` column from `attribution_verdict`; keep existing unix-vs-* rows; print `n` beside medians so tiny-n cells are visible).
- **Approach:** per (tier,cat) print the three medians + lift + typed verdict; `OPEN:*` structural cells are the loop's work list.
- **Test scenarios:** *Happy:* a 4-mode fixture renders the verdict column; *Integration:* report + analyze agree via shared `agg_util`.
- **Verification:** the slice names which structural cells aren't attribution-closed, and why.

### U4 — re-gate the loop spec on attribution
- **Goal:** `loop/ACCEPTANCE.md` + `loop/PROMPT.md` close on md-lift, not bare ≥-unix.
- **Dependencies:** U2, U3.
- **Files:**
  - Modify: `loop/ACCEPTANCE.md` (north star → "md adds attributable value beyond a unix-capable agent with the same prompt"; structural `AC-{tier}-{cat}` pass_evidence = verdict `CLOSES`; add `hybrid-no-md` to every cell's verifier run; list tie-acceptable categories)
  - Modify: `loop/PROMPT.md` (per-iteration runs all **4** modes; oracle-drift guard gains "don't game the attribution baseline"; seed cell now also needs positive lift; document neuter→stays-OPEN)
- **Approach:** docs only; the gate logic lives in U2/U3 (the oracle).
- **Test scenarios:** `Test expectation: none — docs`. Sanity: grep that no structural criterion's pass_evidence omits the lift requirement.
- **Verification:** a loop session reading these cannot close a structural cell by neutering the prompt.

## Scope Boundaries
- **No Rust `md` changes** — this is the measurement/gate; the loop improves `md` later against it.
- **No task/scorer/threshold changes** — the oracle is fixed.
- Tier classification + bench-v2 cost basis unchanged.

### Deferred to Follow-Up Work
- **Tail-aware per-task paired cost** (second-opinion H5): failures shown as *dominated* not dropped; surface `max` beside median so a +70k tail isn't hidden. Separate methodology PR. *Cheap interim done in U3:* print `n` per median.

## System-Wide Impact
- **Interaction graph:** new mode flows through the same `run_agent → BenchResult → report`; only `agg_util` gains the lattice. No existing field changes meaning.
- **API parity:** `report.py` + `analyze.py` both consume `agg_util` — verdict logic lives once.
- **Unchanged invariants:** unix/mdtools/hybrid modes, `pass_rate`, dual-scoring, the bench-v2 cost slice all still work; this adds a 4th mode + a gate.

## Risks & Dependencies
| Risk | Mitigation |
|---|---|
| Denied-md attempts inflate hybrid-no-md cost → fake lift | guard #3 (no-md ≈ unix) + `policy_violations` measured; `SUSPECT` verdict if baseline flails |
| Prompt-length overhead makes no-md > unix without flailing | generous guard tol absorbs prompt-token delta; lift margin above it |
| 4 modes double runtime/cost | hybrid-no-md only run for structural cells under diagnosis, per-cell not blanket |
| Tier conflict makes a cell unwinnable | per-(tier×cat) cells independent; unwinnable → STUCK/documented, not loop-wide block |
