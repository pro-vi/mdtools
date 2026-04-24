## Legacy N=3 Haiku snapshot (2026-04-02)

> **Split disclosure:** this snapshot predates the search/holdout split and uses `bench/tasks/tasks_v1.json` (20-task corpus). T4, T14, T20 appear here among search-mixed rows; they are now members of the current holdout set (`bench/holdout/task_ids.json`), and T22–T24 did not exist yet. Treat the snapshot's per-task numbers as pre-split observations, not as a validated split result for the current thesis.

### Per-task results (N=3, 20-task v1 snapshot)

| Task | hybrid pass | hybrid time | hybrid calls | mdtools pass | mdtools time | mdtools calls | unix pass | unix time | unix calls |
|------|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| T1 | 100% | 15s | 1.0 | 100% | 15s | 1.0 | 33% | 175s | 10.0 |
| T2 | 100% | 18s | 3.3 | 100% | 20s | 3.7 | 0% | 31s | 6.0 |
| T3 | 100% | 37s | 8.3 | 100% | 40s | 9.7 | 100% | 34s | 8.0 |
| T4 | 100% | 36s | 12.0 | 100% | 42s | 10.0 | 100% | 34s | 7.3 |
| T5 | 100% | 16s | 1.0 | 100% | 13s | 1.0 | 0% | 36s | 4.0 |
| T6 | 0% | 49s | 14.0 | 0% | 45s | 13.3 | — | — | — |
| T7 | 100% | 16s | 3.0 | 100% | 16s | 3.0 | — | — | — |
| T8 | 33% | 33s | 8.0 | 33% | 26s | 5.3 | — | — | — |
| T9 | 100% | 19s | 2.0 | 100% | 21s | 2.3 | — | — | — |
| T10 | 100% | 18s | 3.3 | 100% | 18s | 3.0 | — | — | — |
| T11 | 67% | 60s | 4.0 | 100% | 22s | 3.7 | — | — | — |
| T12 | 100% | 26s | 5.7 | 100% | 32s | 6.0 | — | — | — |
| T13 | 100% | 20s | 4.0 | 100% | 17s | 3.3 | — | — | — |
| T14 | 100% | 13s | 1.0 | 100% | 11s | 1.0 | — | — | — |
| T15 | 100% | 35s | 7.0 | 100% | 29s | 7.0 | — | — | — |
| T16 | 100% | 24s | 5.7 | 100% | 27s | 7.3 | — | — | — |
| T17 | 100% | 20s | 2.0 | 100% | 15s | 2.0 | — | — | — |
| T18 | 100% | 25s | 5.7 | 100% | 22s | 5.7 | — | — | — |
| T19 | 33% | 22s | 2.7 | 0% | 19s | 2.7 | — | — | — |
| T20 | 100% | 26s | 5.3 | 100% | 23s | 4.7 | — | — | — |

### Aggregate

| Mode | Pass% | Avg Time | Avg Calls | Avg Obs KB | Requery% |
|------|------:|--------:|---------:|----------:|--------:|
| hybrid | 87% | 26s | 5.0 | 8KB | 62% |
| mdtools | 87% | 24s | 4.8 | 7KB | 58% |
| unix | 50% | 64s | 7.3 | 4KB | 7% |

### By task family

| Family | Tasks | hybrid | mdtools | unix |
|--------|------:|---:|---:|---:|
| Extraction | 6 | 83% (n=18) | 83% (n=18) | 20% (n=5) |
| Targeted mutation | 4 | 100% (n=12) | 100% (n=12) | — |
| Batch mutation | 1 | 100% (n=3) | 100% (n=3) | — |
| Multi-step | 2 | 100% (n=6) | 100% (n=6) | — |
| Content delivery | 4 | 83% (n=12) | 83% (n=12) | 50% (n=6) |
| Safe-fail | 1 | 100% (n=3) | 100% (n=3) | — |
| Text manipulation | 2 | 50% (n=6) | 50% (n=6) | 100% (n=3) |

*Generated from N=3 Haiku 4.5 runs on 2026-04-02 against `bench/tasks/tasks_v1.json` (20-task snapshot). Unix mode incomplete — only T1-T5 finished (14/60 runs). T5 run 3/3 timed out. Unix aggregates are over completed runs only.*

## Cross-model matrix (N=1, 20-task v1 snapshot)

