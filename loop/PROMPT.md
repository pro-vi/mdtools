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
   inventory in one repo state at N≥3. **No n=1 claims, ever.** A 1-task or
   single-replicate **probe is diagnosis only** — it can never set or advance a
   cell's status; status changes require the full category × 3-gate-mode ×
   tier-runner × N≥3 verifier.

## Terminal contract

Complete only when **every** criterion in `loop/ACCEPTANCE.md` reaches `PASS`
via the final-verify. Then emit `criteria-met` → `stop-and-summarize`, halt
cause `criteria-met`. Not for partial completion, not for "the easy cells", not
for a single green run.

## Acceptance + verifier discipline

`loop/ACCEPTANCE.md` is the live anchor. The verifier for a cell = run that
(tier × category)'s tasks × **{unix, hybrid, hybrid-no-md}** × the tier's runner
at **N≥3**, then `python3 bench/report.py <outputs>` and read the cell's
**md-attribution verdict**. The source of truth is
`agg_util.attribution_verdict`'s **return value** — `report.py` only *renders*
it; never treat report.py's printed text as authoritative if you've touched the
renderer. **Valid pass evidence:** verdict == `CLOSES` at N≥3
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
   Confirm the `goal_version:` field matches. **Check omlx is up** (`curl -s -m4
   -H "Authorization: Bearer 215069" http://127.0.0.1:10240/v1/models`); if down
   → `BLOCKED_EXTERNAL`. **Inventory gate (where am I?):** if the AC inventory is
   not yet instantiated — STATE.md `iteration: 0`, or no `loop/runs/init-*` slice
   exists — run the **Bootstrap** (§ below) to instantiate it, write the
   `AC-{tier}-{cat}` rows + bump `iteration` in STATE.md, and END this iteration
   there (do not also pick a cell). Otherwise skip the Bootstrap entirely and
   continue to step 2 — the Bootstrap is iteration-0-only and must NOT re-run.
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
baseline being worse); refresh an expected fixture to current wrong output;
**edit `report.py`'s rendering to alter the printed verdict**; **hand-write,
edit, copy, or synthesize any run-record JSON** under `loop/runs/**` (every
record `report.py` consumes must be the verbatim stdout of a harness run you
executed *this* session — "report.py printed CLOSES over files I own" is not a
real pass); or claim a front at n=1. Any change to the **measurement** (not the
thing measured) requires an inline Oracle Change Note in `loop/STATE.md`, and is
admissible ONLY if it passes a mechanical strictness check the agent cannot fake:
**(a)** every existing `bench/test_agg_util.py` test still passes **unmodified**
(editing that file to accommodate the change is itself `oracle-drift`); **(b)**
you add a sentinel that is RED both before and after on the current gap; **(c)**
the previously-recorded failing evidence still yields a non-`CLOSES` verdict
under the new code. If you can't prove all three, restore the old verifier or
emit `oracle-drift` and stop.

## Rules / scope

- **Allowed to change (the ONLY two lever surfaces):** `src/**` (the `md` tool)
  and, inside `bench/harness.py`, **only the `MDTOOLS_DOCS` and `HYBRID_DOCS`
  string literals** (the hybrid prompting) — plus `md`'s own `--help`/docs.
  Nothing else in `harness.py`. Inner-channel check: `git diff bench/harness.py`
  must show changes confined to those two literals; a diff anywhere else =
  `oracle-drift`, revert.
