# Candidate Normalization

- Assigned canonical task ID `C-T10-28` and family slug `error-logging-format-relocation`; the generator emitted a generic slug/task ID.
- Replaced the generator's non-Markdown triple-quote archive block with a normal blockquote archive note, and preserved that note in the expected output.
- Tightened the decoy heading to `### Error Logging Exceptions`, added a table and checklist to the moved subsection, and kept the generated intent: move `### Error Logging Format` from `## Data Processing` to `## Reporting` before `### Report Generation Schedule`.
- Added an explicit normalized-text scorer policy because the generator omitted the requested `scorer_policy` key.
