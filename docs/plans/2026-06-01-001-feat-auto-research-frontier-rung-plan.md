---
title: Auto-research harness → generalizing candidate pipeline (frontier rung + N≥3 + attribution gate)
type: feat
status: active
date: 2026-06-01
origin: loop/STATE.md:67-76 (hybrid-pareto loop `autoresearch_harness_NEEDS_UPDATE`)
---

# Auto-research harness upgrade — frontier rung + N≥3 + attribution gate

## Context & Requirements

The hybrid-pareto loop concluded `md value ∝ 1/model-capability` and flagged that
`bench/auto_research.py` **mines weak-model gaps that don't generalize**: it
measures candidates only on a weak local model at N=1 with a lenient `any()`-pass
binary `hybrid−unix` gap. The relocation candidates proved the failure — promoted
as "AST-structural gaps", they collapsed on Sonnet (md loses even on move-section).
This upgrade makes candidate promotion *trustworthy enough to evolve the eval-set*.

Requirements (loop `autoresearch_harness_NEEDS_UPDATE` + resolved decisions):

- **R1** — re-measure on `Qwen3.6-35B-A3B-8bit` thinking-off (runner fix already shipped `b1c8ccb`); many prior `rejected-*` are thinking-on flail artifacts.
- **R2** — measure at **N≥3** with a **stability gate** (kill the N=1 noise trap).
- **R3** — add a **frontier measurement rung** (claude-cli) so a candidate must show a gap that survives a strong model before promotion.
- **RA** — both rungs gate on the real bench oracle `agg_util.attribution_verdict` (md-LIFT over `hybrid-no-md`), not the gameable binary `hybrid>unix`.
- **RB** — the frontier rung is **budget-gated** (real $): opt-in, token-denominated cap, defer-don't-block, write-ahead ledger, **default off**.
- **RC** — a `--remeasure <slug>` path to re-run existing wrongly-rejected candidates under the new setup.

## Architecture Decision

**Approach:** A **two-rung cheap→expensive pipeline** unified on `attribution_verdict`.
Replace `step_measure`'s 3-mode `{mdtools,hybrid,unix}` / N=1 / binary-gap with a
`{unix,hybrid,hybrid-no-md}` / N≥3 measurement scored by `attribution_verdict`.
The **local rung** (free, oai-loop/35b) runs always; only a **local-passing**
candidate pays for the **frontier rung** (claude-cli, budget-gated). A candidate
promotes only when the gap **closes on the strong model**.

**Rationale (criterion: consistency + simplicity):** reusing the production oracle
(`attribution_verdict` + `_aggregate_replicates` + `intersection_cost`) means
candidate promotion uses the *exact same* honesty gates the main bench does — the
attribution gate already kills baseline-gaming (→SUSPECT), prompt-pushing
(→no-lift), and now (via the frontier rung) weak-model-only gaps. The cheap→
expensive tiering bounds spend: no claude $ on candidates that don't even show a
free local gap.

**The one oracle extension (additive, strengthening, test-guarded):**
`category_for("C-AR-042") == "other"`, which is **not** in `STRUCTURAL_CATEGORIES`,
so `attribution_verdict` would take the tie-acceptable (Pareto-only, no-lift) path
and **skip the md-lift gate** — defeating the purpose. And `_cell_records` filters
by `category_for(task_id)==category`, so passing a different category string drops
every record. Fix: add a keyword-only `structural_override: bool | None = None` to
`attribution_verdict` — `structural = structural_override if structural_override is
not None else (category in STRUCTURAL_CATEGORIES)`. Callers pass
`category="other"` (so `_cell_records` keeps the records) + `structural_override=
True` (forces the lift gate). **Default `None` preserves byte-identical behavior**
for every existing caller; the 21 `test_agg_util` tests stay green unmodified, plus
one new test for the override. This is a deliberate, human-directed *strengthening*
of the bench tooling (sanctioned eval-evolution per `STATE.md:82`), not loop
oracle-drift.

