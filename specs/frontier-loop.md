# mdtools Frontier Loop Prompt

## Rationale

`mdtools` is suitable for a benchmark-driven frontier loop because it already has a working cheap inner channel (`cargo test`, harness unit tests, scorer validation), a real expensive outer channel (the benchmark corpus, search pilots, and mode/model matrix), an explicit search/holdout split, and durable typed run bundles under `bench/runs/`. The repo is not in closure mode. It still has both product and evaluator frontier to move. I classify evaluator maturity at `T6`: search-style benchmark work, surfaced efficiency / failure metrics, and holdout protection now exist. The main remaining risk is not missing evaluator substrate, but failing to use the holdout as a real acceptance gate. That makes this a `mixed` loop with a cash-out bias toward trustworthy benchmark and product moves, not an evaluator-bootstrap loop.

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

Current tier: T6.
The loop is above ramp threshold. Search/holdout and durable artifact organization now exist. The main anti-overfitting risk is procedural: accepting search-set wins without actually confirming them on holdout.

## Reward channels

- **Cheap inner channel:** `cargo test -q`, `python3 -m unittest bench.test_command_policy bench.test_oai_loop bench.test_harness_json bench.test_harness_run_artifacts bench.test_harness_task_split bench.test_analyze_inputs bench.test_report_inputs`, and `python3 bench/harness.py --md-binary target/release/md`. Run every iteration.
- **Expensive outer channel:** `python3 bench/harness.py --run ...` on search manifests under `bench/search/` with persisted `bench/runs/` bundles, followed by `bench/holdout/task_ids.json` confirmation after any accepted search-set gain. Run on accepted changes or at checkpoints.

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

Strong typed surfaces exist in-repo under `bench/runs/`, and external review findings can be layered on top of them. Anti-collapse can key on those typed surfaces directly. `bench/ledger.md` does not exist yet; if a concise human-memory surface becomes useful, treat it as auxiliary memory rather than as stronger evidence than the run bundles.

## Homeostasis

The loop maintains the repository's frontier health across these axes.
Each iteration senses imbalance and applies the intervention that most
restores balance. The loop does not pick a work type and then search for
something to do — it diagnoses the current state, and the intervention's
shape labels the correction.

### Axes

- **Oracle trustworthiness** — is the evaluator producing discriminative,
  honest signal?
  Disturbance signs: search-set wins that are not reconfirmed on holdout, scorer blind spots, mode/model comparisons that are not apples-to-apples, claim inflation from pilot manifests being narrated as full-corpus wins, false greens, or policy-deny / parser-failure behavior that is not accounted for in analysis.

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
  Disturbance signs: repeated matrix-filling or metric polish without cashing out into holdout-confirmed claims, repeated scorer or log work without a fresh blocked benchmark move, or repeated product tweaks against the same search tasks without widening or validating the claim.

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
- `specs/**`
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
- a missing evaluator artifact such as a holdout confirmation bundle, a comparable harness axis, or a durable summary for a newly-run comparison,
- or a benchmark category whose measured outcomes are not yet trustworthy enough to support the repo's stated claim.

Free-floating hardening, refactoring, or legibility polish without an
anchor is valid only when another homeostasis axis is actively
disturbed and the work is the cheapest restoration.

### Additional rules

- Use traces and outputs as truth; use notes as memory.
- Treat the loop as a Pareto frontier, not a single scalar.
- Prefer additive ratchets over broad rewrites unless evidence strongly
  favors a rewrite.
- A search-set gain is not a durable win unless the holdout stays green, or the loop explicitly records why holdout could not yet be run.
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

- benchmark task corpora: `bench/tasks/`, `bench/search/`, `bench/holdout/`
- benchmark harness and analysis: `bench/harness.py`, `bench/analyze.py`, `bench/report.py`
- published narrative results: `bench/RESULTS.md`, `README.md`
- durable per-run bundles: `bench/runs/` containing `run.json`, `results.json`, and `task_ids.json`
- local debug residue: `bench/runs/**/logs/` containing `prompt.txt`, `agent_output.txt`, and `guard.log` when enabled; do not treat these logs as commit-worthy evidence by default
- optional human-memory surface: `bench/ledger.md` if repeated loops need concise unresolved-findings prose
```

## Current Artifact Contract

This repo now has the core artifact contract in place. The missing pieces are operational discipline and, optionally, a lightweight human-memory layer.

Current structure:

- `bench/runs/`
  Machine-readable benchmark outputs per run or matrix slice.
- `bench/search/`
  The visible search set the loop is allowed to optimize against, including pilot manifests.
- `bench/holdout/`
  The iteration-forbidden validation set for anti-overfitting checks.
- `bench/ledger.md`
  Optional concise findings and unresolved questions. This stays weaker than typed artifacts and does not yet exist.

Known false-green zone:

- benchmark wins on the search split can still overstate the claim if the holdout is declared but not actually run after accepted search-set changes.

Forbidden shortcuts:

- claiming benchmark improvement from incomplete matrices
- changing tasks and then presenting old results as if they still certify the claim
- treating a declared holdout split as real anti-overfitting protection without actually exercising it
- comparing different harnesses or models without normalizing the benchmark setup
- treating README prose as stronger evidence than run artifacts
