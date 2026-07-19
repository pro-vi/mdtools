# Non-Block Target Identity Deterministic Manifest Protocol

Date locked: 2026-07-19

## Scope

This probe is a complete static specification for a local-only experiment that
reuses the four candidate families from `probes/target_state_etag` without
changing `mdtools` behavior or selecting a new etag format:

- `content_only`
- `target_local`
- `ambiguity_reject`
- `document_target_state`

The experiment is limited to the non-block read surfaces whose authority comes
from live `md ... --json` output:

- `section`
- `table`
- `task`

The source-construction phase is source-only and limited authorship to
`probes/non_block_target_identity/PROTOCOL.md`,
`probes/non_block_target_identity/cases.json`, and
`probes/non_block_target_identity/probe.py`.

This phase did not execute the new runner.

That non-execution boundary is strict:

- do not import `probe.py`
- do not execute `probe.py`
- do not author `results.json`
- do not author `RESULTS.md`
- do not treat this phase as candidate selection

The hypothesis is bounded and falsifiable: if the reused candidate framing is
applied to live section/table/task descriptor authority, unchanged rereads
should remain same-target accepts, duplicate cross-target copies should expose
wrong-target accepts for insufficiently bound candidates, same-locator
duplicate shifts should require explicit lineage proof before any identity claim
is trusted, and unrelated edits should surface whole-document false-conflict
cost only where full-document binding is used.

Disconfirming evidence is explicit:

- any unchanged reread that rejects
- any wrong-target case that accepts without being counted as a wrong-identity
  failure
- any same-locator case whose lineage proof does not mechanically reconstruct
  the current document from the observed duplicate spans
- any result artifact whose bytes are not canonical and reproducible from the
  pinned manifest plus live `md` reads

Current local reproduction authority is future-facing and non-mutating:

- `cargo build --release --locked --offline`
- `python3 probes/non_block_target_identity/probe.py --md-binary target/release/md --check probes/non_block_target_identity/results.json`

Authorized regeneration is explicit and separate from reproduction:

- `cargo build --release --locked --offline`
- `python3 probes/non_block_target_identity/probe.py --md-binary target/release/md --output probes/non_block_target_identity/results.json`
- `python3 probes/non_block_target_identity/probe.py --md-binary target/release/md --check probes/non_block_target_identity/results.json`

Execution, regeneration, and result promotion require separate authorization
after source inspection.

## Live Descriptor Authority

All descriptor authority must come from live `md ... --json` read commands over
temporary documents materialized from `cases.json`. Manifest text never
substitutes for live JSON authority.

### Section

- Target query command:
  `md section SELECTOR FILE --occurrence N --json`
- Domain query command:
  `md section SELECTOR FILE --occurrence N --json`, enumerating one-based
  occurrences until the command stops resolving a match
- Canonical descriptor: `{ "selector", "heading", "span" }`
- Exact target bytes: slice the temporary document raw bytes at the returned
  `span.byte_start..span.byte_end`

### Table

- Target query command:
  `md table FILE --index BLOCK_INDEX --json`
- Domain query command:
  `md table FILE --json`
- Canonical descriptor: `{ "block_index", "span" }`
- Exact target bytes: slice the temporary document raw bytes at the returned
  `span.byte_start..span.byte_end`

### Task

- Target query command:
  `md tasks FILE --json`
- Domain query command:
  `md tasks FILE --json`
- Canonical descriptor: `{ "loc", "child_path", "span" }`
- Exact target bytes: slice the temporary document raw bytes at the returned
  `span.byte_start..span.byte_end`

Forbidden authority is unchanged from `probes/target_state_etag`:

- returned `content` strings
- `preview` fields
- `summary_text` fields
- `etag` fields
- manifest-owned span literals
- handwritten Markdown parsing

## Manifest Contract

`cases.json` is the deterministic manifest. Each case must carry:

- stable `case_id`
- `case_class`
- `identity_truth`
- `surface`
- `observed_document_utf8`
- `current_document_utf8`
- `observed_target_query`
- `current_target_query`
- `current_domain_query`

All three same-locator cases must also carry runner-owned exact:

- `same_locator_preconditions`
- `same_locator_lineage`

The runner owns the exact ordered 12-case vocabulary. The manifest must declare
and realize this exact order:

1. `section-unchanged-reread`
2. `section-duplicate-cross-target-copy`
3. `section-same-locator-duplicate-shift`
4. `section-unrelated-edit-false-conflict`
5. `table-unchanged-reread`
6. `table-duplicate-cross-target-copy`
7. `table-same-locator-duplicate-shift`
8. `table-unrelated-edit-false-conflict`
9. `task-unchanged-reread`
10. `task-duplicate-cross-target-copy`
11. `task-same-locator-duplicate-shift`
12. `task-unrelated-edit-false-conflict`

Closed vocabularies are fixed at the manifest boundary:

