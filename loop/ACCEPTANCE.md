# ACCEPTANCE — hybrid-attribution-v3

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

> INSTANTIATED iteration 1 (2026-05-30). Frontier = claude-cli **Sonnet 4.6**; local =
> oai-loop **Qwen3.5-27B**. authority = `agg_util.attribution_verdict` via report.py.
> CURRENT frontier measurement = `loop/runs/iter1b-frontier/` (neutral lean HYBRID_DOCS,
> ACCEPTED). iter1b N3 verdicts: **Batch=CLOSES (PASS_PENDING_FINAL)**; Content/Metadata/
> Multi-step/Table = OPEN:no-lift; Extraction/Safe-fail/Targeted/Text = OPEN:loses-unix; no
> SUSPECT (baseline clean). Per-cell `last_verification` below may show the init/iter0
> baseline; STATE.md `iter1_verifier_result` + `last_action_iter1b` hold the authoritative
> current numbers. Local cells PENDING (sweep relaunching under neutral docs).

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
  depends_on: [all AC-frontier-*, all AC-local-*]
  reopen_condition: any md/prompt edit after this passed
  last_verification: none

# ===== FRONTIER tier (claude-cli / Sonnet 4.6) — measured N=3 2026-05-30 =====
# Root cause across the cost-losing cells: HYBRID_DOCS (67 lines / ~1284+ tok md
# reference) is re-sent every turn vs UNIX_DOCS (6 lines). T7 proof: hybrid==no-md
# ==101k, identical 5-turn behavior, +20k over unix 81k => PROMPT tax, not md usage.
# Lead lever (next iteration): lean + sharper-tool-selection HYBRID_DOCS.

- id: AC-frontier-Extraction
  statement: "STRUCTURAL. Extraction (T1,T5,T9,T11,T16,T19): hybrid Pareto-dominates unix AND md-lift over no-md."
  authority: attribution_verdict
  verifier: T1,T5,T9,T11,T16,T19 × {unix,hybrid,hybrid-no-md} × claude-cli N≥3
  pass_evidence: verdict CLOSES
  fail_evidence: OPEN:loses-unix (current)
  status: OPEN
  depends_on: []
  reopen_condition: any md/HYBRID_DOCS change
  last_verification: "2026-05-30 frontier N3: OPEN:loses-unix — CORRECTNESS regress: hybrid 83% < unix 100% (md makes hybrid FAIL 1 task no-md passes); cost on n=5 intersect hybrid 50944 < unix 62906 (md cheaper where it passes). no-md 100%."

- id: AC-frontier-Targeted-mutation
  statement: >
    frontier cell CLOSES on Targeted-mutation (T7,T10,T13,T20): hybrid
    Pareto-dominates unix AND shows positive md-lift over hybrid-no-md.
  source: observed n=1 (Opus, historical) — claude T7 mdtools +70427 tok, hybrid +6500 tok vs unix (md HURT the strong model)
  authority: attribution_verdict
  verifier: run T7,T10,T13,T20 × {unix,hybrid,hybrid-no-md} × claude-cli at N≥3; report.py verdict
  pass_evidence: verdict CLOSES (closing needs leaner HYBRID_DOCS to cut the per-turn prompt tax)
  fail_evidence: OPEN:loses-unix (current) / OPEN:no-lift / SUSPECT
  status: OPEN   # SEED — diagnosed: prompt-size tax (HYBRID_DOCS), not md output. md helps vs no-md but both > unix.
  depends_on: []
  reopen_condition: any md output-verbosity or HYBRID_DOCS change
  last_verification: "2026-05-30 frontier Sonnet N3: OPEN:loses-unix — hybrid 95612 vs unix 61062 (+56%), no-md 106166. All 100% pass. md HELPS vs no-md (95612<106166) but loses unix on cost. (Opus historical: +6500.)"

- id: AC-frontier-Batch-mutation
  statement: "STRUCTURAL (1-task: T12). hybrid Pareto-dominates unix AND md-lift over no-md."
  authority: attribution_verdict
  verifier: T12 × {unix,hybrid,hybrid-no-md} × claude-cli N≥3
  pass_evidence: verdict CLOSES
  fail_evidence: OPEN:loses-unix
  status: PASS_PENDING_FINAL   # CLOSES under neutral lean HYBRID_DOCS (iter1b). md set-task genuinely beats unix sed.
  depends_on: []
  reopen_condition: any md/HYBRID_DOCS change
  last_verification: "2026-05-30 iter1b frontier N3 (neutral lean docs): CLOSES — hybrid 70639 < unix 80787 (cost-lift vs unix) AND < no-md 93431 (cost-lift vs clean baseline), md-probe=0. md set-task loop genuinely fewer ops than unix sed. (1-task cell: N3 within-task stability.)"