**Trade-offs accepted:** (1) a candidate is a **single-task cell** — N≥3 replicates
aggregate to n=1 at `intersection_cost`, satisfying `min_overlap=1`; cost-lift on a
single task is weaker evidence than a category, mitigated by the stability gate +
the frontier rung (two independent confirmations). (2) The frontier rung is
**operator-gated** — a plain run never spends; full promotion requires an explicit
`--frontier-confirm` + cap. (3) auto_research is a **one-shot CLI**, so the budget
ledger is a per-run JSON file (resumable), not the loop's re-entrant STATE.md.

## High-Level Technical Design

**Pipeline data flow (new):**

```
generate → realism → [LOCAL RUNG] → [compose] → [FRONTIER RUNG?] → [compose] → manifest
                         │                            │ (only if local CLOSES
                         │                            │  AND --frontier-confirm
                         │                            │  AND fits cap)
   3 modes {unix,hybrid,hybrid-no-md}          same 3 modes, claude-cli,
   oai-loop Qwen3.6-35B-A3B-8bit, N≥3          N≥3, WAL-ledgered spend
   → re-load results.json → raw records        → re-load → raw records
   → attribution_verdict(tier=local,           → attribution_verdict(tier=frontier,
       structural_override=True)                   structural_override=True)
   → + stability gate (per-replicate)          → frontier verdict
```

**Composed status decision matrix** (replaces the binary-gap status logic):

| realism | stability | local verdict | frontier | → status |
|---|---|---|---|---|
| no | — | — | — | `rejected-planning` |
| yes | unstable | — | — | `rejected-unstable` |
| yes | stable | not CLOSES | — | `rejected-local-no-gap:<verdict>` (cheap; **no frontier spend**) |
| yes | stable | CLOSES | not run (default / no `--frontier-confirm`) | `pending-frontier-confirm` |
| yes | stable | CLOSES | over-cap (deferred) | `pending-frontier-confirm` (+ ledger `deferred` entry, ready-cmd) |
| yes | stable | CLOSES | SUSPECT:baseline-flails | `rejected-frontier-suspect:<reason>` |
| yes | stable | CLOSES | OPEN:loses-unix / OPEN:no-lift | **`rejected-weak-model-only-gap`** ← the relocation case |
| yes | stable | CLOSES | CLOSES | **`promoted-frontier-confirmed`** (the only true promote) |

`pending-cross-seed` (legacy "ready for N=3") is **subsumed**: local-CLOSES at N≥3
+ stable is now the local bar; the remaining gate is frontier confirmation.

## Composition Matrix (model-shape change)

This changes candidate acceptance from a **scalar** (`hybrid_minus_unix_pp > 0`) to
a **composed verdict** `{local: attribution_verdict, frontier: attribution_verdict,
stability: bool}`.

- **Old scalar assumptions:** `step_measure` returns an outcomes dict + `gap_pp`;
  `step_assemble_manifest` branches on `hybrid_pass`/`unix_pass`/`gap_pp`/`ast_structural`;
  `manifest.measurements.outcome` renders `hybrid_minus_unix_pp`; CLAUDE.md docs the
  `pending-cross-seed` / `rejected-*` vocabulary.
- **New composed model:** members `local_verdict` (required), `frontier_verdict`
  (optional/candidate — present only when the frontier rung ran), `stability`
  (required). Provenance: each member tags its tier + N + the bundle dirs. Trust:
  frontier > local (frontier is the generalization authority).
- **Consumer surfaces:** `manifest.json` (status + per-rung verdicts), the CLI stdout
  summary, the frontier ledger JSON, CLAUDE.md "Auto-research" section, the
  `bench/search/candidates/<slug>/manifest.json` schema readers (if any).