- `case_class`: `unchanged_reread`, `duplicate_cross_target_copy`,
  `same_locator_duplicate_shift`, `unrelated_edit_after_unchanged_target`
- `identity_truth`: `same_target`, `wrong_target`

The manifest must not contain candidate expected decisions, candidate credits,
or any other pre-scored selection oracle.

Validation is mechanical and fail-closed:

- reject `required_case_ids` unless it matches the runner-owned ordered
  12-case vocabulary exactly
- reject duplicate `case_id` values
- reject case IDs that do not match `^[a-z0-9]+(?:-[a-z0-9]+)*$`
- reject unknown `case_class` or `identity_truth` values
- reject unknown `surface` values
- reject query shapes whose `surface` or `command` does not match the declared
  query type
- reject missing same-locator mappings for the three same-locator cases
- reject any extra same-locator mapping on non-same-locator cases

### Same-Locator Lineage Proof

Each same-locator case must prove that the first observed duplicate disappeared
and the second observed duplicate survived into the current locator. The runner
must mechanically enforce:

- the observed document has exactly two exact-byte matches for the observed
  target bytes
- the current document has exactly one exact-byte match for the observed target
  bytes
- the first and second observed matches have distinct increasing byte starts
- the current document equals
  `observed_prefix_before_first_matching_target + observed_suffix_starting_at_second_matching_target`
- the SHA-256 of that reconstructed byte string matches the
  runner-owned `require_reconstructed_current_document_sha256`

The lineage proof is evidence, not a new mutation mechanism. It exists to
demonstrate wrong-target survival under the same live locator after the first
duplicate is removed.

## Token Framing Contract

Every candidate token preimage uses:

- ASCII domain label `non-block-target-identity-token`
- unsigned 64-bit big-endian byte length plus raw bytes for probe schema
  version
- unsigned 64-bit big-endian byte length plus raw bytes for candidate name
- unsigned 64-bit big-endian byte length plus raw bytes for target surface
- unsigned 64-bit big-endian byte length plus raw bytes for each candidate
  payload field

Candidate payload continuity with `probes/target_state_etag` is exact:

- `content_only`: `target_bytes`
- `target_local`: `canonical_descriptor_utf8`, `target_bytes`
- `ambiguity_reject`: `target_bytes`
- `document_target_state`: `canonical_descriptor_utf8`, `target_bytes`,
  `document_bytes`

`ambiguity_reject` does not hash the current match set. It reuses the
`target_bytes` framing and fails closed unless the current-domain exact-byte
match count is exactly one.

## Candidate Decisions And Scoring

For each case, the runner must:

1. Materialize the observed and current document strings exactly as UTF-8.
2. Resolve the observed target from the observed document using
   `observed_target_query`.
3. Resolve the current target from the current document using
   `current_target_query`.
4. Resolve the current domain from the current document using
   `current_domain_query`.
5. Count current exact-byte matches against the observed target bytes.
6. For same-locator cases, resolve the observed domain with the same domain
   query shape and enforce the lineage proof before scoring.

The candidate decisions are continuous with the prior probe:

- `content_only`: accept when the current target bytes reproduce the observed
  target-bytes token
- `target_local`: accept when the current target bytes and canonical
  descriptor both reproduce the observed token
- `ambiguity_reject`: accept only when the current target bytes reproduce the
  observed token and the current-domain match count is exactly one
- `document_target_state`: accept when the current document bytes, current
  target bytes, and current canonical descriptor all reproduce the observed
  token

Scoring is identity-first:

- a same-target reject is a required-same-state failure
- a wrong-target accept is a wrong-identity failure
- an unrelated-edit reject is a whole-document false-conflict cost when it
  comes from `document_target_state`

Promotion and demotion are deterministic:

- promote a candidate only if every required case avoids wrong-identity accept
  and avoids required same-state reject
- demote `document_target_state` if any unrelated-edit false conflict appears
- otherwise demote the candidate that incurred the failure

## Deterministic Artifact Contract

The canonical report is deterministic JSON emitted only by the authorized
`--output PATH` mode. `--check PATH` compares bytes in memory and never
rewrites.

The report must preserve:

- top-level `candidate_summary`
- top-level `cases`
- top-level `overall_graduation_verdict`
- per-case candidate token digests
- per-case wrong-identity and same-target-reject evidence
- per-case same-locator lineage evidence when applicable

Canonical JSON uses sorted object keys except for the fixed candidate ordering,
two-space indentation, UTF-8, and a trailing newline.

## Security Boundary

`probe.py` is local-only and Python-stdlib-only. It may run only an explicit
caller-supplied repository-local `md` binary via `subprocess.run` with an
argument vector, `shell=False`, `env={}`, captured output, and an ephemeral
temporary working directory.

The only durable write mode is `--output PATH` via atomic same-directory
replacement. The probe must not write any other durable artifacts and must not
perform network access, dependency installation, or environment-derived
authority lookup.
