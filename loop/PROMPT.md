<!--
PROVENANCE (loopgen)
- archetype: goal (terminal acceptance loop)
- divergences:
  - verifier-is-a-benchmark (task): the per-criterion oracle is the bench-v2 cost
    slice (quantitative, N≥3), not a unit test. Binary at the cell level
    (hybrid Pareto-dominates unix? yes/no); statistical underneath (N≥3, no n=1 claims).
  - per-criterion work is frontier-style (from frontier archetype): diagnose →
    ONE principled fix → verify-no-regression. The "make it cheaper" work is
    open-ended, but each front HAS a fixed pass line (hybrid ≤ unix), so it stays goal-shaped.
  - cross-criterion Pareto guard (task): closing one cell must not reopen another
    (an md change touches every cell). This is the goal impact-guard, load-bearing here.
  - oracle-drift guard == "never weaken the bench" (task): the bench tasks/scorers/
    agg_util cost logic/thresholds ARE the acceptance contract; editing them is oracle drift.
- consult-capability: tier-2 (agentify GPT-Pro available; PAL/trio may be down — single programmatic consult)
- evaluator-maturity: high (dual-scored bench + agg_util intersection logic, 7 fixture tests)
- frontload gaps: (1) omlx server must be running each session (BLOCKED_EXTERNAL if down);
  (2) claude-cli runs cost real API $ — treat as a soft budget; (3) local tier is tool_calls
  basis until AC-INSTR-pi-tokens; (4) claude-cli tool_mix not captured (cost/tokens ARE).
-->

You are running a **terminal goal loop** on this repository (mdtools).

Your job is not to explore the frontier.
Your job is to make a finite acceptance inventory — `loop/ACCEPTANCE.md`,
goal version `hybrid-attribution-v3` (clean-ablation gate) — pass **without weakening it**.

## Motive

Make `md` **earn its place** in hybrid mode on the mdtools bench-v2: hybrid never
worse than unix on correctness or cost, AND — on structural tasks — md adds
**attributable value** beyond a unix-capable agent given the *same* prompt
(hybrid must beat a `hybrid-no-md` baseline). By improving `md` and its hybrid
prompting — **never** by weakening the bench, and **never** by neutering md via
the prompt. A bare "hybrid ≥ unix" goal is gameable: hybrid *contains* unix, so
steering the agent away from md wins it while md never improves — which is why
structural cells gate on md-lift, not just ≥unix.

## Runner contract

Runner-agnostic; you are one iteration. Durable state lives in `loop/` files,
not memory. Emit `stop-and-summarize` when no useful iteration remains; emit
`escalate: <reason>` only for genuinely irreversible/external blocks (paid-API
with no budget cap, secrets, public publish). External ceilings (tokens, time)
are runner concerns — preserve the tree and summarize for the next run.

## Judgment default

On a taste/inferred call: take the smallest reversible action consistent with
the strongest source, log an Alignment Review (problem · options · chosen ·
alignment cost · rollback trigger · review question), and continue. Escalate
only for irreversible/external/authority-needed actions. **Running claude-cli
cells costs real API $** — keep iterations small; if an iteration would spend
heavily, prefer the local tier or a 1-task probe first, and log it.

## Oracle principles (honest by construction)

1. **Binary** — each cell criterion is pass/fail via `attribution_verdict`
   (verdict == `CLOSES`?), never self-assessment.
2. **Independence** — the oracle (bench-v2 slice) already FAILS the unmet
   behavior (claude T7 hybrid loses on cost). A verifier that can't fail can't
   prove. If you add/extend a verifier, show it red on the current gap first.
3. **Consumer-side** — "if this cell passes, would a user genuinely be better
   off using `md` here than plain unix?" If it requires inference, the verifier
   is wrong.
4. **Anti-theater** — `FIXED ≠ CLOSED`. A cell's own run passing is
   `PASS_PENDING_FINAL`. `PASS` requires the **final-verify** to prove the whole
   inventory in one repo state at N≥3. **No n=1 claims, ever.**

## Terminal contract

Complete only when **every** criterion in `loop/ACCEPTANCE.md` reaches `PASS`
via the final-verify. Then emit `criteria-met` → `stop-and-summarize`, halt
cause `criteria-met`. Not for partial completion, not for "the easy cells", not
for a single green run.

## Acceptance + verifier discipline

`loop/ACCEPTANCE.md` is the live anchor. The verifier for a cell = run that
(tier × category)'s tasks × **{unix, hybrid, hybrid-no-md}** × the tier's runner
at **N≥3**, then `python3 bench/report.py <outputs>` and read the cell's
**md-attribution verdict**. **Valid pass evidence:** verdict == `CLOSES` at N≥3
(structural ⇒ hybrid beat unix AND beat hybrid-no-md AND the baseline didn't
flail; tie-acceptable ⇒ hybrid ≥ unix). **Invalid:** "looks better", n=1, a cost
win from hybrid *failing* an expensive task, a relaxed scorer, a narrowed task
set, or a `CLOSES` faked by neutering the prompt (impossible — that yields
`OPEN:no-lift`).

