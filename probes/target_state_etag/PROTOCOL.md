# Target-State Etag Authority Protocol

Date locked: 2026-07-18

## Scope

This probe is a review-only scaffold for comparing four stateless authority
candidates against the read projections that mdtools already exposes through
`md ... --json`.

Authoritative projection surfaces for this probe:

- `md blocks FILE --json` -> `blocks[*]` with `index`, `span`, `etag`, and exact
  block bytes recoverable from the source slice.
- `md section SELECTOR FILE --json` -> `section` plus `content`, selector
  metadata, heading metadata, `span`, and `etag`.
- `md table FILE --json` and `md table FILE --index N --json` -> table list/read
  descriptors with `block_index`, `span`, `etag`, `headers`, and row/column data.
- `md tasks FILE --json` -> `results[*].tasks[*]` with `loc`, `child_path`,
  `span`, `etag`, `summary_text`, and nearest-heading context.

Frontmatter is out of scope. `md frontmatter --json` owns a whole-frontmatter
state token already, and this probe must not reinterpret or replace it.

## Hypothesis

`document_target_state` is the only stateless candidate that can reject copied or
shifted wrong-identity substitutions across block, section, table, and task
surfaces while still accepting unchanged rereads, exact-byte reversion, CRLF
bytes, and multibyte UTF-8 bytes when descriptors come from actual mdtools
projections.

## Candidate Framing

All candidates must use explicit domain separation and unambiguous length framing.
The framing notation below is descriptive, not yet executable:

- `content_only`:
  `content_only\0surface\0u32(len(target_bytes))\0target_bytes`
- `target_local`:
  `target_local\0surface\0u32(len(locator_descriptor))\0locator_descriptor\0u32(len(target_bytes))\0target_bytes`
- `ambiguity_reject`:
  `ambiguity_reject\0surface\0u32(len(target_bytes))\0target_bytes\0u32(len(current_match_set))\0current_match_set`
- `document_target_state`:
  `document_target_state\0surface\0u32(len(locator_descriptor))\0locator_descriptor\0u32(len(target_bytes))\0target_bytes\0u32(len(document_bytes))\0document_bytes`

Locator descriptors must be taken from the real projection surface for the target:

- block: `index` plus `span`
- section: selector metadata plus heading metadata plus `span`
- table: `block_index` plus `span`
- task: `loc` plus `child_path` plus `span`

## Minimum Experiment

1. Read the target only through the projection authority above.
2. Materialize the exact document bytes declared in `cases.json`.
3. Evaluate each candidate against the case's observed target and candidate
   document state without inventing a shadow parser.
4. Record both the candidate decision (`accept` or `reject`) and whether that
   decision deserves credit.

## Disconfirming Evidence

The hypothesis is false if any of the following occurs:

- `content_only`, `target_local`, or `ambiguity_reject` rejects every wrong
  identity case in the manifest without adding hidden state.
- `document_target_state` accepts the wrong target in a copied-token or
  same-locator substitution.
- `document_target_state` cannot be reconstructed from actual mdtools
  projections plus exact document bytes.
- Any candidate fails unchanged reread, exact-byte reversion, CRLF, or multibyte
  UTF-8 cases for byte-identity reasons alone.

## Required Scenarios

The deterministic manifest must cover:

- unchanged reread
- duplicate-content targets
- copied-token substitution
- same-locator substitution
- unrelated edit conflict cost
- exact-byte reversion
- CRLF bytes
- multibyte UTF-8 bytes
- descriptor availability for block, section, table, and task projections

The same-locator substitution case is special: if uniqueness returns after the
wrong target occupies the same locator, `ambiguity_reject` and `target_local`
must not receive credit for accepting that shifted identity.

## Actual Projection Requirement

Later authorized execution may only derive target descriptors from the live
output of the commands listed in Scope. The probe must not substitute:

- a handwritten Markdown parser
- filesystem metadata
- path-bound durable identity
- nonces
- hidden persistence

## Security And Execution Boundary

This run is static review only. The scaffold may describe later use of a local
`md` binary, but it must not be executed, imported, or compiled during this run.
Later authorized execution must remain local-only, use explicit paths, avoid
network access, avoid environment-derived authority, avoid `shell=True`, and use
ephemeral temporary files only.

## Evidence Fields

Each later measured row should record at least:

- `case_id`
- `surface`
- `projection_command`
- `locator_descriptor`
- `target_bytes_utf8`
- `document_bytes_utf8`
- `candidate`
- `decision`
- `credit`
- `reason`

## Promotion / Demotion Rule

Promote a candidate only if it:

- accepts unchanged reread, exact-byte reversion, CRLF, and multibyte UTF-8
- rejects every wrong-identity substitution case
- uses only the declared stateless framing

Demote a candidate immediately on the first silent wrong-target accept, or if it
depends on frontmatter state, hidden persistence, or a fifth hybrid authority.
