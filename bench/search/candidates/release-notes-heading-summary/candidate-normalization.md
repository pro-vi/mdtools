# Candidate Normalization

Before measurement, the generated candidate was normalized for a valid scorer target:

- Corrected the generated subsection heading count from 7 to 8; the source document has eight H3 headings outside ignored regions.
- Renamed `first_level_headers` to `section_headings` and `second_level_header_count` to `subsection_heading_count` so the output contract matches Markdown heading levels unambiguously.
- Added explicit heading-like lines inside the fenced code block and blockquoted archival note so the generated "ignore code and quoted archival notes" requirement is actually exercised.

The realism review in `realism-review.json` was run against the normalized task before any target-model measurement.
