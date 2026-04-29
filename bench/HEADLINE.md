# mdtools Headline Metric

The number(s) this repo's loop is hill-climbing.

## Phase

phase: baseline-buildup

Fixed-anchor 18-task baseline does not yet exist (17/18 measured).
Cross-model triggers, auto-research, and hill-climb interpretation are
deferred to steady-state. The only admissible move is to extend
baseline coverage. Phase flips to `phase: steady-state` when the
missing 1 task lands.

## The numbers

T10 declares two gaps:

- **Fixed-anchor gap** = hybrid pass rate − unix pass rate on the
  original 18-task search corpus (`bench/tasks/tasks.json` minus
  `bench/holdout/task_ids.json` IDs). This is the legitimate
  hill-climb signal — moves only via product/scorer/agent change,
  never via composition. **Currently undefined** (baseline-buildup).
- **Current-corpus gap** = hybrid pass rate − unix pass rate on the
  currently-measured corpus. Descriptive only — moves with composition.

Per Pro consensus (T9 iter 3 second-opinion, both lanes 0.84
confidence): the iter 2→3 +10.1pp current-corpus delta was a
denominator change (added the multi-step family, +100pp per-family),
not a hill-climb signal. T10 rebuilds the metric with explicit phases
to prevent this drift.

## Target

| Field | Value |
|---|---|
| Primary model | `Qwen3.5-27B-4bit` (small dense, established T7 baseline) |
| Endpoint | `http://localhost:10240/v1` (OAI-compatible MLX) |
| Modes | `unix` / `mdtools` / `hybrid` (all three required for gap calc) |
| Search corpus | 18 tasks (`bench/tasks/tasks.json` minus 6 holdout IDs) |
| Holdout | `bench/holdout/` — never read by the loop, only by post-run audit |
| Cross-model stability check | `Qwen3.5-122B-A10B-4bit` (same family, larger; isolates model-size effect when **fixed-anchor** gap moves ≥+5pp in steady-state) |

## Missing primary-baseline tasks

1 of 18 search-corpus tasks remains unmeasured. Subsequent buildup
iterations must close this list before phase flips to `steady-state`.

| Task | Family (per CLAUDE.md) | Expected mdtools advantage |
|---|---|---|
| T8 | content-delivery | Moderate |

## Current value

| Metric | Value | As of | Bundle |
|---|---:|---|---|
| Fixed-anchor gap | _undefined_ (baseline-buildup, 17/18) | 2026-04-28 | — |
| Current-corpus gap (hybrid − unix) | **+41.2pp** | 2026-04-28 | T9-1+2+3 + T10-2+3+4+6+8+9 bundles |
| Current-corpus hybrid | 64.7% (11/17) | 2026-04-28 | — |
| Current-corpus unix | 23.5% (4/17) | 2026-04-28 | — |
| Measured subset | T1, T2, T3, T5, T6, T7, T9, T10, T11, T12, T13, T15, T16, T17, T18, T19, T21 | 2026-04-28 | — |
| Search corpus size | 18 (24 total − 6 holdout) | — | — |

## Hill-climb history

_(loop appends one row per iteration that extends baseline, moves a
gap, or grows the corpus. Every row carries a `cause` label.)_

