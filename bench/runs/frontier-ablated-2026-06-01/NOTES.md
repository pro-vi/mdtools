# Frontier ablated-baseline re-run + cache-regime analysis — 2026-06-01

The first valid md-attribution measurement (after the `./md` ablation fix
`611c2c3` on top of the isolation fix `4e20adf`), **plus** the cache-regime
analysis that followed once we noticed the cost metric was 84% cache-read.

> **TL;DR.** Under the gate, **no targeted-edit cell ever closes** (any model,
> any cache regime). The one frontier `CLOSES` is **Sonnet · Batch**, and it is
> **cache-regime-conditional**: it closes under realistic warm/cached pricing
> (Anthropic read ≈ 0.1×, robust until the read price hits ~0.42× fresh) and is
> `SUSPECT` only in a cold / one-off / cache-disabled world. The stronger Opus
> does not close Batch even warm. **`md ∝ 1/capability` holds as a gradient;**
> the one win is repeated-use-only and **n=1-provisional**.

## 1. Why this run exists

The earlier clean re-run (`../frontier-clean-2026-06-01`) was isolated correctly
but its `hybrid-no-md` baseline was **bypassed**: the harness copies `md` to
`./md` in the workdir and the prompt advertises `./md`, so a no-md agent that ran
`./md` (as told) hit the **real** binary. `611c2c3` makes the `./md` copy the
soft stub in `hybrid-no-md` too. Verified: a no-md agent now runs `./md tasks` →
`md: unavailable here…` → falls back to grep/sed, `md_probe_count == 1`.

## 2. Setup (minimal, reuse-where-sound)

`611c2c3` changes **only** the `hybrid-no-md` branch; `unix`/`hybrid` code paths
are byte-identical, so those bundles are **reused** from the clean run and only
`hybrid-no-md` was re-measured: `T7,T10,T13,T20` (Targeted, n=4) + `T12` (Batch,
n=1) × N=3, two models. 30 runs, **0 errors, all 3/3 correct, every
`md_probe_count == 1`**. Render is per-model (both models are tier `frontier`).

## 3. The cache-accounting discovery (what reframed everything)

The cost basis is `tokens_in = input + cache_creation + cache_read`, all weight
1.0. But **84% of those tokens are cache_READS** (the static md-docs prompt
prefix re-read every turn), and reads bill at ~0.1×. So raw-tokens over-weights
the prompt-heavy modes (`hybrid`, `hybrid-no-md`) ~3–4× vs what you actually pay.

Key fact: **prompt caching is output-invariant** — the model can't see cache
state, so token counts and pass/fail are identical across regimes; only the
*price* of the prefix moves. So "which cost basis" **is** "which cache regime",
and every regime is reconstructable by re-pricing the logged per-run usage
(`usage_breakdown.json`, all 90 runs) — no re-run needed:

`cost(r) = input + cache_creation + r·cache_read + output`
- `r = 1.0` → cold / cache-disabled = raw tokens = a **one-off** invocation
- `r = 0.1` → Anthropic warm read price = **repeated / cached** production use

Reproduce: `python3 regime_sensitivity.py`.

## 4. Result — verdict vs cache regime (the main finding)

| cell | COLD / one-off (r=1, raw tokens) | WARM / repeated (r=0.1) | breakpoint r* | regime-robust? |
|---|---|---|---|---|
| Sonnet · Targeted | `OPEN:loses-unix` | `OPEN:loses-unix` | — | ✅ never closes |
| **Sonnet · Batch** | `SUSPECT:baseline-flails(cost)` | **`CLOSES`** | **≈0.42** | ⚠️ **regime-dependent** |
| Opus · Targeted | `SUSPECT:baseline-flails(cost)` | `SUSPECT:baseline-flails(cost)` | — | ✅ un-attributable |
| Opus · Batch | `OPEN:loses-unix` | `OPEN:no-lift` | (n=1 `CLOSES` sliver ~r∈[0.10,0.24] fine sweep; coarse 0.05-grid shows 0.15–0.20 — starts just above the real warm r≈0.1, where it's no-lift) | ✅ no clean win at the real operating point |

Sonnet · Batch flips at **r\* ≈ 0.42**. Anthropic's real read price is **r ≈ 0.1**
— far below 0.42 — so under realistic caching it `CLOSES` with margin (cached
reads would have to cost **several×** their actual price to break it). The per-run
separation is clean (every Sonnet hybrid Batch run cheaper than every unix run:
hybrid $0.064–0.076 vs unix $0.113–0.119), not a cold/warm fluke. The warm
result is **conservative** — true steady-state amortizes run-1's cache-creation,
which would make `md` look *better*, not worse.

## 5. Adversarial second opinion (GPT-5 Pro, extended thinking)

Consulted neutrally (three bases as competing hypotheses). It **rejected
cache-weighted tokens as the primary basis** (it smuggles a provider-specific
0.1× coefficient while pretending neutrality, and isn't truly dollars either) and
recommended **billed `$` as the primary product-cost metric**, with raw tokens
**renamed to a "context-footprint" diagnostic** (not "cost") and a cache-read
weight *sweep* as the sensitivity view (§4 above). Deeper structural points it
raised, logged as follow-ups (§7):

- **Cache state is an experimental *treatment*, not a logging detail** — declare
  a regime (cold / warm steady-state / amortized-over-K) instead of the current
  uncontrolled N=3 back-to-back mix.
- The **both-passed intersection** conditions on tasks both modes solved, so it
  structurally misses `md`'s value when `md` changes *which* tasks pass — prefer
  **expected cost per successful task**.
- The **no-md ablation is contaminated** (blends docs-effect + tool-effect +
  dead-probe-effect); factorialize into `unix / docs-only / stubbed-tool /
  hybrid` to separate them.
- A single cost scalar hides the real shape — `md` buys **reliability / edit
  locality / batch efficiency**, not universal cost reduction.

## 6. Honest headline (regime-aware)

> **`md` earns no clean frontier win on targeted edits — any model, any cache
> regime.** On **batch** structural ops, `md` cleanly wins on the mid-frontier
> model (**Sonnet**) **for repeated / cached use** (`CLOSES` at the realistic
> r≈0.1, robust to r<0.42), but not for a cold one-off, and the stronger **Opus**
> does not close even warm. The **`md ∝ 1/capability` gradient holds**; the one
> frontier win is cache-regime-conditional and **n=1-provisional**.

This supersedes the prior absolute line "no frontier cell closes / `md` earns no
clean win on a strong model" — true only under cold/raw-token accounting.

## 7. Caveats & follow-ups

- **Batch is n=1** (T12 only). The `CLOSES` is **provisional** until Batch has
  more tasks — queued as a follow-up (generate + N≥3-measure new batch-mutation
  tasks; that kills the single-task uncertainty before the win is asserted hard).
- Pin the **cache regime** as a declared treatment (§5); adopt **billed `$`** as
  primary with raw-tokens relabeled "context footprint"; consider
  **expected-cost-per-success** over the both-passed intersection; **factorialize
  the ablation** (docs-only vs stubbed-tool). These are gate/metric design
  changes — deferred, not made unilaterally here.
- `unix`/`hybrid` reused from the clean run (unchanged code path); only
  `hybrid-no-md` re-measured. All N=3.
