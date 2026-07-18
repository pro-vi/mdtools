# Target-State Etag Deterministic Manifest Protocol

Date locked: 2026-07-18

## Scope

This probe is a complete static specification for a local-only experiment that
compares four stateless candidates:

- `content_only`
- `target_local`
- `ambiguity_reject`
- `document_target_state`

The original source-construction phase was source-only and limited authorship
to `probes/target_state_etag/PROTOCOL.md` and
`probes/target_state_etag/probe.py`. That phase did not authorize executing or
importing `probe.py`, did not authorize parsing `cases.json` through the probe,
and did not authorize authoring result artifacts.

A later separately authorized execution used hash-locked
`probes/target_state_etag/probe.py`, `probes/target_state_etag/cases.json`, and
the execution-time `probes/target_state_etag/PROTOCOL.md` to produce the
committed canonical `probes/target_state_etag/results.json` and
`probes/target_state_etag/RESULTS.md`.

Current local reproduction authority is non-mutating:

- `cargo build --release --locked --offline`
- `python3 probes/target_state_etag/probe.py --md-binary target/release/md --check probes/target_state_etag/results.json`

The build boundary is fail-closed: `--locked` prevents `Cargo.lock` edits or
dependency-resolution drift, and `--offline` prevents Cargo registry or
download access during local reproduction. A missing dependency cache is a
failed reproduction precondition, not an invitation to relax the build
contract. Any dependency acquisition or cache priming is operator setup outside
the local probe reproduction boundary and requires separate authorization.

Authorized regeneration is explicit and separate from reproduction:

- `cargo build --release --locked --offline`
- `python3 probes/target_state_etag/probe.py --md-binary target/release/md --output probes/target_state_etag/results.json`
- `python3 probes/target_state_etag/probe.py --md-binary target/release/md --check probes/target_state_etag/results.json`

Regeneration is not an incidental read operation. Any authorized refresh must
stay inside the same locked offline build boundary, review the resulting byte
diff, and synchronize the ledger and protocol hashes before treating the
regenerated JSON as canonical again.

All authorized execution stays inside the local security boundary: use the
repository-local `target/release/md` binary, explicit repository paths,
ephemeral temporary files only, no network, no environment-derived authority,
and `shell=False` subprocess execution.

## Read Authority

All descriptor authority must come from live `md ... --json` read commands over
the observed and current documents materialized from `cases.json`. The runner
must not embed projection snapshots, invent a handwritten Markdown parser, or
substitute filesystem metadata for descriptor authority.

### Block

- Target query command: `md blocks FILE --json`
- Query key: zero-based `block_index`
- Exact target bytes: source slice at `blocks[block_index].span`
- Canonical descriptor: `{ "index", "span" }`
- Current-domain query: `md blocks FILE --json`

### Section

- Target query command:
  `md section SELECTOR FILE [--occurrence N] [--contains] [--ignore-case] --json`
- Query keys: `selector`, optional one-based `occurrence`, `contains`,
  `ignore_case`
- Exact target bytes: source slice at returned `section.span`
- Canonical descriptor: `{ "selector", "heading", "span" }`
- Current-domain query:
  `md section SELECTOR FILE --occurrence N --json`, enumerating one-based
  occurrences until the command stops resolving a match

### Table

- Target query command: `md table FILE --index BLOCK_INDEX --json`
- Query key: `block_index`
- Exact target bytes: source slice at returned `span`
- Canonical descriptor: `{ "block_index", "span" }`
- Current-domain query: `md table FILE --json`

### Task

- Target query command: `md tasks FILE --json`
- Query keys: zero-based `result_index` and zero-based `task_index`
- Exact target bytes: source slice at
  `results[result_index].tasks[task_index].span`
- Canonical descriptor: `{ "loc", "child_path", "span" }`
- Current-domain query: `md tasks FILE --json`

