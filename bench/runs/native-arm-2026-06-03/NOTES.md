# Native-rooted arm — first live frontier sweep (2026-06-03)

FRAC-194 U7. `claude-cli` (Sonnet 4.6), N=3, the 5 fixed anchor tasks
(T7/T10/T13/T20 = Targeted, T12 = Batch) × 3 native modes. **45/45 sessions PASS,
0 errors.**

## Headline — `md` *loses* to native `Edit` at the frontier

| cell | `native_root` verdict | `native` cost | `native+md` cost |
|---|---|---|---|
| Targeted mutation (n=4) | **`OPEN:loses-native`** | 38,568 | 66,603 (**+73%**) |
| Batch mutation (n=1) | **`OPEN:loses-native`** | 49,781 | 59,235 (**+19%**) |

`native+md` is **more expensive** than `native` (native `Edit` alone). `md` does
**not** help an agent that already has native `Edit` — it costs *more*. This is a
**Pareto fail** (decided before the lift/ablation is consulted), so it is robust to
the ablation bug below. The **`md ∝ 1/capability`** thesis is confirmed and
*strengthened*: at the frontier `md` is **net-negative**, not merely no-lift.

## Adoption — `md` advertised ⇒ `md` used exclusively (zero native `Edit`)

| mode | tool_mix (transcript-derived) |
|---|---|
| `native` (no md) | `Read 15, Edit 18, grep 2` — native tools, ~2 calls/task, cheap |
| `native+md` | `md tasks 36, md set-task 12, md outline 2, cd 7` — **md only, zero native Edit**, 12 mutations |

With `NATIVE_MD_DOCS` advertising the md subcommands, Sonnet used `md` for
everything — including `md set-task` (a mutation verb) — and **never touched native
`Edit`** — but that `md` path cost ~73% more than the native-`Edit` path. `md`'s
availability + advertisement *pulled* the agent into a costlier route. (Contrast the
2026-05-28 verb-adoption memo, which observed frontier agents using native Edit over
md — there, md wasn't this prominently advertised in-prompt. The lesson: adoption is
prompt-sensitive; the *cost* verdict is not.)

## Bug found + fixed (`b362292`) — `native+md-no-md` ablation bypass

`native+md-no-md` cost ≈ `native+md` (66,599 ≈ 66,603) — the clean ablation isolated
nothing. The `./md` workdir-copy stub was gated on `mode == "hybrid-no-md"` only, so
`native+md-no-md` got the **real** md via `./md` (the `611c2c3` bypass, not extended
to the new mode). Fixed. **Does not change the verdict** (Pareto-fail). A re-run
would give a clean — but, here, *unused* — lift baseline.

## Caveat / what a re-run would add

The clean ablation (`native+md-no-md`, now fixed) is what separates "`md` is
intrinsically costlier" from "the prompt nudged `md`." But since `native+md` already
**loses** `native` on Pareto, the deployment takeaway holds either way: **adding +
advertising `md` made the frontier agent more expensive, not less.** Also: harness
fix `8de8922` — claude-cli's Bash tool doesn't source `BASH_ENV`, so prior claude-cli
`tool_mix`/`mutations` were undercounted; this run uses the transcript-derived signal.