## Channels

- **Cheap inner channel** (after any edit, before the bench): if `md` changed,
  `cargo build --release && cargo test` (337 Rust tests) ; always
  `python3 -m bench.test_agg_util` (the oracle's own tests must stay green).
- **Per-criterion verifier:** the cell's bench run over **4 modes** (unix,
  hybrid, hybrid-no-md, + tier runner) → the attribution verdict, N≥3.
- **Final-verify:** full suite × **4 modes** × both tiers at N≥3 →
  `python3 bench/report.py loop/runs/final/*.txt` → assert EVERY cell verdict ==
  `CLOSES`. Run for terminal completion and as a checkpoint after any `md`
  change (which touches every cell).

## Dependency topology

`AC-MASTER` depends on every `AC-{tier}-{cat}`. The cells are otherwise
independent **except**: any `md` change or `HYBRID_DOCS`/`MDTOOLS_DOCS` change
affects **every** cell — so after such an edit, the impact guard = re-run the
**already-passing** cells of that tier and confirm none regressed (the
**Pareto-no-regression guard**, load-bearing). `AC-INSTR-*` are enablers, not
fronts; do them only if they sharpen measurement.

## Iteration protocol

1. Read `loop/ACCEPTANCE.md`, `loop/STATE.md`, the latest `loop/runs/` outputs.
   Confirm goal version matches. **Check omlx is up** (`curl -s -m4 -H
   "Authorization: Bearer 215069" http://127.0.0.1:10240/v1/models`); if down →
   `BLOCKED_EXTERNAL`.
2. **Oracle integrity check** before editing: bench tasks/scorers unchanged,
   `agg_util` cost logic + thresholds unchanged, no task removed from a cell, no
   tolerance loosened. Any such change = `oracle-drift` (see guard).
3. If every cell is `PASS_PENDING_FINAL`/`PASS`, run the **final-verify**. If it
   proves the whole inventory in one repo state at N≥3: set all `PASS`, write
   `loop/VERIFY.md` with the slice, emit `criteria-met` → `stop-and-summarize`.
4. Else pick **one** primary failing/`OPEN` cell — strongest failing evidence
   first (seed: `AC-frontier-Targeted-mutation`), cheapest verifier feedback,
   highest regression risk. If every unpassed cell is `STUCK`/`BLOCKED`/
   `QUARANTINED` → halt classification.
5. Before editing write one line:
   `cell-id | failing-evidence | hypothesis-for-WHY | edit-surface (md or HYBRID_DOCS) | rollback`.
   **Diagnose the WHY first** — run the failing cell with `--log-dir
   loop/runs/diag/` and read the agent trace + per-call token/tool-call
   breakdown. (e.g. did `md tasks --json` emit a huge payload? did the agent
   re-query? did it pick `md` where `grep` was one line?)
6. Make **one** small reversible change to `md` (src/) or the hybrid prompting
   (bench/harness.py `HYBRID_DOCS`/`MDTOOLS_DOCS`) targeting that root cause.
   Run the cheap inner channel; fix or revert if it fails.
7. Run the cell's verifier — **all 4 modes** (unix, hybrid, hybrid-no-md, tier
   runner) at N≥3 → the attribution verdict. Then run the impact guard for
   already-passing cells of the affected tier.
8. Accept only if: the cell moves toward pass (or gains sharper evidence), **no
   passing cell regresses** (Pareto guard), and the oracle was not weakened.
   Else revert and record the failed hypothesis.
9. Cell verifier passing → `PASS_PENDING_FINAL`, not `PASS`.
10. 3 consecutive failures with no new evidence → `STUCK`; if the cell is one
    where unix is structurally better and no admissible md/prompt change closes
    it, document it as a **tie-target** (hybrid ≥ unix is the win) and move on.

## Oracle-drift guard == "never weaken the bench"

The headline failure mode. The loop must NOT: remove/relax a bench task or
scorer; change `agg_util`'s intersection/cost/attribution logic to flatter `md`;
loosen the 5% tolerance; drop tasks from a cell to shrink the comparison; count a
cost "win" that came from hybrid **failing** an expensive task; **game the
attribution baseline** — push md in `HYBRID_DOCS` so `hybrid-no-md` FAILS tasks
or repeatedly hits the md stub (the verdict catches this as
`SUSPECT:baseline-flails(correctness|probes|cost)`; the baseline must stay a
clean unix fallback, so the win must come from hybrid being BETTER, not the
baseline being worse); refresh an expected fixture to current wrong output; or
claim a front at n=1. Any change to the
**measurement** (not the thing measured) requires an inline Oracle Change Note
in `loop/STATE.md` proving new-strictness ≥ old (red/green or sentinel). If you
can't prove it, restore the old verifier or emit `oracle-drift` and stop.

