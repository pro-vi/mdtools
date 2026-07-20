# Stateless Target Identity Observability Limit Protocol

Date locked: 2026-07-20
Accepted base commit: `4dc46369c54ea35414bc4909974f8ecc988f7bef`
Accepted base tree: `08aa84f8aac8e788bedb462e3efc24524e8fcaf4`
Accepted branch: `probe/stateless-target-identity-observability-4dc4636`

## Scope and grounding

This phase is source-only at accepted base
`4dc46369c54ea35414bc4909974f8ecc988f7bef` on branch
`probe/stateless-target-identity-observability-4dc4636`. It may author exactly
one tracked file: `probes/stateless_target_identity_observability/PROTOCOL.md`.
It does not authorize any runner, manifest, fixture, result artifact, test,
production change, helper script, workflow file, or probe execution.

This protocol is grounded only in these accepted authorities and exact
SHA-256 hashes:

- `CLAUDE.md`:
  `38de34f5d7e911ea19352a8e8ad34a455286320a05aa09f0a60462e525a115d1`
- `specs/mdtools.md`:
  `eb8911743a392de3cd3f182ba3ef9c7e652c39dee66db3b12ae8c6e4231d4af3`
- `probes/position_bound_target_identity/PROTOCOL.md`:
  `579dd0bc5eb8cfa2e65834fd976517af10c008c2006e3687e124c911b2b77445`
- `probes/position_bound_target_identity/RESULTS.md`:
  `0477dcacdecc783429f1c0766fb402eea4998de9072cd478c4b96257610dbfeb`
- `probes/target_state_etag/RESULTS.md`:
  `6df1de92b94afe571583522157870e9c2edb757dcf9406aba02da027d983fb9c`

The protocol preserves these accepted predecessor facts without broadening
them:

- `loc` carries no identity.
- `etag` fingerprints content rather than lineage.
- `preceding_block`, `following_block`, `adjacent_blocks`, and
  `byte_window_64` were demoted.
- the bounded predecessor did not prove that all larger or non-bounded
  mechanisms fail.

This phase chooses an observational-equivalence probe. The intended claim is an
observability limit over one closed enumerated public-contract stateless
surface. It is not an unrestricted impossibility proof, not another token
proposal, and not a production identity architecture decision. It does not
claim that persistent IDs, history, operation receipts, embedded markers, VCS
ancestry, external coordination, or newly discovered public input cannot
distinguish lineage.

## Source-only boundary

This phase is a static protocol only.

- Create exactly `probes/stateless_target_identity_observability/PROTOCOL.md`.
- Do not modify any other tracked file and do not add `.braid/`.
- Do not author `probe.py`, `cases.json`, `results.json`, `RESULTS.md`,
  fixtures, tests, production code, helper scripts, or workflow files.
- Do not execute, import, compile, or otherwise evaluate newly authored probe
  code.
- Do not invoke `md` for the proposed experiment in this phase.
- Allow only static Markdown checks and Git diff checks.

Protocol authorship alone does not validate the hypothesis, does not select an
identity mechanism, and does not authorize production work.

## Hypothesis and claim ledger

Hypothesis: for the closed stateless observation tuple defined in this
protocol, if two mechanically constructed histories have different lineage
truths but identical canonical observations, then no deterministic stateless
function of that tuple can accept the same-lineage world and reject the
wrong-lineage world.

Claim ledger:

| Claim | Status | Scope |
| --- | --- | --- |
| `loc` carries no identity | Accepted | Preserved from accepted grounding |
| `etag` fingerprints content rather than lineage | Accepted | Preserved from accepted grounding |
| `preceding_block`, `following_block`, `adjacent_blocks`, and `byte_window_64` did not graduate | Accepted | Preserved predecessor result only |
| Closed-tuple observability limit over the current public-contract stateless surface | Probing | New claim in this protocol only |
| Choice of persistent identity architecture | Open | Explicitly not decided here |

The new claim is intentionally narrower than "identity is impossible." It asks
only whether the later execution can validate that the currently enumerated
stateless public observation surface is insufficient to distinguish
`same_target` from `wrong_target` when both worlds expose byte-equal canonical
observations.

## Authorized observation surface

Define one closed canonical stateless observation tuple `O`. The later runner
must compare `O_same` and `O_wrong` only after canonicalization removes
incidental serialization noise while retaining every semantically public field.
Unknown or future public fields must be included canonically or the experiment
must fail closed rather than silently discarding them.

`O` must include exactly these component classes:

1. `B_md`: the exact authenticated repository-local `md` binary bytes, or
   equivalently the exact accepted SHA-256 that authenticates those bytes
   before any child invocation.
2. `path_rel`: the same logical relative file path in both worlds.
3. `selectors`: the same explicit read or mutation selector inputs, including
   the observed selector, the current selector, target kind, occurrence
   choices, and any fixed CLI arguments.
