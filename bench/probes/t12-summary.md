# T12 Halt Summary

## Halt record

- **Halt type:** #1 — Gap saturation (3 consecutive stalled corpus-growth iterations)
- **Halt fired after:** T12-5 (2026-05-03)
- **Final fixed-anchor gap:** +38.9pp (unchanged since T11)
- **Final current-corpus gap:** +45.0pp (20 tasks, unchanged)
- **New corpus members added:** 0
- **Saturation counter at halt:** 3/3

## Launch state

- Phase: `steady-state`
- Fixed-anchor baseline: **18 tasks** (stamp from T10-10)
- Fixed-anchor gap: **+38.9pp** (inherited from T11, no movement)
- Current-corpus gap: **+45.0pp** on **20 tasks**
- Primary model: `Qwen3.5-27B-4bit`
- Cross-model: `Qwen3.5-122B-A10B-4bit` (trigger: fixed-anchor ≥+5pp)
- OAI endpoint: `http://localhost:10240/v1`, timeout 180s
- Auto-research: `bench/auto_research.py` — verified end-to-end

## Iteration log

| T12 Iter | Cause | Candidate | Status | Saturation counter |
|---|---|---|---|---|
| T12-1 | product (mandatory sweep) | 3 lock-blocked reruns | all rejected | non-counting |
| T12-2 | infra | `auto_research.py` hardening | — | non-counting |
| T12-3 | corpus-growth | `relocate-authentication-section-under-user-management` | rejected-cross-seed-instability | 1/3 |
| T12-4 | corpus-growth | `relocate-metrics-section-into-reliability-heading` | rejected-hybrid-fail-no-gap | 2/3 |
| T12-5 | corpus-growth | `reorder-changelog-sections-chronologically` | rejected-cross-seed-instability | 3/3 → **HALT** |

## Iter 1 mandatory product-axis sweep

| Candidate | Prior status | T12-1 result |
|---|---|---|
| `certificate-rotation-runbook-relocation` | `rejected-lock-blocked` | rejected-AST-structural |
| `pager-rotation-review-relocation` | `rejected-lock-blocked` | rejected-hybrid-fail-no-gap |
| `getting-started-installation-relocation` | `rejected-planning-mdtools-fail` | rejected-both-pass-no-gap |

## Diagnosis

The subsection-relocation candidate family (the gap source identified in F10-1) continues to fail the promotion gate. The pattern across T12:

- **Cross-seed instability** is the dominant failure mode: candidates pass seed-1 but hybrid drops to FAIL on seeds 2-3. The gap is real on one seed but the task is borderline enough that small model variation flips the hybrid agent.
- **No-gap candidates**: some tasks are solved trivially by both unix and hybrid, confirming the generator is not consistently targeting the difficulty sweet spot.
- `md move-section` shipped in T11 but the corpus has not grown — the model still cannot reliably chain the tool for multi-heading relocation patterns.

## Recommended next actions

1. **Strengthen the generator prompt** to target subsection-relocation tasks with longer headings and deeper nesting (harder for unix, structure-accessible for mdtools).
2. **Raise N=3 promotion gate to N=5** or apply a quorum threshold (3/5) instead of any-fail rejection.
3. **Inspect cross-seed agent traces** for the 2 rejected-cross-seed-instability candidates to understand where hybrid drops — likely the same `section` command arg-ordering failure.
4. Consider T13 with a fresh model variant (e.g., `Qwen3.6-35B-A3B-8bit`) to shift the cross-seed instability distribution.
