---
title: bench-v2 — add the cost axis (token-cost-to-success + tool-choice), sliced
type: feat
status: active
date: 2026-05-29
origin: docs/decisions/2026-05-28-verb-adoption-signal-response.md
---

# bench-v2 — measure cost + choice, sliced by tier and category

## Why

mdtools' bench scores **output correctness** only. For strong models correctness
saturates (they pass everything), so the bench is blind to mdtools' actual claimed
value for them — efficiency ("Opus: efficiency only", CLAUDE.md:90). The goal sits
on an axis the instrument doesn't have: **cost**. bench-v2 adds that axis.

Geometry: we measure the correctness band; the goal (`strong × cost-saved`) floats
off-chart because there is no cost axis. The gap is a missing dimension, not a
distance — so the move is instrument-first.

## Architecture Decision

**Approach:** add a **cost axis** to the existing `BenchResult` record and a
**sliced-intersection cost report**, and **adopt `hybrid` as the canonical
free-choice / cost-measurement regime — no new mode.**

**Rationale (consistency + simplicity):**
- **No new mode.** `hybrid` already grants `set(UNIX_TOOLS) | set(MDTOOLS_TOOLS)`
  with neither forced (`bench/command_policy.py:59-64`), and `HYBRID_DOCS` says
  "choose the best tool for each step" (`bench/harness.py:356-365`). Building a
  "free-choice mode" would duplicate it. **Decision: hybrid IS free-choice.**
- **Cost = tokens (primary) + tool_calls/turns (always-present proxies).**
  `BenchResult` already carries `tool_calls`, `turns`, `bytes_*`, `mutations`
  (`bench/harness.py:226-228`). The one missing signal is real token counts —
  `response.usage` is available but unread at `bench/oai_loop.py:177`. Tokens map
  to $ and to the efficiency claim; proxies keep the slice working for runners
  without usage.
- **Category from the existing `TASK_FAMILIES` dict** (`bench/report.py:16-26`) —
  do not add a tasks.json field (the avoidable fourth pillar). **Tier** from a new
  small `TIER_MAP`.
- **Factor a shared aggregation util** — the one real coupling risk (both
  `report.py` and `analyze.py` group results); extraction prevents drift.

**Headline metric:** median cost on the **both-passed intersection**, sliced
`(tier × category × mode)`.

