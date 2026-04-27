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
| Primary model | `Qwen3.6-35B-A3B-8bit` (small MoE, 3B active params) |
| Endpoint | `http://localhost:10240/v1` (OAI-compatible MLX) |
| Modes | `unix` / `mdtools` / `hybrid` (all three required for gap calc) |
| Corpus | `bench/tasks/tasks.json` minus `bench/holdout/task_ids.json` IDs |
| Holdout | `bench/holdout/` — never read by the loop, only by post-run audit |
| Cross-model stability check | `Qwen3.5-27B-4bit` (single confirmation run when gap moves ≥+5pp) |

## Current value

| Metric | Value | As of | Bundle |
|---|---:|---|---|
| hybrid pass rate | _pending first run_ | — | — |
| unix pass rate | _pending first run_ | — | — |
| **Gap (hybrid − unix)** | **_pending_** | — | — |
| Corpus size | 18 (24 total − 6 holdout) | 2026-04-27 | — |

## Hill-climb history

_(loop appends one row per iteration that moves the gap or grows the corpus)_

| Iter | Date | Gap | Corpus | Δ Gap | Δ Corpus | Move | Bundle |
|---:|---|---:|---:|---:|---:|---|---|
| _none yet_ | | | | | | | |

## Hill-climb rules (delta from T9 spec)

- An iteration that **moves the gap up** (via product hardening, scorer fix
  on existing surface, or unix-mode degradation discovered) records its bundle
  pointer in the history table.
- An iteration that **grows the corpus** with a candidate that survived
  realism + unix-adversary review records the candidate pointer.
- An iteration that does **neither** is inadmissible (per T9 core law).
- Cross-model stability check fires automatically when the gap moves ≥+5pp:
  re-run the full corpus on `Qwen3.5-27B-4bit` and record both numbers.
  If the cross-model gap diverges from the primary by >10pp, file a finding.

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
