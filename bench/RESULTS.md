### Per-task results (N=3)

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

| Family | N | hybrid | mdtools | unix |
|--------|--:|---:|---:|---:|
| Extraction | 6 | 83% | 83% | 20% |
| Targeted mutation | 4 | 100% | 100% | — |
| Batch mutation | 1 | 100% | 100% | — |
| Multi-step | 2 | 100% | 100% | — |
| Content delivery | 4 | 83% | 83% | 50% |
| Safe-fail | 1 | 100% | 100% | — |
| Text manipulation | 2 | 50% | 50% | 100% |

*Generated from N=3 Haiku 4.5 runs on 2026-04-02. Unix mode incomplete (5/20 tasks due to timeouts).*

## Cross-model matrix (N=1, 20 tasks)

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

The benefit is inversely proportional to model capability.
Weaker models gain correctness. Stronger models gain speed.