4. `D_obs`: the exact observed Markdown document bytes.
5. `J_obs`: the complete deterministic public `md` structural JSON projection
   family derived from `D_obs` through explicit live authenticated
   `md ... --json` reads under the locked public contract, the same
   authenticated binary, the same runner-owned relative path, and the fixed
   explicit inputs.
6. `target_obs_bytes`: the exact observed target bytes derived only from live
   spans plus checked raw-byte slicing.
7. `target_obs_descriptor`: the canonical observed live target descriptor
   derived only from the public projections plus checked raw-byte slicing.
8. `A_obs`: the expected token or other deterministic artifact derived from the
   observed components under the fixed candidate framing and decision rule.
9. `D_cur`: the exact current Markdown document bytes.
10. `J_cur`: the complete deterministic public `md` structural JSON projection
    family derived from `D_cur` through the same authenticated binary, the same
    runner-owned relative path, and the same explicit live `md ... --json`
    reads and fixed inputs.
11. `target_cur_bytes`: the exact current target bytes derived only from live
    spans plus checked raw-byte slicing.
12. `target_cur_descriptor`: the canonical current live descriptor derived only
    from the public projections plus checked raw-byte slicing.
13. `match_domain_cur`: the same-kind live match domain for the current target
    resolution.
14. `match_counts_cur`: the live current match counts required by the selector
    and match domain.
15. `R_other`: all other in-scope repository file bytes that the later runner
    authorizes as readable input, held byte-identical across the two worlds.
16. `candidate_spec`: the deterministic candidate name, version, framing, and
    decision rule.

The complete deterministic public projection family in `J_obs` and `J_cur`
must dominate all currently proposed stateless target tokens. At minimum it
must include every locked-spec deterministic public JSON read surface reachable
from the same path and explicit inputs. Target bytes and descriptors must still
be derived from live spans plus raw bytes only, not from previews, prose, or
handwritten parsing.

Any deterministic derivative of the equal components above is already covered,
including full-document hashes, unbounded context hashes, Merkle structures,
cryptographic digests, `loc`s, `etag`s, ambiguity counts, and reformatted
structural projections.

## Excluded authority and non-claims

The later execution must exclude all authority outside the closed tuple.

- No persistent ID, history database, receipt log, embedded marker, VCS
  ancestry, editor metadata, filesystem timestamp, external service, or human
  assertion may supply target identity truth.
- No manifest-owned span, expected verdict, lineage truth, candidate credit,
  or aggregate conclusion is trusted.
- No hidden input or nondeterministic side channel may distinguish the two
  worlds once `O_same` and `O_wrong` are byte-equal.

This protocol does not claim that those excluded authorities are impossible or
bad. It claims only that they are outside the enumerated stateless public
surface being tested here.

## Canonical two-history construction

The later runner must construct one observed world and two history narratives
over the same observed bytes:

`D0 = P || T_a || M || T_b || S`

Requirements for the construction are exact:

- `P`, `M`, and `S` are nonempty runner-owned exact byte segments.
- `D0` must be valid Markdown with live top-level block boundaries.
- `T_a` and `T_b` are distinct observed block occurrences with byte-identical,
  nonempty exact block slices and `T_a == T_b` as bytes.
- `T_a` is the observed intended target, resolved by a runner-owned selector
  through live authenticated `md blocks FILE --json` plus any other required
  locked-contract `md ... --json` reads and checked slicing.
- The live observed projection must prove exactly two same-kind exact-byte
  target matches at pinned ordinals and must prove their ranges and ordering.
- All non-target repository files remain byte-identical across both worlds.

Define the two histories by exact deletion on `D0`:

- `H_same`: delete exactly `M || T_b`, preserving `T_a`.
- `H_wrong`: delete exactly `T_a || M`, preserving `T_b`.

The segment notation is conceptual only. The later runner must derive actual
byte boundaries from live block spans and runner-owned exact construction, not
from manifest-owned spans or handwritten Markdown parsing.

Both histories must reconstruct byte-for-byte the same current document:

`D1 = P || T || S`

The current selector must be identical in both worlds, must resolve exactly one
same-kind exact-byte target, and must produce the same complete current
projection, target bytes, descriptor, match domain, and match counts. The
observed document, observed selector, and complete observed projection must
also be identical across worlds.

## Mechanical lineage proof

Lineage truth must be proven mechanically and independently of candidate
output.

- The later runner must track the two original occurrence ordinals through the
  exact deletion operation.
- In `H_same`, it must prove that the surviving current target range is the
  translated survivor of observed occurrence `T_a` and therefore
  `same_target`.
