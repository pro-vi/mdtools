# mdtools Frontier Loop Prompt — T10 (Phased Headline-Metric Hill-Climb)

## Rationale

T9 launched a single-metric hill-climb on the gap (hybrid pass rate − unix
pass rate). It ran 3 iterations and self-flagged a structural defect via
its own `/meta` lens at end of iter 3, escalated to paired GPT Pro
consultations: **denominator drift / unratified phase drift**. The loop's
5 admissible moves were written for steady-state on a complete baseline,
but iter 1 substituted a 6-task subset (full corpus was multi-hour and
infeasible in one iteration). Iter 2/3 extended the subset by one family
each. The iter 2→3 +10.1pp gap movement was not a hill-climb signal — it
was a composition delta from adding the multi-step family (which has a
+100pp per-family gap) to the denominator. The cross-model trigger fired
on a denominator change, not a real improvement.

Both Pro lanes (0.84 confidence, independent agreement) recommended
halting iter 4, repairing the spec, and resuming with explicit phases
before any cross-model interpretation. T10 is that repair.

T10 differs from T9 in four structural ways:

1. **Two declared phases.** `baseline-buildup` (until full primary
   18-task baseline measured once on target model) vs `steady-state`
   (after). Admissible moves and trigger semantics differ between phases.
2. **Two declared gaps.** `fixed-anchor gap` = hybrid − unix on the
   original 18-task corpus (the legitimate hill-climb signal).
   `current-corpus gap` = hybrid − unix on the currently-grown corpus
   (descriptive only, can move via composition). HEADLINE.md tracks both.
3. **Cross-model trigger is phase-gated.** Fires only in steady-state,
   on `fixed-anchor gap` movement. During buildup, gap deltas are
   composition deltas, not improvement deltas, and trigger nothing.
4. **Iter 1 is bounded.** T10 iter 1 = "complete the missing 7 primary-
   baseline tasks (T2, T3, T6, T8, T12, T17, T21) on Qwen3.5-27B-4bit
   in hybrid + unix modes." No other moves admissible until baseline
   exists.

T7+T8 substrate (dual scorer with F8 fixes, L1 holdout guard,
holdout_version stamping, PI runner with audit, opener-stack JSON
extractor, cross-executor comparability rule) carries forward as frozen
baseline. T9's auto-research discipline rules carry forward unchanged
and become active in steady-state.

## Prompt

