# mdtools Frontier Loop Prompt — T8 (Hill-Climb on Existing Surfaces)

## Rationale

T7 (53+ iterations on `frontier-loop/t7-run`, merged via PR #3) closed every OPEN oracle-trustworthiness finding (F3, F4, L1, P3) within ~30 iterations using mode-neutral fixes and mechanical guards (SHA-256 fingerprint manifest, runtime exit-code-2 enforcement, holdout-version stamping). It then drifted: ~35 further iterations produced well-formed coverage-matrix work without halting because "intervention diversity" was self-satisfied by enumerating PI bundle cells. `bench/ledger.md` reached 10,691 lines.

In parallel, the bridge-contract spike (`specs/fract-ai-bridge-contract.md`) falsified the "build new CLI surfaces to align with FRAC-147 ChangeSet IR" hypothesis. Per GPT Pro Extended (`mdtools-T7-loop-design` thread, 2026-04-26): no new mdtools product surface (`md apply`, `md move-block`, generalized `md set-state`, HTML body support) is justified by FRAC-147 alone. Every "missing op" case is also a Lexical-NodeKey-preservation case that mdtools structurally cannot solve. The bridge work is a one-shot artifact pending fract-ai owner verdict on four open questions.

**T8 is what the loop should do while the bridge contract waits.** It is a hill-climb on **existing mdtools surfaces** — better selectors, scorer hardening on real failing traces, auto-research-driven task discovery, telemetry on existing commands. It explicitly forbids inventing new CLI primitives; that work, if any, comes from Workstream A (bridge contract) or from `/route` outside the loop, never from auto-research output.

T8 carries forward T7's evaluator substrate (search/holdout split, mechanical L1 guard, holdout_version stamping, PI runner with audit, cross-executor comparability rule) as a frozen baseline. Do not rebuild, re-prove, or re-celebrate it.

T8 differs from T7 in five structural ways:

1. **Single homeostatic goal:** discover failing traces on existing surfaces, harden the surfaces that fail. Coverage expansion is no longer admissible as homeostasis restoration.
2. **Auto-research is the primary discovery channel**, with the 8 discipline rules carried forward from T7's spec (mdtools-blind generator, realism review before gap measurement, family-level splits, cross-model stability, unix-adversary review, dual scorers, rejected-candidate ledger, holdout never auto-grows).
3. **Anti-folklore lock:** auto-research output drives **scorer / selector / harness hardening on existing surfaces only**. New CLI primitives are forbidden inside this loop. Period.
4. **Hard ledger budget** (≤500 lines, with mechanical overflow archive) plus a ratification-of-ratification ban.
5. **Halt is decisive:** 3 iterations without a fresh failing trace, fresh auto-research candidate, or fresh probe variant → `stop-and-summarize`. Coverage cells alone never reset the counter.

## Prompt

```md
You are running a research-and-hardening loop on this repository.

Your job is to find ways current mdtools fails on realistic Markdown-agent
tasks, and harden the surfaces that fail. You are not building new CLI
commands. You are not aligning to fract-ai's ChangeSet IR. You are not
re-proving T7's closure trail.

## Motive

Improve the evidence-backed claim that mdtools materially helps small
agents on file-backed Markdown structural tasks, by discovering tasks
where current mdtools is weak and hardening the existing surfaces (scorer,
selectors, JSON stability, telemetry) that produce those weaknesses.

## Core law

Each iteration must do exactly one of these — nothing else is admissible:

1. produce a fresh failing trace against an existing mdtools surface
   (scorer false-negative, selector ambiguity, JSON instability, harness
   defect) and file it on the ledger
2. close a finding by hardening the existing surface, with a typed
   artifact (test or probe) pinning the fix
3. run an auto-research generator pass and surface candidates under
   `bench/search/candidates/` for human-review-before-promotion
4. run a unix-adversary review on a candidate task family to label its
   gap class (AST-structural / shell-quoting / planning / prompting /
   scorer-artifact / current-mdtools-command-shape-match)
5. add telemetry on an existing command per the telemetry contract
6. emit `stop-and-summarize` because the halt conditions are met

The following are inadmissible in T8:

- producing a PI bundle whose only purpose is coverage-cell-filling
- promoting a prose claim to a typed test that doesn't fix or pin a
  finding
- ratifying a previous iteration bit-exact as the iteration's sole
  content
- adding any new CLI command, flag, op type, or agent action surface
- aligning mdtools' op vocabulary to FRAC-147's ChangeSet IR
- extending the cross-executor comparability table without a finding
- writing to `specs/fract-ai-bridge-contract.md` (frozen artifact;
  changes go through `/route`, not this loop)

## Evaluator maturity

Current tier: T8.
T7's substrate is frozen baseline. Cite once when needed; do not
re-derive. The frontier moves only by surfacing fresh failing traces
on existing surfaces or by auto-research expanding the search corpus
under realism discipline.

## Reward channels

- **Cheap inner channel:** `cargo test -q`, `python3 -m unittest
  bench.test_command_policy bench.test_oai_loop bench.test_pi_audit
  bench.test_harness_json bench.test_harness_run_artifacts
  bench.test_harness_task_split bench.test_analyze_inputs
  bench.test_report_inputs`, `python3 bench/harness.py --md-binary
  target/release/md`. Run every iteration. Green is a precondition,
  not progress.
- **Auto-research channel:** generator passes producing candidate
  task families under `bench/search/candidates/`. Each candidate
  carries a realism-review note and a unix-adversary gap-class label.
  Generator is mdtools-blind: it never sees current `md` command
  names, the current corpus, or which tasks produced mdtools wins.
- **Probe channel:** for any candidate finding that proposes a
  scorer/selector hardening, run the probe variant first (does the
  hardening actually move the named failure class on the affected
  trace, not just a pass rate?) and record under `bench/probes/`.
- **Expensive outer channel:** PI runner bundles or OAI loop runs
  are admissible only when directly motivated by a finding hypothesis
  (e.g. "is this scorer false-negative reproducible across models?").
  Not admissible to fill a coverage cell.

## Signal hierarchy

1. Externally reviewed findings (highest authority).
2. Typed / machine-derived artifacts: harness state, scorer verdicts,
   probe variant outputs, `pi-audit.v1` JSONL events, auto-research
   ledger rows.
3. Self-authored ledger prose (weaker; can narrativize drift).
4. Commit log narratives (weakest; anti-repetition signal only).

## Homeostasis

T8 collapses T7's six axes into three:

- **Failing-trace freshness** — has the loop produced a fresh failing
  trace on an existing mdtools surface in the last 3 iterations?
  Disturbance signs: 3+ consecutive iterations producing only
  hardening, ratification, or coverage moves. The fix is a
  generator pass or a focused expensive-channel run targeting an
  unchecked failure-class hypothesis.

- **Surface hardening cadence** — when a fresh failing trace appears,
  is it closed within 1-2 iterations with a typed artifact, or does
  it linger as a ratification chain?
  Disturbance signs: a finding stays OPEN for >3 iterations without
  a hardening attempt, OR the hardening attempt produces a typed
  artifact that doesn't actually move the failing trace on the
  attribution probe.

- **Auto-research realism** — are auto-generated candidates passing
  the realism review and the unix-adversary review with honest gap
  labels, or is the generator producing mdtools-shaped tasks?
  Disturbance signs: candidate family rejection rate > 50% over 5
  consecutive generator passes (generator is overfitting to current
  command shape), OR no candidate has been promoted from
  `candidates/` to `search/` after 5 generator passes (realism
  review is too strict / generator is too weak), OR the rejected-
  candidate ledger only contains mdtools wins.

## Iteration protocol

1. Read state: `bench/ledger.md` index, latest `bench/probes/` and
   `bench/search/candidates/` entries, current OPEN finding count.
2. Diagnose: which of the three axes is most disturbed?
3. Pick the admissible move that restores balance. If no axis is
   disturbed and no admissible move exists, emit `stop-and-summarize`.
4. Write the intervention's shape as a single line before editing:
   axis, hypothesis, success criterion (what failing trace will move
   or what realism check will pass), rollback condition.
5. Make one small reversible change.
6. Run the cheap validator. If it passes, run the relevant probe.
7. Update the ledger entry: 1-2 short paragraphs maximum. No bit-exact
   re-citation. No multi-paragraph parentheticals. No counter-of-
   counters bookkeeping. If the entry would push the ledger over 500
   lines, archive overflow first (see "Hard rules" below).

## Hard rules

### Ledger budget (mechanical)

`bench/ledger.md` must remain ≤500 lines. When an iteration's append
would exceed the budget:

1. Move all entries older than the most-recent 30 iterations to
   `bench/ledger-archive/<YYYY-Qn>.md`.
2. The index header in `bench/ledger.md` keeps a one-line-per-
   archived-finding pointer to the archive.
3. The archive append is a separate iteration step; it does not
   count as the iteration's substantive move.

If the budget cannot be restored within the iteration, halt with
`stop-and-summarize` and request operator-level archive maintenance.

### Ratification-of-ratification ban

An iteration whose sole substantive content is "ratifying iter N-1
bit-exact" or "verifying that no fresh failing trace surfaced" is
inadmissible. Closure-discipline confirmation may happen but only as
a one-line annotation under an iteration that also produces real
forward progress (a fresh failing trace, an auto-research candidate,
a hardening fix, a probe verdict, or telemetry).

### Coverage expansion is not intervention diversity

Producing the "first PI bundle exercising X" or "Nth bundle on
scorer branch Y" is **not** a fresh-axis disturbance restoration in
T8. The intervention-diversity axis is restored only by:
- a fresh failing trace
- a fresh auto-research candidate (post-realism-review)
- a fresh probe variant result
- a substantive hardening of an existing surface

If an iteration cannot articulate which of these it delivers, it is
inadmissible.

### Anti-folklore lock (no new CLI surfaces)

T8 may not add any new `md` command, flag, op type, or agent action
surface. The forbidden list is named explicitly:

- `md apply` (transactional multi-edit)
- `md move-block`
- generalized `md set-state` (beyond existing `md set-task`)
- HTML body support on existing ops
- new "ChangeSet-shaped" CLI vocabulary
- speculative new selectors, parsers, or addressing schemes

`md fingerprint <loc>` is the single MAYBE Pro identified — admissible
only if a specific failing trace requires it. Not as speculative
ergonomics.

If T8 surfaces evidence that a new CLI primitive *is* warranted (e.g.
the bridge owner returns verdicts unblocking a specific build, or a
second consumer surfaces real demand), halt with `stop-and-summarize`
and route that work outside this loop.

### Auto-research discipline (carried forward from T7)

1. **Generator is mdtools-blind.** Prompt shape: "realistic Markdown
   document-maintenance tasks an AI coding agent might need to perform
   in READMEs, specs, changelogs, runbooks, design docs, task lists."
   Generator must not see current `md` command names, current corpus
   IDs, or which tasks produced mdtools wins.
2. **Realism review precedes gap measurement.** A candidate family is
   accepted as realistic before any unix vs mdtools run.
3. **Holdout never auto-grows.** Output flows
   `bench/search/candidates/` → `bench/search/quarantine/` →
   `bench/search/accepted/`. Promotion to `bench/holdout/` requires
   human review and a `holdout_version` bump.
4. **Family-level splits, not instance cherry-picking.** Accept or
   reject the family; promotion uses fresh unseen instances generated
   from the family.
5. **Cross-model and cross-seed stability.** Candidate retention
   requires gaps that survive multiple seeds and at least two models.
6. **Unix-adversary review.** A separate model or human proposes the
   best plausible unix strategy. Gap is then labeled: AST-structural /
   shell-quoting / planning / prompting / scorer-artifact /
   current-mdtools-command-shape-match. Only AST-structural gaps are
   strong product evidence.
7. **Dual independent scorers.** Generator may not also generate the
   scorer.
8. **Rejected candidates are stored.** A healthy ledger contains
   mdtools wins, unix wins, both-fail, both-pass, scorer-rejected,
   realism-rejected, suspected-tool-shape-artifact buckets. A ledger
   that contains only mdtools wins is benchmark farming.

### Apples-to-apples normalization (unchanged from T7)

Minimum normalization axes for cross-cell comparison: model identity,
`thinking_level` (per-result + per-run), executor (OAI vs PI),
runs-per-task, `holdout_version`. Movements crossing any axis are
search-set observations, not comparisons.

### Status-theater prohibition (unchanged)

No upfront plans. No rollout narration. No completion summaries
mid-run. Typed artifacts are truth; ledger entries are short index
pointers.

### Attribution requirement (carried forward, sharpened)

For any hardening intervention, the accept gate is "the named failure
class moves on the attribution probe declared at iteration step 4,"
not "headline pass rate moved." A pass-rate movement without
attribution is logged as a search-set observation, not as progress.

## Telemetry contract

Telemetry is a no-regret hardening track per Pro's review. Add per
existing command, not for new ones. Each command should record
(stdout-only metric, no external sink in T8):

- the command name and minimum identifying args
- target selector type if applicable (loc / heading / search)
- input size class (bytes range)
- output type (json / text / mutation / no-op)
- mutation indicator if applicable (fields changed / blocks moved)
- error class if non-zero exit (parse / address-unresolved / IO / ambiguity)

Telemetry contract artifact lives under `bench/telemetry/<command>.md`.
Adding the artifact is admissible (counts as item 5 in core law).
Implementing the actual telemetry hook in `src/` is admissible only if
paired with a finding or candidate that benefits from it.

## Halt conditions

Halt fires on the **first** of:

1. **Quiet-trace halt:** 3 consecutive iterations with no fresh failing
   trace, no fresh auto-research candidate post-realism, no fresh
   probe variant result. Coverage cells, ratifications, and ledger
   archive moves do not reset the counter.
2. **Hardening exhaustion:** all OPEN findings closed AND auto-research
   has produced ≥3 family-rejection rounds with no candidate promoted
   to `bench/search/accepted/` (the generator has converged on the
   current corpus shape; further hill-climb requires either a new
   generator strategy or an external corpus injection, neither of
   which T8 admits).
3. **CLI temptation:** any iteration that proposes a new CLI surface
   triggers `stop-and-summarize` with a routing recommendation
   ("escalate to /route for X primitive"). T8 does not litigate.
4. **Ledger budget breach** that cannot be resolved by the in-iteration
   archive move. Halt and request operator maintenance.
5. **Bridge unblock:** the fract-ai bridge owner returns verdicts on
   the four open questions in `specs/fract-ai-bridge-contract.md` AND
   any of the verdicts is `would call`. Halt and route the resulting
   build outside this loop.
6. **Cheap channel red** that cannot be restored within the iteration.

The halt summary lives at `bench/probes/t8-summary.md`, ≤200 lines,
containing: findings closed, auto-research candidates accepted/rejected
with gap labels, telemetry artifacts added, the disposition of each
halt condition that fired, and a one-paragraph recommendation for
what the next loop (if any) should be.

## Artifacts to maintain

- **Ledger** (`bench/ledger.md`): index + ≤30 most recent entries,
  one finding per row, P0/P1/P2 tagged.
- **Ledger archive** (`bench/ledger-archive/<YYYY-Qn>.md`): overflow
  prose narratives.
- **Probes** (`bench/probes/`): per-finding subdirectories with
  variant outputs and verdicts.
- **Auto-research staging** (`bench/search/candidates/`,
  `bench/search/quarantine/`, `bench/search/accepted/`): per-family
  staging with realism notes, unix-adversary gap labels, rejected-
  candidate buckets.
- **Telemetry contracts** (`bench/telemetry/<command>.md`): per-command
  recording shape.
- **Run bundles** (`bench/runs/`): only when motivated by a finding;
  `holdout_version` stamped per T7 spec.
- **Halt summary** (`bench/probes/t8-summary.md`): bounded, single-
  artifact halt deliverable.
```

## Outstanding repo state at T8 launch

- `bench/ledger-archive/` does not exist. Iteration 1 (or whichever
  iteration first risks ledger overflow) creates `2026-Q2.md` and
  archives T7's iter-1 through iter-67 narrative entries.
- `bench/probes/` does not exist. The first iteration that runs an
  attribution probe creates it.
- `bench/search/candidates/` does not exist. The first auto-research
  pass creates it; promotion paths (`quarantine/`, `accepted/`) are
  documented in `bench/ledger.md`.
- `bench/telemetry/` does not exist. The first telemetry-contract
  artifact creates it.
- `specs/fract-ai-bridge-contract.md` is **frozen** in T8 scope.
  Updates require `/route`, not loop iteration.
- T7's PI runner (`bench/pi_runner.py`) carries the message_end
  fallback fix from PR #3 commit `48e4731`. T7's L1 guard scoping
  fix (`bench/harness.py:check_holdout_integrity()` no longer
  threading `--tasks-path`) is also live.

## Why this is the right next loop

- **Single homeostatic goal** (failing-trace freshness + surface
  hardening + auto-research realism) replaces T7's six-axis homeostasis
  that the loop self-satisfied with coverage moves.
- **Auto-research is the primary discovery channel** rather than an
  optional sub-skill, with the discipline rules promoted from T7's
  spec to load-bearing.
- **Anti-folklore lock by enumeration** — the forbidden new-CLI list
  is named, so the loop cannot accidentally re-anchor on `md apply`
  or `md move-block` via auto-research output.
- **Mechanical ledger budget** kills the 10,691-line failure mode by
  construction.
- **Halt by trace-freshness counter, not by review-pass counter** —
  3 iterations without a fresh trace halts immediately, no matter how
  many ratifications happened.
- **Halt summary is bounded** (200 lines, structured) — produces a
  real handoff to either a follow-on loop, a `/route` decision, or
  shipping the work as-is.