Separate single runs per model. Not part of the N=3 Haiku dataset above.

| Model | unix | hybrid | Δ | Tool value |
|-------|-----:|-------:|--:|------------|
| Haiku 4.5 | 50% | 87% | +37pp | Correctness + speed |
| Sonnet 4.6 | 80% | 85% | +5pp | Slight correctness lift + speed |
| Opus 4.6 | 89% | 83% | -6pp | Efficiency only |

Sonnet timing evidence (N=1, selected structural tasks):

| Task | Sonnet unix | Sonnet hybrid | Speedup |
|------|------------|--------------|---------|
| T9 (extraction) | 111s / 5 calls | 22s / 2 calls | 5.0x |
| T11 (aggregation) | 95s / 9 calls | 28s / 2 calls | 3.4x |
| T12 (batch) | 149s / 6 calls | 26s / 3 calls | 5.7x |
| T18 (re-query) | 72s / 10 calls | 22s / 4 calls | 3.3x |

The historical frontier-model snapshot suggests weaker frontier models gain more correctness while stronger ones mostly gain speed, but the local search-pilot matrix below shows that mode choice also depends strongly on model family and task family.

## Local OAI search-pilot matrix (2026-04-21, search split)

These results come from committed durable bundles under `bench/runs/` on the default 24-task corpus's search split. They are narrower pilot manifests, not a rerun of the full 20-task v1 snapshot above.

> **Holdout status (2026-04-24):** first holdout run against Qwen3.5-122B-A10B-4bit (both modes) scored **50%** against the search-pilot's 100%. The drop traces primarily to scorer-surface defects surfaced for the first time; see the "Holdout confirmation" section at the bottom of this file. Remaining cells are search-split observations without matching holdout bundles and are not durable claims.

### Extraction pilot (T1, T9, T16)

| Model | unix | mdtools | hybrid | Notes |
|-------|-----:|--------:|-------:|-------|
| Qwen3.5-27B-4bit | 0% (0/3, 411s) | 100% (3/3, 70s) | 100% (3/3, 71s) | Clear tool-value lift over unix; hybrid ties mdtools |
| Qwen3.5-122B-A10B-4bit | — | 100% (3/3, 30s) | 100% (3/3, 34s) | Fastest passing local model on the committed extraction pilot |
| Hermes-4-70B-4bit | — | 67% (2/3, 169s) | 33% (1/3, 98s) | Hybrid underperforms mdtools on extraction |
| magnum-v4-123b-4bit | — | 33% (1/3, 621s) | 33% (1/3, 358s) | Hybrid fails faster, but does not improve correctness |

### Targeted mutation pilot (T7, T10, T13)

| Model | mdtools | hybrid | Requery | Notes |
|-------|--------:|-------:|--------:|-------|
| Qwen3.5-27B-4bit | 100% (3/3, 41s) | 100% (3/3, 41s) | 67% / 67% | Stable tie between mdtools and hybrid |
| Qwen3.5-122B-A10B-4bit | 100% (3/3, 33s) | 100% (3/3, 29s) | 100% / 100% | Strongest committed mutation cell |
| Hermes-4-70B-4bit | 67% (2/3, 28s) | 33% (1/3, 126s) | 0% / 0% | Hybrid underperforms mdtools on mutation too |
| magnum-v4-123b-4bit | 100% (3/3, 128s) | 100% (3/3, 145s) | 67% / 33% | Correct in both modes, slower in hybrid |

### Multistep pilot (T15, T18)

| Model | mdtools | hybrid | Requery | Notes |
|-------|--------:|-------:|--------:|-------|
| Qwen3.5-27B-4bit | 100% (2/2, 64s) | 100% (2/2, 59s) | 100% / 100% | First executable moat-use evidence on the search split |
| Hermes-4-70B-4bit | 100% (2/2, 34s) | 100% (2/2, 29s) | 100% / 100% | Removes the Hermes hybrid deficit on this family |

### Local search-pilot takeaways

- `mdtools` materially improves the extraction pilot for Qwen3.5-27B-4bit versus raw unix (`0/3` to `3/3`).
- `hybrid` is not a universal upgrade: it ties `mdtools` for the Qwen-family pilots, underperforms for Hermes on extraction and mutation, and is correctness-neutral but slower for magnum on mutation.
- Qwen3.5-122B-A10B-4bit is strong and fast across every committed pilot cell in this branch; its 100% search-pilot number dropped to 50% on holdout. The holdout gap is unresolved pending a mode-neutral scorer fix (F3 in `bench/ledger.md`). See "Holdout confirmation" below.
- magnum-v4-123b-4bit is strong on mutation but weak on extraction, with the failure mode now visible in durable bundle metrics instead of only raw logs.

