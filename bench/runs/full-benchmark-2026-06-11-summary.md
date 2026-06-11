# Full Benchmark Rerun - 2026-06-11

Corpus: `bench/tasks/tasks.json` at current worktree state, 28 tasks.

Modes: `unix`, `mdtools`, `hybrid`.

Runs per task/mode: N=1.

Scorer dry run: `bench/runs/full-corpus-dry-run-2026-06-11`.

Run bundles:
- Haiku: `bench/runs/full-haiku-2026-06-11`
- GPT 5.4 mini: `bench/runs/full-gpt54mini-2026-06-11`

## Aggregate Values

| Model | Runner | Mode | Pass | Pass rate | Avg time | Avg calls | Avg obs | Avg deny | Requery |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| claude-haiku-4-5-20251001 | claude-cli | unix | 15/28 | 54% | 51s | 7.9 | 4KB | 0.0 | 36% |
| claude-haiku-4-5-20251001 | claude-cli | mdtools | 26/28 | 93% | 24s | 5.3 | 7KB | 0.0 | 54% |
| claude-haiku-4-5-20251001 | claude-cli | hybrid | 27/28 | 96% | 29s | 6.7 | 6KB | 0.0 | 46% |
| openai-codex/gpt-5.4-mini | pi-json, thinking=minimal | unix | 15/28 | 54% | 27s | 4.6 | 2KB | 1.1 | 57% |
| openai-codex/gpt-5.4-mini | pi-json, thinking=minimal | mdtools | 24/28 | 86% | 19s | 4.4 | 7KB | 0.5 | 64% |
| openai-codex/gpt-5.4-mini | pi-json, thinking=minimal | hybrid | 27/28 | 96% | 17s | 4.0 | 4KB | 0.4 | 64% |

## Failed Tasks

| Model | Mode | Failed tasks |
|---|---|---|
| Haiku | unix | T1, T2, T5, T8, T9, T11, T15, T17, T21, T22, T23, T24, C-AR-040 |
| Haiku | mdtools | T8, T23 |
| Haiku | hybrid | T8 |
| GPT 5.4 mini | unix | T1, T2, T5, T8, T9, T12, T16, T19, T21, T22, T24, C-AR-040, C-AR-041 |
| GPT 5.4 mini | mdtools | T8, T11, T12, T19 |
| GPT 5.4 mini | hybrid | T8 |

## Notes

- `T8` failed in every model/mode cell in this rerun.
- Haiku shows the strongest `md` lift on this corpus: `unix` 54% to `mdtools` 93% and `hybrid` 96%.
- GPT 5.4 mini matches Haiku in `unix` and `hybrid` pass rate, but its `mdtools` mode is lower at 86%.
- GPT 5.4 mini produced guard denials in all modes; Haiku produced only one denial, in hybrid T1.
- `bench/analyze.py` and `bench/report.py` were updated so candidate task IDs such as `C-T10-28` and `C-AR-040` are sortable/reportable.