```md
You are running a phased hill-climb loop on this repository.

Your job is to make ONE number go up over time, but only after the
denominator is fixed:

- **fixed-anchor gap** = hybrid mode pass rate − unix mode pass rate,
  on the original 18-task search corpus, on the target small model.
  This is the legitimate hill-climb signal — it can only move via
  product/scorer/agent change, never via composition.

The loop has two phases:

- **baseline-buildup:** the fixed-anchor gap does not yet exist
  because not all 18 tasks have been measured. The only admissible
  move is to extend coverage of the primary baseline. No cross-model
  triggers, no auto-research, no hardening interpretation tied to
  headline movement.
- **steady-state:** the fixed-anchor gap exists. T9's full set of
  moves becomes admissible. The current-corpus gap may also be tracked
  as a descriptive companion when corpus grows beyond 18.

Phase is declared in `bench/HEADLINE.md` and gates trigger semantics.

## Motive

Build a defensible, growing evidence base for the headline claim
("agents perform measurably better on Markdown editing with mdtools
than without"). The fixed-anchor gap is the metric; corpus growth is
the moat. A fixed-anchor +20pp on 18 tasks plus +20pp on 100 grown
tasks is stronger evidence than either alone.

## Core law

### Phase 1: baseline-buildup

Active until all 18 primary-baseline tasks (`bench/tasks/tasks.json`
minus `bench/holdout/task_ids.json`) have been measured on the target
model in hybrid + unix modes. Admissible moves:

1. **Extend baseline coverage:** run one or more of the missing
   primary-baseline tasks in hybrid + unix on the target model, dual
   scorer, holdout_version stamped. Append the bundle to HEADLINE.md
   under the buildup-phase row format.
2. **Stop-and-summarize** if halt conditions fire.

Inadmissible during buildup:

- Auto-research (no candidates, no realism review, no unix-adversary
  review). Defer to steady-state.
- Cross-model checks. Defer to steady-state.
- Hardening tied to "the gap moved" — only attribution-probe-grounded
  hardening is admissible, and only if the iteration also extends
  baseline coverage.
- Treating any composition-driven HEADLINE delta as a hill-climb
  signal. All buildup-phase Δ columns are labeled `composition` and
  do not count as movement.

T10 launches into buildup phase with 11/18 tasks measured. The
remaining 7 are: **T2, T3, T6, T8, T12, T17, T21**. When all 18 are
measured, transition to steady-state by stamping the fixed-anchor
baseline row and flipping `phase: steady-state` in HEADLINE.md.

### Phase 2: steady-state

Activates once the fixed-anchor 18-task baseline exists. Each
iteration must do exactly one of these substantive moves:

1. **Grow the corpus:** propose a new realistic Markdown-agent task,
   verify realism (LLM judge or heuristic), run all 3 modes against
   the target model, label gap class via unix-adversary review, and if
   the family qualifies, file under `bench/search/candidates/`.
2. **Promote a candidate:** move a candidate from
   `bench/search/candidates/` → `bench/search/accepted/` →
   `bench/search/` proper after dual scorer agreement and family-level
   stability across multiple seeds.
3. **Harden against a fresh failing trace:** if a candidate or existing
   task surfaces a real scorer/selector/JSON-stability defect (T8-style
   F-series finding), close it with a typed artifact and an attribution
   probe.
4. **Cross-model stability check:** when the **fixed-anchor gap** has
   moved ≥+5pp since the last check, run the full 18-task fixed-anchor
   corpus on the cross-model target and record both numbers. If
   divergence > 10pp, file a finding. Current-corpus gap movement does
   NOT trigger this — only fixed-anchor.
5. Emit `stop-and-summarize` because the halt conditions are met.

**Required tail action on items 1–4:** if the iteration moved a gap
or grew the corpus, append a row to `bench/HEADLINE.md`'s history
table with the bundle pointer AND a `cause` label
(`product` / `scorer` / `agent` / `composition` / `corpus-growth`).
HEADLINE.md update is never a standalone iteration; it is the
bookkeeping that closes a real move.

The following are inadmissible in T10 (both phases):

- producing a bundle whose only purpose is coverage cell-filling
  (does NOT apply to baseline-buildup; extending the missing 7 is
  legitimate completion of the spec-mandated baseline)
- promoting a prose claim to a typed test that doesn't fix or pin a
  finding the gap surfaced
- ratifying a previous iteration bit-exact as the iteration's sole
  content
- adding any new CLI command, flag, op type, or agent action surface
  (the T8 anti-folklore lock carries forward)
- aligning mdtools' op vocabulary to FRAC-147's ChangeSet IR
- writing to `specs/fract-ai-bridge-contract.md` (frozen artifact)
- modifying `bench/holdout/` or any holdout fingerprints (L1 guard)
- treating a current-corpus gap delta as a hill-climb signal
  (current-corpus is descriptive only; only fixed-anchor moves count)

## Evaluator maturity

Current tier: T10.
T7+T8's substrate is frozen baseline. T9's headline-as-single-metric
framing is preserved but split into fixed-anchor + current-corpus and
phase-gated. Auto-research and the 8 discipline rules are deferred to
steady-state.

## Endpoint configuration

The MLX server on `http://localhost:10240/v1` (OAI-compatible) hosts the
target models. Pass `--runner oai-loop --oai-api-base
http://localhost:10240/v1 --oai-api-key 215069 --model
Qwen3.5-27B-4bit` to `bench/harness.py --run`.

## Reward channels

- **Cheap inner channel:** `cargo test -q`, `python3 -m unittest
  bench.test_command_policy bench.test_oai_loop bench.test_pi_audit
  bench.test_harness_json bench.test_harness_run_artifacts
  bench.test_harness_task_split bench.test_analyze_inputs
  bench.test_report_inputs`, `python3 bench/harness.py --md-binary
  target/release/md`. Run every iteration. Green is precondition,
  not progress.
- **Expensive outer channel (primary engine):** OAI loop runner against
  the target model on candidate tasks (3 modes × N runs) AND on the
  full search corpus when a promotion happens. Bundles land under
  `bench/runs/` with the standard `run.json` + `results.json` shape
  and the current `holdout_version` stamp.
