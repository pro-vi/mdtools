# mdtools Frontier Loop Prompt — T11 (Saturation-Aware Hill-Climb)

## Rationale

T10 codex run (PR #7) hit fixed-anchor saturation at **+38.9pp** after
27 substantive iterations. Codex completed the 18-task baseline,
flipped to steady-state, ran 25 auto-research passes, and surfaced two
durable observations:

1. **The candidate generator kept rejecting on the same locked surface.**
   3 distinct candidates (T6 baseline + iter 18 C-T10-26 + iter 27
   C-T10-34) hit the same compound subsection-relocation gap — heading
   + body must move as a unit, with optional level-shift to match the
   destination's hierarchy. This was a real product surface gap, not
   agent planning, but the anti-folklore lock (`md move-block`
   forbidden) blocked the natural fix. Codex labeled these
   `mdtools-fail` rather than escalating, so the loop quietly burned
   compute spinning on rejections instead of routing the work out.
   **Resolved post-T11:** F10-1 was routed to `/code-architect` as
   ordinary product work, `md move-section` shipped at master `9369af6`,
   and the lock has been amended to admit it (see § Anti-folklore lock
   below). T12+ runs can target this primitive directly.
2. **+38.9pp is the legitimacy claim won, not a stalled climb.** Per
   CLAUDE.md "tool benefit inversely proportional to model capability,"
   a strong gap on a small dense model is the right shape of evidence.
   Corpus growth 18 → 20 under discipline (2 AST-structural promotions,
   ~22 honest rejections) demonstrates the moat without overfit. The
   loop reached structural equilibrium for this model + lock combination.

T11 patches three discipline gaps surfaced by T10:

1. **`lock-blocked` rejection category.** When a candidate fails because
   the necessary primitive is on the anti-folklore forbidden list,
   label it `lock-blocked`, not `mdtools-fail`. Triggers escalation
   counter; 3 lock-blocked instances fire stop-and-summarize per halt #6.
2. **Tighten halt #1.** Count ALL iterations that produce no fixed-anchor
   movement AND no corpus growth, not just promotion-gate attempts.
3. **Equilibrium-as-valid halt (new #9).** If fixed-anchor has been
   stable for 5 consecutive iterations and corpus has grown by ≥2
   under discipline, declare the legitimacy claim satisfied, halt with
   `stop-and-summarize`, and route further work as scope expansion
   (cross-model, lock-lift, or new generator) outside this loop.

The anti-folklore lock is **preserved unchanged** in T11 with one
admission: F10-1 (compound subsection-relocation gap) was routed to
ordinary product work via `/code-architect`, the design landed in
`specs/move-section-design.md` (Pro 2-lane review, 0.83/0.83 confidence),
and `md move-section` shipped at master `9369af6` with 32 dedicated tests
and zero regressions vs the prior 283-test baseline. Per the original
T11 conditional ("future T12 [...] may admit `md move-section` after
F10-1's design lands and ships"), that condition is now met:
`md move-section` is **admitted** to the supported surface and removed
from the forbidden list.

Other locked surfaces (`md apply`, `md move-block`, generalized
`md set-state`, HTML body support, ChangeSet vocabulary) remain
forbidden. The lock-blocked category and escalation counter still apply
to those.

T7+T8 substrate (dual scorer with F8 fixes, L1 holdout guard,
holdout_version stamping, PI runner with audit, opener-stack JSON
extractor, cross-executor comparability rule) carries forward as frozen
baseline. T9's auto-research discipline rules carry forward unchanged.
T10's two-phase + two-gap + composition-discipline framing carries
forward; T11 only adds the saturation-aware patches above.

## What T11 lets the loop change vs. what stays locked

T11 is a hill-climb tier: the loop has **broad freedom over the tool
substrate** and **zero freedom over the measurement substrate**.
Anything below the line is a legitimate `product`/`agent`/`scorer`
move; anything above is fixed and tampering is treated as cheating
under the composition-discipline rule.

**Loop may change (with discipline)**

- **Product** (`cause: product`): mdtools binary — new flags on
  *admitted* commands, bug fixes, performance work, output-format
  refinements, new test fixtures pinning behavior. Adding a *new*
  CLI primitive is still a halt-#6 escalation (route via
  `/code-architect` → ship → re-launch with the lock amended, the
  way `md move-section` arrived in T11).
- **Agent** (`cause: agent`): runner prompt, system prompt, tool-use
  policy, retry/backoff strategy, turn budget, chain-of-thought
  surface, mode-specific instructions. Same-binary, same-corpus,
  same-scorer.
- **Scorer** (`cause: product`, scorer sub-type): hardening fixes that
  pin a real divergence between md-side and neutral-side scoring
  (T8 F8-1..F8-8 are the canonical shape). New scorer rules require
  an attribution probe + typed test that pins the finding.
- **Corpus growth** (`cause: corpus-growth`): new candidate families
  via the auto-research path. Realism review + unix-adversary review +
  cross-seed stability + dual scorer agreement still gate promotion;
  only `AST-structural` candidates land in `bench/search/`.

**Locked (tampering = cheating)**

- **Fixed-anchor identity.** The 18-task fixed-anchor corpus is the
  set stamped at T10-10 (`bench/tasks/tasks.json` minus the 6 holdout
  IDs at that stamp). Adding/removing tasks from this set is a
  composition move, never a hill-climb signal. New corpus members
  enter `current-corpus` only.
- **Holdout** (`bench/holdout/`). L1 mechanically guarded; never read
  by the loop; only post-run audit. Promotion to holdout requires
  human review and a `holdout_version` bump — outside the loop.
- **Composition discipline.** Any fixed-anchor delta produced by
  re-running a different subset, swapping seeds, or changing the
  measurement axis (model, executor, `holdout_version`,
  `thinking_level`) is a composition delta and does NOT count as
  movement. Cross-cell comparisons require all five axes equal.
- **Scorer dual-agreement requirement.** Both md-side and neutral
  markdown-it-py scorers must agree for a result to gate `correct`.
  Disabling, bypassing, or unilaterally tuning one scorer to match
  the other is cheating.
- **Realism + unix-adversary review.** Generator must be
  mdtools-blind; realism precedes gap measurement; unix-adversary
  labels are the only AST-structural verdict that admits a corpus
  member. Skipping or self-judging either is cheating.

**Counter resets at T11 launch**

T10's `stop-and-summarize` was a tier exit, not a halt that
T11 inherits. T11 launches with: 0 lock-blocked rejections, 0 stalled
iterations, 0 buildup stalls. The "5 consecutive stable + corpus grew
by ≥2" equilibrium counter (halt #9) is measured **from this T11
launch baseline**, not cumulatively across tiers — the 2 promoted
candidates from T10 (C-T10-15, C-T10-28) are part of the launch
state, not partial credit toward this tier's equilibrium.

**Expected first iteration shape**

Because `md move-section` shipped between T10 and T11, the cleanest
first move is `cause: product` — re-measure T6 (canonical
subsection-relocation failure in the fixed-anchor baseline) against
the new binary. If hybrid flips FAIL → PASS, fixed-anchor moves
+5.6pp, and the cross-model trigger fires for iteration 2.
This is illustrative; the loop chooses its own move.

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

T11 launches into **steady-state** with 18/18 tasks measured at
**+38.9pp** fixed-anchor (stamped at T10-10) and current-corpus +45.0pp
on 20 tasks. Baseline-buildup is complete; the buildup-phase rules above
are documented for completeness and for any future tier that re-opens
the baseline (e.g. a new model target whose own baseline must be
established before steady-state activates).

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

The following are inadmissible in T11 (both phases):

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

Current tier: T11.
T7+T8's substrate is frozen baseline. T9's headline-as-single-metric +
T10's two-phase + two-gap + composition-discipline framing all preserved.
T11 adds: `lock-blocked` rejection category, tightened gap-saturation
halt, lock-blocked accumulation halt, and equilibrium-as-valid halt.
Auto-research and the 8 discipline rules are deferred to steady-state.

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
     is there an open failing trace from a recent run, OR has the
     product surface changed since the last fixed-anchor measurement
     (new admitted CLI primitive, scorer hardening, agent prompt
     revision)?
   - Pick the admissible move that most directly addresses the
     diagnosis:
     - Product surface changed → re-measure affected fixed-anchor
       tasks with `cause: product` (or `agent` / `scorer` per the
       sub-axis). Existing candidates labeled `mdtools-fail` /
       `lock-blocked` whose failure mode the new surface addresses
       become re-evaluation targets — sweep them before generating
       new candidates.
     - Stalling on both → propose new task family (auto-research).
     - Candidate accumulating in `candidates/` → promote one.
     - Failing trace open → harden the surface.
   - Make the change. Run the cheap validator. If green, run the
     expensive channel for the candidate (3 modes × N).
   - If the change moved a gap or grew the corpus, append a row to
     HEADLINE.md with the bundle pointer AND cause label.
   - If **fixed-anchor** gap moved ≥+5pp since last cross-model
     check, run cross-model on the 18-task fixed-anchor corpus.
   - If the iteration fails due to infrastructure (`cause: infra`):
     log the failure, do NOT increment the halt #1 saturation counter,
     bump `--oai-request-timeout` if the cause is an OAI wall-time
     overrun, and retry the same iteration intent.

### Product-axis protocol

When a new admitted CLI primitive or scorer fix ships between tiers,
the **first** steady-state move MUST be a product-axis re-measurement
sweep (`cause: product`) over every fixed-anchor task that the prior
tier labeled `mdtools-fail` or `lock-blocked` for a reason the new
surface addresses. Only after that sweep is the loop free to generate
new candidates or declare saturation. Skipping the sweep mis-attributes
the halt reason (tool gap vs. search exhaustion) and wastes the next
tier's planning.

Concretely:
1. Build the new binary.
2. Filter `bench/search/candidates/*/manifest.json` for `status:
   rejected-lock-blocked` and `status: rejected-mdtools-fail` entries.
3. For each matching family, re-run `harness.py --run` in all three
   modes against the fixed-anchor corpus.
4. Update each candidate's manifest with the new verdict.
5. Only then proceed to corpus-growth or halt-condition evaluation.

### Agent-axis protocol

When a runner prompt, system prompt, or tool-policy file changes
between tiers, the loop must re-run at minimum the two lowest-pass-rate
fixed-anchor tasks in `cause: agent` mode to verify the change did not
regress existing passes. If those two re-runs both still pass, treat
the agent change as neutral and continue. If either regresses, the
agent change is the iteration's substantive content and the fixed-anchor
gap movement (positive or negative) is the row's Δ.

### Infra-failure recovery protocol

1. Tag the failed measurement `cause: infra` in HEADLINE.md (or omit
   the row entirely — infra rows are optional bookkeeping).
2. Do NOT count the failed iteration toward halt #1's no-movement streak.
3. Diagnose: OAI timeout → bump `--oai-request-timeout`; file vanished
   from workdir → check harness kill-TERM side effects; harness crash
   → check scorer error logs.
4. Retry with corrected parameters before advancing the iteration counter.

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

### Anti-folklore lock (T8, with one T11 admission)

No new `md` commands, flags, op types, or agent action surfaces. Forbidden
list: `md apply`, `md move-block`, generalized `md set-state`, HTML body
support, ChangeSet-shaped CLI vocabulary, `md fingerprint <loc>` (the
single MAYBE Pro identified is admissible only if a specific failing
trace requires it — not as speculative ergonomics).

**Admitted in T11 (post-shipment):** `md move-section` — section-aware
heading + body relocation with optional level cascade. Distinct from the
forbidden `md move-block` (arbitrary block movement); shipped with
`specs/move-section-design.md` and 32 dedicated tests after F10-1
escalated as ordinary product work. Now part of the supported surface;
candidate generators may target it freely.

If T11+ surfaces evidence that another new CLI primitive *is* warranted,
halt with `stop-and-summarize` and route that work outside this loop via
`/route` or the bridge-contract escalation path. Do not in-loop expand
the admitted list.

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

### Composition discipline (T10) + lock-blocked category (new in T11)

Every HEADLINE.md history row in steady-state MUST carry a `cause`
label. Allowed values:

- `product` — mdtools binary or scorer changed; same denominator
- `agent` — runner/prompt/policy changed; same denominator
- `corpus-growth` — denominator changed (current-corpus only)
- `composition` — denominator changed via re-running a different
  subset (current-corpus only; flagged as descriptive)
- `baseline-buildup` — buildup-phase row, no improvement claim
- `cross-model` — cross-model verification, no headline movement
- `lock-blocked` (T11) — candidate showed real AST-structural
  hybrid-vs-unix gap, BUT mdtools mode failed the cross-seed promotion
  gate because the necessary primitive is on the anti-folklore
  forbidden list. Label this honestly instead of `mdtools-fail` —
  the failure is a lock issue, not a tool/agent issue. Triggers the
  T11 escalation counter (see halt #6 below).
- `infra` — infrastructure failure prevented a valid measurement: OAI
  endpoint timeout, workdir file disappearance, harness crash, or
  scorer error unrelated to the candidate. **Does NOT count toward
  halt #1 saturation** (the no-movement streak must be composed of
  genuine zero-delta iterations, not failed measurements). Log the
  iteration as `cause: infra` with a one-line failure description,
  retry with corrected parameters (e.g., bumped `--oai-request-timeout`),
  and continue the iteration counter from the pre-failure state.

Iterations that move only `current-corpus` gap via `composition` are
admissible only as bookkeeping during buildup. In steady-state,
moving only current-corpus is admissible but does not constitute
hill-climb progress for halt-condition counting.

## Halt conditions

Halt fires on the **first** of:

1. **Gap saturation (steady-state only, T11 tightened):** 3 consecutive
   iterations produce no fixed-anchor gap movement AND no corpus growth
   surviving review. Counts ALL iterations, not just promotion-gate
   attempts. (T10 interpreted this narrowly and burned 25 steady-state
   iters at flat fixed-anchor.)
2. **Cross-model divergence:** primary-vs-cross-model fixed-anchor
   gap diverges by >10pp without a clean explanation. Halt and
   investigate.
3. **Endpoint failure:** MLX server unreachable for >5 consecutive
   iterations.
4. **Cheap channel red** that cannot be restored within the iteration.
5. **Ledger budget breach** unrepairable in iteration.
6. **CLI temptation / lock-blocked accumulation (T11 expanded):**
   any iteration proposing a new CLI primitive triggers
   `stop-and-summarize` with a routing recommendation. Additionally,
   3 cumulative `lock-blocked` rejections (per the new composition
   discipline cause label) fire stop-and-summarize with a routing
   recommendation, even if individual iterations didn't propose a
   primitive directly. The compounding signal IS the proposal.
7. **Spec incoherence:** the loop discovers that this spec's own rules
   contradict each other or block all admissible moves. Halt and
   request a spec-level repair from the operator rather than
   inventing a workaround. (T9's iter 3 self-flagging of denominator
   drift is the canonical precedent — that disposition produced T10.)
8. **Buildup stall:** in baseline-buildup phase, 3 consecutive
   iterations fail to extend baseline coverage. Halt with a routing
   recommendation rather than carrying an incomplete baseline into
   steady-state.
9. **Fixed-anchor equilibrium (new in T11):** if the fixed-anchor gap
   has been stable for 5 consecutive iterations AND the corpus has
   grown by ≥2 members under discipline, the legitimacy claim is
   considered satisfied for this model+lock combination. Halt with
   `stop-and-summarize`, ship the result, and route any further
   hill-climb work as scope expansion (cross-model expansion, lock-
   lift via product work, or new generator/model) outside this loop.
   This is the success-shaped halt, not the failure-shaped halts above.

The halt summary lives at `bench/probes/t11-summary.md`, ≤200 lines,
with: final fixed-anchor gap, final current-corpus gap, final corpus
size, phase at halt, families accepted/rejected with gap labels
(including any `lock-blocked` instances), cross-model divergence at
halt (if measured), telemetry/findings delta, the disposition of
each fired halt condition, and a one-paragraph recommendation for
the next loop or scope-expansion work.

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
- **Halt summary** (`bench/probes/t11-summary.md`): bounded.
```

## Outstanding repo state

- **`bench/HEADLINE.md` is canonical runtime state.** Read it for
  current `phase`, fixed-anchor gap, current-corpus gap, history, and
  per-family table. Do NOT assume the launch-time numbers — they evolve
  every iteration. As of this T11 launch revision: phase is
  steady-state, fixed-anchor +38.9pp, current-corpus +45.0pp on 20
  tasks (original 18 + C-T10-15 server-setup-relocation + C-T10-28
  error-logging-relocation, both promoted under T10's auto-research
  discipline as `AST-structural`). All 18 fixed-anchor tasks measured.
- **Search staging is populated.** `bench/search/candidates/`,
  `bench/search/quarantine/`, and `bench/search/accepted/` already
  exist. T10 left ~22 candidate bundles in `bench/search/candidates/`
  rejected as `mdtools-fail`/`hybrid-fail` — many describe
  subsection-relocation shapes that the newly-admitted `md move-section`
  primitive may now unblock. T11 may re-evaluate these as a `product`-
  cause sweep before generating new candidates.
- **MLX endpoint live on port 10240.** Primary target
  `Qwen3.5-27B-4bit` and cross-model target `Qwen3.5-122B-A10B-4bit`
  are both loaded.
- **T7+T8 evaluator substrate carries forward intact:** dual scorer
  with F8 fixes, mechanical L1 guard, holdout_version stamping, PI
  runner with audit, cross-executor comparability rule, opener-stack
  JSON extractor.
- **Product surface as of T11 launch:** `md move-section` is shipped at
  master `9369af6` and admitted to the supported surface. Other locked
  surfaces (`md apply`, `md move-block`, generalized `md set-state`,
  HTML body support, ChangeSet vocabulary) remain forbidden — see
  § Anti-folklore lock. Bench harness `command_policy.py` already
  classifies `move-section` as a mutation; agent's `md --help` lists it.
- **Known agent-side workload risks** (from prior runs, not blocking):
  Qwen3.5-27B-4bit unix mode is prone to invalid-response loops on
  multi-step tasks; content-delivery tasks (e.g. T3, T8) can take
  >20 min wall clock per mode. Plan iteration scope accordingly.

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
