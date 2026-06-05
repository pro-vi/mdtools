# md's frontier edge vs native Edit — falsified (Sonnet 4.6)

**Date:** 2026-06-04
**Status:** decided (negative result — a reposition, not a failure)
**Method:** the bench-v2 native arm (md vs native Read/Edit/Write, claude-cli),
billed-$ basis, N=3 per cell, gated by the new native-adversary.
**Question:** with native `Read`/`Edit`/`Write` available to a frontier agent, does
`md` (the structural CLI) have a *structural-entropy* edge — a family where
`native+md` Pareto-beats `native` (a cost win or correctness lift), holdout-confirmed?

## Answer

**No — not at the Sonnet 4.6 frontier, across the three most plausible families.**
md's claimed structural edge does not survive contact with a capable model that
has native file tools. The one apparent win did not replicate on a faithful
holdout. This is the honest falsification the FRAC-194 loop set out to find.

## What was measured

A native-adversary was added to `bench/auto_research.py` (the gate that was
missing): run `native` mode on each candidate; a task native Edit solves at
parity is `REJECTED_NO_MD_EDGE`, not an md edge. Three native-adversary-vetted
hard families were generated (deterministic generators = de-bias provenance) and
measured native vs native+md at N=3 billed-$:

| Family | native+md vs native (billed) | md adopted? | Verdict |
|--------|------------------------------|-------------|---------|
| duplicate-heading disambiguation | — (native solves 3/3) | n/a | `REJECTED_NO_MD_EDGE` |
| conditional many-checkbox batch *(md's flagship)* | **+25.8%** | yes (`set-task` loop) | `REJECTED_NO_MD_EDGE` |
| large-file slicing — search (C-LF-01) | **−21.3%** | yes (`outline`×3) | CLOSES (search only) |
| large-file slicing — holdout v1 (C-LF-02) | −11.9% | **no** (greppable target) | CONFOUNDED |
| large-file slicing — corrected holdout (C-LF-03) | **+5.2%** | **yes** (`outline`×3–7) | `REJECTED_NO_MD_EDGE` |

## Why native wins (the mechanism)

1. **Native Edit fuses.** For localized edits — even a 10-item conditional batch —
   a capable model reads the relevant region once and emits **one** multi-line
   `Edit` on already-read content. md's per-primitive model (`md tasks` +
   N×`md set-task`, or `section`+`replace`) is N separate Bash round-trips →
   more cache-read re-reads and turns → md costs **+25.8%** on its own flagship
   batch family. (Same mechanism as the +28% easy-edit baseline, FRAC-194 N1.)

2. **The model adapts when locating is hard.** Native isn't just `Edit` — it's
   `Bash`+`grep`/`sed`/`awk`+`Read`+`Edit`. On a large doc it greps to locate,
   reads a slice, and edits. It only thrashes when the target value is ambiguous
   *and* it greps the value instead of the section heading — which is variance,
   not a structural wall.

3. **md's structural navigation backfires at depth.** On the corrected large-file
   holdout (deeply nested, non-greppable target), md *was* adopted (3/3) — but
   the agent called `md outline` **3–7×** on a large outline tree *and still
   grepped*. md added exploration cost instead of replacing it → **+5.2%**. The
   −21.3% search win was an efficient-adoption outlier, not a robust effect; the
   family's clean cells straddle zero with high variance.

## The honesty checkpoint that mattered

The first holdout (v1) reported `CLOSES −11.9%` and would have looked like a
confirmation. The **adoption split** refuted it: native+md never invoked md (a
unique greppable marker let it grep straight to the target), so the −11.9% was
within-strategy run variance. The verdict turns on *whether md was used and
helped*, not on the cost number alone. Future large-file cells must report the
adoption split.

## Reposition (what to tell users)

This **sharpens and confirms** the standing thesis — *tool benefit is inversely
proportional to model capability* — now with the deployment-true native oracle:

- **For frontier agents with native file tools, `md` is not a cost or correctness
  win over native `Edit`** — not for localized edits, not for batches, and not
  (reliably) for large/deep documents. Do **not** position md as "faster/cheaper
  for capable coding agents."
- **md's real value is unchanged and elsewhere:** (a) weaker / tool-poor models
  (the Haiku unix→hybrid +37pp headline), (b) contexts without native file tools
  (plain shells, restricted runners, `--from` ergonomics), and (c) humans/scripts
  doing structural queries (`outline`, `tasks`, `table`, `--json` for `jq`).
- **Drop the implied "md beats native Edit on structure" claim** from any frontier
  pitch. The structural-entropy edge is unproven at the frontier and, where tested,
  absent.

## Caveats (scope of the claim)

- One model (`claude-sonnet-4-6`), N=3, high variance; three families. Not "md
  never helps any agent" — it is "md shows no robust edge vs native Edit at this
  frontier on these families." A weaker model, or a regime not tested (e.g. very
  large files >10k lines where even sliced reads dominate, or transactional
  multi-file edits), could differ — but the burden of proof is now on finding one.
- The native-adversary + the three generators are **standing assets**: re-runnable
  to re-test on a new model or family without re-deriving the harness.

## Follow-ups (not blocking)

- If chasing a residual edge: test **>10k-line** docs (where even native's sliced
  reads may dominate) and **transactional multi-file** edits (md's `--expect-etag`
  drift-safety vs native's no-rollback) — the two regimes this run did not reach.
- The leaner-md cost lever (N6) is **not** worth pursuing: there is no winning
  family to optimize cost on.

## The dedicated attack — the obvious levers are verified no-ops (2026-06-05)

After the loop falsified the *families*, a follow-up swarm attacked the three
closest losers directly — designing the single strongest targeted `md` fix for
each, then red-teaming it against the actual source + the N=3 transcripts. **All
three levers are no-ops.** This upgrades the result from "no winning family" to
"the obvious fixes provably cannot move the measured cost." Do not re-attempt
these — the engineering is wasted:

- **large-file — a `--lean outline` flag is a no-op.** The premise (verbose
  span-JSON balloons context) is false: every `md outline --json` call is already
  piped through `jq`/`head`, so the raw span JSON never enters context (run1's
  total `tokens_in`=206,883 can't even contain the claimed bloat). The real driver
  is **turns × jq-schema-rediscovery** (the agent cycling `.[]` → `.entries[].title`
  → `.entries[].heading.text` across round-trips) — tokens track outline-call-count
  exactly (3 calls/92.7k beats native; 7 calls/206.9k blows up). A `--lean` flag
  removes **zero** context tokens and cannot cut turn count; a second schema could
  *worsen* the thrash.
- **conditional-batch — a fused `set-tasks LOCS…` verb is a no-op.** The premise
  (N separate file-re-reading round-trips) is false: every run already issues
  **one** `for loc in …; do ./md set-task …; done` Bash call. The +25.8% is the
  `md tasks --json` enumeration blob (3.9× the file) native's single `Read` avoids,
  plus a redundant `Read` + one extra turn. A batch verb swaps one single-call loop
  for one single-call verb — **zero turns, zero meaningful tokens** removed; best
  case still ~+10–18%.
- **duplicate-heading — unflippable.** Native passes 3/3, so correctness-lift is
  impossible under the frozen gate; only a cost-lift remains, and `md` is the
  *wrong sign* (it adds a locate-probe + md-output re-read on a ~1.2 KB file). The
  proposed lever (`--occurrence`) **already exists and is already advertised** in
  the prompt — a literal no-op.

**The architectural through-line:** `md` adds discover→re-read→mutate **round-trips
it cannot amortize** at the capable-model frontier, while native `Edit` fuses
discovery and mutation on content it has **already `Read`**. This is the *shape* of
the tool, not a tuning gap — confirmed at the mechanism level, $0, with no new
sweeps. Cost: ~$2 of compute across the whole hunt + attack, to avoid building
three useless features.
