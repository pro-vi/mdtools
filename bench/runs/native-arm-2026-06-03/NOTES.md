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
2. **The clean ablation was bypassed on TWO axes** (so the +27% no-md ≈ +md figure is
   meaningless): (a) the `./md` workdir copy was the real binary, fixed in `b362292`;
   (b) **deeper** — a later cross-model review found that even with `./md` stubbed, the
   agent reaches **bare `md`** (the form `NATIVE_MD_DOCS` advertises) which resolves to
   the **real md on the system PATH**, because claude-cli's Bash never sources `BASH_ENV`
   so the guard's PATH restriction never fires. b362292 did **not** close (b). Both are
   now fixed in code (workdir prepended to PATH so bare `md`→the workdir stub); but this
   run's *data* is contaminated on both axes. **No valid md-causal lift estimate** from
   it, and a re-run was impossible-to-trust until the PATH fix landed.
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

## Mechanism — why md costs more (code/architecture/model)

Grounded in the transcripts (the *why*, not just the *what*). The +28% billed delta
decomposes as **~52% cache_read, ~30% output, ~18% cache_creation** (median Targeted:
cache_read 1.86×, output 1.74×, turns 1.67×, **cache_creation only 1.11×**):

- **Architecture (dominant).** md is **stateless + Bash-mediated**. `md set-task` has
  no "where am I" context, forcing a **discover→mutate→verify loop**: `md tasks` (find
  the loc) → `md set-task <loc>` → `md tasks` (verify). Each is a Bash round-trip, and
  `md tasks --json` returns a ~1,600-token blob (`loc/status/span/heading_block_index`
  per task, ~4× the native Read) that **re-enters cached context every turn**
  (cache_read 1.86×). Native `Edit` **fuses** discovery+mutation — it addresses by
  *string* on content already Read, so no round-trip and a compact result. That
  cache_creation is only 1.11× is the tell: the cost is **re-reading the verbose md
  output**, not the bigger prompt. (One run even shows `md tasks --json | jq` failing
  on a schema mismatch, then retrying — a wasted call.)
- **Model behavior.** Every native+md run was **Bash-only, zero native Edit** — the
  ~17-line md reference + "md handles structural markdown operations" *induced* an
  exclusive-md policy, talking a capable model out of its fluent first-class editor.
- **Task-fit (root — `md ∝ 1/capability` made concrete).** md is a structural-
  complexity tool (loc addressing, atomic `move-section`, drift-safe re-query, etag).
  The anchor tasks toggle ONE checkbox / mark one section — none need any of that — so
  md's machinery is **pure overhead with no offsetting benefit** (CLAUDE.md: "don't
  replace `sed` for simple edits"), and 45/45-pass means there's no *correctness* win
  for that cost to buy back.

**Where md would flip positive** (the inverse the mechanism predicts): section/
subsection moves (atomic `move-section` vs delete+reinsert+re-derive locs), multi-step
edits with inter-edit drift (re-query beats stale-loc failures), duplicate-heading/
content files (Edit's `old_string` is ambiguous; loc+etag precise), large files
(`md section`'s slice vs a context-blowing Read), and weaker models that fail
structural tasks outright (md buys a correctness lift native lacks). None are in this
task set — which is *why* the easy-edit ceiling makes md look purely costly here.

## What a bulletproof re-run needs (FRAC follow-up)
Billed-$ as the primary metric (done here for re-score; wire it into the gate);
the **fixed** ablation with a **preflight no-md proof** in every artifact that checks
**both axes** — `./md` **and** bare `md` (`command -v md`, `md --version`, hash, alias)
must all hit the stub / fail closed, not just `./md` (the PATH-axis bypass, now closed
by the workdir-PATH-prepend, is exactly what a single-axis preflight would have missed);
a small prompt factorial to
decompose prompt-length vs md-doc-steering vs functional-md (A native-docs/no-md,
B +length-matched neutral/no-md, C +md-docs/no-md, D +md-docs/md, E native-docs/md);
paired per-task stats (ratios, bootstrap CI, tails, sign count); and harder task
strata (many checkboxes, large files, tables, section moves, duplicate headings)
where native Edit is *not* trivially sufficient.

(Harness fix `8de8922`: claude-cli's Bash tool doesn't source `BASH_ENV`, so the
guard never fires for claude-cli — prior claude-cli tool_mix/mutations were
undercounted; this run uses the transcript-derived signal.)
