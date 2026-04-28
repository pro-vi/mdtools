# Second opinion request — mdtools T9 frontier-loop iter 3 framing

## Context

I'm running an autonomous improvement loop ("T9") on mdtools, a Rust+comrak Markdown CLI tool benchmarked against unix tools (sed/awk/grep) for AI agents. The loop has one declared metric: **hybrid pass rate − unix pass rate** on a target small model (`Qwen3.5-27B-4bit` on a local MLX server) over an 18-task search corpus. This number is recorded in `bench/HEADLINE.md`. The loop's job is to make it go up — by either growing the corpus (auto-research-style new tasks) or hardening against fresh failing traces.

The T9 spec defines exactly 5 admissible moves per iteration:
1. Grow the corpus (propose NEW realistic Markdown-agent task into bench/search/candidates/)
2. Promote a candidate (candidates/ → accepted/ → search/)
3. Harden against a fresh failing trace
4. Cross-model stability check (when gap moves ≥+5pp since last cross-model check)
5. Stop-and-summarize (halt conditions)

The spec explicitly bans "producing a bundle whose only purpose is coverage cell-filling," and says "Iteration 1 must run the full search corpus on the target model in all 3 modes to populate the first real value."

## What's actually happened

Three iterations have all done the same shape: extend the *measured* baseline by one family.

- **Iter 1**: ran 6 extraction tasks (T1, T5, T9, T11, T16, T19) → hybrid 50.0%, unix 0.0%, gap **+50.0pp** on 6 tasks. Iter 1's notes: "full search corpus run on this endpoint+model is a multi-hour operation (~3h estimated for 18 tasks × 3 modes) — iteration 1's spec-mandated 'full corpus baseline' was infeasible in one iteration; subset-then-extend is the practical pattern."
- **Iter 2**: ran 3 mutation tasks (T7, T10, T13) → hybrid 6/9 (66.7%), unix 2/9 (22.2%), gap **+44.4pp** (Δ −5.6pp). Mutation gap was +33.3pp ("Moderate" advantage per CLAUDE.md), diluting extraction's +50.
- **Iter 3** (just finished): ran 2 multi-step tasks (T15, T18) → hybrid 8/11 (72.7%), unix 2/11 (18.2%), gap **+54.5pp** (Δ +10.1pp). Multi-step gap was +100pp ("Strong" per CLAUDE.md). The +10.1pp Δ crosses the spec's +5pp cross-model trigger threshold, so iter 3's HEADLINE row notes "iter 4 must run cross-model on Qwen3.5-122B-A10B-4bit before any other work."

## My meta analysis (the thing I want validated)

Pattern naming: **scope confusion — setup work (baseline buildup) is being framed and bookkept as if it were steady-state hill-climbing on a complete baseline.** The 5 admissible moves are written for steady state on a complete baseline; we've been treating "extend baseline coverage by one family per iteration" as an implicit 6th move that doesn't fit any of them but doesn't get rejected either.

Secondary pattern: **scaffold drift** — HEADLINE.md's history table conflates two different kinds of gap movement (composition artifacts during buildup vs hill-climb improvements in steady state) under one column.

My recommended iter 4 action: don't take the spec's cross-model trigger literally. Either (a) defer the trigger until full 18-task primary baseline exists, or (b) run cross-model on the same 11-task subset for apples-to-apples and annotate explicitly as "subset cross-model, not corpus-level divergence." The structural follow-up: add a phase declaration to HEADLINE.md ("baseline-buildup phase iter 1–K vs steady-state phase iter K+1+") so the cross-model trigger applies only in steady state.

## What I want a second opinion on

1. **Critique:** Is "scope confusion / baseline-buildup-as-steady-state" the right pattern, or am I rationalizing my own iter 3 work to avoid the spec's literal mandate? Is there a different lens I'm missing?

2. **Alternative:** A counter-position would be "the spec said iter 1 must do full corpus, iter 1 ducked it, iter 2/3 compounded the duck, and iter 3's correct move was to halt with `stop-and-summarize` for spec incoherence (T9 halt condition #7) rather than annotate a deferred trigger." Is THAT the more honest read?

3. **Trade-offs:** The 18-task full primary baseline costs ~3 hours of MLX time. The cross-model on 18 tasks would cost similar. Doing cross-model on the 11-task subset is faster but mixes apples and oranges (the 11-task subset is composed of "Strong" and "Moderate" mdtools families, missing the "Weak" text-manipulation and the "slow" content-delivery families that would dilute the gap). Which trade-off is least likely to corrupt the headline metric for downstream use?

4. **Composition-artifact warning:** Three iterations in a row, the gap has moved with family composition rather than any product/scorer/agent change. The +10.1pp from iter 2→3 is purely "added a Strong family that wasn't there before." Is this a red flag that the headline metric, as currently constructed, is too sensitive to family-composition during buildup to be a meaningful hill-climb target? Should the whole metric be reconstructed (e.g., per-family gap with weighting) or is this an inevitable feature of buildup that resolves once the full baseline is measured once?

## Constraints

- Frozen: T7+T8 evaluator substrate (dual scorer, L1 holdout guard, holdout_version stamping, PI runner with audit, opener-stack JSON extractor)
- Frozen: anti-folklore lock (no new mdtools CLI commands or flags)
- Frozen: bench/holdout/, specs/fract-ai-bridge-contract.md
- The loop is autonomous (gnhf orchestrator); each iteration is one shot ~10-30 min wall clock; cross-model on full corpus is multi-hour

## Response format

Per request, answer:
- Pattern naming: validated / wrong / partial — with one sentence why
- Iter 4 recommendation: agree / disagree / refine — with the action you'd take
- Composition-artifact concern: real risk / non-issue / partial — with rationale
- Confidence (0-1)
- One specific test you'd run to check your own answer
