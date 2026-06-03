# Native-rooted arm — first live frontier sweep (2026-06-03)

FRAC-194 U7. `claude-cli` (Sonnet 4.6), N=3, the 5 fixed anchor tasks
(T7/T10/T13/T20 = Targeted, T12 = Batch) × 3 native modes. **45/45 sessions PASS,
0 errors.**

> **⚠️ CORRECTED 2026-06-03 after a billed-$ re-score + a GPT-5 Pro adversarial
> review.** The first write of this file claimed "md loses to native Edit at the
> frontier, +73%, net-negative." That **overclaimed** — the +73% was a raw-token
> artifact, and the design can't attribute the cost to md-the-tool. The honest
> result is below.

## What this run actually shows (and doesn't)

It is a **prompt-induced-over-adoption warning**, not a clean tool-causal result:
with the md-advertising prompt (`NATIVE_MD_DOCS`), Sonnet — which *had* native
`Edit` available — used **md exclusively (zero native Edit)** and cost more on
five easy markdown-edit tasks. It does **not** establish that md-the-tool is
intrinsically costlier than native editing.

## Cost: raw tokens vs billed-$ (the magnitude was inflated ~2.5×)

Re-scored from the transcripts in Sonnet 4.6 input-equivalents (input×1, cache_create×1.25, **cache_read×0.1**, output×5):

| cell | mode | RAW Δ vs native | **BILLED-$ Δ vs native** |
|---|---|---|---|
| Targeted (n=4) | native+md | +73% | **+28%** |
| Targeted (n=4) | native+md-no-md | +73% | +27% (ablation invalid — see below) |
| Batch (n=1, anecdotal) | native+md | +19% | +44% |

Why raw lied: native+md's raw cost is **~88% `cache_read`** (re-reading `md tasks`
output + the larger md-docs prompt). Raw counts those at 1.0×; real billing is
0.1×. So **+73% (raw) → +28% (billed)**. The **sign survives** (md still
Pareto-loses native, +28% > the 5% tolerance) but the headline number was wrong —
the same raw-token error this project corrected for the POSIX arm, repeated here.

## Adoption — md advertised ⇒ md used exclusively (zero native Edit)

| mode | tool_mix (transcript-derived) | calls/task |
|---|---|---|
| `native` (no md) | Read 15, Edit 18, grep 2 | ~2 |
| `native+md` | md tasks 36, md set-task 12, md outline 2, cd 7 — **md only, zero native Edit** | ~4 |

The agent ran `md tasks` ~3× per mutation (discover the loc, then `md set-task`),
vs native's Read-then-Edit. **But this is the policy the *prompt* induced**, not
proof the agent weighed both tools — native Edit was available and unused.

## Confounds that block the strong claim (mine + GPT-5 Pro's)

1. **The prompt asymmetry IS the treatment, not a side-confound.** `native+md`
   differs from `native` in four ways (md exists, prompt longer, prompt advertises
   md, agent invited to a new policy). So this measures the *bundle*, not md.
2. **The clean ablation was bypassed** (`./md` stub bug, fixed in `b362292` but the
   *data* is already contaminated): native+md-no-md ran real md, so it ≈ native+md
   (+27% billed). There is **no valid md-causal lift estimate** from this run.
3. **Ceiling effect**: 45/45 pass → the design *cannot* show correctness lift, only
   cost. It is structurally biased toward the simplest adequate tool; a CLI can't
   beat a first-class `Edit` on trivial single-edits.
4. **"native Edit" is mislabeled** — the baseline used Read + Edit + **grep** (native
   toolchain, not native Edit alone; grep did some of the location work).
5. **Induced policy ≠ tool capacity**; the stub is itself an intervention ("broken
   advertised md", needs a SUSPECT path); Batch n=1 is anecdotal; medians hide tails;
   `md tasks` ×3/mutation may be a *docs* problem (inefficient usage), not a tool
   penalty; **one model, one runtime (Sonnet 4.6 / Claude Code) ≠ "frontier" generally.**

## What is honestly claimable today

- In this sweep, all modes passed; under **billed-$** the md-advertising config cost
  **~28% more** than the native toolchain on easy edits (down from raw +73%).
- The `NATIVE_MD_DOCS` prompt **strongly steered tool choice** — Sonnet abandoned
  native Edit entirely. As a *default*, that prompt looks net-harmful on simple edits.
- **NOT** claimable: md intrinsically loses; md is net-negative; the attribution gate
  was validly exercised; frontier agents generally don't benefit; the clean ablation
  shows no lift.

## What a bulletproof re-run needs (FRAC follow-up)
Billed-$ as the primary metric (done here for re-score; wire it into the gate);
the **fixed** ablation with a **preflight no-md proof** in every artifact
(`command -v md` / `./md` / hash / alias — fail closed); a small prompt factorial to
decompose prompt-length vs md-doc-steering vs functional-md (A native-docs/no-md,
B +length-matched neutral/no-md, C +md-docs/no-md, D +md-docs/md, E native-docs/md);
paired per-task stats (ratios, bootstrap CI, tails, sign count); and harder task
strata (many checkboxes, large files, tables, section moves, duplicate headings)
where native Edit is *not* trivially sufficient.

(Harness fix `8de8922`: claude-cli's Bash tool doesn't source `BASH_ENV`, so the
guard never fires for claude-cli — prior claude-cli tool_mix/mutations were
undercounted; this run uses the transcript-derived signal.)
