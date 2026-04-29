# Candidate Normalization

Before measurement, the generated candidate was normalized for a valid
Markdown hierarchy and a benchmark-stable task definition:

- Normalized generated family slug `documentation_maintenance` to
  `server-guide-system-configuration-relocation`.
- Normalized generated task id `md-move-nested-subsection-20240515` to
  benchmark id `C-T10-17`.
- Changed the two generated `## System Configuration` headings to
  `### System Configuration` so both are true subsections under their
  nearest parent sections (`## Setup` and `## Maintenance`).
- Clarified the insertion point as immediately after
  `### Backup Procedures` and before the existing Maintenance
  `### System Configuration` subsection.
- Preserved the generator's core task shape: move one complete
  subsection, keep its list and fenced code block intact, and ignore
  a similarly named subsection plus fenced-code mentions.

The realism review in `realism-review.json` was run against this
normalized task before any target-model measurement.
