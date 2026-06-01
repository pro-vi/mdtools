# Frontier clean-measurement re-run — 2026-06-01

Validation after the claude-cli **isolation fix** (`4e20adf`, PR #10 / Codex P1).

## Why

The prior frontier evidence was measured with `claude -p … --settings ""`, which did
**not** isolate: every cell discovered ~94 slash-commands, 5 agents, 6 MCP servers +
CLAUDE.md (~32k input tokens). Critically the **mdtools CLAUDE.md documents `md`** and
tells the agent to prefer it, so that text leaked into the `unix` and `hybrid-no-md`
baselines — a mode-isolation breach. Fix: keep real HOME (auth) + clean cwd and use
`--disable-slash-commands --strict-mcp-config --setting-sources "" --agents "{}"`
(keeps token usage, unlike `--bare`). Trivial prompt: 32k → ~3.5k input tokens,
discovery 0, model confirms no md/mdtools docs in context.

## Setup

`T7,T10,T13,T20` (Targeted mutation, n=12) + `T12` (Batch, n=3) × {unix, hybrid,
hybrid-no-md} × N=3, isolated claude-cli. Two models to separate *contamination* from
*model-capability*: `claude-sonnet-4-6` (matches the contaminated baseline) and
`claude-opus-4-8` (a stronger model).

## Result (median cost on the both-passed intersection; verdict per `attribution_verdict`)

| cell | Sonnet contaminated | **Sonnet clean** | **Opus clean** |
|---|---|---|---|
| Targeted mutation | `loses-unix` (hybrid +56%) | `loses-unix` (hybrid +23%; 48154 vs 59350) | `no-lift` (hybrid −5%; 16058 vs 15314) |
| Batch mutation | `CLOSES` (hybrid −30%) | `SUSPECT:baseline-flails(cost)` (no-md +31% over unix) | `loses-unix` (hybrid +27%; 19745 vs 25060) |

## Findings

1. **`md ∝ 1/model-capability` holds and is cleaner.** Under the gate, **no frontier
   cell CLOSES** on either strong model. The stronger the model the less md helps:
   Sonnet *taxes* (Targeted +23%) → Opus *neutral* (`no-lift`).
2. **Contamination effect, isolated (Sonnet vs Sonnet):** it *exaggerated magnitude*
   (Targeted +56% → +23%) and *flipped one verdict* (Batch `CLOSES` → `SUSPECT`). It did
   NOT flip Targeted — that cell is `loses-unix` clean too.
3. **The "md wins batch-structural ops even for a strong model" claim was an
   artifact.** Clean, Batch never closes on the frontier (`SUSPECT` on Sonnet — the
   `hybrid-no-md` ablation flails +31%, so the apparent md-lift can't be credited;
   `loses-unix` on Opus). md's batch advantage is **weak-model only**. README corrected.

Caveat: Batch is a single task (n=1 intersection); Targeted is n=12. All runs N=3.
