# Multifile Drift Probe Results

| model | task | mode | n | pass | pass^n | alt counts | proof OK | md before mutation | clobbers | notes |
| --- | --- | --- | ---: | ---: | --- | --- | ---: | ---: | ---: | --- |
| omlx/Qwen3.6-35B-A3B-8bit | MF-ETAG-01 | native | 3 | 0 | no | none:3 | 0 | 3 | 3 |  |
| omlx/Qwen3.6-35B-A3B-8bit | MF-ETAG-01 | native+md | 3 | 1 | no | none:2, retry:1 | 1 | 1 | 2 |  |
| omlx/Qwen3.6-35B-A3B-8bit | MF-ETAG-02 | native | 3 | 0 | no | none:3 | 1 | 1 | 2 |  |
| omlx/Qwen3.6-35B-A3B-8bit | MF-ETAG-02 | native+md | 3 | 0 | no | none:3 | 3 | 3 | 0 |  |
| openai-codex/gpt-5.4 | MF-ETAG-01 | native | 3 | 0 | no | none:3 | 0 | 2 | 3 |  |
| openai-codex/gpt-5.4 | MF-ETAG-01 | native+md | 3 | 1 | no | retry:3 | 1 | 3 | 0 |  |
| openai-codex/gpt-5.4 | MF-ETAG-02 | native | 3 | 0 | no | none:3 | 0 | 2 | 3 |  |
| openai-codex/gpt-5.4 | MF-ETAG-02 | native+md | 3 | 0 | no | none:3 | 0 | 1 | 2 |  |

## Kill-Condition Check

- `omlx/Qwen3.6-35B-A3B-8bit`: CLOSED: neither arm reached pass^n reliability; no native+md edge was demonstrated.
- `openai-codex/gpt-5.4`: CLOSED: neither arm reached pass^n reliability; no native+md edge was demonstrated.
