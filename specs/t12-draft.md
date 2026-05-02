# mdtools Frontier Loop Prompt — T12 (Automated Pipeline + Cross-Model Expansion)

## Rationale

T11 fired halt #1 (gap saturation) after 8 iterations. The fixed-anchor gap
held at **+38.9pp** across all 8 iterations; 4 consecutive iterations produced
zero fixed-anchor movement and no corpus growth. Halt #6 (lock-blocked
accumulation) did not fire (counter reached 2/3; both candidates were
re-labeled `rejected-cross-seed-instability` after `md move-section` admitted,
which removed them from the lock-blocked count).

Three root causes explain the T11 saturation:

1. **Infra failures counted as saturation.** T6 retest (iter T11-7) failed due
   to an OAI wall-time overrun (60s default), not a genuine zero-delta result.
   This inflated the no-movement streak and contributed to halt #1 firing
   earlier than the underlying measurement evidence warranted.
2. **No automated pipeline.** Each candidate required 5–8 manual steps
   (generate, realism-review, measure, adversary-review, manifest, promote).
   Codex ran these ad-hoc per iteration; the corpus grew by only 2 members in
   ~25 attempts. The generator could have produced 5–10× more candidates if
   the pipeline were automated.
3. **Product-axis sweep was partial.** After `md move-section` shipped,
   only 3 lock-blocked candidates were re-evaluated. The spec's product-axis
   protocol (sweep ALL `mdtools-fail`/`lock-blocked` before declaring
   saturation) was not codified at T11 launch; it was added post-halt. T12
   starts with this protocol in place.

T12 resolves all three:

1. **`cause: infra` is non-counting** toward halt #1 saturation (codified in
   `specs/frontier-loop.md` § Cause labels).
2. **`bench/auto_research.py` replaces the manual pipeline.** One command
   runs generator → realism → measurement → adversary → manifest. Verified
   end-to-end against gemma-4-e4b-it-8bit.
3. **Product-axis sweep is iter 1.** Before generating new candidates, T12
   must re-run every candidate in `bench/search/candidates/` with status
   `rejected-lock-blocked` or `rejected-mdtools-fail` against the fixed-
   anchor corpus now that `md move-section` is available.

## What T12 lets the loop change vs. what stays locked

T12 is a **steady-state hill-climb tier** — no buildup phase, no phase flip.
The 18-task fixed-anchor corpus and +38.9pp baseline are inherited from T11.

**Free to change:**
- Generate new candidate families via `bench/auto_research.py`
- Re-evaluate prior `lock-blocked`/`mdtools-fail` candidates with new binary
- Runner prompt, system prompt, tool-policy refinements (agent-axis)
- Cross-model verification on `Qwen3.5-122B-A10B-4bit` if fixed-anchor moves ≥+5pp
- Batch-generate via multiple `auto_research.py` invocations

**Locked (unchanged from T11):**
- Fixed-anchor corpus (18 tasks from T10-10 stamp)
- Anti-folklore surface (all T11 locked primitives still forbidden)
- `md move-section` is **admitted** — fully part of the supported surface
- Measurement substrate (dual scorer, holdout guard, apples-to-apples rules)
- Promotion gate (realism + AST-structural + 3-seed cross-seed)

## Phase

T12 launches directly into **steady-state**. The buildup phase is complete;
all 18 fixed-anchor tasks are measured at the T10-10 stamp.

## Launch state