- **Priority lattice:** frontier verdict **overrides** local when both present (a
  weak-model CLOSES that the frontier refutes → `rejected-weak-model-only-gap`).
  Stability is a **pre-gate** — `unstable` short-circuits before either verdict is
  trusted. Realism is the outermost pre-gate (unchanged).
- **Ownership boundary:** `step_measure_rung` (new) builds each rung's raw records;
  `compose_candidate_status` (new) decides policy; `step_assemble_manifest` only
  renders the composed result.

| Mixed case | Visible contract | Typed decision | Test |
|---|---|---|---|
| local CLOSES, no frontier run | status `pending-frontier-confirm`; both rungs' verdicts shown; ledger untouched | frontier=absent | `test_compose_local_closes_pending_frontier` |
| local CLOSES + frontier CLOSES | status `promoted-frontier-confirmed`; both CLOSES rendered | promote | `test_compose_both_closes_promotes` |
| local CLOSES + frontier OPEN | status `rejected-weak-model-only-gap`; frontier named as the refuter, local not "blamed" | frontier overrides | `test_compose_frontier_refutes_weak_gap` |
| local not-CLOSES | status `rejected-local-no-gap:<verdict>`; frontier never ran (no spend) | local short-circuit | `test_compose_local_no_gap_skips_frontier` |
| unstable replicates | status `rejected-unstable`; neither verdict trusted | stability pre-gate | `test_compose_unstable_rejects` |
| legacy manifest read | manifest renders the full composed verdict set, not a lone `hybrid_minus_unix_pp` | typed verdicts overlay legacy gap field (kept for back-compat, marked advisory) | `test_manifest_renders_composed` |

## Implementation Units

### U1. `attribution_verdict` structural-override (oracle extension)

- **Goal:** let a caller force the structural (md-lift) gate for a cell whose
  `category_for` is `"other"`, without touching record filtering.
- **Requirements:** RA
- **Dependencies:** None
- **Files:** Modify `bench/agg_util.py`; Test `bench/test_agg_util.py`
- **Approach:** add keyword-only `structural_override: bool | None = None` to
  `attribution_verdict`; replace `structural = category in STRUCTURAL_CATEGORIES`
  with `structural = structural_override if structural_override is not None else
  (category in STRUCTURAL_CATEGORIES)`. No other line changes.
- **Patterns to follow:** `bench/agg_util.py:190` (the `structural = ...` line); keep
  the verdict-precedence block untouched.
- **Test scenarios:**
  - *Happy path:* `attribution_verdict(recs, "local", "other", structural_override=True)` on a neuter fixture (hybrid≈no-md) → `OPEN:no-lift` (NOT `CLOSES`) — proves the gate now fires for `other`.
  - *Edge:* `structural_override=False` on a real structural category → forces tie-acceptable (`CLOSES` on Pareto alone) — symmetric override.
  - *Regression:* default (`None`) reproduces all 21 existing verdicts unchanged (the existing suite passes unmodified).
- **Verification:** new test green; existing 21 `test_agg_util` tests pass unmodified.

### U2. Local rung → 3-mode attribution measurement + N≥3 + stability gate

- **Goal:** replace `step_measure`'s `{mdtools,hybrid,unix}`/N=1/binary-gap with
  `{unix,hybrid,hybrid-no-md}`/N≥3 scored by `attribution_verdict`, plus a stability
  gate; pin the local model to `Qwen3.6-35B-A3B-8bit`.
- **Requirements:** R1, R2, RA
- **Dependencies:** U1
- **Files:** Modify `bench/auto_research.py`; Test `bench/test_auto_research.py` (Create)
- **Approach:** (a) measure over `("unix","hybrid","hybrid-no-md")`; (b) add
  `_load_bundle_records(bundle_dir) -> list[dict]` that re-reads `results.json`
  (raw BenchResult dicts) — the existing scalar-outcomes parse stays for the stdout
  summary but the verdict path consumes raw records; (c) default
  `runs_per_task=3`; (d) `local_verdict = attribution_verdict(all_records, "local",
  "other", structural_override=True)`; (e) **stability gate:** recompute
  `attribution_verdict` on each replicate independently (N=1 each) and require
  `≥⌈2N/3⌉` per-replicate verdicts agree (CLOSES-vs-not) with the aggregate, else
  `unstable`; (f) ensure `--model` is always passed explicitly (default
  `Qwen3.6-35B-A3B-8bit`) so `extract_model_tier` → `local` deterministically.
