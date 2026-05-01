# Candidate Normalization

Before measurement, the generated candidate was normalized for a valid scorer target:

- Renamed the generic generated family slug/name to `server-setup-subsection-relocation` / `Server setup subsection relocation` for ledger readability.
- Corrected the generated expected document so `## Logging` and `## Database Backups` remain under `# System Configuration` after `## Database Configuration` is moved.
- Made the insertion point explicit: place `## Database Configuration` under `# Server Setup`, immediately after `## Network Configuration` and before `## Service Status`.
- Removed the generated trailing spaces after `## Application Settings` so exact block comparison is not affected by incidental whitespace.

The realism review in `realism-review.json` was run against the normalized task before any target-model measurement.