- In `H_wrong`, it must prove that observed occurrence `T_a` lies inside the
  deleted range and that the surviving current target range is the translated
  survivor of `T_b`, and therefore `wrong_target`.
- It must reconstruct both current documents from `D0`, the two pinned deletion
  ranges, and raw bytes.
- It must prove that both reconstructions equal the exact runner-owned `D1`,
  that the deletion ranges are distinct, and that survivor mapping plus all
  mechanical preconditions hold before comparing observations or drawing any
  conclusion.

Failure of construction, live resolution, byte equality, projection equality,
or lineage proof is an operational failure and never evidence for the
hypothesis.

The manifest must not supply trusted expected candidate decisions, credits,
aggregate verdicts, or lineage truth.

## Minimum future experiment

Later work must stay phase-separated:

1. A separately authorized source phase may author a deterministic runner and
   manifest.
2. A separate execution phase may run them.

The later runner requirements are exact:

- Authenticate an explicit repository-local release `md` binary by exact
  SHA-256 before any child invocation.
- Use only local temporary files, argument-list subprocesses with
  `shell=False`, an empty child environment, no `PATH` lookup, and no network.
- Materialize `D0`, execute each exact deletion construction in memory, and
  materialize the two `D1` worlds at the same logical path sequentially.
- Obtain complete public structural projections from explicit live `md` JSON
  reads, validating JSON shapes, UTF-8, spans, byte bounds, field types, and
  unknown fields.
- Derive target bytes and descriptors only from live spans plus raw bytes.
- Prove independently that lineage truths differ and that the complete
  canonical tuples are byte-equal before emitting a validated observability
  result.
- Emit a mechanically derived deterministic canonical JSON result.
- Provide a non-mutating byte-identical `--check` mode.

The later protocol may specify `cases.json`, `probe.py`, `results.json`, and
`RESULTS.md` as later artifacts, but this phase must not create them.

## Deterministic conclusion rule

The later runner must apply this exact conclusion rule:

If and only if all construction checks, authority checks, canonicalization
checks, tuple-equality checks, and independent lineage proofs pass, and
`O_same == O_wrong` while the mechanically derived truths differ, conclude that
no deterministic stateless function of the enumerated tuple can both accept the
same-lineage world and reject the wrong-lineage world.

The forced tradeoff must be stated explicitly:

- Equal input forces equal output.
- If the function accepts both worlds, `H_wrong` becomes a wrong-identity
  accept.
- If the function rejects both worlds, `H_same` becomes a false conflict.

An always-reject rule is safe against wrong-target mutation, but it is not a
durable identity discriminator because it cannot authorize the mechanically
proven same-target world.

The result labels are closed:

- `validated_observability_limit`
- `falsified_distinguishing_input_found`
- `inconclusive_mechanical_failure`

No label is pre-marked in this source-only phase.

## Disconfirming evidence

Disconfirming evidence narrows or kills the hypothesis. At minimum it includes:

- a legitimate in-contract deterministic distinguishing input inside the closed
  tuple that yields a different canonical component across the two worlds
  before hidden input is invoked
- a supposedly equal tuple component that is not equal after canonicalization
- failure to establish lineage truth independently
- an apparent different decision from genuinely byte-equal tuples that must
  first be treated as hidden input, nondeterminism, or runner error

Operational failure is not validation. Any failure of construction, projection
validation, tuple equality, or lineage proof routes to
`inconclusive_mechanical_failure`, not to
`validated_observability_limit`.

## Evidence log schema

The later evidence log must record at least:

- exact base commit and exact head commit
- protocol, manifest, runner, and result hashes
- authenticated `md` binary path and SHA-256
- command argv schema
- observed and current document hashes
- both deletion ranges
- survivor ordinals
- independent lineage proofs
- canonical tuple component hashes
- complete tuple hash and tuple equality result
- mechanical preconditions and their pass or fail status
- result label
- independent review ranges and gate outcomes

Tracked artifacts must not contain absolute temporary paths, environment dumps,
credentials, personal telemetry, or machine-specific configuration.

## Promotion, demotion, and next-decision path

Promotion and demotion are closed:

- A `validated_observability_limit` result promotes only the scoped
  observability-limit claim and demotes further token proposals that are
  deterministic functions solely of the same tuple.
- A validated result does not select or authorize a persistent identity
  architecture.
- Any stateful design may be considered only through a new probe that compares
  explicit authority, lifecycle, corruption, portability, cleanup, and
  concurrency costs.
- A `falsified_distinguishing_input_found` result routes to a preregistered
  candidate probe for the newly discovered distinguishing input.
- An `inconclusive_mechanical_failure` result authorizes repair or replay only.

The accepted predecessor fact remains separate: the bounded candidates were
demoted, but that result neither validates this closed-tuple claim nor answers
which stateful authority, if any, is worth its costs.
