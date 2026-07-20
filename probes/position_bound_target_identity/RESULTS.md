# Position-Bound Target Identity Results Ledger

## Hypothesis and Candidate Boundary

This probe asks whether any exact bounded positional-context candidate can
preserve target identity across unchanged and unrelated edits while rejecting
mechanically proven wrong-target substitutions.

- `preceding_block`
- `following_block`
- `adjacent_blocks`
- `byte_window_64`

One wrong-target accept or one same-target reject globally demotes a candidate.
This is an evidence-only probe; it does not authorize production identity work.

## Exact Braid Build

- merged Braid source commit:
  `dd26937d3322dc1fe7307cf6478322d8b8caa5bb`
- merged Braid source tree:
  `5aced3c2cb8730fba899f61d8c6abd595f8f222c`
- exact Braid binary SHA-256:
  `f7fecdf0628aebaa7463ddc100815e27a0c650bdd87b9de0533d75dde76971c4`
- exact adapter SHA-256:
  `9ceeec5c4b7dd6962d586cca0f5409cab744d75cd95d3728a64b99641e4f6972`

## Source Construction and Repair Replays

Three separate source-only Braid runs constructed and repaired the runner and
manifest before execution authorization.

### Initial Runner and Manifest Construction

- intention SHA-256:
  `8577c68bdd42add53d2938523e3ec1a229089bc7bafa01646dc3abc16bb76d41`
- run: `run-1784560584.353976000`
- manifest commit: `7ce4a3fa8e20a7c831ca6a70cf25d039958057e8`
- runner commit: `17fea07435fc9d76e0048a580c68b1c2a98d3e25`
- reviewed repair commit: `bf04c092d6ecbbfecfaa8c16eac0259585186ecb`
- first review started at sequence `33`
- repaired clean review started at sequence `40` and was recorded at `43`
- phase-oracle pass: sequence `44`
- durable `RunCompleted`: sequence `45`
- repair explanation: native review found and repaired disclaimer and
  path-authority issues before completion

### Runner Contract Repair

- intention SHA-256:
  `c6108133d8377c7555ffa6f8c85b94e1cea09d35ee7b1bf149ebf4ce23ea1d6e`
- run: `run-1784563713.866381000`
- builder commit: `c84d6cb4928b3f1e66de86104a30455e927371f1`
- first fixer commit: `5444f21487e678f31583aba3610660dfde8a1923`
- final fixer commit: `347e77fcc79ca2597931d4f5773f119e4730f909`
- task-oracle pass: sequence `60`
- review rounds started at sequences `65`, `72`, and `79`
- final clean review recorded at `82`
- phase-oracle pass: sequence `83`
- durable `RunCompleted`: sequence `84`
- repair explanation: review first corrected report insertion authority and
  then removed an invalid dependence on live JSON object-key order

### Final Pre-Execution Hardening

- intention SHA-256:
  `83cea2672526c286fd24b478159ae70b90cc13de4ad7123ad5cca342673a4be9`
- run: `run-1784565437.328585000`
- final execution-source commit:
  `4308df5874a032c67a3fb0b1aa28ba303ed3bd56`
- exact native-review range:
  `347e77fcc79ca2597931d4f5773f119e4730f909..4308df5874a032c67a3fb0b1aa28ba303ed3bd56`
- task-oracle pass: sequence `95`
- clean review sequences: `100-103`
- phase-oracle pass: sequence `104`
- durable `RunCompleted`: sequence `105`
- repair explanation: this run hardened duplicate-key rejection, manifest
  validation order, fixed child argv and document authority, live block order
  and span proofs, and collision-safe framed relation measurements

All source construction and repair runs were source-only. The newly changed
runner was not imported, compiled, or executed in them. Every runner line and
every repair delta was inspected as inert source before the execution run was
authorized.

## Exact Execution Replay

- execution intention SHA-256:
  `9b8c383abb3387a7af8fe98b26f48fac144d6d85abc5804f7b58d8db257c3071`
- sealed Weave draft:
  `weave-draft-1784566968665639000-17663-00000000`
- typed RunSpec version: `8`
- execution run: `run-1784567352.820022000`
- exact source commit: `4308df5874a032c67a3fb0b1aa28ba303ed3bd56`
- canonical result commit:
  `d49236658f307d1d1c4882d88a30fdeba0aeed7c`
- exact native-review range:
  `4308df5874a032c67a3fb0b1aa28ba303ed3bd56..d49236658f307d1d1c4882d88a30fdeba0aeed7c`
- typed task-command SHA-256:
  `d01c2f0e016742659ef7d6d0580d6899719de414ffcd937dd237fd49e79da211`
- typed phase-command SHA-256:
  `322e0d72abdd437afbf1b1ac12bee2271c4fbc785ae869e7583b4798c65c53c4`