- **The immutable oracle (canonical list — Forbidden to change):**
  `bench/tasks/**`; **all of `bench/harness.py` except the two doc literals**
  (the scorers `StructuralDiffPolicy` / `score_task` / `score_json_canonical` /
  `score_structural_json` / the independent scorer all live here — they are
  oracle, not lever); `bench/agg_util.py` (intersection/cost/**attribution**
  logic + thresholds); `bench/report.py` (it only *renders* the verdict —
  editing it to change what verdict prints is drift); `bench/command_policy.py`
  (the mode guard + soft-stub + policy-sync check); `loop/ACCEPTANCE.md`
  criteria/thresholds; and **every run-record under `loop/runs/**`** (evidence,
  never a lever — see oracle-drift guard).
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
- **Seed cell `AC-frontier-Targeted-mutation` (the gated delta):** once the
  inventory is instantiated, start closing here. The delta to target is the
  **gated** one: on T7, **hybrid** costs ~+6500 tok over unix (82367) — just over
  the 5% tolerance (~4118) — so the cell reads `OPEN:loses-unix`. (The
  often-quoted +70427 is **mdtools** mode, which is diagnosis-only and NOT in the
  gate — don't chase it; tuning mdtools output won't move the verdict.) Diagnose
  why hybrid's `md` usage adds ~6.5k tokens (md `--json` payload size, or an extra
  re-query) from the `--log-dir` per-call breakdown, then make it leaner.
- **Consult (tier-2):** for a genuinely stuck diagnosis (why is md a trap on a
  cell?), you may fire one `/agentify` GPT-Pro consult; log it.

## Bootstrap (iteration-0 only — gated by the inventory gate in step 1)

Run this block ONLY on the instantiation iteration (STATE.md `iteration: 0` / no
`loop/runs/init-*` slice). Every later iteration skips it and goes straight to
the cell cycle (protocol steps 2→10). All Python is **`python3`** (this machine
has no `python` on PATH). Follow the artifact convention verbatim: stdout redirected to one `.txt`
per mode, with `--log-dir` traces in a sibling `logs/` subdir the report glob
ignores — `report.py` raises an uncaught `FileNotFoundError` on any dir lacking
`results.json`, so never point it at a results parent with a bare `*`; glob
`*.txt`.

1. **Clean baseline.** `git status` — the only expected modification is
   `tests/cli_contracts.rs` (pre-existing rustfmt whitespace; harmless). Confirm
   `cargo test` is GREEN now, so a later RED is attributable to your own edit.
2. **omlx up** (else `BLOCKED_EXTERNAL`; do not start it yourself):
   `curl -s -m4 -H "Authorization: Bearer 215069" http://127.0.0.1:10240/v1/models`
3. **Build md:** `cargo build --release` (binary `target/release/md`).
4. **Smoke the local runner reaches omlx** before the full sweep (pi resolves its
   endpoint from pi's own config, NOT the curl'd URL):
   `python3 bench/harness.py --run --runner pi-json --model Qwen3.6-35B-A3B-8bit --mode hybrid --task T1 -N 1 --json --log-dir loop/runs/smoke/logs/ > loop/runs/smoke/probe.txt`
   — confirm the record's `model` is the Qwen id. If pi does NOT route to omlx,
   fall back to `--runner oai-loop --oai-api-base http://127.0.0.1:10240/v1
   --oai-api-key 215069 --model Qwen3.5-27B-4bit` (verified-working; oai-loop
   drops `tool_mix`, so run adoption diagnosis on a pi-json run).
5. **Instantiate the inventory** — full suite, **3 gate modes × tier runner**,
   N≥3 (NOT `mdtools` mode — diagnosis-only, the gate never reads it; on the paid
   frontier tier it burns budget for nothing). LOCAL first (cheap; tool_calls):
   `for MODE in unix hybrid hybrid-no-md; do python3 bench/harness.py --run --runner pi-json --model Qwen3.6-35B-A3B-8bit --mode $MODE --md-binary target/release/md -N 3 --json --log-dir loop/runs/init-local/logs/ > loop/runs/init-local/$MODE.txt; done`
   FRONTIER (claude-cli — real $; tokens; keep tight):
   `for MODE in unix hybrid hybrid-no-md; do python3 bench/harness.py --run --runner claude-cli --mode $MODE --md-binary target/release/md -N 3 --json --log-dir loop/runs/init-frontier/logs/ > loop/runs/init-frontier/$MODE.txt; done`
6. **Render + sanity-check:**
   `python3 bench/report.py loop/runs/init-local/*.txt loop/runs/init-frontier/*.txt`
   Confirm the claude rows land under a `frontier` tier (not `unspecified`); if
   not, the model stamp failed — pass an explicit `--model claude-...` on the
   claude-cli loop so the tier is deterministic, and re-render.
7. **Attack the seed cell** `AC-frontier-Targeted-mutation` per the gated-delta
   note in the overlay above. Heavy-spend guard — probe one task first:
   `python3 bench/harness.py --run --runner claude-cli --mode hybrid --md-binary target/release/md --task T7 -N 3 --json --log-dir loop/runs/diag/logs/ > loop/runs/diag/T7_hybrid.txt`