Frontmatter is excluded. This probe does not reinterpret `md frontmatter`.

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
- per-candidate expected `decision` and `credit`

Validation is mechanical and fail-closed:

- reject missing required keys
- reject duplicate `case_id` values
- reject case IDs that do not match `^[a-z0-9]+(?:-[a-z0-9]+)*$`
- reject unknown `surface` values
- reject unknown query `type` values
- reject query shapes that do not match the declared `surface`
- reject out-of-range block, table, result, or task indexes
- reject section queries whose selector or occurrence inputs do not resolve
- reject cases whose candidate matrix omits any of the four candidate names

`observed_document_utf8` and `current_document_utf8` must use standard JSON
escapes only. `json.load` must yield the exact LF, CRLF, and multibyte UTF-8
text that the runner writes to disk. The runner must never apply a second escape
decoder.

Target bytes are always authoritative byte slices from the temporary document
raw bytes at the resolved `span.byte_start..span.byte_end`. Returned `content`,
`preview`, `summary_text`, `etag`, manifest-owned spans, or handwritten parsing
must never replace that slice.

## Token Framing Contract

Every candidate token preimage uses:

- ASCII domain label `target-state-etag-token`
- unsigned 64-bit big-endian byte length plus raw bytes for probe schema
  version
- unsigned 64-bit big-endian byte length plus raw bytes for candidate name
- unsigned 64-bit big-endian byte length plus raw bytes for target surface
- unsigned 64-bit big-endian byte length plus raw bytes for each candidate
  payload field

The payload field lists are:

- `content_only`: `target_bytes`
- `target_local`: `canonical_descriptor_utf8`, `target_bytes`
- `ambiguity_reject`: `target_bytes`
- `document_target_state`: `canonical_descriptor_utf8`, `target_bytes`,
  `document_bytes`

`ambiguity_reject` does not hash a current match set. It reuses the
`target_bytes` token framing and fails closed unless the current-domain exact
byte match count is exactly one.

## Candidate Decision Rules

For each case, the runner must:

1. Materialize the observed and current document strings exactly as UTF-8.
2. Resolve the observed target from the observed document using
   `observed_target_query`.
3. Resolve the current target from the current document using
   `current_target_query`.
4. Resolve the current domain from the current document using
   `current_domain_query`.
5. Count current exact-byte matches against the observed target bytes.

The candidate decisions are:

- `content_only`: accept when the current target bytes selected by
  `current_target_query` reproduce the observed target-bytes token.
- `target_local`: accept when the current target bytes and canonical descriptor
  both exactly match the observed target bytes and canonical descriptor.
- `ambiguity_reject`: accept only when the current target bytes selected by
  `current_target_query` reproduce the observed target-bytes token and the
  current-domain match count is exactly one.
- `document_target_state`: accept when the current document bytes, current
  target bytes, and current canonical descriptor all exactly match the observed
  document bytes, target bytes, and canonical descriptor.

Credits come from the manifest:

- `correct`
- `wrong_identity`
- `false-conflict`

## Aggregate Report Contract

The canonical report is the deterministic JSON artifact at
`probes/target_state_etag/results.json`, produced by the authorized output
command and revalidated by the canonical check command. Each candidate has one
global disposition across the required ten-case matrix, and any
wrong-identity acceptance in any required case is sufficient to demote that
global candidate. The block cases contain the wrong-target, duplicate, and
same-locator identity challenges. The section, table, and task cases validate
live descriptor reconstruction in unchanged state only and do not
independently prove wrong-target identity on those surfaces. No per-surface
candidate graduation or identity claim is made, and position-bound target identity is a separate focused architectural decision.

The report must contain:

- top-level `candidate_summary`
- top-level `cases`
- one singular top-level `overall_graduation_verdict` object

Per-case `candidate_results` are evidence only. They must preserve:

- `decision`
- `expected_decision`
- `expectation_match`
- `credit`
- token digests
- `wrong_identity_accept`
- `required_same_state_reject`
- `unrelated_edit_conflict`
- `whole_document_false_conflict_cost`