| Field | Value |
|---|---|
| Fixed-anchor gap | **+38.9pp** (inherited from T11, no movement) |
| Current-corpus gap | **+45.0pp** on 20 tasks |
| Primary model | `Qwen3.5-27B-4bit` |
| Cross-model | `Qwen3.5-122B-A10B-4bit` (trigger: fixed-anchor ≥+5pp) |
| OAI endpoint | `http://localhost:10240/v1`, API key in `~/.omlx/settings.json` |
| OAI timeout | 180s (bumped from T11's 60s default) |
| Auto-research | `bench/auto_research.py` — verified end-to-end |
| Halt #1 saturation counter | Reset to 0 (infra iterations do not count) |
| Lock-blocked counter | Reset to 0 |

## Iteration protocol (steady-state)

Each iteration must do exactly one of:

1. **Product-axis sweep (iter 1 only, mandatory):** Re-run every candidate
   in `bench/search/candidates/` with status `rejected-lock-blocked` or
   `rejected-mdtools-fail` against the fixed-anchor corpus. Update each
   manifest. Append a `cause: product` HEADLINE.md row if any fixed-anchor
   gap moves. This sweep is required before generating new candidates.

2. **Auto-research candidate:** Run `bench/auto_research.py` to generate,
   review, and measure one new candidate. If status is `pending-cross-seed`,
   run the N=3 promotion gate and promote if it passes.

3. **Agent-axis:** Change runner prompt, system prompt, or tool-policy. Re-run
   the two lowest-pass-rate fixed-anchor tasks. If both still pass, the change
   is neutral. If either regresses, this is the iteration's substantive content.

4. **Cross-model verification:** If fixed-anchor gap moved ≥+5pp, run
   `Qwen3.5-122B-A10B-4bit` on the full 18-task fixed-anchor corpus.

5. **Corpus-growth bookkeeping:** Promote a `pending-cross-seed` candidate
   that has passed the N=3 gate (3 seeds hybrid PASS, 0 seeds unix PASS,
   dual-scorer agreement). Update HEADLINE.md with `cause: corpus-growth`.

If an iteration fails due to infrastructure (`cause: infra`): log it, do NOT
count it toward halt #1, bump `--oai-request-timeout` if OAI timeout was the
cause, and retry.

## Auto-research usage

```bash
# Single new candidate (primary model resolves from /models)
python bench/auto_research.py \
  --md-binary target/release/md \
  --api-base http://localhost:10240/v1 \
  --api-key <key from ~/.omlx/settings.json>

# Use gemma for fast generation, Qwen for quality measurement
python bench/auto_research.py \
  --md-binary target/release/md \
  --api-base http://localhost:10240/v1 \
  --api-key <key> \
  --model gemma-4-e4b-it-8bit

# Dry-run (skip measurement — useful for previewing generator output)
python bench/auto_research.py \
  --md-binary target/release/md \
  --api-base http://localhost:10240/v1 \
  --api-key <key> \
  --skip-measure
```

Outputs land in `bench/search/candidates/<slug>/`. Status after one run:
- `pending-cross-seed` — gap found, AST-structural, run N=3 gate next
- `rejected-hybrid-fail-no-gap` — task too hard for all modes; try simpler doc
- `rejected-both-pass-no-gap` — unix also solved it; no structural advantage
- `rejected-planning` / `rejected-<gap-label>` — generator or adversary rejection

## Cause labels (HEADLINE.md rows)

All steady-state HEADLINE.md rows must carry one of:

- `product` — binary or scorer changed; same denominator
- `agent` — runner/prompt/policy changed; same denominator
- `corpus-growth` — denominator changed (current-corpus only)
- `composition` — denominator changed via different subset (descriptive only)
- `cross-model` — cross-model verification run
- `lock-blocked` — candidate blocked by anti-folklore forbidden list
- `infra` — infrastructure failure; does NOT count toward halt #1 saturation

## Halt conditions (inherited + T12 additions)

Inherited from T11 (full set in `specs/frontier-loop.md`):

1. **Gap saturation:** 3 consecutive iterations with no fixed-anchor movement
   AND no corpus growth. `infra` iterations do not count. Product-axis sweep
   (iter 1) does not count unless it moves the gap.
2. **Cross-model divergence:** >10pp between primary and cross-model
   fixed-anchor gaps.
3. **Endpoint failure:** MLX unreachable >5 consecutive iterations.
6. **Lock-blocked accumulation:** 3 cumulative `lock-blocked` rejections.
9. **Fixed-anchor equilibrium:** gap stable for 5 consecutive non-infra
   iterations AND corpus has grown by ≥2 under discipline since T12 launch.
   (T11's 2 promotions are part of T12's inherited baseline, not partial
   credit toward T12's equilibrium counter.)

**T12 addition:** if `auto_research.py` consistently produces
`rejected-hybrid-fail-no-gap` for 5 consecutive attempts (generator is
producing tasks too hard for all modes), switch generator model or adjust
the generator system prompt before continuing. Log the switch as
`cause: agent` in HEADLINE.md.

## What success looks like

- Fixed-anchor gap moves above +38.9pp via a genuine product or agent change.
- OR corpus grows to ≥22 members under discipline (2 new promotions since T12
  launch, same AST-structural gate as T10-T11).
- OR halt #9 fires (equilibrium with ≥2 new promotions): declared satisfied,
  ship result, route further work as scope expansion.

## Artifacts to maintain

Same as T11 — `bench/HEADLINE.md`, `bench/ledger.md`, `bench/search/`, 
`bench/runs/`, `bench/probes/`. HEADLINE.md is canonical runtime state.

Halt summary file: `bench/probes/t12-summary.md`.

## Why this is the right next loop

- **Pipeline bottleneck removed.** `auto_research.py` compresses 5–8 manual
  steps into one command. T10 produced 2 promotions from ~25 attempts (8%
  yield). T12 targets ≥5 attempts per iteration, which at even 8% yield gives
  ≥1 new promotion per 2-3 iterations.
- **Infra failures no longer stall the counter.** The T11 halt may have fired
  one iteration early due to OAI timeout counting as saturation. T12 starts
  clean.
- **Product-axis sweep may immediately move the gap.** Several T11 `rejected-
  lock-blocked` candidates were subsection-relocation shapes that `md move-
  section` directly addresses. If any of the ~4 remaining lock-blocked
  candidates pass their mdtools run now, the fixed-anchor gap could move
  before the first new candidate is generated.
- **Epoch framing deferred.** GPT Pro recommended an epoch-frozen surface +
  experimental lane model (Layer A/B/C). This is sound for a longer series
  but adds spec complexity T12 doesn't need. T12 stays with the T11 framing
  (single locked surface, one admitted primitive) and revisits epoch-framing
  if lock-blocked accumulation fires again.
