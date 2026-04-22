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
- Qwen3.5-122B-A10B-4bit is strong and fast across every committed pilot cell in this branch.
- magnum-v4-123b-4bit is strong on mutation but weak on extraction, with the failure mode now visible in durable bundle metrics instead of only raw logs.
