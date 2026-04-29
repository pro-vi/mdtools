# Candidate Normalization

- Renamed the generator's generic `ops-markdown-edit` slug to `release-database-migration-relocation`.
- Renamed the generator's generic `move-subsection-with-decoy` task ID to loop-local `C-T10-22`.
- Made the insertion point explicit in the benchmark description: the moved subsection is inserted as the first subsection under `Maintenance`, immediately before `Log Rotation`.
- Kept the generated input and expected Markdown content unchanged, aside from adding final trailing newlines in the file artifacts.
- Converted the generator's exact-match scorer policy to the repo's dual normalized-text scorer policy: heading tree, block order, and block text.