- **Patterns to follow:** `agg_util._aggregate_replicates` (majority/median/MAX-probe);
  the loop's per-cell verifier (4-mode, N≥3); harness invocation at
  `auto_research.py:377-391`.
- **Test scenarios:**
  - *Happy path:* a fixture bundle where hybrid genuinely beats unix+no-md → `local_verdict==CLOSES`, stable.
  - *Edge (single-task n=1 intersection):* N=3 of one task aggregates to n=1 pair → no `OPEN:insufficient-evidence`; cost basis = tool_calls (oai-loop tokens=0).
  - *Stability:* replicate verdicts [CLOSES, CLOSES, OPEN] (2/3) → stable; [CLOSES, OPEN, OPEN] → `unstable`.
  - *Error:* a mode's `results.json` missing → that mode contributes no records → verdict reflects missing baseline (`OPEN:insufficient-evidence`), surfaced not crashed.
- **Verification:** local rung emits a typed `attribution_verdict` + `stable` flag per candidate; `mdtools` mode no longer run; model field on every record tiers to `local`.

### U3. Composed candidate status + manifest rendering

- **Goal:** replace the binary-gap status logic with the composed decision matrix;
  add the new status vocabulary; render both rung verdicts in the manifest.
- **Requirements:** RA, R3
- **Dependencies:** U2
- **Files:** Modify `bench/auto_research.py`; Test `bench/test_auto_research.py`
- **Approach:** new `compose_candidate_status(realism, stability, local_verdict,
  frontier_verdict) -> str` implementing the decision matrix; `step_assemble_manifest`
  calls it and renders `measurements.local_verdict` + `measurements.frontier_verdict`
  (keep the legacy `hybrid_minus_unix_pp` field marked `"advisory": true` for
  back-compat). New statuses: `rejected-unstable`, `rejected-local-no-gap:<verdict>`,
  `pending-frontier-confirm`, `rejected-weak-model-only-gap`,
  `rejected-frontier-suspect:<reason>`, `promoted-frontier-confirmed`.
- **Patterns to follow:** `auto_research.py:502-513` (current status branch);
  `manifest` schema at `auto_research.py:517-569`.
- **Test scenarios:** the six composition-matrix rows above, each → its exact status.
- **Verification:** every matrix row maps to its named status; manifest carries both
  verdicts; reading an old manifest (pre-upgrade) doesn't crash the readers.

### U4. Frontier rung — budget-gated claude-cli confirmation

- **Goal:** when a candidate's local rung CLOSES and the operator opts in, re-measure
  it on claude-cli (3 modes, N≥3) under a token-denominated cap with a write-ahead
  ledger; defer (not block) when over cap.
- **Requirements:** R3, RB
- **Dependencies:** U2, U3
- **Files:** Modify `bench/auto_research.py`; Create `bench/frontier_budget.py`; Test `bench/test_frontier_budget.py`
- **Approach:** CLI flags `--frontier-confirm` (opt-in, default off), `--frontier-model`
  (explicit, e.g. a claude model id so tier→frontier), `--frontier-token-cap`
  (default `0` = local-only/defer all). `frontier_budget.py` owns a per-run ledger
  JSON (`bench/runs/frontier-ledger.json`): fields `frontier_tokens_cumulative`,
  `per_task_tok_est` (seed `153_000`, lower-only recalibration), `entries`
  (`{slug, status: launched|reconciled, debit_tok}`), `deferred` (`{slug, ready_cmd,
  est_tok}`). Per candidate: `est = 1×3×N×per_task_tok_est`; run iff `cap>0 AND est ≤
  remaining AND remaining ≥ est×1.5`; **WAL: pre-debit worst-case → flush → run →
  reconcile to real `tokens_in+tokens_out` → flush**; a `launched`-but-unreconciled
  entry on restart is assumed spent and **not re-run**. Over-cap → status
  `pending-frontier-confirm` + a `deferred` entry with the verbatim ready-cmd. Never
  `AskUserQuestion`, never shrink N or drop a mode to fit (oracle-drift).
