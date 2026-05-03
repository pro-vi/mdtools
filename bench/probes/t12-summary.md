# T12 Halt Summary

_In progress. Updated when halt fires._

## Launch state

- Phase: `steady-state`
- Fixed-anchor baseline: **18 tasks** (stamp from T10-10)
- Fixed-anchor gap: **+38.9pp** (inherited from T11, no movement)
- Current-corpus gap: **+45.0pp** on **20 tasks**
- Primary model: `Qwen3.5-27B-4bit`
- Cross-model: `Qwen3.5-122B-A10B-4bit` (trigger: fixed-anchor ≥+5pp)
- OAI endpoint: `http://localhost:10240/v1`, timeout 180s
- Auto-research: `bench/auto_research.py` — verified end-to-end
- Halt #1 saturation counter: 0 (infra iterations non-counting)
- Lock-blocked counter: 0

## Iter 1 mandatory product-axis sweep

Candidates to re-run before generating new candidates:

| Candidate | Prior status | Shape |
|---|---|---|
| `certificate-rotation-runbook-relocation` | `rejected-lock-blocked` | subsection relocation |
| `pager-rotation-review-relocation` | `rejected-lock-blocked` | subsection relocation |
| `getting-started-installation-relocation` | `rejected-planning-mdtools-fail` | subsection relocation |
