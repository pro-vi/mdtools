# STATE — hybrid-attribution-v3

```yaml
goal_version: hybrid-attribution-v3
goal_fingerprint: "bench-v2 @ feat/bench-v2-cost-axis a4686a0; oracle=agg_util.attribution_verdict rendered by report.py COST SLICE; tolerance=5%; N>=3 (hash is a provenance marker, NOT a HEAD/drift assertion — the drift check keys ONLY on goal_version)"
archetype: goal (terminal acceptance loop), frontier-style per-criterion work
iteration: 0
current_cell: AC-frontier-Targeted-mutation   # known seed failure
stuck_attempt_n: 3
last_action: "round-3 consult tightening — md-lift now requires beating the BETTER of unix and hybrid-no-md (closes tolerance-arbitrage: a within-tolerance-degraded ablation can no longer be the source of lift while hybrid just ties unix); probe gate is MAX-across-runs not median ([1,1,5]->SUSPECT); ablation stub de-branded (no 'ablation mode' string to branch on). 154 bench tests green incl. cost-arbitrage->OPEN:no-lift + probe-tail->SUSPECT. agg_util verifier and ACCEPTANCE.md re-synced (oracle-drift guard)."
next_action: >
  Iteration 1: confirm omlx is up; run a first FULL-suite pass (both tiers, 3
  modes, N>=3) to instantiate the AC-{tier}-{cat} inventory from real cells;
  then diagnose AC-frontier-Targeted-mutation — why does claude burn +70k tokens
  using md on T7 vs unix? (read --log-dir trace + per-call token breakdown).

frontload:
  resolved:
    - oracle: bench-v2 cost slice (report.py + agg_util); 7 fixture tests green
    - run commands + two tiers (claude-cli frontier, pi-json Qwen local): in PROMPT.md overlay
    - levers fenced: src/ md + bench/harness.py HYBRID_DOCS/MDTOOLS_DOCS only
    - consult-capability: tier-2 (agentify GPT-Pro present)
    - evaluator-maturity: high
  gaps_flagged:
    - BLOCKED_EXTERNAL risk: omlx server (127.0.0.1:10240) must be running each session
    - soft budget: claude-cli cells cost real API $ — keep iterations small, prefer local/1-task probes
    - local tier is tool_calls basis until AC-INSTR-pi-tokens (pi token capture)
    - claude-cli tool_mix/mutations not captured (cost/tokens ARE) — adoption signal is local-tier-only
    - EVIDENTIARY CAVEAT (round-3 consult, NOT a gate bug): min_overlap=1 is weak
      evidence for 1-task categories (Batch mutation = T12 only) — there is no
      cross-task data to require. A CLOSES on a 1-task cell rests on N>=3 within-task
      replicate stability, not breadth. Treat such cells as lower-confidence; do not
      tighten the gate (it would make the category permanently unclosable).
    - OUT OF SCOPE for the ablation gate (round-3 consult): the gate proves md
      AVAILABILITY mattered on THIS task suite — it does NOT prove md generalized.
      Category-level overfitting (md tuned to fixed benchmark shapes) is only
      caught by hidden/rotating eval tasks — a separate workstream, not gate logic.
      Diagnose root cause, keep md general-purpose, to stay honest meanwhile.

known_baseline:  # n=1 probe 2026-05-29 (NOT a claim — seed evidence only)
  - "frontier T7: unix=82367 tok PASS | hybrid=88867 (+6500) PASS | mdtools=152794 (+70427) PASS  -> hybrid LOSES cost front"
  - "frontier T1: unix FAIL(135k) | mdtools PASS(48k) | hybrid PASS(48k)  -> md helps; no unix-pass intersection"
  - "local  T7: unix=10 calls | hybrid=4 (-6) | mdtools=6 (-4)  -> md HELPS local on calls"
  - "hybrid free-choice adoption (local): md = 42% of 55 tool-calls (md outline 21, jq 20, ...)"

oracle_change_notes: []   # appended inline when a verifier must change (with strictness proof)
```

## Notes for iteration 1

The seed insight to chase: on the **same** task (T7) `md` **helps the local
model** (−calls) but **taxes the frontier model** (+tokens). The frontier tax is
the trap to remove — likely `md`'s output verbosity (a big `--json` payload the
strong model pays to read) or an extra re-query. Diagnose before editing. The
win condition for the frontier cost front is making `md`'s presence cost ≤ unix
— e.g. leaner/streamable output, or HYBRID_DOCS steering claude to `md` only
where it saves tokens. Do NOT close it by making hybrid fail T7.