- **Patterns to follow:** `loop/PROMPT.md` `## Budget policy` (token cap, WAL
  pre-debit→reconcile, defer-don't-block, per-atom re-check); harness claude-cli
  invocation contract (validated: `--runner claude-cli --mode hybrid-no-md
  --tasks-path … --task-ids-path … --results-dir … -N … --model …`).
- **Test scenarios:**
  - *Happy path:* cap fits → frontier runs, ledger reconciles to real tokens, `per_task_tok_est` lowers.
  - *Defer:* `cap==0` or `est>remaining` → candidate `pending-frontier-confirm`, `deferred` entry has the exact re-launch command, **no claude invoked**.
  - *Crash-safety:* a `launched` entry without `reconciled` → next run skips that slug, worst-case debit stands.
  - *Tier:* records from claude-cli carry a frontier-tier model → `attribution_verdict(tier="frontier")` filters correctly.
  - *Edge:* over-cap must NOT be "solved" by N=2 or dropping `hybrid-no-md` — assert the batch is deferred whole.
- **State-Action Contract Matrix (ledger):**
  - Invariant: `entry.status=='reconciled' iff frontier_tokens_cumulative includes the entry's REAL token sum` (else the pre-debit worst-case stands).
  - Invariant: `slug ∈ deferred iff its frontier batch did not run this invocation AND no reconciled entry exists`.
  - `(run, cap-fits)` → claude invoked; pre-debit then reconcile; cumulative += real; ledger entry reconciled; test `test_frontier_runs_and_reconciles`.
  - `(run, over-cap)` → no claude; deferred entry written with ready-cmd; cumulative unchanged; test `test_frontier_defers_over_cap`.
  - `(restart, entry==launched)` → no claude; slug skipped; cumulative keeps worst-case debit; test `test_frontier_wal_no_respend`.
- **Verification:** a plain run (no `--frontier-confirm`) spends $0 and never calls claude; an authorized run reconciles real spend; over-cap defers with a resumable command; cap is never exceeded by more than ~one candidate's headroom (documented).

### U5. `--remeasure <slug>` path

- **Goal:** re-run an existing candidate through the new local (and optionally
  frontier) rungs without regenerating it — recover wrongly-rejected candidates.
- **Requirements:** RC
- **Dependencies:** U2, U3, U4
- **Files:** Modify `bench/auto_research.py`; Test `bench/test_auto_research.py`
- **Approach:** `--remeasure <slug>` skips generate+realism, loads the existing
  `candidates/<slug>/` (tasks.json, input.md, expected.md, prior realism), re-runs
  the local rung (+ frontier if `--frontier-confirm`), recomposes status, and
  **rewrites** `manifest.json` (preserving `generator-output.json` + the original
  realism verdict; appending a `remeasured_at` + prior-status note). Refuses if the
  candidate dir is missing required artifacts.
- **Patterns to follow:** `main()` flow at `auto_research.py:635-705` (reuse the
  measure→compose→manifest tail, skip the generate/realism head).
- **Test scenarios:**
  - *Happy path:* a prior `rejected-cross-seed-instability` candidate re-measures stable+CLOSES → status flips to `pending-frontier-confirm`.
  - *Error:* `--remeasure nonexistent-slug` → clear error, exit non-zero, no dir mutation.
  - *Idempotency:* re-running `--remeasure` twice yields the same status (manifest rewrite is deterministic; `remeasured_at` updates).
- **Verification:** an existing candidate's manifest is rewritten with the new
  composed verdict; generator/realism artifacts preserved; prior status retained in a
  note.

### U6. Tests harness + docs

- **Goal:** lock the pipeline behavior with runnable tests and update the operator docs.
- **Requirements:** R1–R3, RA–RC
- **Dependencies:** U1–U5
- **Files:** Modify `bench/test_auto_research.py`, `CLAUDE.md`; Test (self)
- **Approach:** ensure `python3 -m pytest bench/test_auto_research.py` covers the
  composition matrix + ledger + remeasure with fixture bundles (no live LLM/claude —
  synthesize `results.json` records). Update CLAUDE.md "Auto-research" section: the
  two-rung pipeline, the new status vocabulary, `--frontier-confirm` /
  `--frontier-token-cap` / `--remeasure`, and the "$0 by default" guarantee.
- **Test scenarios:** `Test expectation: aggregation only` — the unit's own tests are
  the matrix/ledger/remeasure tests from U1–U5; this unit ensures they run green
  together and the docs match the shipped flags.
- **Verification:** `python3 -m pytest bench/ -q` green; `python3 -m bench.test_agg_util`
  green (21 + 1); CLAUDE.md reflects the shipped CLI.

## Scope Boundaries

- **No `src/` (md core) changes.** This is bench/ tooling only.
- **No weakening the canonical oracle** — `bench/tasks/**`, scorers, thresholds, the
  existing `attribution_verdict` behavior (the `structural_override` param is
  additive, default-preserving, and *strengthening*).
- **No auto-promotion into the canonical corpus.** This pipeline produces a
  trustworthy `promoted-frontier-confirmed` *verdict*; the act of copying a candidate
  into `bench/tasks/tasks.json` (eval-set evolution) remains a separate, operator-gated
  step — sanctioned per `STATE.md:82` (strengthening), but out of this plan's scope.

### Deferred to Follow-Up Work

- **Batch `--remeasure-all`** over every existing candidate under the new setup: a
  thin loop over U5; ship after U5 proves single-slug remeasure.
- **`per_task_tok_est` cross-run persistence** beyond a single invocation's ledger
  (a learned-cost cache): the one-shot ledger already lower-calibrates within a run;
  a durable cost model is a later optimization.
- **Frontier-rung concurrency** (parallel candidate confirmation): serial first; the
  ledger's WAL is the prerequisite for safe parallelism.

## System-Wide Impact

- **Interaction graph:** `main()` → (generate, realism) → `step_measure` (now
  3-mode, N≥3) → `_load_bundle_records` → `attribution_verdict` (U1) → stability gate
  → `compose_candidate_status` (U3) → `frontier_budget` (U4) → `step_assemble_manifest`.
  `--remeasure` (U5) enters at `step_measure`.
- **Error propagation:** a missing/empty `results.json` per mode → that mode yields no
  records → verdict degrades to `OPEN:insufficient-evidence` (surfaced as a status,
  never a crash). Harness non-zero exit already warns (`auto_research.py:394-396`);
  keep that, and treat an empty bundle as insufficient-evidence.
- **State lifecycle risks:** the frontier ledger is the only durable state — WAL
  pre-debit→reconcile prevents double-spend on crash; a `launched`-but-unreconciled
  entry is the documented fail-closed case (worst-case debit stands, slug skipped).
- **API surface parity:** `attribution_verdict`'s new param is keyword-only and
  optional — `report.py` and the loop's callers are unaffected (they don't pass it).
