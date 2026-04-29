# Candidate Normalization

- Renamed the generic generated slug/name to `api-rate-limit-policy-relocation` / `API rate-limit policy relocation`.
- Renamed the generic generated task ID to loop-local `C-T10-27`.
- Added the destination parent `## Operational Policies` to the input because the raw generator output referenced it only in the expected output.
- Added the similarly named decoy heading `### API Rate Limiting Exceptions` under `## Appendix` so the candidate satisfies the decoy-heading requirement.
- Kept the generator's fenced literal mention of `# API Rate Limiting` inside the moved subsection; it must move as owned content, not be edited as a heading.
- Tightened the task description to place the moved subsection immediately after `### Data Retention` under `## Operational Policies` while leaving the appendix decoy untouched.
- Switched the scorer policy from the generator's generic binary wording to the repo's `normalized_text` structural policy with heading tree, block order, and block text checks.
