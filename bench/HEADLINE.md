# mdtools Headline Metric

The single number this repo's loop is hill-climbing.

## The number

**Hybrid pass rate minus unix pass rate**, on the target small model, over the
full search corpus (`bench/search/` + `bench/tasks/tasks.json`-listed members
of `bench/holdout/task_ids.json` excluded).

This is the *gap* — the legitimacy claim is "agents perform measurably better
on Markdown editing with mdtools than without," and a gap directly measures
that claim. The gap can grow either by hybrid going up (mdtools wins more) or
by unix going down (corpus added tasks unix is structurally bad at) — both
count.

## Target

| Field | Value |
|---|---|
| Primary model | `Qwen3.5-27B-4bit` (small dense, established T7 baseline) |
| Endpoint | `http://localhost:10240/v1` (OAI-compatible MLX) |
| Modes | `unix` / `mdtools` / `hybrid` (all three required for gap calc) |
| Corpus | `bench/tasks/tasks.json` minus `bench/holdout/task_ids.json` IDs |
| Holdout | `bench/holdout/` — never read by the loop, only by post-run audit |
| Cross-model stability check | `Qwen3.5-122B-A10B-4bit` (same family, larger; isolates model-size effect when gap moves ≥+5pp) |

## Current value

| Metric | Value | As of | Bundle |
|---|---:|---|---|
| hybrid pass rate | 72.7% (8/11) | 2026-04-27 | iter 1 + iter 2 + iter 3 hybrid bundles |
| unix pass rate | 18.2% (2/11) | 2026-04-27 | iter 1 + iter 2 + iter 3 unix bundles |
| **Gap (hybrid − unix)** | **+54.5pp** | 2026-04-27 | both |
| Measured subset | extraction + mutation + multi-step (11/18 search corpus): T1, T5, T7, T9, T10, T11, T13, T15, T16, T18, T19 | 2026-04-27 | iter 3 |
| Corpus size (search total) | 18 (24 total − 6 holdout) | 2026-04-27 | — |
| Cross-model trigger | **fired** — iter 2→3 gap moved +10.1pp; iter 4 must run cross-model check before any other work | 2026-04-27 | per T9 spec § "Cross-model trigger" |

## Hill-climb history

_(loop appends one row per iteration that moves the gap or grows the corpus)_

| Iter | Date | Gap | Corpus | Δ Gap | Δ Corpus | Move | Bundle |
|---:|---|---:|---:|---:|---:|---|---|
| _none yet_ | | | | | | | |
| 1 | 2026-04-27 | +50.0pp | 6 | corpus baseline | 6 | first baseline (extraction subset) | bench/runs/headline-baseline-hybrid-Qwen3.5-27B-4bit-2026-04-27/ + bench/runs/headline-baseline-unix-Qwen3.5-27B-4bit-2026-04-27/ |
| 2 | 2026-04-27 | +44.4pp | 9 | −5.6pp | +3 | extend baseline to mutation family (T7, T10, T13); hybrid 3/3, unix 2/3 (T13 fails) — mutation gap +33.3pp dilutes the extraction-only +50.0pp toward a more honest 9-task baseline | bench/runs/headline-mutation-hybrid-Qwen3.5-27B-4bit-2026-04-27/ + bench/runs/headline-mutation-unix-Qwen3.5-27B-4bit-2026-04-27/ |
| 3 | 2026-04-27 | +54.5pp | 11 | +10.1pp | +2 | extend baseline to multi-step family (T15, T18); hybrid 2/2, unix 0/2 — multi-step gap +100pp confirms CLAUDE.md "Strong" rating: re-query pattern after delete-section is the moat. Unix T15 made 7 mutations across 9 calls but couldn't recover from index drift; T18 hit policy denials (deny:2) and produced 0 mutations. Cross-model trigger fired (Δ +10.1pp ≥ +5pp threshold) — iter 4 must run cross-model on `Qwen3.5-122B-A10B-4bit` before any other work | bench/runs/headline-multistep-hybrid-Qwen3.5-27B-4bit-2026-04-27/ + bench/runs/headline-multistep-unix-Qwen3.5-27B-4bit-2026-04-27/ |

## Hill-climb rules (delta from T9 spec)

- An iteration that **moves the gap up** (via product hardening, scorer fix
  on existing surface, or unix-mode degradation discovered) records its bundle
  pointer in the history table.
- An iteration that **grows the corpus** with a candidate that survived
  realism + unix-adversary review records the candidate pointer.
- An iteration that does **neither** is inadmissible (per T9 core law).
- Cross-model stability check fires automatically when the gap moves ≥+5pp:
  re-run the full corpus on `Qwen3.5-122B-A10B-4bit` (same family, larger) and
  record both numbers. If the cross-model gap diverges from the primary by
  >10pp, file a finding. Per CLAUDE.md the gap is expected to *shrink* on the
  larger model ("tool benefit inversely proportional to model capability") —
  the divergence guard is for the unexpected case (gap *grows* on larger
  model, suggesting the corpus is overfit to small-model pathologies rather
  than measuring real structural advantage).

## Halt criteria for the headline

The loop should stop hill-climbing when either:

1. **Gap saturates:** 3 consecutive promotion attempts produce no gap movement
   AND no corpus growth survives review. The corpus is converged for the
   target model.
2. **Cross-model divergence:** primary-vs-cross-model gap diverges by >10pp,
   suggesting the primary metric is overfit to the target model. Halt and
   investigate before continuing.
3. **Endpoint failure:** MLX server unreachable for >5 consecutive iterations.
4. **Cheap channel red:** any committed change breaks `cargo test` or the
   python unittest suite.

## What this file is NOT

- Not the ledger (findings live in `bench/ledger.md`).
- Not the spec (rules live in `specs/frontier-loop.md`).
- Not a snapshot of past runs (history lives in `bench/runs/` per-bundle).
- Just the one number, what it currently is, and how it has moved over time.
