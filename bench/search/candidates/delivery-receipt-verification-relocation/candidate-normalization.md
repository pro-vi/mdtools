# Candidate Normalization

- Renamed the generic generated slug/name to `delivery-receipt-verification-relocation` / `Delivery receipt verification relocation`.
- Renamed the generic generated task ID to loop-local `C-T10-30`.
- Repaired the generated expected output so the source subsection is removed instead of duplicated.
- Replaced the exact duplicate decoy heading with the similarly named `### Receipt Verification Exceptions`, which satisfies the decoy requirement without creating two identical headings under the destination parent.
- Added a real Markdown table and fenced text block inside the moved subsection to make the structural unit nontrivial.
- Moved the archived literal `### Delivery Receipt Verification` mention into a quoted archive outside the moved block so it must be ignored and remain in place.
- Tightened the task description to move the subsection from `## Intake` to `## Completion`, immediately before `### Customer Handoff`.
- Switched the scorer policy from the generator's missing/generic policy to the repo's `normalized_text` structural policy with heading tree, block order, and block text checks.