- **Unchanged invariants:** existing `attribution_verdict` verdicts for all real
  TASK_FAMILIES categories; the holdout integrity guard (canonical-only, doesn't trip
  on candidate `--tasks-path`); the realism + unix-adversary steps (untouched).

## Risks & Dependencies

| Risk | Mitigation |
|---|---|
| Frontier rung overspends the cap | Token-denominated cap + 1.5× headroom + per-candidate atom re-check; documented bound = ~one candidate's headroom over cap (no mid-flight kill, by design — already-spent tokens aren't refunded). `--frontier-token-cap 0` default = $0 unless explicitly authorized. |
| `attribution_verdict` on a single-task candidate is thin evidence | Two independent confirmations (local + frontier) + the stability gate (per-replicate verdict agreement at N≥3); a single-task cost-lift is treated as supporting, not sole, evidence. |
| Stability gate too strict/lenient | Concrete rule: ≥⌈2N/3⌉ per-replicate verdicts agree with the aggregate; tunable; tested with [CLOSES,CLOSES,OPEN] (stable) and [CLOSES,OPEN,OPEN] (unstable) fixtures. |
| Model field empty → tier `unspecified` (records dropped) | Always pass `--model` explicitly on both rungs; U2/U4 assert the record `model` tiers correctly before trusting the verdict; a record with `model=None` is a hard error, not silent drop. |
| `structural_override` mistaken for oracle-drift | Additive, default-preserving, test-guarded; 21 existing tests stay green *unmodified* (editing them would itself be drift); documented as sanctioned strengthening. |
| Re-load reads a stale/partial `results.json` | Bundle dir is per-(slug,mode,run) and written by the harness atomically per run; U2 treats a missing file as insufficient-evidence, not a crash. |

