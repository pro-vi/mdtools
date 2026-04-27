# mdtools Frontier Loop Prompt — T9 (Headline-Metric Hill-Climb)

## Rationale

T7 (PR #3) closed every OPEN oracle-trustworthiness finding and built the
mechanical guards. T8 (PR #4) closed 8 latent scorer/extractor bugs that T7
had never surfaced (F8-1 through F8-8), including 2 false-POSITIVES that
compromised past benchmark verdicts. Both runs hardened the *evaluator*; the
*product* was untouched.

T9 turns the loop outward. The repo's legitimacy claim is one sentence:
*"agents perform measurably better on Markdown editing with mdtools than
without."* That claim is a **gap** — hybrid-mode pass rate minus unix-mode
pass rate on a target small model, on a growing realistic corpus. The gap is
the headline number. The loop's only job is to make it go up over time, by
either growing the corpus (auto-research-style task discovery surviving
realism + unix-adversary review) or by hardening existing surfaces against
specific failing traces that the gap surfaced.

T7's evaluator substrate (search/holdout split, mechanical L1 guard,
holdout_version stamping, PI runner with audit, cross-executor comparability
rule, dual scorer with F8-series fixes) is treated as frozen baseline. T8's
"hill-climb on existing surfaces" framing carries forward as a sub-channel,
but the *primary* discovery engine is now auto-research producing realistic
task families that distinguish mdtools from unix.

T9 differs from T8 in three structural ways:

1. **Single declared metric.** `bench/HEADLINE.md` carries one number — the
   gap on the primary target model. Every accepted iteration must move it
   up or grow the corpus underneath it.
2. **Auto-research is the primary engine, not a sub-channel.** T8 admitted
   it but never ran it. T9 makes it the iteration default. The 8 discipline
   rules from T7's spec are now load-bearing operational requirements, not
   guardrails for an optional sub-flow.
3. **The endpoint is real.** MLX server on port 10240 is reachable; the OAI
   loop runner already speaks to it. Iterations that need to score a
   candidate task on the target model can run it cheaply.

## Prompt

```md
You are running a hill-climb loop on this repository.

Your job is to make ONE number go up over time:
**hybrid mode pass rate minus unix mode pass rate**, on the target small
model, over the full search corpus, recorded in `bench/HEADLINE.md`.

That gap is the legitimacy claim — "agents perform measurably better on
Markdown editing with mdtools than without." Each iteration must move
the gap up OR grow the corpus underneath it with a task that survived
realism + unix-adversary review.

## Motive

Build a defensible, growing evidence base for the headline claim. The
gap is the metric; the corpus growth is the moat. A gap of +20pp on
24 tasks is weaker evidence than +20pp on 100 tasks of demonstrably
realistic Markdown-agent work — both the *level* and the *breadth* matter.

## Core law

Each iteration must do exactly one of these substantive moves — nothing
else is admissible:

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
4. **Cross-model stability check:** when the gap has moved ≥+5pp since
   the last check, run the full corpus on the cross-model target and
   record both numbers. If divergence > 10pp, file a finding.
5. Emit `stop-and-summarize` because the halt conditions are met.

**Required tail action on items 1–4:** if the iteration moved the gap
or grew the corpus, append a row to `bench/HEADLINE.md`'s history table
with the bundle pointer in the same iteration. HEADLINE.md update is
never a standalone iteration; it is the bookkeeping that closes a real
move.

The following are inadmissible in T9:

- producing a bundle whose only purpose is coverage cell-filling
- promoting a prose claim to a typed test that doesn't fix or pin a
  finding the gap surfaced
- ratifying a previous iteration bit-exact as the iteration's sole
  content
- adding any new CLI command, flag, op type, or agent action surface
  (the T8 anti-folklore lock carries forward)
- aligning mdtools' op vocabulary to FRAC-147's ChangeSet IR
- writing to `specs/fract-ai-bridge-contract.md` (frozen artifact)
- modifying `bench/holdout/` or any holdout fingerprints (L1 guard)

## Evaluator maturity

Current tier: T9.
T7+T8's substrate is frozen baseline. The headline metric is the new top
of the funnel; the loop's job is to climb it. Auto-research is the
primary engine, with the 8 discipline rules as load-bearing requirements.

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

1. Read `bench/HEADLINE.md` to know the current gap and corpus size.
2. Diagnose: is the gap stalling, the corpus stalling, or is there an
   open failing trace from a recent run?
3. Pick the admissible move that most directly addresses the diagnosis:
   - Stalling on both → propose new task family (auto-research generator
     pass).
   - Candidate accumulating in `candidates/` → promote one through
     realism + unix-adversary review.
   - Failing trace open → harden the surface.
4. Make the change. Run the cheap validator. If green, run the expensive
   channel for the candidate (3 modes × N).
5. If the change moved the gap or grew the corpus, append a row to
   `bench/HEADLINE.md`'s history table with the bundle pointer.
6. If gap moved ≥+5pp since last cross-model check, run cross-model.

## Auto-research discipline (load-bearing in T9)

These were guardrails in T8; they are operational requirements in T9.

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

### Promotion gate (new in T9)

A candidate task family does not enter `bench/search/` proper unless
ALL of these are true:

- Realism review verdict: yes (logged with reviewer model + prompt).
- Unix-adversary review label: AST-structural (other labels reject).
- Cross-seed stability: gap appears across at least 3 seeds.
- Dual scorer agreement on at least one mdtools win in the family.
- Family is named in `bench/search/accepted/<family>/manifest.json`
  with input docs, expected outputs, scorer policies, and per-instance
  bundle pointers.

### Cross-model trigger (new in T9)

When the headline gap moves ≥+5pp since the last cross-model check,
the next iteration MUST run the cross-model check before any other
work. If divergence > 10pp, file a finding (P0 if it crosses an
acceptance metric, P1 otherwise) and halt corpus growth until resolved.

## Halt conditions

Halt fires on the **first** of:

1. **Gap saturation:** 3 consecutive promotion attempts produce no gap
   movement AND no corpus growth surviving review. Corpus has converged
   on the current generator's reach for the target model.
2. **Cross-model divergence:** primary-vs-cross-model gap diverges by
   >10pp without a clean explanation. Halt and investigate.
3. **Endpoint failure:** MLX server unreachable for >5 consecutive
   iterations.
4. **Cheap channel red** that cannot be restored within the iteration.
5. **Ledger budget breach** unrepairable in iteration.
6. **CLI temptation:** any iteration proposing a new CLI primitive
   triggers `stop-and-summarize` with a routing recommendation.
7. **Spec incoherence:** the loop discovers that T9's own rules
   contradict each other or block all admissible moves (e.g. cross-
   model trigger stuck because endpoint timeout, or all admissible
   moves blocked by an open finding that nothing in scope can close).
   Halt and request a spec-level repair from the operator rather than
   inventing a workaround.

The halt summary lives at `bench/probes/t9-summary.md`, ≤200 lines,
with: final gap, final corpus size, families accepted/rejected with
gap labels, cross-model divergence at halt, telemetry/findings
delta, the disposition of each fired halt condition, and a one-paragraph
recommendation for the next loop (if any).

## Artifacts to maintain

- **HEADLINE.md** (`bench/HEADLINE.md`): the one number, current and
  history. Updated only on iterations that move the gap or grow the
  corpus.
- **Ledger** (`bench/ledger.md`): index of findings, ≤500 lines.
- **Ledger archive** (`bench/ledger-archive/<YYYY-Qn>.md`): overflow.
- **Probes** (`bench/probes/`): per-finding directories with variant
  outputs and verdicts.
- **Auto-research staging** (`bench/search/candidates/`,
  `bench/search/quarantine/`, `bench/search/accepted/`): per-family
  staging with realism notes, unix-adversary gap labels, rejected-
  candidate buckets.
- **Telemetry contracts** (`bench/telemetry/<command>.md`): per-command
  recording shape — admissible to add when a finding requires it.
- **Run bundles** (`bench/runs/`): per-iteration with `holdout_version`.
- **Halt summary** (`bench/probes/t9-summary.md`): bounded.
```

## Outstanding repo state at T9 launch

- `bench/HEADLINE.md` exists (created at T9 launch with placeholder
  current value). Iteration 1 must run the full search corpus on the
  target model in all 3 modes to populate the first real value.
- `bench/search/candidates/`, `bench/search/quarantine/`, and
  `bench/search/accepted/` do not exist. The first auto-research pass
  creates them.
- MLX endpoint live on port 10240. Primary target `Qwen3.5-27B-4bit`
  and cross-model target `Qwen3.5-122B-A10B-4bit` are both loaded.
  `Qwen3.6-35B-A3B-8bit` and Gemma-4 are downloading and may join the
  matrix later as additional cross-model checks.
- T7+T8 evaluator substrate carries forward intact: dual scorer with F8
  fixes, mechanical L1 guard, holdout_version stamping, PI runner with
  audit, cross-executor comparability rule, opener-stack JSON extractor.

## Why this is the right next loop

- **Single declared metric** (the gap in HEADLINE.md) replaces T8's
  diffuse "research and harden" framing. There is one number, and the
  loop's only job is to make it go up.
- **Auto-research as primary engine** finally exercises the channel T8
  spec admitted but never ran. The 8 discipline rules become operational
  requirements, not guardrails.
- **Endpoint is real.** No more theoretical hill-climb — Qwen3.5-27B-4bit
  on MLX is one curl away. T8's blocker is gone.
- **Anti-overfit by construction** via realism review + unix-adversary
  review (only AST-structural gaps count) + cross-model trigger
  (≥+5pp moves require Qwen3.5-122B-A10B confirmation, where the gap
  is expected to shrink rather than grow).
- **Halts cleanly** when the corpus saturates against the current
  generator + target model. No drift mode, because every iteration must
  cite a gap movement, a corpus growth, or a hardening fix tied to the
  gap.
