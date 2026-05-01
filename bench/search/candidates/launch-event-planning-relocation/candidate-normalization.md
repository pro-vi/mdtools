# C-T10-36 Candidate Normalization

The raw generator output was accepted as the theme seed, not as a directly
runnable fixture. Normalization happened before realism review and before any
gap measurement.

- Replaced generic slug/name with `launch-event-planning-relocation`.
- Added the required similarly named decoy heading:
  `### Launch Event Planning Exceptions`.
- Replaced placeholder `Lorem ipsum...` prose with realistic marketing and
  events guide text.
- Preserved the generated moved subsection family shape: one level-3 subsection
  with paragraph, table, fenced archive mention, and checklist content moves
  from `## Marketing` to `## Events`.
- Changed the fenced archive text to mention the exact heading
  `### Launch Event Planning`, which must remain untouched and must not be
  interpreted as a real heading.
- Kept both input and expected Markdown under 50 lines.