**Trade-offs accepted:** category lives in two places until a later tasks.json
migration; tokens absent for non-oai runners (proxies cover it); "free-choice" here
measures **md-vs-unix** choice (what this bench can offer), not md-vs-native-Edit
(editor-bench's regime) — named in Scope Boundaries.

## Implementation Units

### U1 — Capture token cost (oai_loop → BenchResult)
- **Goal:** every run record carries `tokens_in` / `tokens_out`.
- **Dependencies:** None.
- **Files:**
  - Modify: `bench/oai_loop.py` (`LoopTrace` ~35-46; capture at ~177; thread through `_snapshot` ~149-160)
  - Modify: `bench/harness.py` (`BenchResult` ~216-235 add `tokens_in/out`; wire trace→result ~1345-1355)
- **Approach:** after `response = _request_json(...)` (`oai_loop.py:164-176`), read
  `response.get("usage", {})`; accumulate per turn; surface in the trace; assign
  into `BenchResult`. `asdict` persists them automatically.
- **Patterns to follow:** the existing `bytes_output` accumulation at `oai_loop.py:177-178`.
- **Test scenarios:**
  - *Happy path:* response `usage:{prompt_tokens:120,completion_tokens:40}` over 3 turns → tokens accumulate; persisted in result JSON.
  - *Edge:* response missing `usage` → fields default to 0, no crash (proxies still populate).
- **Verification:** a persisted record shows non-zero `tokens_in/out` for an oai-loop run; zero (not error) when usage absent.

### U2 — Persist per-cell tool-choice / verb mix
- **Goal:** each record carries *which* tools/verbs the agent chose (not just the `mutations` count) — the verb-adoption signal, measured in free-choice (hybrid).
- **Dependencies:** None.
- **Files:**
  - Modify: `bench/command_policy.py` (`classify_command_kind` ~128-156 to also return the verb/subcommand from `GuardEvent.raw_command`)
  - Modify: `bench/harness.py` (add `tool_mix: dict[str,int]` to `BenchResult`; populate from guard events ~1433-1443)
- **Approach:** reuse the guard-event stream that already feeds `mutations`; bucket
  by verb (`md outline`, `md replace-block`, `sed`, `grep`, …) into a small count
  map. No new log.
- **Patterns to follow:** the `mutations` count loop at `harness.py:1439-1443`.
- **Test scenarios:**
  - *Happy path:* a hybrid cell running `md blocks` then `sed` → `tool_mix={"md blocks":1,"sed":1}`.
  - *Edge:* a unix-mode cell → only unix verbs; an mdtools-mode cell → only md verbs (mode gating still holds).
- **Verification:** `tool_mix` sums consistent with `tool_calls`; in hybrid the md-vs-unix split is visible per cell.

### U3 — Shared aggregation util (the load-bearing pure function)
- **Goal:** one module computing tier, category, and **cost-on-both-passed-intersection**, consumed by both report and analyze.
- **Dependencies:** None (pure; fixture-tested).
- **Files:**
  - Create: `bench/agg_util.py` (`extract_model_tier(model)→tier`, `category_for(task_id)` reusing `TASK_FAMILIES`, `intersection_cost(records, mode_a, mode_b)`)
  - Test: `bench/test_agg_util.py`
- **Approach:** key each task-instance by `(task_id, model, thinking_level)` (the
  cross-mode join key, per `analyze.py:46-68`). For a mode pair, the comparison set
  = task-instances where **both** modes have `correct=True`; return median cost per
  mode over that set.
- **Patterns to follow:** the `TASK_FAMILIES` lookup at `report.py:16-26`.
- **Test scenarios (this unit gets the most — it is where the number can lie):**
  - *Happy path:* 5 tasks, both modes pass the same 3 → intersection size 3; median over exactly those 3 per mode.
  - *Superset:* mode A passes {T1,T2,T3}, mode B passes {T1,T2} → intersection {T1,T2}; A's cost NOT averaged over T3. (The named correctness case.)
  - *Disjoint:* A passes {T1}, B passes {T2} → empty intersection → report `n=0`, not a misleading number.
  - *Edge:* token field 0 (usage absent) → util falls back to `tool_calls` proxy; record which cost basis was used.
  - *Tier:* "claude-…"→frontier, "Qwen…"→local, unknown→"unspecified" (never crashes).
- **Verification:** `intersection_cost` over a fixture matches hand-computed medians; empty intersection yields `n=0`.

### U4 — Wire the sliced cost report
- **Goal:** `report.py` (and `analyze.py`) emit cost + pass-rate per `(tier × category × mode)` and the both-passed cost comparison, using U3.
- **Dependencies:** U1 (tokens), U3 (util); U2 for the verb-adoption column.
- **Files:**
  - Modify: `bench/report.py` (grouping ~540-570 to call `agg_util`)
  - Modify: `bench/analyze.py` (~340-418 to use the same util — kills the drift) — **DEFERRED to v2; not shipped. `analyze.py` still computes inline.**
- **Approach:** add a report section: rows `(tier, category)`, columns per mode
  showing pass-rate, median tokens, and the intersection-cost delta vs unix.
  Surface `tool_mix` adoption % for hybrid.
- **Test scenarios:**
  - *Happy path:* a small fixture run renders the tier×category table with non-empty intersection cells.
  - *Integration:* report and analyze produce identical tier/category/intersection numbers from the same records (proves the shared util removed drift). **[Deferred to v2 with the analyze wiring — this parity test is not yet written.]**
- **Verification:** the report shows, for a frontier tier, mode-vs-mode median-token deltas on the both-passed set — the missing Y-axis is now plotted.

## Scope Boundaries
- **No new bench mode** — `hybrid` is adopted as free-choice; only relabeled in the report.
- **No Rust core changes** (`src/` untouched); **no runner rewrite** — token capture is a few-line read of an existing response field.
- **"Free-choice" = md vs unix**, the choice this bench can offer; **md-vs-native-Edit stays editor-bench's** to measure.

### Deferred to Follow-Up Work
- Move `family` into `tasks.json` as the single source of truth (then `TASK_FAMILIES` reads from it) — separate cleanup PR.
- Token capture for non-oai runners (`pi_runner`) — proxies cover the gap until then.

## System-Wide Impact
- **Interaction graph:** `oai_loop` → `BenchResult` → `report`/`analyze`. Only additive fields; no existing field changes meaning.
- **API surface parity:** verdict logic lives once in `agg_util`, consumed by `report.py` (shipped). `analyze.py` still computes inline — wiring it through + the report↔analyze parity test are **deferred to v2**, so parity is not yet enforced.
- **Unchanged invariants:** existing `pass_rate` (`harness.py:976`), the three modes, and dual-scoring are untouched; bench-v2 only *adds* axes.

## Risks & Dependencies
| Risk | Mitigation |
|------|------------|
| Intersection cost computed over wrong subset (the number lies) | U3 is a pure function with the superset/disjoint cases as the gate |
| report/analyze drift | `report.py` uses `agg_util`; `analyze.py` inline — shared wiring + parity test **deferred to v2** (drift currently possible, tracked) |
| tokens absent for some runners | tool_calls/turns proxies always present; util records which basis it used |
| category mapping duplicated | accepted short-term; deferred tasks.json migration noted |

## Acceptance (end-to-end, beyond unit tests)
Run the instrumented bench against two model paths and confirm the cost slice renders:
- **pi-local** (local Qwen via the OAI endpoint) — the weak/local tier.
- **claude path** — the frontier tier.
Expected: a `(tier × category × mode)` table with median token cost on the
both-passed intersection, and a hybrid tool-mix/adoption column. The frontier row
is the one the old bench could not produce.
