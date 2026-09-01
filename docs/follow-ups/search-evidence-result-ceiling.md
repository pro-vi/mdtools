# Search evidence result ceiling

Status: deferred to I2 U7 performance and release review.

## Problem

Search returns every overlapping match and advances by one source byte after
each match. A large parser-unrepresented region containing repetitive text can
therefore allocate a result vector proportional to the number of overlapping
matches. U5 makes parser-unrepresented bytes searchable, widening this existing
search behavior to source-gap evidence.

## Affected invariant

One bounded query must not consume unbounded memory merely because valid source
contains many overlapping matches.

## Concrete failure sequence

1. Parse a multi-megabyte document whose source is one parser-unrepresented
   region containing repeated `a` bytes.
2. Search for `aa` with `include_source_gaps: true`.
3. The matcher emits nearly one result per source byte and retains every result
   in memory before returning.

## Why U5 triage does not set a limit

A fixed ceiling would introduce truncation or rejection semantics that the v3
wire does not currently represent. Choosing a number without workload evidence
would create a new public contract rather than repair the reviewed wire.

## Required seams

- Decide whether the contract is bounded rejection, explicit truncation state,
  pagination, or non-overlapping matches.
- Define how the CLI and Rust API report the bound.
- Measure target-backed and source-gap searches over repetitive inputs.
- Keep ordering and evidence-family behavior identical below the bound.

## Acceptance tests

- A documented large repetitive input stays within the selected memory/result
  bound.
- The response explicitly distinguishes complete from incomplete results if
  truncation is permitted.
- Target-backed and source-gap evidence obey the same result policy.
- Ordinary searches preserve exact spans, ordering, and overlapping behavior
  unless the accepted contract deliberately changes it.

## Temporary containment

No containment is added. Callers should avoid highly repetitive multi-megabyte
queries until U7 resolves the contract.

## Relationship and land impact

The behavior predates U5 for indexed blocks and is widened to parser-
unrepresented regions by U5. The v3 authority boundary remains sound, so this
is non-blocking for U5 and must be decided before I2 U7 completes.

Owner/backlog location: I2 U7.
