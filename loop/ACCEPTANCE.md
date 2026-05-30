# ACCEPTANCE — hybrid-attribution-v2

Goal version: `hybrid-attribution-v3` (2026-05-29; bench-v2 @ `feat/bench-v2-cost-axis`; CLEAN-ablation gate per `bench/BENCH_V2_CLEAN_ABLATION.md`)

## North star (frozen)

Hybrid mode (`md` + unix together) must **earn its place**: never worse than
pure-unix on correctness or cost, AND — on structural tasks — md must add
**attributable value** beyond a unix-capable agent given the *same* prompt.

Why a bare "hybrid ≥ unix" goal is insufficient (the /second-opinion finding):
hybrid *contains* unix, so the loop could "win" by editing the hybrid prompt to
steer the agent AWAY from md (adoption is observed but not gated) → hybrid ≈
unix → every cell ties → md never improves. **Prompt-neutering is the dominant
strategy.** The objective must require md to be *causally* responsible. So:
structural cells gate on md-lift over a **`hybrid-no-md`** baseline (same prompt,
md disabled). Honest claim: *does access to `md` add value beyond a unix-capable
agent with the same prompt?*

Closed by improving `md` and its hybrid prompting — **never** by weakening the
bench, and **never** by making md irrelevant.

## Criterion schema

One criterion per **(tier × task-category)** cell. Tiers: `frontier`
(claude-cli, token cost) · `local` (pi-json Qwen, tool_calls cost). Categories:
the 9 `TASK_FAMILIES`.

**The verifier is `agg_util.attribution_verdict(records, tier, category)`** over
**4 modes** (`unix`, `hybrid`, `hybrid-no-md`, run with the tier's runner) at
**N≥3**. A cell PASSES iff its verdict == `CLOSES`. The gate differs by category:

- **STRUCTURAL** (Extraction · Targeted mutation · Batch mutation · Multi-step ·
  Content delivery · Safe-fail · Metadata · Table projection) — closes only when
  md is causally responsible, all three:
  1. **Pareto:** hybrid pass-rate ≥ unix AND hybrid cost ≤ unix +5%.
  2. **md-lift (vs the BETTER baseline):** hybrid strictly beats **`max(unix,
     hybrid-no-md)` on correctness OR `min(unix, hybrid-no-md)` on cost** (margin
     5%) — NOT merely the weaker `hybrid-no-md`. This kills *tolerance arbitrage*:
     a `hybrid-no-md` degraded only WITHIN baseline tolerance (≤10pp pass, ≤20%
     cost) can no longer be the *source* of the lift while hybrid just ties unix.
     md-lift now means md produced a real, attributable win over a unix-only
     agent — not "the ablation looked a bit worse."
  3. **clean baseline:** `hybrid-no-md` must be a COMPETENT unix fallback —
     **correctness parity** (no-md pass ≥ unix − 10pp) AND **≤1 canonical md
     probe in EVERY run** (max-across-runs, not median — one stuck run is dirty)
     AND cost parity (≤ unix +20%); else
     `SUSPECT:baseline-flails(correctness|probes|cost)`. The baseline advertises
     md but runs it as a SOFT STUB (exit 1, message indistinguishable from a
     plain "command not found" so a loop-edited prompt can't branch on it) — so
     making the baseline WORSE (failing tasks, or repeatedly banging into md) no
     longer closes a cell. Only making **hybrid genuinely better** does.
  **Neuter the prompt → hybrid ≈ hybrid-no-md → lift≈0 → `OPEN:no-lift` → the
  cell does NOT close.**
- **TIE-ACCEPTABLE** (Text manipulation, T4/T6) — closes on Pareto alone
  (hybrid ≥ unix; a tie is the win — "don't replace sed"). No lift required.

`pass_evidence`: `report.py` md-attribution verdict == `CLOSES` at N≥3.
`fail_evidence`: any other verdict — `OPEN:loses-unix` / `OPEN:no-lift` /
`OPEN:insufficient-evidence` / `SUSPECT:baseline-flails`.

Statuses: `OPEN` · `PASS_PENDING_FINAL` · `PASS` (only via final-verify) ·
`STUCK` (3 failed hypotheses; e.g. a cell where unix is structurally better and
no admissible md/prompt change yields lift — document as a tie-target) ·
`BLOCKED_EXTERNAL` (omlx down / no claude budget) · `QUARANTINED`.

## Criteria

> The full inventory is **instantiated from the first 4-mode full-suite run**
> (one `AC-{tier}-{cat}` row per cell the suite produces, with measured verdict).
> Do NOT invent cells the task suite doesn't produce.

```yaml
- id: AC-MASTER
  statement: >
    Across the full task suite × 4 modes (unix, hybrid, hybrid-no-md, +tier
    runner) × both tiers at N≥3, EVERY (tier×category) cell verdict == CLOSES.
  source: user "hybrid to beat unix on all fronts" + /second-opinion attribution gate
  authority: agg_util.attribution_verdict (the oracle) via report.py
  verifier: FINAL-VERIFY (see loop/PROMPT.md Channels)
  pass_evidence: report.py md-attribution section shows CLOSES for every cell, one repo state, N≥3
  fail_evidence: any cell not CLOSES
  status: OPEN
  depends_on: [all AC-{tier}-{cat}]
  reopen_condition: any md/prompt edit after this passed
  last_verification: none

- id: AC-frontier-Targeted-mutation
  statement: >
    frontier (claude) cell CLOSES on Targeted-mutation (T7,T10,T13,T20): hybrid
    Pareto-dominates unix AND shows positive md-lift over hybrid-no-md.
  source: observed n=1 — claude T7 mdtools +70427 tok, hybrid +6500 tok vs unix (md HURT the strong model)
  authority: attribution_verdict
  verifier: run T7,T10,T13,T20 × {unix,hybrid,hybrid-no-md} × claude-cli at N≥3; report.py verdict
  pass_evidence: verdict CLOSES (note: this cell currently LOSES the cost front; closing likely needs leaner md output)
  fail_evidence: OPEN:loses-unix (current) / OPEN:no-lift / SUSPECT
  status: OPEN   # KNOWN-FAILING seed — diagnose the +70k token tax first
  depends_on: []
  reopen_condition: any md output-verbosity or HYBRID_DOCS change
  last_verification: "n=1 probe 2026-05-29: hybrid +6500, mdtools +70427 vs unix 82367 — loses cost front"

- id: AC-INSTR-pi-tokens
  statement: pi-json captures real token cost (so the local tier gates in tokens, not tool_calls)
  authority: pi --mode json usage shape (probe like claude option 2)
  verifier: a pi-json run record shows tokens_in>0
  pass_evidence: BenchResult.tokens_in non-zero for a pi-json cell
  status: OPEN   # enabler only; skip if pi emits no usage
  depends_on: []
  reopen_condition: none
  last_verification: "local tier currently tool_calls basis (pi tokens=0)"
```

## Final-verify (terminal)

The full-suite 4-mode slice, one repo state, proving every cell:

```
cargo build --release
# full suite, both tiers, ALL FOUR modes (unix hybrid hybrid-no-md + tier runner), N≥3 -> loop/runs/final/
python3 bench/report.py loop/runs/final/*.txt   # md-attribution: assert EVERY cell == CLOSES
```

`AC-MASTER` reaches PASS only when this proves the whole inventory together at
N≥3. A single cell's own run passing is `PASS_PENDING_FINAL`, never `PASS`.
Closing a structural cell requires a `CLOSES` verdict — which a neutered prompt
can never produce.