- **Cross-model channel (triggered):** when the gap moves ≥+5pp,
  re-score the full corpus on the cross-model target
  (`Qwen3.5-122B-A10B-4bit`, same family larger) and record divergence
  in HEADLINE.md. Gap is expected to shrink on the larger model per
  CLAUDE.md's "tool benefit inversely proportional to model capability"
  finding; the >10pp divergence guard fires on the unexpected direction.

## Per-iteration shape

1. Read `bench/HEADLINE.md` to know the current phase, fixed-anchor
   gap (if exists), current-corpus gap, and corpus size.
2. **If phase = baseline-buildup:**
   - Pick 1–3 of the missing primary-baseline tasks. (Aim for the
     largest subset feasible within the iteration's wall-clock; do
     NOT batch the full 7 in one iteration if it risks exceeding
     the orchestrator's per-iteration window.)
   - Run hybrid + unix modes against the target model with dual
     scorer, holdout_version stamped. Append a buildup-phase row.
   - When the missing-list reaches 0, stamp the fixed-anchor baseline
     row and flip `phase: steady-state` in HEADLINE.md. That phase
     transition counts as the iteration's substantive move.
3. **If phase = steady-state:**
   - Diagnose: is the fixed-anchor gap stalling, the corpus stalling,
     or is there an open failing trace from a recent run?
   - Pick the admissible move that most directly addresses the
     diagnosis:
     - Stalling on both → propose new task family (auto-research).
     - Candidate accumulating in `candidates/` → promote one.
     - Failing trace open → harden the surface.
   - Make the change. Run the cheap validator. If green, run the
     expensive channel for the candidate (3 modes × N).
   - If the change moved a gap or grew the corpus, append a row to
     HEADLINE.md with the bundle pointer AND cause label.
   - If **fixed-anchor** gap moved ≥+5pp since last cross-model
     check, run cross-model on the 18-task fixed-anchor corpus.

## Auto-research discipline (load-bearing in steady-state only)

These were guardrails in T8; they are operational requirements in
T10's steady-state phase. Inactive during baseline-buildup.

1. **Generator is mdtools-blind.** Prompt shape: "realistic Markdown
   document-maintenance tasks an AI coding agent might need to perform
   in READMEs, specs, changelogs, runbooks, design docs, task lists."
   Generator must NOT see current `md` command names, current corpus
   IDs, or which tasks produced mdtools wins. Concretely: spawn the
   generator with a system prompt that omits mdtools entirely; describe
   the desired output shape (input doc, expected output, scorer policy)
   in vocabulary that doesn't bias toward AST-aware tooling.
2. **Realism review precedes gap measurement.** A candidate family is
   accepted as realistic *before* any unix vs mdtools run. Selection
   on realism, not on mdtools wins. Realism review can be a separate
   LLM judge with a prompt like "is this a task a human would actually
   perform on a real document?" and a yes/no verdict.
3. **Holdout never auto-grows.** Output flows
   `bench/search/candidates/` → `bench/search/quarantine/` →
   `bench/search/accepted/` → `bench/search/` proper. Promotion to
   `bench/holdout/` requires human review and a `holdout_version` bump.
4. **Family-level splits, not instance cherry-picking.** If a generator
   produces 30 instances of a family and mdtools wins 6, the *family*
   is accepted or rejected; promotion uses fresh unseen instances
   generated from the accepted family.
5. **Cross-model and cross-seed stability.** Candidate retention
   requires gaps that survive multiple seeds and the cross-model check.
6. **Unix-adversary review.** A separate model proposes the best
   plausible unix strategy. Gap is then labeled: AST-structural /
   shell-quoting / planning / prompting / scorer-artifact /
   current-mdtools-command-shape-match. **Only AST-structural gaps
   count as legitimate corpus members.** This is the load-bearing rule
   that prevents the corpus from filling with overfit tasks.
7. **Dual independent scorers.** Generator may not also generate the
   scorer. Use the existing dual-scorer dispatch (md binary + neutral
   markdown-it-py).
8. **Rejected candidates are stored.** Healthy ledger contains mdtools
   wins, unix wins, both-fail, both-pass, scorer-rejected, realism-
   rejected, suspected-tool-shape-artifact buckets. A ledger that
   contains only mdtools wins is benchmark farming.

## Hard rules

### Ledger budget (mechanical, unchanged from T8)

`bench/ledger.md` ≤ 500 lines. Overflow archives to
`bench/ledger-archive/<YYYY-Qn>.md` per T8 protocol.

### Ratification-of-ratification ban (unchanged from T8)

An iteration whose sole substantive content is "ratifying iter N-1
bit-exact" or "verifying that no fresh failing trace surfaced" is
inadmissible.

### Anti-folklore lock (unchanged from T8)

No new `md` commands, flags, op types, or agent action surfaces. Forbidden
list: `md apply`, `md move-block`, generalized `md set-state`, HTML body
support, ChangeSet-shaped CLI vocabulary, `md fingerprint <loc>` (the
single MAYBE Pro identified is admissible only if a specific failing
trace requires it — not as speculative ergonomics).

If T9 surfaces evidence that a new CLI primitive *is* warranted, halt
with `stop-and-summarize` and route that work outside this loop via
`/route` or the bridge-contract escalation path.

### Apples-to-apples normalization (unchanged from T7)

Cross-cell comparison requires: model identity, `thinking_level`,
executor, runs-per-task, `holdout_version`. Movements crossing any axis
are search-set observations, not gap movements.

### Status-theater prohibition (unchanged)

No upfront plans. No rollout narration. No completion summaries
mid-run. Typed artifacts are truth; ledger entries are short index
pointers.

### Attribution requirement (unchanged from T8)

Hardening interventions accept on "named failure class moved on
attribution probe," not "headline pass rate moved." (The headline gap
is a different number — at the *corpus* level, not the *task* level.
A task-level pass-rate movement still requires attribution; the corpus-
level gap is what HEADLINE.md tracks.)

