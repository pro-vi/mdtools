# Skill Harvest — the hybrid cost tax is mostly PROMPT SIZE, not md output

- **target skill:** mdtools bench-v2 design (`bench/BENCH_V2_ATTRIBUTION.md` /
  `harness.py` HYBRID_DOCS) + the hybrid-pareto loop prompt (`loop/PROMPT.md`
  seed note).
- **observed gap:** The seed note and ACCEPTANCE seed cell framed the frontier
  cost tax as **md output verbosity** ("a big `--json` payload the strong model
  pays to read") or an **extra re-query**. The N=3 frontier sweep (Sonnet 4.6,
  2026-05-30) shows the dominant tax is neither — it is **the size of the
  HYBRID_DOCS tool-reference prompt, re-sent every turn**.
- **evidence iteration:** bootstrap (iteration 0), `loop/runs/init-frontier/`.
  T7: unix and hybrid both do **exactly 5 turns / 4 calls / correct**, yet
  hybrid tok_in = 101k vs unix 81k (**+20k**), and **hybrid == hybrid-no-md ==
  101k**. Since no-md never invokes md, the +20k cannot be md output — it is the
  HYBRID_DOCS prompt (67 lines / ~1284 tok vs UNIX_DOCS 6 lines / ~54 tok)
  re-sent across turns. Second component: T4/T6 (text-manip) hybrid takes 8-15
  turns vs unix 4-5 — the md-heavy prompt steers the model to over-explore
  structural approaches on text tasks (sed/awk would be direct).
- **proposed rule:** When diagnosing a "tool taxes the strong model" cost gap,
  **separate per-turn FIXED cost (prompt/tool-docs size, paid every turn) from
  per-action VARIABLE cost (tool output size, paid per call)** before choosing a
  lever. The cheap discriminator: compare `hybrid` vs `hybrid-no-md` at equal
  turn counts — if they match, the tax is the shared PROMPT, not md. A long
  tool-reference prompt is O(turns) in cost; trimming it is often higher-leverage
  than trimming any single tool's output.
- **why it generalizes:** every agent harness that injects a static tool
  reference pays it on every turn of a multi-turn task. The bigger the reference
  and the more turns, the worse — independent of whether the tool is even used.
  This is a property of turn-based agent loops, not of md.
- **bonus invariant (lever/baseline safety):** `unix` mode uses `UNIX_DOCS` + no
  md, so it is **invariant to both loop levers** (HYBRID_DOCS edits, md src). A
  running sweep's unix records survive a lever edit, and editing HYBRID_DOCS
  while a sweep is in `unix` mode lets the later `hybrid`/`hybrid-no-md` modes
  (separate harness procs) auto-measure the new prompt — one sweep yields a
  consistent post-edit baseline. Useful for sequencing edits against the slow
  (~22h) local sweep without double-running it.
- **accidental-encouragement risk:** trimming HYBRID_DOCS to cut the tax can
  tip into **neutering md** (steer the agent away from md → hybrid≈no-md →
  `OPEN:no-lift`, the cell does NOT close). The harvest rule must always pair
  "lean the prompt" with "keep md the obvious choice for structural ops" — the
  attribution gate is precisely the guard that catches over-trimming.
