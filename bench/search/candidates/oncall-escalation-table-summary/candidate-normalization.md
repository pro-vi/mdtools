# Candidate Normalization

- Normalized generated family slug `doc-maintenance` to `oncall-escalation-table-summary`.
- Normalized generated task id `mdt-2023-05` to benchmark id `C-T10-14`.
- Replaced the generated metrics table with an on-call handoff table to match the prompt's on-call/release-engineering target.
- Added an explicit `## Active Escalations` heading, a blockquoted fake table, and a fenced fake table.
- Added escaped-pipe and inline-code-pipe cells to exercise real Markdown table parsing rather than simple pipe splitting.
- Made the row-selection predicate explicit: include rows whose Severity is `1` or `2`, in source order.
- Converted the expected output to the exact canonical JSON object used by the structural scorer.

No target-model measurements were run before this normalization or before the realism review.