- id: AC-frontier-Multi-step
  statement: "STRUCTURAL. Multi-step (T15,T18): hybrid Pareto-dominates unix AND md-lift over no-md."
  authority: attribution_verdict
  verifier: T15,T18 × {unix,hybrid,hybrid-no-md} × claude-cli N≥3
  pass_evidence: verdict CLOSES
  fail_evidence: OPEN:no-lift (current)
  status: OPEN
  depends_on: []
  reopen_condition: any md/HYBRID_DOCS change
  last_verification: "2026-05-30 frontier N3: OPEN:no-lift — pareto OK (hybrid 87434 < unix 105915) but hybrid≈no-md (87074): md adds NO lift, the PROMPT does the work. Closing needs md to beat no-md by 5%."

- id: AC-frontier-Content-delivery
  statement: "STRUCTURAL. Content delivery (T2,T3,T8,T17): hybrid Pareto-dominates unix AND md-lift over no-md."
  authority: attribution_verdict
  verifier: T2,T3,T8,T17 × {unix,hybrid,hybrid-no-md} × claude-cli N≥3
  pass_evidence: verdict CLOSES
  fail_evidence: OPEN:loses-unix (current)
  status: OPEN
  depends_on: []
  reopen_condition: any md/HYBRID_DOCS change
  last_verification: "2026-05-30 frontier N3: OPEN:loses-unix — n=3 intersect hybrid 65368 vs unix 59935 (+9%>5%). All modes 75% pass (T-fails shared)."

- id: AC-frontier-Safe-fail
  statement: "STRUCTURAL (1-task: T14). hybrid Pareto-dominates unix AND md-lift over no-md."
  authority: attribution_verdict
  verifier: T14 × {unix,hybrid,hybrid-no-md} × claude-cli N≥3
  pass_evidence: verdict CLOSES
  fail_evidence: OPEN:loses-unix (current)
  status: OPEN
  depends_on: []
  reopen_condition: any md/HYBRID_DOCS change
  last_verification: "2026-05-30 frontier N3: OPEN:loses-unix — hybrid 28645 vs unix 25156 (+14%), no-md 47408. All 100% pass. (1-task cell.)"

- id: AC-frontier-Metadata
  statement: "STRUCTURAL. Metadata (T21,T22,T24): hybrid Pareto-dominates unix AND md-lift over no-md."
  authority: attribution_verdict
  verifier: T21,T22,T24 × {unix,hybrid,hybrid-no-md} × claude-cli N≥3
  pass_evidence: verdict CLOSES
  fail_evidence: OPEN:loses-unix (current)
  status: OPEN
  depends_on: []
  reopen_condition: any md/HYBRID_DOCS change
  last_verification: "2026-05-30 frontier N3: OPEN:loses-unix — md gives CORRECTNESS LIFT (hybrid 100% vs unix 67%!) but cost on n=2 intersect hybrid 46282 vs unix 42284 (+9%) fails Pareto. Closing = keep the +33pp pass while trimming md cost ≤unix+5%."

- id: AC-frontier-Table-projection
  statement: "STRUCTURAL (1-task: T23). hybrid Pareto-dominates unix AND md-lift over no-md."
  authority: attribution_verdict
  verifier: T23 × {unix,hybrid,hybrid-no-md} × claude-cli N≥3
  pass_evidence: verdict CLOSES
  fail_evidence: OPEN:loses-unix (current)
  status: OPEN
  depends_on: []
  reopen_condition: any md/HYBRID_DOCS change
  last_verification: "2026-05-30 frontier N3: OPEN:loses-unix — hybrid 27454 vs unix 24702 (+11%); no-md FAILS (0%/84342) so md is ESSENTIAL here. Closing = trim md cost ≤unix+5%. (1-task cell.)"

- id: AC-frontier-Text-manipulation
  statement: "TIE-ACCEPTABLE (T4,T6). hybrid ≥ unix on Pareto (a tie is the win — 'don't replace sed'). No md-lift required."
  authority: attribution_verdict
  verifier: T4,T6 × {unix,hybrid,hybrid-no-md} × claude-cli N≥3
  pass_evidence: verdict CLOSES (Pareto alone)
  fail_evidence: OPEN:loses-unix (current)
  status: OPEN   # tie-target candidate: unix structurally better; goal is hybrid≥unix, not lift
  depends_on: []
  reopen_condition: any md/HYBRID_DOCS change
  last_verification: "2026-05-30 frontier N3: OPEN:loses-unix — hybrid 218294 vs unix 84636 (+158%!). md-heavy prompt steers Sonnet to over-explore structural approaches on text tasks (8-15 turns vs unix 4-5). Closing = HYBRID_DOCS must steer unix for plain text."

# ===== LOCAL tier (oai-loop / Qwen3.5-27B) — PENDING sweep bgqm9t2kg =====
# 9 cells, same families. Verdicts instantiated when the local sweep completes
# (~22h; weak model flails to max-turns in unix/no-md modes — that IS the md-signal).
- id: AC-local-Extraction
  statement: "STRUCTURAL. Extraction × oai-loop/Qwen3.5-27B N≥3."
  authority: attribution_verdict
  verifier: T1,T5,T9,T11,T16,T19 × {unix,hybrid,hybrid-no-md} × oai-loop N≥3
  pass_evidence: verdict CLOSES
  fail_evidence: pending
  status: OPEN
  depends_on: []
  reopen_condition: any md/HYBRID_DOCS change
  last_verification: "pending — local sweep bgqm9t2kg running"
