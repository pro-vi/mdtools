# Target-State Etag Results Ledger

- Date: 2026-07-18
- Purpose: Factual ledger for the ten-case stateless-candidate target-state etag probe recorded in `probes/target_state_etag/results.json`.
- Accepted execution base commit: `2891a3e1454ef0c88481f4cf3e389a423f1c0319`

## Immutable Inputs And Commands

- `probes/target_state_etag/probe.py` SHA-256: `339185b1c0be604272b91e9c2edae896de290ce77ba0298d8d1306586fa32461`
- `probes/target_state_etag/cases.json` SHA-256: `287031f5e85d6ab32f394eaac0245fde4177eb4fb88d1049d79b242463f11d56`
- execution-time `PROTOCOL.md` SHA-256: `1c891a9f46fcb0cf0fca916a1a78efc3da008254246d9932698039e00095c3b5`
- current PROTOCOL.md SHA-256: `33af966231b641a2205d5e224d81993036414dab595fdcecb74b026ad0923adc`
- Build command: `cargo build --release`
- Output command: `python3 probes/target_state_etag/probe.py --md-binary target/release/md --output probes/target_state_etag/results.json`
- Check command: `python3 probes/target_state_etag/probe.py --md-binary target/release/md --check probes/target_state_etag/results.json`
- `probes/target_state_etag/results.json` SHA-256: `68ea7de2aa473e4c746c08bbefc0a022beb9d84f53ebd4cd9ed6fcb1087b6b6e`

## Candidate Summary

| candidate | accepts | rejects | expectation matches | wrong.identity accepts | required-same-state rejects | unrelated.edit conflicts | graduation verdict | disposition |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| `content_only` | 10 | 0 | 10 | 2 | 0 | 0 | `fails_wrong_identity` | `demote` |
| `target_local` | 9 | 1 | 10 | 1 | 0 | 0 | `fails_wrong_identity` | `demote` |
| `ambiguity_reject` | 9 | 1 | 10 | 1 | 0 | 0 | `fails_wrong_identity` | `demote` |
| `document_target_state` | 7 | 3 | 10 | 0 | 1 | 1 | `fails_whole_document_false_conflict` | `demote` |

## Overall Verdict

These verdicts are global candidate outcomes across the required ten-case
matrix. Any wrong-identity acceptance in any required case is sufficient to
demote that candidate globally. The block cases carry the wrong-target,
duplicate, and same-locator identity challenges; the section, table, and task
cases validate live descriptor reconstruction in unchanged state only and do
not independently prove wrong-target identity on those surfaces.

- `overall` verdict: `no_candidate_graduates`
- graduating candidates: none
- demoted candidates: `content_only`, `target_local`, `ambiguity_reject`, `document_target_state`
- selected candidate: `null`
- whole-document false-conflict cost: `1`
- candidate verdicts: `content_only=fails_wrong_identity`, `target_local=fails_wrong_identity`, `ambiguity_reject=fails_wrong_identity`, `document_target_state=fails_whole_document_false_conflict`

## Case Observations

- `duplicate_copy` observation: `block-duplicate-cross-target-copy` had `ambiguity_match_count=2`; `content_only` accepted with `credit=wrong_identity`, while `target_local`, `ambiguity_reject`, and `document_target_state` rejected.
- `same.locator` observation: `block-same-locator-duplicate-shift` had `ambiguity_match_count=1`; `content_only`, `target_local`, and `ambiguity_reject` accepted with `credit=wrong_identity`, while `document_target_state` rejected.
- `unrelated.edit` observation: `block-unrelated-edit-false-conflict` kept `identity_truth=same_target`; `document_target_state` rejected with `required_same_state_reject=true`, `unrelated_edit_conflict=true`, and `whole_document_false_conflict_cost=true`, while `content_only`, `target_local`, and `ambiguity_reject` accepted.

## Conclusion

This ledger is bounded to the ten-case stateless-candidate experiment recorded
in `results.json`. No tested candidate graduated, so the probe does not itself
choose or implement a production etag format, and position-bound target-state
semantics remain a separate architectural decision.