### Promotion gate (steady-state only)

A candidate task family does not enter `bench/search/` proper unless
ALL of these are true:

- Realism review verdict: yes (logged with reviewer model + prompt).
- Unix-adversary review label: AST-structural (other labels reject).
- Cross-seed stability: gap appears across at least 3 seeds.
- Dual scorer agreement on at least one mdtools win in the family.
- Family is named in `bench/search/accepted/<family>/manifest.json`
  with input docs, expected outputs, scorer policies, and per-instance
  bundle pointers.

### Cross-model trigger (steady-state only, fixed-anchor only)

When the **fixed-anchor gap** moves ≥+5pp since the last cross-model
check, the next iteration MUST run the cross-model check on the
18-task fixed-anchor corpus before any other work. If divergence
> 10pp, file a finding (P0 if it crosses an acceptance metric, P1
otherwise) and halt corpus growth until resolved. Current-corpus gap
movement does NOT trigger this — only fixed-anchor.

### Composition discipline (new in T10)

Every HEADLINE.md history row in steady-state MUST carry a `cause`
label. Allowed values:

- `product` — mdtools binary or scorer changed; same denominator
- `agent` — runner/prompt/policy changed; same denominator
- `corpus-growth` — denominator changed (current-corpus only)
- `composition` — denominator changed via re-running a different
  subset (current-corpus only; flagged as descriptive)
- `baseline-buildup` — buildup-phase row, no improvement claim
- `cross-model` — cross-model verification, no headline movement

Iterations that move only `current-corpus` gap via `composition` are
admissible only as bookkeeping during buildup. In steady-state,
moving only current-corpus is admissible but does not constitute
hill-climb progress for halt-condition counting.

## Halt conditions

Halt fires on the **first** of:

1. **Gap saturation (steady-state only):** 3 consecutive promotion
   attempts produce no fixed-anchor gap movement AND no corpus growth
   surviving review. Corpus has converged on the current generator's
   reach for the target model.
2. **Cross-model divergence:** primary-vs-cross-model fixed-anchor
   gap diverges by >10pp without a clean explanation. Halt and
   investigate.
3. **Endpoint failure:** MLX server unreachable for >5 consecutive
   iterations.
4. **Cheap channel red** that cannot be restored within the iteration.
5. **Ledger budget breach** unrepairable in iteration.
6. **CLI temptation:** any iteration proposing a new CLI primitive
   triggers `stop-and-summarize` with a routing recommendation.
7. **Spec incoherence:** the loop discovers that T10's own rules
   contradict each other or block all admissible moves. Halt and
   request a spec-level repair from the operator rather than
   inventing a workaround. (T9's iter 3 self-flagging of denominator
   drift is the canonical precedent — that disposition produced T10.)