- id: AC-local-Targeted-mutation
  statement: "STRUCTURAL. Targeted mutation × oai-loop/Qwen3.5-27B N≥3."
  authority: attribution_verdict
  verifier: T7,T10,T13,T20 × {unix,hybrid,hybrid-no-md} × oai-loop N≥3
  pass_evidence: verdict CLOSES
  fail_evidence: pending
  status: OPEN
  depends_on: []
  reopen_condition: any md/HYBRID_DOCS change
  last_verification: "pending — local sweep bgqm9t2kg running. (historical pi probe: md HELPED local on calls T7 unix10/hybrid4)"
- id: AC-local-Batch-mutation
  statement: "STRUCTURAL (1-task: T12) × oai-loop/Qwen3.5-27B N≥3."
  authority: attribution_verdict
  verifier: T12 × {unix,hybrid,hybrid-no-md} × oai-loop N≥3
  pass_evidence: verdict CLOSES
  fail_evidence: pending
  status: OPEN
  depends_on: []
  reopen_condition: any md/HYBRID_DOCS change
  last_verification: "pending — local sweep bgqm9t2kg running"
- id: AC-local-Multi-step
  statement: "STRUCTURAL. Multi-step (T15,T18) × oai-loop/Qwen3.5-27B N≥3."
  authority: attribution_verdict
  verifier: T15,T18 × {unix,hybrid,hybrid-no-md} × oai-loop N≥3
  pass_evidence: verdict CLOSES
  fail_evidence: pending
  status: OPEN
  depends_on: []
  reopen_condition: any md/HYBRID_DOCS change
  last_verification: "pending — local sweep bgqm9t2kg running"
- id: AC-local-Content-delivery
  statement: "STRUCTURAL. Content delivery (T2,T3,T8,T17) × oai-loop/Qwen3.5-27B N≥3."
  authority: attribution_verdict
  verifier: T2,T3,T8,T17 × {unix,hybrid,hybrid-no-md} × oai-loop N≥3
  pass_evidence: verdict CLOSES
  fail_evidence: pending
  status: OPEN
  depends_on: []
  reopen_condition: any md/HYBRID_DOCS change
  last_verification: "pending — local sweep bgqm9t2kg running"
- id: AC-local-Safe-fail
  statement: "STRUCTURAL (1-task: T14) × oai-loop/Qwen3.5-27B N≥3."
  authority: attribution_verdict
  verifier: T14 × {unix,hybrid,hybrid-no-md} × oai-loop N≥3
  pass_evidence: verdict CLOSES
  fail_evidence: pending
  status: OPEN
  depends_on: []
  reopen_condition: any md/HYBRID_DOCS change
  last_verification: "pending — local sweep bgqm9t2kg running"
- id: AC-local-Metadata
  statement: "STRUCTURAL. Metadata (T21,T22,T24) × oai-loop/Qwen3.5-27B N≥3."
  authority: attribution_verdict
  verifier: T21,T22,T24 × {unix,hybrid,hybrid-no-md} × oai-loop N≥3
  pass_evidence: verdict CLOSES
  fail_evidence: pending
  status: OPEN
  depends_on: []
  reopen_condition: any md/HYBRID_DOCS change
  last_verification: "pending — local sweep bgqm9t2kg running"
- id: AC-local-Table-projection
  statement: "STRUCTURAL (1-task: T23) × oai-loop/Qwen3.5-27B N≥3."
  authority: attribution_verdict
  verifier: T23 × {unix,hybrid,hybrid-no-md} × oai-loop N≥3
  pass_evidence: verdict CLOSES
  fail_evidence: pending
  status: OPEN
  depends_on: []
  reopen_condition: any md/HYBRID_DOCS change
  last_verification: "pending — local sweep bgqm9t2kg running"
- id: AC-local-Text-manipulation
  statement: "TIE-ACCEPTABLE (T4,T6) × oai-loop/Qwen3.5-27B N≥3. hybrid ≥ unix; no lift required."
  authority: attribution_verdict
  verifier: T4,T6 × {unix,hybrid,hybrid-no-md} × oai-loop N≥3
  pass_evidence: verdict CLOSES (Pareto alone)
  fail_evidence: pending
  status: OPEN
  depends_on: []
  reopen_condition: any md/HYBRID_DOCS change
  last_verification: "pending — local sweep bgqm9t2kg running"

- id: AC-INSTR-pi-tokens
  statement: local tier captures real token cost (so it gates in tokens, not tool_calls)
  authority: runner usage shape
  verifier: a local run record shows tokens_in>0
  pass_evidence: BenchResult.tokens_in non-zero for a local cell
  status: SATISFIED-via-oai-loop   # enabler: oai-loop captures tokens (17326/2748 on smoke) -> local gates in TOKENS. pi-json gives 0; we use oai-loop.
  depends_on: []
  reopen_condition: none
  last_verification: "2026-05-30: oai-loop/Qwen3.5-27B emits tokens_in>0 (smoke 17326). Local tier gates in tokens basis — richer than the spec's tool_calls fallback. (pi-json still tokens=0; moot since local=oai-loop.)"
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
