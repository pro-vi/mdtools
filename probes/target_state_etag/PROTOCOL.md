# Target-State Etag Deterministic Manifest Protocol

Date locked: 2026-07-18

## Scope

This probe is a complete static specification for a local-only experiment that
compares four stateless candidates:

- `content_only`
- `target_local`
- `ambiguity_reject`
- `document_target_state`

The mutable probe surface for this run is limited to:

- `probes/target_state_etag/PROTOCOL.md`
- `probes/target_state_etag/cases.json`
- `probes/target_state_etag/probe.py`

This run is source-only. Do not execute or import `probe.py`, do not parse
`cases.json` through the probe, and do not author result artifacts in this run.

## Read Authority

All descriptor authority must come from live `md ... --json` read commands over
the observed and current documents materialized from `cases.json`. The runner
must not embed projection snapshots, invent a handwritten Markdown parser, or
substitute filesystem metadata for descriptor authority.

### Block

- Target query command: `md blocks FILE --json`
- Query key: zero-based `block_index`
- Exact target bytes: source slice at `blocks[block_index].span`
- Canonical descriptor: `{ "kind", "index", "span" }`
- Current-domain query: `md blocks FILE --json`

### Section

- Target query command:
  `md section SELECTOR FILE [--occurrence N] [--contains] [--ignore-case] --json`
- Query keys: `selector`, optional one-based `occurrence`, `contains`,
  `ignore_case`
- Exact target bytes: returned `content`
- Canonical descriptor:
  `{ "kind", "selector", "heading", "depth", "block_indices", "span" }`
- Current-domain query:
  `md section SELECTOR FILE --occurrence N --json`, enumerating one-based
  occurrences until the command stops resolving a match

### Table

- Target query command: `md table FILE --index BLOCK_INDEX --json`
- Query key: `block_index`
- Exact target bytes: source slice at returned `span`
- Canonical descriptor: `{ "block_index", "span", "headers" }`
- Current-domain query: `md table FILE --json`

### Task

- Target query command: `md tasks FILE --json`
- Query keys: zero-based `result_index` and zero-based `task_index`
- Exact target bytes: source slice at
  `results[result_index].tasks[task_index].span`
- Canonical descriptor:
  `{ "loc", "block_index", "child_path", "task_index", "depth", "nearest_heading", "nearest_heading_block_index", "span" }`
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

- `content_only`: accept when the current-domain match count is at least one.
- `target_local`: accept when the current target bytes and canonical descriptor
  both exactly match the observed target bytes and canonical descriptor.
- `ambiguity_reject`: accept when the current-domain match count is exactly one.
- `document_target_state`: accept when the current document bytes, current
  target bytes, and current canonical descriptor all exactly match the observed
  document bytes, target bytes, and canonical descriptor.

Credits come from the manifest:

- `correct`
- `wrong_identity`
- `false-conflict`

## Load-Bearing Same-Locator Case

The `same-locator` duplicate-shift case is mandatory and must fail
mechanically unless all of the following are observed from real `md` output:

- observed and current target exact bytes are equal
- observed and current canonical block descriptors are equal
- current ambiguity count is exactly one
- observed and current whole-document bytes differ

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

Later local execution may use a repository-local `md` binary plus ephemeral
temporary files only. It must remain local-only, use explicit paths, avoid
network access, avoid environment-derived authority, and keep `shell=False`.
