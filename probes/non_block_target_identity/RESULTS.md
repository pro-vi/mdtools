# Non-Block Target Identity Results Ledger

## Hypothesis

The pinned hypothesis was falsifiable and narrow: unchanged rereads should stay
same-target accepts; duplicate cross-target copies should expose wrong-target
accepts for underbound candidates; same-locator duplicate shifts should require
explicit lineage proof before any identity claim is trusted; unrelated edits
should surface whole-document false-conflict cost only where full-document
binding is used.

## Canonical Artifact

- source commit: `ca2d2e022696d5464edb2b334b284787e4d88323`
- source tree: `01bc6c3e6ef8f9a0c31a0d0d1631f748e82349e0`
- source-run ID: `run-1784509369.151240000`
- source intention SHA-256: `2c25a942446e0abf55f35795e2db374bf15a40c70797a109bd1d6c5ec3179669`
- review range: `55efe31a59aef6e05f2e8884f2526ec0d0af2b17..ca2d2e022696d5464edb2b334b284787e4d88323`
- mismatch sequence: `25`
- native spec-repair sequences: `29-33`
- review sequence: `46`
- phase-oracle sequence: `47`
- RunCompleted sequence: `48`
- Braid merge: `057a737aea27c07226a088d3a2c6004b6ad15212`
- binary SHA-256: `ebeff7065581d30124bfa88fdaaf89e1ad422e4fe3832a048faec615187e4b90`
- adapter SHA-256: `9ceeec5c4b7dd6962d586cca0f5409cab744d75cd95d3728a64b99641e4f6972`
- `PROTOCOL.md` SHA-256: `9612eb3b268c20f87dd428a40a0db8a63d4a8da0f80f2ecbb7e2f28b684f2d7b`
- `cases.json` SHA-256: `01e42dff414ca6491b6013243b9b6233e1b42f813fa67669bdbefa9ac92084a3`
- `probe.py` SHA-256: `060170d2527db7e20f1d23367a8601b209b2de7c61a46d3e72216eaa6d1cbc16`
- results.json SHA-256: e9efed5b9d57c3b45399d7c8f684a943536b18f1b2786bd2de22bd0ebee16153

## Candidate Set

- `content_only`
- `target_local`
- `ambiguity_reject`
- `document_target_state`

## 12-Case Three-Surface Matrix

| Case ID | Surface | Case class | Identity truth | `content_only` | `target_local` | `ambiguity_reject` | `document_target_state` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `section-unchanged-reread` | section | `unchanged_reread` | `same_target` | accept, correct | accept, correct | accept, correct | accept, correct |
| `section-duplicate-cross-target-copy` | section | `duplicate_cross_target_copy` | `wrong_target` | accept, wrong_identity | reject, correct | reject, correct | reject, correct |
| `section-same-locator-duplicate-shift` | section | `same_locator_duplicate_shift` | `wrong_target` | accept, wrong_identity | accept, wrong_identity | accept, wrong_identity | reject, correct |
| `section-unrelated-edit-false-conflict` | section | `unrelated_edit_after_unchanged_target` | `same_target` | accept, correct | accept, correct | accept, correct | reject, false_conflict |
| `table-unchanged-reread` | table | `unchanged_reread` | `same_target` | accept, correct | accept, correct | accept, correct | accept, correct |
| `table-duplicate-cross-target-copy` | table | `duplicate_cross_target_copy` | `wrong_target` | accept, wrong_identity | reject, correct | reject, correct | reject, correct |
| `table-same-locator-duplicate-shift` | table | `same_locator_duplicate_shift` | `wrong_target` | accept, wrong_identity | accept, wrong_identity | accept, wrong_identity | reject, correct |
| `table-unrelated-edit-false-conflict` | table | `unrelated_edit_after_unchanged_target` | `same_target` | accept, correct | accept, correct | accept, correct | reject, false_conflict |
| `task-unchanged-reread` | task | `unchanged_reread` | `same_target` | accept, correct | accept, correct | accept, correct | accept, correct |
| `task-duplicate-cross-target-copy` | task | `duplicate_cross_target_copy` | `wrong_target` | accept, wrong_identity | reject, correct | reject, correct | reject, correct |
| `task-same-locator-duplicate-shift` | task | `same_locator_duplicate_shift` | `wrong_target` | accept, wrong_identity | accept, wrong_identity | accept, wrong_identity | reject, correct |
| `task-unrelated-edit-false-conflict` | task | `unrelated_edit_after_unchanged_target` | `same_target` | accept, correct | accept, correct | accept, correct | reject, false_conflict |

## Candidate Verdicts

### `content_only`

- observed correct decisions: all three unchanged rereads and all three
  unrelated-edit same-target cases
- observed incorrect decisions: all three duplicate cross-target copies and all
  three same-locator duplicate shifts accepted the wrong target
- graduation verdict: `fails_wrong_identity`
- disconfirming evidence: six `wrong_identity_accepts`
- promotion or demotion conclusion: demote

### `target_local`

- observed correct decisions: all three unchanged rereads, all three duplicate
  cross-target copies, and all three unrelated-edit same-target cases
- observed incorrect decisions: `section-same-locator-duplicate-shift`,
  `table-same-locator-duplicate-shift`, and
  `task-same-locator-duplicate-shift` accepted the wrong target
- graduation verdict: `fails_wrong_identity`
- disconfirming evidence: three `wrong_identity_accepts`
- promotion or demotion conclusion: demote

### `ambiguity_reject`

- observed correct decisions: all three unchanged rereads, all three duplicate
  cross-target copies, and all three unrelated-edit same-target cases
- observed incorrect decisions: `section-same-locator-duplicate-shift`,
  `table-same-locator-duplicate-shift`, and
  `task-same-locator-duplicate-shift` accepted the wrong target
- graduation verdict: `fails_wrong_identity`
- disconfirming evidence: three `wrong_identity_accepts`
- promotion or demotion conclusion: demote

### `document_target_state`

- observed correct decisions: all three unchanged rereads, all three duplicate
  cross-target copies, and all three same-locator duplicate shifts
- observed incorrect decisions: `section-unrelated-edit-false-conflict`,
  `table-unrelated-edit-false-conflict`, and
  `task-unrelated-edit-false-conflict` rejected a same-target reread
- graduation verdict: `fails_whole_document_false_conflict`
- disconfirming evidence: three `false_conflicts`, three
  `required_same_state_rejects`, and three `unrelated_edit_conflicts`
- promotion or demotion conclusion: demote

## Candidate Summary

| Candidate | Accepts | Rejects | Correct | Wrong-identity accepts | False conflicts | Required same-state rejects | Unrelated-edit conflicts | Disposition |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `content_only` | 12 | 0 | 6 | 6 | 0 | 0 | 0 | demote |
| `target_local` | 9 | 3 | 9 | 3 | 0 | 0 | 0 | demote |
| `ambiguity_reject` | 9 | 3 | 9 | 3 | 0 | 0 | 0 | demote |
| `document_target_state` | 3 | 9 | 9 | 0 | 3 | 3 | 3 | demote |

## Promotion Outcome

No candidate graduates. `content_only`, `target_local`, and
`ambiguity_reject` fail on wrong-target acceptance. `document_target_state`
fails on whole-document false conflict under unrelated edits. The evidence-only
conclusion is to demote all four candidates and not promote an identity token
from this run.
