# Frontier ablated-baseline re-run — 2026-06-01

The **first valid md-attribution measurement.** Re-run after the `./md` ablation
fix (`611c2c3`, PR #10 / Codex P1), on top of the isolation fix (`4e20adf`).

## Why this run exists

The clean re-run (`frontier-clean-2026-06-01`) was isolated correctly but its
`hybrid-no-md` baseline was **bypassed**: the harness copies `md` to `./md` in the
workdir for every mode and the prompt advertises `./md`, so a no-md agent that ran
`./md` (exactly as told) hit the **real** binary, not the stub. The no-md baseline
silently had `md`, so every **md-lift / attribution verdict** in that run measured
nothing (no-md ≈ hybrid because both had md). `611c2c3` makes the `./md` copy the
soft stub in `hybrid-no-md` too. Verified end-to-end: a no-md agent now runs
`./md tasks` → `md: unavailable here; use standard unix tools…` → falls back to
`grep`/`sed`, with `md_probe_count=1`.

The **Pareto headline** (`md ∝ 1/model-capability`) never used `hybrid-no-md` — it
is hybrid-vs-**unix** — so it stood through both methodology bugs. This run produces
the first *attribution* verdicts that are actually valid.

## Setup (minimal, reuse-where-sound)

`611c2c3` changes **only** the `hybrid-no-md` branch of the md-copy block; the
`unix` and `hybrid` code paths are byte-identical. So:

- **`unix` + `hybrid`: reused** from `frontier-clean-2026-06-01` (already isolated,
  unaffected by the `./md` fix — re-running them would only add nondeterminism, not
  remove a bug).
- **`hybrid-no-md`: re-measured here** with the fixed ablation — `T7,T10,T13,T20`
  (Targeted, n=4) + `T12` (Batch, n=1) × N=3, two models (`claude-sonnet-4-6`,
  `claude-opus-4-8[1m]`). 30 runs, **0 runner errors, all 30 cells 3/3 correct,
  every `md_probe_count == 1`** (each run tried `./md` once, hit the stub, fell back
  → `probe_ok` holds, the baseline is a competent unix fallback by the probe gate).

Render: `report.py` is run **per model** (both models are tier `frontier`, so one
combined call would mix them). Bundles: prior-clean `unix.txt`+`hybrid.txt` ⊕ this
run's merged `hybrid-no-md.txt`. See `render_verdicts.py`. (The raw 5-array harness
output is preserved as `*.raw5.txt`; the `.txt` was merged to one JSON array so
`report.py`'s loader reads all 15 records, not just the last.)

## Result — first VALID attribution verdicts (median tokens on both-passed intersection)

| cell | pareto unix→hybrid | lift no-md→hybrid | no-md vs unix | **verdict (valid)** | prior (invalid) |
|---|---|---|---|---|---|
| Sonnet · Targeted | n=4  48154→59350 (+23%) | n=4  66536→59350 (−11%) | **+38%** | `OPEN:loses-unix` | `loses-unix` |
| Sonnet · Batch    | n=1  55109→53124 (−4%)  | n=1  74969→53124 (−29%) | **+36%** | `SUSPECT:baseline-flails(cost)` | `SUSPECT` |
| Opus · Targeted   | n=4  16058→15314 (−5%)  | n=4  25810→15314 (−41%) | **+61%** | `SUSPECT:baseline-flails(cost)` | `no-lift` |
| Opus · Batch      | n=1  19745→25060 (+27%) | n=1  24719→25060 (+1%)  | **+25%** | `OPEN:loses-unix` | `loses-unix` |

All four cells: `md-probe=1`, `baseline_ok=False (reason=cost)`. **No structural cell
`CLOSES` on either frontier model.**

## Findings

1. **`md ∝ 1/model-capability` holds — now under a *valid* attribution gate.** No
   frontier structural cell `CLOSES`. The bottom line is unchanged from the
   Pareto-only finding; the valid re-run **confirms** it rather than overturning it.

2. **One verdict label flipped: Opus Targeted `no-lift` → `SUSPECT:baseline-flails`.**
   The invalid run had no-md ≈ hybrid (both ran real md) → apparent lift ≈ 0 →
   `no-lift`. Valid, the no-md baseline is a true unix fallback costing **+61% over
   pure unix**, so even though `md` *appears* to lift −41% over it, the gate refuses
   to credit a win measured against a flailing baseline → `SUSPECT`. The two
   `loses-unix` cells (Sonnet/Opus, pareto-fail) were immune to the bug — pareto
   short-circuits before the ablation is consulted.

3. **New, newly-valid finding — the md-documented prompt is itself a cost tax.**
   Across **all four** cells the `hybrid-no-md` baseline (the hybrid/md prompt with
   `md` stubbed) costs **+25% to +61% more than pure unix**. An agent primed on the
   md docs reads them, reflexively tries `./md` (the +1 probe), gets the stub, then
   hand-rolls unix — paying for the longer prompt and the wasted call. So the clean
   ablation baseline is **not** a competent unix-equivalent (guard #3's "no-md ≈
   unix" assumption is violated empirically), which is *why* md-lift is
   un-attributable on the frontier: every structural cell is either `loses-unix`
   (hybrid > unix) or `SUSPECT` (baseline flails on cost). The gate is working as
   designed — it declines to claim a win it cannot attribute.

Caveat: Batch is a single task (n=1 intersection); Targeted is n=4. All N=3.
`unix`/`hybrid` reused from the clean run (unchanged code path); only `hybrid-no-md`
re-measured.