| Iter | Date | Phase | Gap (kind, value) | Δ | Cause | Bundle |
|---:|---|---|---|---:|---|---|
| T9-1 | 2026-04-27 | buildup | current-corpus +50.0pp (6/18) | — | baseline-buildup | bench/runs/headline-baseline-{hybrid,unix}-Qwen3.5-27B-4bit-2026-04-27/ |
| T9-2 | 2026-04-27 | buildup | current-corpus +44.4pp (9/18) | −5.6 | baseline-buildup | bench/runs/headline-mutation-{hybrid,unix}-Qwen3.5-27B-4bit-2026-04-27/ |
| T9-3 | 2026-04-27 | buildup | current-corpus +54.5pp (11/18) | +10.1 | baseline-buildup (originally mis-classified as hill-climb; retroactively re-labeled per T10 spec — composition, not improvement) | bench/runs/headline-multistep-{hybrid,unix}-Qwen3.5-27B-4bit-2026-04-27/ |
| T10-2 | 2026-04-28 | buildup | current-corpus +50.0pp (12/18) | −4.5 | baseline-buildup (T21 added; both modes failed scoring with `frontmatter_json: MISMATCH` — Qwen3.5-27B emitted just the parsed `data` payload instead of the full `md frontmatter --json` envelope; both modes had equal-shape failure so denominator-only delta) | bench/runs/headline-buildup-T21-{hybrid,unix}-Qwen3.5-27B-4bit-2026-04-28/ |
| T10-3 | 2026-04-28 | buildup | current-corpus +53.8pp (13/18) | +3.8 | baseline-buildup (T12 added; hybrid PASS in 176s/15 calls/8 mut, unix FAIL after 1237s/30 turns/29 invalid responses/1 tool call — model produced 81KB of malformed output and never reached a working sed/awk plan; per-family gap = +100pp on the batch-mutation family. Δ is composition only, not a hill-climb signal) | bench/runs/headline-buildup-T12-{hybrid,unix}-Qwen3.5-27B-4bit-2026-04-28/ |
| T10-4 | 2026-04-28 | buildup | current-corpus +50.0pp (14/18) | −3.8 | baseline-buildup (T17 added; hybrid PASS in 42.7s/2 calls/1 mut/3 turns, unix PASS in 187.8s/11 calls/1 mut/13 turns/2 deny — agent tried `md replace-section` twice (denied), then succeeded via head/cat/tail/mv splice on the 19-line file; per-family gap = 0pp on this single content-delivery instance. Δ is composition only, not a hill-climb signal) | bench/runs/headline-buildup-T17-{hybrid,unix}-Qwen3.5-27B-4bit-2026-04-28/ |
| T10-6 | 2026-04-28 | buildup | current-corpus +46.7pp (15/18) | −3.3 | baseline-buildup (T6 added; hybrid FAIL in 679.7s/19 calls/8 mut/30 turns/11 invalid responses, unix FAIL in 382.2s/17 calls/5 mut/30 turns/13 invalid/1 policy-deny — both modes max-turn out, scorer agreement: dual scorers BOTH false. block_order MISMATCH: actual output has 1 extra block (44 vs expected 43), neither mode removed the Phase 3 thematic break. Per-family gap = 0pp on this single text-manipulation instance, contradicting CLAUDE.md's "unix wins simple sed/awk" prediction — Qwen3.5-27B can't plan the compound 3-step edit in either mode. Δ is composition only, not a hill-climb signal) | bench/runs/headline-buildup-T6-{hybrid,unix}-Qwen3.5-27B-4bit-2026-04-28/ |
| T10-8 | 2026-04-28 | buildup | current-corpus +43.8pp (16/18) | −3.0 | baseline-buildup (T2 added — runs launched in parallel per iter-7 recommendation; hybrid FAIL in 1260.2s/4 calls/0 mut/30 turns/26 invalid/10 unique/2 policy-deny, unix FAIL in 582.7s/2 calls/0 mut/13 turns/10 invalid/9 unique/1 policy-deny — both modes never wrote to file (mutations=0). dual scorer agreement: md and neutral both false. block_order MISMATCH: actual output is identical to input (7 blocks), expected has 9 (the inserted v2.5 heading + paragraph). Per-family gap = 0pp on this single content-delivery instance — Qwen3.5-27B can't plan the insert-section-after-block-N operation in either mode without writing to disk. Δ is composition only, not a hill-climb signal) | bench/runs/headline-buildup-T2-{hybrid,unix}-Qwen3.5-27B-4bit-2026-04-28/ |
| T10-9 | 2026-04-28 | buildup | current-corpus +41.2pp (17/18) | −2.6 | baseline-buildup (T3 added; hybrid PASS in 696.9s/12 calls/3 mut/30 turns/18 invalid/10 unique/2 policy-deny, unix PASS in 886.1s/22 calls/9 mut/30 turns/8 invalid/7 unique. dual scorer agreement: md and neutral both true. Both modes eventually replaced only the second duplicate `## Methods` section and removed the blank line before `## Changelog`; per-family gap remains 0pp for measured content-delivery tasks because both modes pass T3. Δ is composition only, not a hill-climb signal) | bench/runs/headline-buildup-T3-{hybrid,unix}-Qwen3.5-27B-4bit-2026-04-28/ |

## T9 iter 4 (aborted)

Iter 4 fired a cross-model run on `Qwen3.5-122B-A10B-4bit` for the
6-task extraction subset and hit the 5h wall clock before the harness
finished scoring. No `results.json` / `run.json` were produced;
agent transcripts under `logs/` are gitignored, so no committed
bundle exists for this run. **No headline evidence base entry**.
T10 starts fresh — iter 4's partial activity is not part of the
auditable history.

## Phase transition criteria

Flip `phase: baseline-buildup` → `phase: steady-state` when:

1. All 18 search-corpus tasks measured on `Qwen3.5-27B-4bit` in
   hybrid + unix modes with dual scorer agreement.
2. Fixed-anchor gap stamped as the inaugural steady-state value.
3. Per-family table populated.
4. Spec contract: phase transition counts as that iteration's
   substantive move.

## Per-family pass rate (companion table)

Populated as baseline-buildup completes. Family definitions per
`CLAUDE.md § Task families`.

| Family | Tasks | hybrid | unix | per-family gap |
|---|---|---:|---:|---:|
| Extraction | T1, T5, T9, T11, T16, T19 | 3/6 | 0/6 | +50.0pp |
| Targeted mutation | T7, T10, T13 (T20 holdout) | 3/3 | 2/3 | +33.3pp |
| Batch mutation | T12 | 1/1 | 0/1 | +100.0pp |
| Multi-step | T15, T18 | 2/2 | 0/2 | +100.0pp |
| Content delivery | T2, T3, T8, T17 | 2/3 (T3, T17 pass; T2 fail) | 2/3 (T3, T17 pass; T2 fail) | 0.0pp (T2+T3+T17 measured) |
| Text manipulation | T6 (T4 holdout) | 0/1 | 0/1 | 0.0pp |
| Other | T21 | 0/1 | 0/1 | 0.0pp |

(Holdout-only families: T14 = safe-fail, T22/T23/T24 = misc.)

## Hill-climb rules (delta from T10 spec)

- **Baseline-buildup phase:** only "extend baseline coverage" is
  admissible. All Δ columns are labeled `composition` (since the
  denominator changes by adding tasks). Cross-model triggers, auto-
  research, and hill-climb interpretation deferred to steady-state.
- **Steady-state phase:** T9's 5 admissible moves activate (grow
  corpus, promote candidate, harden trace, cross-model, halt). Every
  history row carries a `cause` label
  (product / agent / corpus-growth / composition / cross-model).
- **Cross-model trigger:** fires only in steady-state, only on
  fixed-anchor gap movement ≥+5pp. Current-corpus deltas never trigger.
- **Composition discipline:** if a steady-state iteration moves only
  `current-corpus` gap via composition, it is admissible bookkeeping
  but does NOT count as hill-climb progress.

## Halt criteria

Per T10 spec § Halt conditions:

1. **Gap saturation (steady-state only):** 3 consecutive promotion
   attempts without fixed-anchor movement or surviving corpus growth.
2. **Cross-model divergence:** primary-vs-cross-model fixed-anchor
   gap diverges >10pp.
3. **Endpoint failure:** MLX unreachable >5 consecutive iterations.
4. **Cheap channel red** unrepairable in iteration.
5. **Ledger budget breach** unrepairable.
6. **CLI temptation:** any new CLI primitive proposal halts.
7. **Spec incoherence:** rules contradict or block all moves.
8. **Buildup stall:** 3 consecutive iterations fail to extend baseline.

## What this file is NOT

- Not the ledger (findings live in `bench/ledger.md`).
- Not the spec (rules live in `specs/frontier-loop.md`).
- Not a snapshot of past runs (history lives in `bench/runs/` per-bundle).
- Just the two numbers, the current phase, and the move history.
