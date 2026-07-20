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

## Exact Execution Replay

- accepted Git review range preserved for this replay:
  `ca2d2e022696d5464edb2b334b284787e4d88323..73755939c0d8d94a07a165420750418312cd5858`
- execution intention SHA-256:
  `04020d97334f9800fa2c70694ddc05c820d6d08f420ff529dc91bbe1f1928933`
- sealed Weave draft: `weave-draft-1784511267401121000-61120-00000000`
- execution run: `run-1784511608.078778000`
- results commit: `f4b5904706be8f2a44834ab87319d32f83cac579`
- ledger commit: `73755939c0d8d94a07a165420750418312cd5858`
- independent offline canonical `--check`: passed at
  `73755939c0d8d94a07a165420750418312cd5858`
- results.json SHA-256 before and after:
  `e9efed5b9d57c3b45399d7c8f684a943536b18f1b2786bd2de22bd0ebee16153`

execution task-oracle sequences: 64, 73
execution completion sequences: 65, 74
execution review sequences: 78-81
execution phase-oracle sequence: 82
execution RunCompleted sequence: 83

Overall verdict: `no_candidate_graduates`.
This appendix is evidence-only: no candidate graduates, and no identity token
is implemented or promoted.

## Exact Review Repair Replay

- original reviewed head: `b3246b0a4a54a4752c08b964bce3c2a4a4970c89`
- Codex review thread IDs: `PRRT_kwDOR1UUic6SIeVx`,
  `PRRT_kwDOR1UUic6SIeVy`
- source intention SHA-256:
  `99bc169daf54aee0c556db239a3b18f652063b9a33b13c8809193f23503ac33b`
- source run: `run-1784515045.184854000`
- repaired source commit: `b9b132f8269aedcc0a4d5629c24cdad08f7ecd83`
- native-review range:
  `b3246b0a4a54a4752c08b964bce3c2a4a4970c89..b9b132f8269aedcc0a4d5629c24cdad08f7ecd83`
- task-oracle sequences: `116`, `125`
- completion sequences: `117`, `126`
- review sequences: `130-133`
- phase-oracle sequence: `134`
- RunCompleted sequence: `135`
- historical `probe.py` SHA-256:
  `060170d2527db7e20f1d23367a8601b209b2de7c61a46d3e72216eaa6d1cbc16`
- repaired `probe.py` SHA-256:
  `86534e49f7306f75f2e1be415a9341736e79f8f74001276980f4ded9d12d197e`
- byte-identical `results.json` SHA-256:
  `e9efed5b9d57c3b45399d7c8f684a943536b18f1b2786bd2de22bd0ebee16153`

Overall verdict: `no_candidate_graduates`.
This review-repair replay is evidence-only: no candidate graduates, the
repaired probe hash differs from the historical hash exactly as recorded
above, the results hash remains byte-identical, and no identity token is
implemented or promoted.

## Exact Adversarial Replay Evidence

- task anchor commit during replay: `b9b132f8269aedcc0a4d5629c24cdad08f7ecd83`
- exact in-memory adversarial manifest checks: passed
- fail-closed mismatch evidence:
  `section-unchanged-reread` replaced with the table duplicate case failed with
  `ProbeError("runner-owned surface mismatch")`
- fail-closed case-class evidence:
  `section-unchanged-reread` replaced with the section duplicate-cross-target
  case failed with `ProbeError("runner-owned case class mismatch")`
- integer guard evidence:
  `expect_nonnegative_int(False, "demo.nonnegative")` and
  `expect_nonnegative_int(True, "demo.nonnegative")` both failed with
  `ProbeError("expected non-negative integer")`
- integer guard evidence:
  `expect_positive_int(True, "demo.positive")` failed with
  `ProbeError("expected positive integer")`
- integer pass evidence:
  `expect_nonnegative_int(0, "demo.nonnegative") == 0` and
  `expect_positive_int(1, "demo.positive") == 1`
- offline replay build:
  `CARGO_NET_OFFLINE=true cargo build --release` passed
- canonical replay check:
  `python3 probes/non_block_target_identity/probe.py --md-binary target/release/md --check probes/non_block_target_identity/results.json`
  passed
- pinned SHA-256 values preserved:
  `target/release/md`:
  `be3ed8d0555233f6e4db5b0cde7a1fe42ce0e24708c60757e53803ec7c7543b9`
- pinned SHA-256 values preserved:
  `probe.py`:
  `86534e49f7306f75f2e1be415a9341736e79f8f74001276980f4ded9d12d197e`
- pinned SHA-256 values preserved:
  `results.json`:
  `e9efed5b9d57c3b45399d7c8f684a943536b18f1b2786bd2de22bd0ebee16153`
- supporting SHA-256 values preserved:
  `PROTOCOL.md`:
  `9612eb3b268c20f87dd428a40a0db8a63d4a8da0f80f2ecbb7e2f28b684f2d7b`
- supporting SHA-256 values preserved:
  `cases.json`:
  `01e42dff414ca6491b6013243b9b6233e1b42f813fa67669bdbefa9ac92084a3`
- evidence-only boundary:
  `results.json` stayed byte-identical under `--check`, and this appendix is the
  only intended worktree delta for the replay.