- task-oracle pass: sequence `118`
- clean native-review sequences: `123-126`
- phase-oracle pass: sequence `127`
- durable `RunCompleted`: sequence `128`
- native review findings: none
- result SHA-256 before and after canonical and adversarial verification:
  `bd3d0d22243f2f6103c60575b1654535c20cf2c7c9ae6a73b8c7e418819d5839`
- authenticated `target/release/md` SHA-256:
  `058277f003303200618ae58c52cba86ce3d78ce403e5a50bb5bbca2460819cd8`

The independent full gate passed:

- `cargo fmt --check`
- `cargo test --offline --package mdtools`
- `cargo clippy --offline --all-targets --all-features` with existing warnings
  only
- `cargo build --release --offline`
- `python3 -m unittest discover -s bench -p 'test_*.py'`: `268` tests run,
  `18` skipped, and every non-skipped test passed
- `python3 bench/harness.py --md-binary target/release/md`: every dual-scorer
  task passed
- the operator-inspected adversarial and canonical verifier passed

## Ten-Case Evidence Matrix

| Case ID | Identity truth | `preceding_block` | `following_block` | `adjacent_blocks` | `byte_window_64` |
| --- | --- | --- | --- | --- | --- |
| `block-unchanged-lf-reread` | `same_target` | accept, correct | accept, correct | accept, correct | accept, correct |
| `block-unchanged-crlf-reread` | `same_target` | accept, correct | accept, correct | accept, correct | accept, correct |
| `block-unchanged-multibyte-utf8-reread` | `same_target` | accept, correct | accept, correct | accept, correct | accept, correct |
| `block-outside-context-unrelated-edit` | `same_target` | accept, correct | accept, correct | accept, correct | accept, correct |
| `block-preceding-neighbor-edit-inside-prefix-window` | `same_target` | reject, false_conflict | accept, correct | reject, false_conflict | reject, false_conflict |
| `block-following-neighbor-edit-inside-suffix-window` | `same_target` | accept, correct | reject, false_conflict | reject, false_conflict | reject, false_conflict |
| `block-forward-survivor-same-locator-duplicate-substitution` | `wrong_target` | accept, wrong_identity | reject, correct | reject, correct | reject, correct |
| `block-backward-survivor-same-locator-duplicate-substitution` | `wrong_target` | reject, correct | accept, wrong_identity | reject, correct | reject, correct |
| `block-cloned-adjacent-blocks-distinguishing-byte-window` | `wrong_target` | accept, wrong_identity | accept, wrong_identity | accept, wrong_identity | reject, correct |
| `block-byte-identical-bounded-neighborhood-duplicate-substitution` | `wrong_target` | accept, wrong_identity | accept, wrong_identity | accept, wrong_identity | accept, wrong_identity |

## Candidate Summaries and Conclusion

| Candidate | Accepts | Rejects | Wrong-identity accepts | Same-target rejects | Neighbor false conflicts | Graduation verdict | Disposition |
| --- | ---: | ---: | ---: | ---: | ---: | --- | --- |
| `preceding_block` | 8 | 2 | 3 | 1 | 1 | `demoted_wrong_identity` | `demote` |
| `following_block` | 8 | 2 | 3 | 1 | 1 | `demoted_wrong_identity` | `demote` |
| `adjacent_blocks` | 6 | 4 | 2 | 2 | 2 | `demoted_wrong_identity` | `demote` |
| `byte_window_64` | 5 | 5 | 1 | 2 | 2 | `demoted_wrong_identity` | `demote` |

Canonical aggregate values:

- verdict: `no_bounded_context_candidate_graduates`
- graduating candidates: empty
- demoted candidates, in order: `preceding_block`, `following_block`,
  `adjacent_blocks`, `byte_window_64`
- selected candidate: `null`

Bounded positional context alone is insufficient for this matrix. No candidate
graduates; no production token, implementation, or selection is authorized.
The result does not prove that all larger or non-bounded identity mechanisms
fail.

## Canonical and Security Evidence

The canonical report uses repository-relative normalized reported command
vectors rather than literal OS argv:

- observed form:
  `target/release/md blocks cases/{case_id}/observed.md --json`
- current form:
  `target/release/md blocks cases/{case_id}/current.md --json`

The canonical report was reproduced byte-identically and retained SHA-256
`bd3d0d22243f2f6103c60575b1654535c20cf2c7c9ae6a73b8c7e418819d5839`
through `--check` and adversarial replay.

The verifier covered:

- manifest omission, extra-field, reorder, type, weakening, semantic-digest,
  duplicate-key, and malformed-JSON failures
- fixed command-path roles and child argv boundaries
- duplicate and extra live JSON keys plus index, order, and span-bounds
  failures
- collision-safe framed relations
- aggregate zero, one, and many branches
- atomic output and no-rewrite checks
- invalid binary path, digest, and executable boundaries

Braid attempts isolate rejected Git state, not hostile execution. The inspected
runner was local-only and invoked an operator-trusted repository build with OS
authority. Hostile binary containment would require an outer VM or container
with network disabled, credentials absent, and scoped mounts.
