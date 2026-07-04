# Long-Document Regime Results

| model | task | mode | n | pass | pass^n | avg byte proxy | avg calls | avg sec | md adopted |
| --- | --- | --- | ---: | ---: | --- | ---: | ---: | ---: | ---: |
| claude-haiku-4-5-20251001 | LD-CHANGELOG-01 | native | 3 | 1 | no | 85570 | 7.7 | 38.9 | 0 |
| claude-haiku-4-5-20251001 | LD-CHANGELOG-01 | native+md | 3 | 0 | no | 125925 | 11.3 | 43.9 | 2 |
| claude-haiku-4-5-20251001 | LD-CHANGELOG-01 | native+md-no-md | 3 | 0 | no | 119601 | 9.7 | 36.4 | 3 |
| claude-haiku-4-5-20251001 | LD-RUNBOOK-02 | native | 3 | 0 | no | 121650 | 9.0 | 59.5 | 1 |
| claude-haiku-4-5-20251001 | LD-RUNBOOK-02 | native+md | 3 | 0 | no | 275490 | 22.0 | 109.0 | 3 |
| claude-haiku-4-5-20251001 | LD-RUNBOOK-02 | native+md-no-md | 3 | 0 | no | 135708 | 11.3 | 59.2 | 3 |
| claude-haiku-4-5-20251001 | LD-SPEC-03 | native | 3 | 0 | no | 69991 | 5.0 | 20.5 | 0 |
| claude-haiku-4-5-20251001 | LD-SPEC-03 | native+md | 3 | 0 | no | 121426 | 10.7 | 42.6 | 3 |
| claude-haiku-4-5-20251001 | LD-SPEC-03 | native+md-no-md | 3 | 0 | no | 95103 | 8.3 | 27.2 | 3 |
| claude-sonnet-4-6 | LD-CHANGELOG-01 | native | 3 | 3 | yes | 37010 | 5.7 | 34.2 | 0 |
| claude-sonnet-4-6 | LD-CHANGELOG-01 | native+md | 3 | 1 | no | 63721 | 9.0 | 43.2 | 3 |
| claude-sonnet-4-6 | LD-CHANGELOG-01 | native+md-no-md | 3 | 0 | no | 44601 | 7.3 | 38.1 | 3 |
| claude-sonnet-4-6 | LD-RUNBOOK-02 | native | 3 | 0 | no | 64776 | 7.0 | 128.4 | 0 |
| claude-sonnet-4-6 | LD-RUNBOOK-02 | native+md | 3 | 0 | no | 121930 | 9.3 | 89.7 | 3 |
| claude-sonnet-4-6 | LD-RUNBOOK-02 | native+md-no-md | 3 | 0 | no | 60878 | 6.0 | 50.0 | 3 |
| claude-sonnet-4-6 | LD-SPEC-03 | native | 3 | 0 | no | 41128 | 6.3 | 39.1 | 0 |
| claude-sonnet-4-6 | LD-SPEC-03 | native+md | 3 | 0 | no | 44014 | 7.0 | 47.2 | 3 |
| claude-sonnet-4-6 | LD-SPEC-03 | native+md-no-md | 3 | 0 | no | 42756 | 6.0 | 26.8 | 3 |

## Model-Level Verdicts

- `claude-haiku-4-5-20251001`: CLOSED — no correctness lift and no byte-cost proxy advantage; no-md control pass=0%, byte proxy=116804, md-attempt rows=9/9.
- `claude-sonnet-4-6`: CLOSED — no correctness lift and no byte-cost proxy advantage; no-md control pass=0%, byte proxy=49411, md-attempt rows=9/9.

## Probe Verdict

CLOSED: native+md showed no correctness lift and no byte-cost proxy advantage on both models.
