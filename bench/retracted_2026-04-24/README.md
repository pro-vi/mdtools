# Retracted bundles (2026-04-24)

These four bundles were produced during a frontier-loop iteration that edited
the `T22` task description in `bench/tasks/tasks.json` after observing T22's
failure on the first holdout run. Because `T22` is in
`bench/holdout/task_ids.json`, that edit constituted post-hoc tuning of a
holdout task. Any subsequent holdout run against the tuned task is therefore
**not valid as holdout confirmation**.

External review flagged the mistake. The task description has been reverted
and these bundles have been moved out of `bench/runs/` so that `bench/analyze.py`,
`bench/report.py`, and any glob-based tooling do not treat them as valid
evidence. They are retained here only as a record of what happened.

## Contents

- `holdout-mdtools-Qwen3.5-122B-A10B-4bit-2026-04-24-postfix/` — 100% (6/6), invalid
- `holdout-hybrid-Qwen3.5-122B-A10B-4bit-2026-04-24-postfix/`  — 100% (6/6), invalid
- `holdout-mdtools-Qwen3.5-27B-4bit-2026-04-24/`               — 83% (5/6),  invalid
- `holdout-hybrid-Qwen3.5-27B-4bit-2026-04-24/`                — 100% (6/6), invalid

Do not cite any of these pass rates as evidence for durability or model
capability. The only valid holdout bundles from this day are the two pre-fix
bundles under `bench/runs/holdout-{mdtools,hybrid}-Qwen3.5-122B-A10B-4bit-2026-04-24/`
(both 50%). See `bench/ledger.md` F3 for the ongoing scorer-layer fix requirement
and L1 for the loop-level learning.
