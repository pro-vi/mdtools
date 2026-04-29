# Candidate Normalization

The generator produced a realistic subsection-relocation task, but the raw output needed small consistency repairs before realism review and measurement:

- Replaced the generic family slug/name with `database-migration-subsection-relocation`.
- Corrected the task description's target heading marker from `## 3.1.2 Database Migration` to the actual nested heading `### 3.1.2 Database Migration`.
- Added an explicit similarly named decoy subsection, `### 3.2.3 Database Migration Checklist`, under the destination parent because the raw candidate only included a code-block mention as a decoy.
- Added a quoted archive mention of `Database Migration` under the destination parent to exercise the requested "ignore quoted archive" condition.
- Made the destination placement explicit: after `### 3.2.2 Data Integrity Checks` and before the decoy subsection.

No Markdown-tooling vocabulary or prior corpus outcomes were introduced into the candidate text.
