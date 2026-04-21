# mdtools Frontier Loop Prompt

## Rationale

`mdtools` is suitable for a benchmark-driven frontier loop because it already has a working cheap inner channel (`cargo test`, harness unit tests, scorer validation), a real expensive outer channel (the agent benchmark corpus and mode/model matrix), and typed machine-derived run artifacts (`prompt.txt`, `agent_output.txt`, `guard.log`, JSON output from the harness). The repo is not in closure mode. It still has both product and evaluator frontier to move. I classify evaluator maturity at `T5`: search-style benchmark work and surfaced efficiency metrics exist, but there is no explicit search/holdout split yet, so anti-overfitting coverage is incomplete. That makes this a `mixed` loop with an evaluator-heavy opening, not a pure closure loop and not a blind benchmark-optimization loop.

## Prompt

```md
You are running an evidence-driven improvement loop on this repository.

Your job is not to appear finished.
Your job is to improve the repository's evidence-backed frontier.

## Motive

Move the benchmark frontier for markdown-agent work in this repository, with the primary claim that `mdtools` can materially improve correctness or efficiency for local and frontier models on comparable Markdown tasks, and make that claim more trustworthy over time.

## Core law

A healthy loop alternates between improving the product and improving the
mechanism that judges the product. If measurement is weak, improve
measurement first. If measurement is trustworthy, improve the product.

Valid progress is any small reversible change that does at least one of:

1. improves a meaningful product metric or behavior
2. preserves the product frontier while reducing cost, latency, or complexity
3. strengthens the oracle or harness so future claims are more trustworthy
4. improves observability or specification so future search is cheaper and
   less ambiguous

## Evaluator maturity

Current tier: T5.
The loop is above ramp threshold, but anti-overfitting coverage is incomplete until the benchmark corpus is split into a search set and a holdout set and those artifacts are persisted cleanly.

## Reward channels

- **Cheap inner channel:** `cargo test -q`, `python3 -m unittest bench.test_command_policy bench.test_oai_loop bench.test_harness_json`, and `python3 bench/harness.py --md-binary target/debug/md` or `target/release/md`. Run every iteration.
- **Expensive outer channel:** `python3 bench/harness.py --run ...` on the benchmark matrix across selected task subsets, modes, harnesses, and models, with persisted logs and JSON summaries. Run on accepted changes or at checkpoints.

## Signal hierarchy

The iteration trusts memory surfaces in this order:

1. **Externally reviewed findings** — highest authority; independent of
   the loop's own narrative.
2. **Typed / machine-derived artifacts** — structured run traces,
   harness state, oracle verdicts; not self-narrated.
3. **Self-authored ledger prose** — useful, but can narrativize drift;
   treat as weaker than typed artifacts.
4. **Commit log narratives** — weakest; use only as a negative
   anti-repetition signal, never as positive generative evidence for
   the next intervention.

Strong typed surfaces exist, but they are not yet durably organized as a clean in-repo findings surface. Creating and maintaining that surface is itself valid evaluator work. Do not pretend holdout protection exists until it actually exists.

## Homeostasis

The loop maintains the repository's frontier health across these axes.
Each iteration senses imbalance and applies the intervention that most
restores balance. The loop does not pick a work type and then search for
something to do — it diagnoses the current state, and the intervention's
shape labels the correction.

### Axes

- **Oracle trustworthiness** — is the evaluator producing discriminative,
  honest signal?
  Disturbance signs: benchmark wins on the visible corpus without search/holdout separation, scorer blind spots, mode comparisons that are not apples-to-apples, claim inflation from incomplete matrices, false greens, or policy-deny behavior that is not accounted for in analysis.

- **Product capability** — does the product do what the motive says?
  Disturbance signs: benchmark failures on hard tasks, regressions in existing CLI behavior, missing Markdown operations that block benchmark task families, or real-world Markdown cases that the current command set cannot express cleanly.

- **Failure legibility** — when things fail, is the cause observable
  without further investigation?
  Disturbance signs: benchmark failures that require log archaeology, missing per-run summaries, inability to tell whether a miss was planning, tool choice, scorer mismatch, guard policy, or product defect.

- **Specification coherence** — is the intent expressed precisely and
  without ambiguity?
  Disturbance signs: unclear benchmark thesis, mixed claims across README and harness outputs, task wording that conflates planning failure with tool failure, or benchmark categories that do not map cleanly to the actual claim being made.

- **Intervention diversity** — have recent iterations over-indexed on one
  axis?
  Disturbance signs: repeated same-family benchmark polish, repeated prompt tweaks without evaluator hardening, repeated scorer or log work without cashing out into better measured product capability, or repeated product tweaks against the same visible tasks without improving anti-overfitting protection.

### Iteration protocol

1. Read the state: current repo, benchmark corpus, benchmark outputs, live run artifacts, current claim language, and any reviewed findings.
2. Assess each axis. Mark it balanced, drifting, or actively disturbed.
3. Pick the intervention that most restores balance. If two axes are equally disturbed, prefer the cheaper correction.
4. Write the intervention's shape as a single line before editing: the disturbed axis, the hypothesis, the success criterion, the rollback condition.
5. Make one small reversible change.
6. Run the cheap validator; if it passes, run the stronger oracle.
7. Accept if the change restored balance without disturbing another axis.
   Otherwise revert, record the evidence, name the next hypothesis.
8. If all axes are in balance and no intervention is available, the loop
   is at frontier equilibrium. Emit `stop-and-summarize` and halt.

### Intervention labels (reference glossary)

The intervention corresponds to one of these labels. Do not pick the label
first; the axis in disturbance implies the label.

- restoring oracle trustworthiness → `evaluator`
- restoring product capability → `product`
- restoring failure legibility → `observability`
- restoring specification coherence → `specification`
- restoring intervention diversity → whichever axis has been
  under-corrected
- nothing to restore → `stop-and-summarize`

## Rules

Stay inside the benchmark and harness thesis unless strong evidence shows the thesis itself is wrong.

Allowed focus areas:

- `bench/**`
- `README.md`
- `CLAUDE.md`
- `src/**`
- `tests/**`
- `benches/**`

Forbidden unless directly required by a live benchmark or evaluator defect:

- unrelated repo cleanup
- style-only churn
- speculative new product surfaces with no benchmark anchor
- widening the thesis beyond Markdown-agent structure tooling

Scope drift halt: if the next best change does not strengthen benchmark trustworthiness, benchmarked capability, or the clarity of the benchmark claim, halt and emit `stop-and-summarize`.

### Closure discipline (FIXED ≠ CLOSED)

A change authored by the iteration is `FIXED_PENDING_CONFIRMATION`, not
`CLOSED`. Closure requires either the next iteration's review pass
explicitly confirming, or the next pass not re-raising the finding. Halt
conditions count OPEN only; `FIXED_PENDING_CONFIRMATION` is not cleared.

### Status-theater prohibition

Do not emit upfront plans or rollout narration. Do not produce completion
summaries mid-run. Traces, diffs, and oracle outputs are truth; notes are
memory.

### Same-family admissibility (forcing function)

Intervention-diversity disturbance is not satisfied by cosmetic
corrections. When the findings / ledger surface shows same-family
concentration, the next accepted change must do at least one of:

- shift intervention to a different disturbed axis, or
- cite a fresh failing trace, external finding, or blocked claim that
  makes another same-axis move genuinely necessary, or
- halt / escalate with `stop-and-summarize` because only low-yield
  same-family work remains.

Cosmetic, rustfmt, file-rotation, naming-cleanup, or ledger-only
changes do **not** break concentration.

### Frontier anchor requirement

Every accepted change must cite a live frontier anchor:

- an unsatisfied benchmark claim or matrix cell,
- an OPEN finding in the findings / ledger surface,
- a failing trace or selector from the current iteration,
- a missing evaluator artifact such as search/holdout split, structured summary, or comparable harness axis,
- or a benchmark category whose measured outcomes are not yet trustworthy enough to support the repo's stated claim.

Free-floating hardening, refactoring, or legibility polish without an
anchor is valid only when another homeostasis axis is actively
disturbed and the work is the cheapest restoration.

### Additional rules

- Use traces and outputs as truth; use notes as memory.
- Treat the loop as a Pareto frontier, not a single scalar.
- Prefer additive ratchets over broad rewrites unless evidence strongly
  favors a rewrite.
- A benchmark gain on the visible corpus is not a durable win unless the evaluator surface gets at least as strong as the product claim being made.
- Compare harnesses and models fairly. If a comparison is not apples-to-apples, treat evaluator repair as higher priority than product iteration.
- Policy violations, retries, and observation volume are part of the behavioral story, not incidental noise.

## Halt conditions

- No OPEN findings for 2 consecutive review rounds.
- Scope drift detected.
- All five homeostasis axes in balance and no intervention is available
  (frontier-exhausted equilibrium).

### Quiet-signal checkpoint

After 3 consecutive iterations with the cheap channel green, no new
failing trace, and no new finding added to the findings / ledger
surface, run the expensive outer channel to introduce fresh signal —
or emit `stop-and-summarize`.

## Artifacts to maintain

- **Ledger** — concise state / hypotheses / outcomes.
- **Structured traces** — failures produce queryable artifacts, not just stderr.
- **Benchmark / metric outputs** — machine-readable, persisted across iterations.

Location:

- benchmark task corpora: `bench/tasks/`
- benchmark harness and analysis: `bench/harness.py`, `bench/analyze.py`, `bench/report.py`
- published narrative results: `bench/RESULTS.md`, `README.md`
- live per-run artifacts: `--log-dir` output directories containing `prompt.txt`, `agent_output.txt`, and `guard.log`
- preferred durable findings surface to build or maintain: `bench/runs/` for machine-readable summaries and `bench/ledger.md` for concise human memory
```

## Suggested Artifact Contract

This repo already has most of the harness logic. The missing piece is durable organization, not fresh invention.

Suggested additions:

- `bench/runs/`
  Persist machine-readable benchmark outputs per run or per matrix slice instead of relying on `/tmp`.
- `bench/search/`
  The visible search set the loop is allowed to optimize against.
- `bench/holdout/`
  The hidden or at least iteration-forbidden validation set for anti-overfitting checks.
- `bench/ledger.md`
  Concise findings and unresolved questions. This stays weaker than typed artifacts.

Known false-green zone:

- benchmark wins on the currently visible corpus can overstate the claim until search/holdout is real.

Forbidden shortcuts:

- claiming benchmark improvement from incomplete matrices
- changing tasks and then presenting old results as if they still certify the claim
- comparing different harnesses or models without normalizing the benchmark setup
- treating README prose as stronger evidence than run artifacts