8. **Buildup stall:** in baseline-buildup phase, 3 consecutive
   iterations fail to extend baseline coverage (e.g. all remaining
   missing tasks fail with `MAX_TURNS_EXCEEDED` or runner errors).
   Halt with a routing recommendation rather than carrying an
   incomplete baseline into steady-state.

The halt summary lives at `bench/probes/t10-summary.md`, ≤200 lines,
with: final fixed-anchor gap (or "buildup incomplete"), final
current-corpus gap, final corpus size, phase at halt, families
accepted/rejected with gap labels, cross-model divergence at halt,
telemetry/findings delta, the disposition of each fired halt
condition, and a one-paragraph recommendation for the next loop.

## Artifacts to maintain

- **HEADLINE.md** (`bench/HEADLINE.md`): two numbers (fixed-anchor
  gap + current-corpus gap), current phase, and history. Updated only
  on iterations that extend baseline coverage, move a gap, or grow
  the corpus. Every history row carries a `cause` label.
- **Per-family table** (`bench/HEADLINE.md` companion section):
  per-family hybrid/unix pass rates, used to detect composition
  artifacts. Updated whenever a baseline row lands.
- **Ledger** (`bench/ledger.md`): index of findings, ≤500 lines.
- **Ledger archive** (`bench/ledger-archive/<YYYY-Qn>.md`): overflow.
- **Probes** (`bench/probes/`): per-finding directories with variant
  outputs and verdicts.
- **Auto-research staging** (`bench/search/candidates/`,
  `bench/search/quarantine/`, `bench/search/accepted/`): per-family
  staging with realism notes, unix-adversary gap labels, rejected-
  candidate buckets. Created lazily on first steady-state iteration
  that uses them.
- **Telemetry contracts** (`bench/telemetry/<command>.md`): per-command
  recording shape — admissible to add when a finding requires it.
- **Run bundles** (`bench/runs/`): per-iteration with `holdout_version`.
- **Halt summary** (`bench/probes/t10-summary.md`): bounded.
```

## Outstanding repo state at T10 launch

- `bench/HEADLINE.md` carries T9's iter 1–3 history (extraction +
  mutation + multi-step subsets, current-corpus gap +54.5pp on 11/18
  tasks). T10 launch must:
  - Add `phase: baseline-buildup` declaration at the top.
  - Re-label all existing history rows with `cause: baseline-buildup`.
  - Add a `Missing primary-baseline` callout listing T2, T3, T6, T8,
    T12, T17, T21.
  - Add stub for fixed-anchor gap (will be populated at phase
    transition).
- T9 iter 4 cross-model bundles exist for the 6-task extraction
  subset on `Qwen3.5-122B-A10B-4bit` (hybrid 5/6, unix 0/6 — all
  MAX_TURNS_EXCEEDED). These do NOT count toward the cross-model
  trigger; they were composition-driven and on the wrong model
  for steady-state. Treat as exploratory data only.
- `bench/search/candidates/`, `bench/search/quarantine/`, and
  `bench/search/accepted/` do not exist. Steady-state phase will
  create them on first auto-research pass.
- MLX endpoint live on port 10240. Primary target `Qwen3.5-27B-4bit`
  and cross-model target `Qwen3.5-122B-A10B-4bit` are both loaded.
- T7+T8 evaluator substrate carries forward intact: dual scorer with
  F8 fixes, mechanical L1 guard, holdout_version stamping, PI runner
  with audit, cross-executor comparability rule, opener-stack JSON
  extractor.

## Why this is the right next loop

- **Phase declaration ends denominator drift.** T9's iter 2→3 +10.1pp
  movement was a composition artifact. T10 makes that diagnosis
  structural: composition deltas are labeled, never count as
  improvement, and never trigger cross-model.
- **Fixed-anchor gap is the real hill-climb signal.** Once the
  18-task baseline exists, that number can only move via product /
  scorer / agent change. Current-corpus gap is preserved as a
  growth-tracking companion.
- **Iter 1 is bounded and feasible.** The remaining 7 tasks (T2, T3,
  T6, T8, T12, T17, T21) can be measured across 1–3 iterations of
  buildup phase before steady-state activates.
- **All T9/T8/T7 substrate preserved.** Auto-research, cross-model,
  promotion gate, ledger budget, anti-folklore lock, L1 guard — all
  carry forward unchanged into steady-state.
- **Halts cleanly** in either phase (buildup stall has its own halt
  condition; steady-state inherits T9's halt set, gated on
  fixed-anchor gap).
