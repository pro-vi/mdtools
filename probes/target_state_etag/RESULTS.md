# Target-State Etag Results Ledger

- Date: 2026-07-18
- Purpose: Factual ledger for the ten-case stateless-candidate target-state etag probe recorded in `probes/target_state_etag/results.json`.
- Accepted semantic-comparison base commit: `c7f08c9e1cfa4803617256c0f943a852c7d6703a`
- Historical execution lineage base commit: `2891a3e1454ef0c88481f4cf3e389a423f1c0319`

## Immutable Inputs And Exact Commands

- `probes/target_state_etag/probe.py` SHA-256: `be59924665a50bd3c38317f8b146a9a9a73b13a049a4ce15ba2dcab12a30faa7`
- `probes/target_state_etag/cases.json` SHA-256: `287031f5e85d6ab32f394eaac0245fde4177eb4fb88d1049d79b242463f11d56`
- historical execution-time `PROTOCOL.md` authority hash (SHA-256): `1c891a9f46fcb0cf0fca916a1a78efc3da008254246d9932698039e00095c3b5`
- current `PROTOCOL.md` SHA-256: `8f40e63bcbad6e5a283e2831732055099f8931578dbbba4b11c1e6c056933b2d`
- Exact locked build command: `cargo build --release --locked --offline`
- Exact regeneration command: `python3 probes/target_state_etag/probe.py --md-binary target/release/md --output probes/target_state_etag/results.json`
- Exact non-mutating check command: `python3 probes/target_state_etag/probe.py --md-binary target/release/md --check probes/target_state_etag/results.json`
- `probes/target_state_etag/results.json` SHA-256: `4a6a3e4c7ec410cd98aa7ab35e55553012dcfc287e9116b1fe8949ee3ba1c98a`

## Order-Only Regeneration Facts

- Parsed current `results.json` is JSON-equal to `git show c7f08c9e1cfa4803617256c0f943a852c7d6703a:probes/target_state_etag/results.json`.
- The regeneration changes candidate-keyed JSON presentation order only; no case, verdict, digest, credit, count, or disposition changed.
- `candidate_names` remains exactly `["content_only","target_local","ambiguity_reject","document_target_state"]`.
- `candidate_summary`, every per-case `candidate_results` map, and `overall_graduation_verdict.candidate_verdicts` now emit keys in that exact protocol order.

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
duplicate, and same-locator identity challenges and provide the block-surface
counterexamples sufficient for global demotion. The unchanged section, table,
and task cases prove live projection and descriptor reproducibility only; they
do not independently prove wrong-target identity on those surfaces.

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