Per-case `candidate_results` must not expose a promotion field and must not
decide experiment-level selection.

Each `candidate_summary[CANDIDATE]` must be emitted in canonical candidate
order and must contain:

- `accepts`
- `rejects`
- `expectation_matches`
- `wrong_identity_accepts`
- `required_same_state_rejects`
- `unrelated_edit_conflicts`
- `graduation_verdict`
- `disposition`

`unrelated_edit_conflicts` counts that candidate's per-case
`unrelated_edit_conflict` evidence. This aggregate count is not restricted to
`document_target_state`, even though only that candidate can fail the
whole-document false-conflict verdict step below.

`graduation_verdict` is derived mechanically from aggregate actual outcomes with
this precedence:

1. `fails_wrong_identity` when `wrong_identity_accepts > 0`
2. `fails_whole_document_false_conflict` only for `document_target_state` when
   `unrelated_edit_conflicts > 0`
3. `fails_required_same_state` when `required_same_state_rejects > 0`
4. `graduates` otherwise

`disposition` is `promote` only when `graduation_verdict` is `graduates`;
otherwise it is `demote`.

The top-level `overall_graduation_verdict` object must contain:

- `candidate_verdicts`
- `graduating_candidates`
- `demoted_candidates`
- `selected_candidate`
- `verdict`
- `whole_document_false_conflict_cost`

`candidate_verdicts` maps each canonical candidate name to its aggregate
`graduation_verdict`.

`graduating_candidates` and `demoted_candidates` are canonical-order lists
derived from each candidate summary `disposition`.

`selected_candidate` is the candidate name only when exactly one candidate
graduates; otherwise it is `null`.

`verdict` is derived only from the count of graduating candidates:

- `no_candidate_graduates`
- `single_candidate_graduates`
- `multiple_candidates_graduate`

No tie-break policy may be invented when multiple candidates graduate.

`whole_document_false_conflict_cost` is the
`candidate_summary["document_target_state"]["unrelated_edit_conflicts"]` count.

## Load-Bearing Same-Locator Case

The `same-locator` duplicate-shift case is mandatory and must fail
mechanically unless all of the following are observed from real `md` output:

- observed and current target exact bytes are equal
- observed and current canonical block descriptors are equal
- current ambiguity count is exactly one
- observed and current whole-document bytes differ

The authorized document pair for this case is a shifted-identity pair: the
observed query selects the first of two duplicate blocks, and the current query
selects the surviving second duplicate after the first duplicate is removed,
while the surviving duplicate now occupies the same canonical block
`index`/`span` locator.

If any precondition is false, the runner must stop with a hard mechanical
failure rather than silently scoring the case.

## Required Case Matrix

The manifest must contain exactly these ten case IDs:

- `block-unchanged-reread`
- `block-duplicate-cross-target-copy`
- `block-same-locator-duplicate-shift`
- `block-unrelated-edit-false-conflict`
- `block-exact-byte-reversion`
- `block-unchanged-crlf-bytes`
- `block-unchanged-multibyte-utf8-bytes`
- `section-unchanged-real-descriptor`
- `table-unchanged-real-descriptor`
- `task-unchanged-real-descriptor`

Those ten cases encode the required scenario matrix:

- unchanged block reread
- static duplicate block cross-target copy
- same-locator duplicate shift
- unrelated edit after an unchanged target
- exact-byte reversion
- unchanged CRLF block bytes
- unchanged multibyte UTF-8 block bytes
- unchanged section descriptor from a real section command
- unchanged table descriptor from a real table command
- unchanged task descriptor from a real tasks command

## Static Boundary

All authorized local execution uses a repository-local `md` binary plus
ephemeral temporary files only. It remains local-only, uses explicit paths,
avoids network access, avoids environment-derived authority, and keeps
`shell=False`.