## Holdout confirmation (2026-04-24, holdout split)

The holdout split is `bench/holdout/task_ids.json` (T4, T14, T20, T22, T23, T24). It is disjoint from the search pilots above and was not optimized against during search. Bundles land under `bench/runs/holdout-*`.

> **Process note (2026-04-24):** an earlier iteration edited T22's task description after observing its holdout failure. T22 is in the holdout set, so that edit constituted post-hoc tuning of a holdout task and the subsequent "post-fix" bundles are **not valid holdout confirmation**. T22's description has been reverted to its original wording. The four invalid bundles have been moved to `bench/retracted_2026-04-24/` (outside `bench/runs/`) so automated analysis tools do not pick them up; see that directory's `README.md` for details.

### Qwen3.5-122B-A10B-4bit (the best-in-class search cell)

| Mode | Search pilots (N=1) | Holdout (N=1, valid) | Δ |
|------|-----:|-----:|---:|
| mdtools | 100% (6/6, ~32s avg) | 50% (3/6, 73s avg) | **−50pp** |
| hybrid  | 100% (6/6, ~32s avg) | 50% (3/6, 68s avg) | **−50pp** |

Valid bundles: `bench/runs/holdout-{mdtools,hybrid}-Qwen3.5-122B-A10B-4bit-2026-04-24/`.

**Per-task failure analysis (mode-independent — identical pass/fail across mdtools and hybrid):**

| Task | Result | Root cause | Appropriate fix layer |
|------|:---:|---|---|
| T4  | FAIL | Text-manipulation family — documented weak cell; agent thrashed at 8/10 mutations. | Product / planning (out of scope this iteration) |
| T22 | FAIL | Task description says "correctness is the ordered list of link kinds and destinations" → agent emits `[{kind, destination}]` list; structural scorer requires a top-level `mdtools.v1` envelope object. | Mode-neutral: scorer should accept semantically equivalent shapes, or system prompt's `json_envelope` instruction should describe envelope shape mode-agnostically. **Not** task-description rewrite (that contaminates the holdout task and leaks mdtools-specific guidance into unix mode). Currently OPEN as F3. |
| T23 | FAIL pre-fix | Agent emitted 54 bytes vs 55 expected (trailing-newline off-by-one); `raw_bytes` scorer had `ignore_trailing_whitespace: true` but implemented per-line rstrip only, ignoring end-of-file whitespace. | `bench/harness.py:339-341` — rstrip the whole string after per-line rstrip. This is a mode-neutral scorer correction (accepted). |

### Retracted bundles (moved to `bench/retracted_2026-04-24/`)

Four bundles produced after the T22 description edit have been moved out of `bench/runs/` so `bench/analyze.py`, `bench/report.py`, and glob-based tooling do not treat them as valid evidence. See `bench/retracted_2026-04-24/README.md` for the full list and reason. Do not cite any pass rate from those bundles.

### Current holdout coverage (valid bundles only)

| Cell | Search → holdout | Status |
|------|---|---|
| Qwen3.5-122B-A10B-4bit mdtools | 100% → 50% | **Valid; −50pp drop unconfirmed as product vs scorer because T22 scorer-layer fix is pending** |
| Qwen3.5-122B-A10B-4bit hybrid  | 100% → 50% | Valid; same caveat |
| Qwen3.5-27B-4bit (either mode) | — | No valid holdout bundle |
| Hermes-4-70B-4bit (either mode) | — | No holdout bundle |
| magnum-v4-123b-4bit (either mode) | — | No holdout bundle |

**What this confirms honestly.** The un-run holdout was hiding evaluator-trust defects that the search split happened not to exercise. The first holdout run made those defects visible. Two of three failures (T22, T23) trace to scorer surfaces; one (T23) has a mode-neutral fix that is retained. T22 still needs a mode-neutral resolution before any holdout rerun can validly reconfirm Qwen3.5-122B-A10B-4bit search-pilot numbers. Until then, the published holdout pass rates stand at 50% for both modes.