## Rules / scope

- **Allowed to change:** `src/**` (the `md` tool), `bench/harness.py`
  `HYBRID_DOCS`/`MDTOOLS_DOCS` (the hybrid prompting), and `md`'s own docs.
- **Forbidden to change:** `bench/tasks/**`, the scorers, `bench/agg_util.py`
  cost/intersection logic, `loop/ACCEPTANCE.md` criteria/thresholds. These are
  the oracle.
- `md` stays **general-purpose markdown tooling** (CLAUDE.md: "markdown
  primitives only"). No task-specific hacks. Honor "hybrid > pure / don't
  replace sed" and "re-query is the moat".
- **No overfitting:** a fix must close the cell's *category* generally, not just
  one task — validate the whole category at N≥3, and run the impact guard.
- Partial completion is not success: continue while any unpassed cell has a
  legal reversible next move in scope.

## Halt conditions / classifier

Halt = `stop-and-summarize` (+`criteria-met` first on terminal success). Label:
- `criteria-met` — final-verify proved every cell at N≥3.
- `partial-deadlock` — remaining cells all `STUCK`/`BLOCKED_EXTERNAL`/
  `QUARANTINED` (e.g. unix-structurally-better tie-targets that already ≥ unix;
  or omlx unavailable). Preserve evidence, list each unpassed cell + latest
  failing evidence + the next required lever. **Do not lower the bar.**
- `oracle-drift` — the bench/criteria can't be preserved without weakening.
- `derivation-gap` — blocked on something this prompt should have specified;
  name it so the next `/loopgen` pass adds it.
- `genuine-escalate` — irreversible/external (claude budget exhausted, omlx
  needs manual start, etc.).
- `wrong-loop` — if a cell genuinely needs open-ended metric-pushing with no
  fixed pass line, reroute that piece via `/loopgen` to the `frontier` archetype.

## Artifacts to maintain

- `loop/ACCEPTANCE.md` — frozen criteria; mutable `status`/`last_verification` only.
- `loop/STATE.md` — goal version, iteration, current cell, stuck counters, Oracle
  Change Notes (inline), last/next action.
- `loop/VERIFY.md` — final-verify slice transcript; written on `criteria-met`.
- `loop/runs/**` — per-cell bench outputs (keep; they are the evidence).
- **Skill Harvest** (`loop/runs/harvest-*.md`): when an iteration exposes a
  reusable lesson (a missing invariant, an md-design pattern that generalizes,
  a bench gap), capture target-skill · observed-gap · evidence-iteration ·
  proposed-rule · why-it-generalizes · accidental-encouragement-risk.

## Repo-specific overlay

- **Build first if `md` changed:** `cargo build --release` (binary
  `target/release/md`). The bench uses `--md-binary target/release/md`.
- **Run a cell (the 3 gate modes):** for `MODE in unix hybrid hybrid-no-md`:
  `python3 bench/harness.py --run --runner <R> --mode $MODE --task <Tn> --md-binary target/release/md -N 3 --json --log-dir loop/runs/<tag>/ > loop/runs/<tag>/<Tn>_$MODE.txt`
  (output is text-summary-then-JSON-array; `report.py` parses it). `mdtools` mode
  is optional, for diagnosis only — the attribution gate uses unix/hybrid/hybrid-no-md.
- **Tiers:** FRONTIER `--runner claude-cli` (default claude; real token cost;
  claude's Bash bypasses the guard so `tool_mix`/`mutations` are blank — cost is
  fine). LOCAL `--runner pi-json --model Qwen3.6-35B-A3B-8bit` (pi on local Qwen
  via omlx; `tool_mix`/`mutations` capture; tokens currently 0 → tool_calls
  basis until `AC-INSTR-pi-tokens`).
- **Slice + verdicts:** `python3 bench/report.py loop/runs/<tag>/*.txt` → read
  `=== COST SLICE (bench-v2) ===` (per-cell cost basis `tokens` vs `tool_calls`,
  never compared across bases) AND the `md-attribution verdicts` section (each
  cell's `CLOSES` / `OPEN:*` / `SUSPECT` — the loop's work list).
- **First iteration:** instantiate the full `AC-{tier}-{cat}` inventory from a
  first 4-mode full-suite run (one row per cell, with its verdict), then start
  closing from `AC-frontier-Targeted-mutation` (the seed: claude burns +70k
  tokens with `md` on T7 → currently `OPEN:loses-unix`; diagnose the verbosity).
- **Consult (tier-2):** for a genuinely stuck diagnosis (why is md a trap on a
  cell?), you may fire one `/agentify` GPT-Pro consult; log it.
