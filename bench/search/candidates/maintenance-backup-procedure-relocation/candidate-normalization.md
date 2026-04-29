# Candidate Normalization

The generator produced a realistic backup-procedure relocation seed but missed two requested constraints:

- The expected output placed `## Backup Procedure` before `## Shutdown Sequence`, contradicting the task description's "immediately before `## Scheduled Tasks`" placement.
- The input lacked a similarly named real heading decoy; it only included backup commands inside a fenced block.

Normalization kept the release-operations backup-procedure relocation shape and made the fixture compact and internally consistent:

- Removed unrelated generated `# Installation` and `# Configuration` sections so the document stays under 70 lines and avoids headings excluded by the generator prompt.
- Moved `## Backup Procedure` under `# Operations` immediately before `## Scheduled Tasks`.
- Added `## Backup Procedure Archive` as a real-heading decoy and a fenced archived mention of `## Backup Procedure` that must be ignored.
- Used the existing normalized-text scorer policy (`heading_tree`, `block_order`, and `block_text`) rather than exact byte comparison.
