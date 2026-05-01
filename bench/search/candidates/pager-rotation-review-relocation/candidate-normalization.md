Before measurement, the generated candidate was normalized into a valid scorer target:

- Renamed the generic generated slug to `pager-rotation-review-relocation` and assigned loop-local task ID `C-T10-34`.
- Replaced numbered/generated alert headings with a compact on-call runbook using the same generated operation class: move one complete level-3 subsection between level-2 parents.
- Fixed the generator's invalid expected output, which had not moved the target subsection and instead inserted a fenced literal mention at the destination.
- Added a source-side similarly named decoy heading and a quoted archive mention of the moved heading that must remain untouched.
- Kept the candidate under 50 lines and selected the repo's normalized-text structural scorer with heading tree, block order, and block text checks.

No target-model measurements were run before this normalization or before the realism review.
