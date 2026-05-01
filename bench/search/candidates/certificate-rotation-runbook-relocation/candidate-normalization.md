# Candidate Normalization

The generator produced a realistic certificate/SSL relocation task shape but missed several constraints and emitted an internally invalid destination decoy:

- Renamed the generic generated family slug/name to `certificate-rotation-runbook-relocation` / `Certificate rotation runbook relocation`.
- Replaced the duplicate destination `### Configuring SSL` heading with a distinct `### Certificate Rotation Archive` decoy so the expected output has one real moved subsection and one similarly named decoy.
- Reframed the document as a compact operations runbook and renamed the moved subsection to `### Certificate Rotation` to avoid the excluded generic setup/configuration wording.
- Added checklist, table, and fenced shell content to the moved subsection so the edit must preserve a complete heading-scoped block.
- Added a fenced archived outline containing a literal `### Certificate Rotation` mention that must remain untouched.
- Made the destination placement explicit: under `## Operations Playbook`, immediately before `### Maintenance Window`.

The realism review in `realism-review.json` was run against this normalized task before any target-model measurement.