**Dependencies:** `agg_util` (U1 extends it), `harness.py` claude-cli + hybrid-no-md
(exist, validated), omlx serving `Qwen3.6-35B-A3B-8bit` (confirmed), a claude budget
for the frontier rung (operator-authorized via the cap).

## Disconfirming Evidence / probe gate

The design's load-bearing claim is *"the frontier rung would have caught the
relocation false-positives."* **Probe (in U4/U5 acceptance):** `--remeasure` one of
the known relocation candidates (`C-AR-040`/`C-AR-041`) with `--frontier-confirm` +
a sufficient cap; the expected outcome is `rejected-weak-model-only-gap` (local
CLOSES, frontier OPEN:loses-unix) — *matching the by-hand Sonnet result the loop
recorded* (`STATE.md:73`). If instead it promotes, the frontier rung is not
reproducing the decisive test → stop and reconcile before trusting promotions.

## Bug-trace cross-check (loop's recorded failures → design)

| Loop finding (STATE.md) | Design clause | Match? |
|---|---|---|
| #1 candidates measured N=1 on 27B thinking-on (flail artifacts) | U2: N≥3, pin 35b/thinking-off, stability gate | ✓ |
| #2 N=1 noise trap; cross-seed-instability = N=1 artifact | U2 stability gate (per-replicate verdict agreement) + `rejected-unstable` | ✓ |
| #3 measured only on weak model → weak-model gaps | U4 frontier rung; U3 `rejected-weak-model-only-gap` | ✓ |
| relocation promoted but collapsed on Sonnet | Composition matrix row "local CLOSES + frontier OPEN" + the probe gate | ✓ |
| gameable hybrid>unix binary signal | RA: `attribution_verdict` md-lift gate via `structural_override` | ✓ |
| frontier costs real $ (unattended block last night) | RB: opt-in, cap, WAL ledger, defer-don't-block, default off | ✓ |

## High-risk checklist (oracle + budget)

- [x] Decision rationale explicit (consistency: reuse production oracle; cheap→expensive tiering)
- [x] Data flow traced end-to-end (generate→…→manifest, with re-load seam named)
- [x] Integration scenarios named (composition matrix + ledger contract matrix)
- [x] Unchanged invariants stated (existing verdicts; holdout guard; report.py callers)
- [x] Failure modes enumerated per boundary (empty bundle, model=None, crash mid-frontier, over-cap)
- [x] Files-to-touch grounded in agent findings (agg_util:190, auto_research:377/502, harness claude-cli contract)
