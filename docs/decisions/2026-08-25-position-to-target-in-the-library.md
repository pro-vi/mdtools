# Position-to-target resolution belongs to the library

**Date:** 2026-08-25
**Status:** Proposed — implemented on `feat/locate-position-to-target`
**Deciders:** Provi (repo owner)

## Context

Every selector mdtools offers is by name: heading text, block index, task loc,
table row index. A reading UI built on this library does not hold names. It
holds positions — a click on a rendered block carrying comrak's
`data-sourcepos`, a scroll line, a byte range.

Only heading clicks had a route in, through `SectionTarget::heading`.
Paragraphs, task checkboxes, and table rows had none, so a consumer that wanted
to turn a click into a guarded edit had to interpret the source a second time:
build its own line index, find block boundaries, decide which task item a byte
belongs to. Two interpretations of one document is exactly the drift the etag
guards exist to prevent.

## Decision

Position-to-target resolution lives in the library, in `src/locate.rs`.

- `locate(document, byte_offset)` returns the enclosing top-level block, the
  innermost section, the innermost task item, and the table data row — each
  `Option`, each carrying the etag its own read path produces.
- `locate_line(document, line)` maps a 1-based line to its first byte through
  the parser's existing line index, so no consumer builds a second one.
- The returned records are the existing read types — `BlockRecord`,
  `SectionEntry`, `TaskRecord` — not new parallel ones. Their etags feed
  `set-task`, `replace-block`, `replace-section`, and the table-row mutations
  unchanged.
- A position between blocks is `Ok` with `block: None`, not an error. Only a
  position at or past the end of the source is an error.
- No `md locate` CLI command. This is a library surface until a consumer has
  confirmed its shape.

## Rationale

**Why the library, not the consumer.** The rejected alternative was a lookup in
the reading UI with its own line index. It fails on two counts: it is a second
interpretation of the source, which the consumer's own design forbids, and
every later consumer would rebuild the same search. The library already owns
`Document::blocks()` (sorted, non-overlapping spans), `BlockInfo.task_items`
(nested spans with child paths), and `extract_table_projection` (per-row
spans). Resolution here is one binary search plus lookups over data that
already exists; anywhere else it is a reimplementation.

**Why a blank line is not an error.** A click on whitespace inside a section is
a meaningful position for a UI — it names where a new block would go. Erroring
on a common click would put the same `match Err(Gap)` arm in every consumer,
and that arm would do nothing but discard information the caller wanted. The
section is still returned, so the position stays useful.

**Why the existing record types.** Introducing `LocatedBlock`, `LocatedSection`,
and so on would fork the etag discipline: two ways to name the same target,
which is the drift the guards exist to catch. Reusing the read types makes the
guarantee structural — a located target is the same value the corresponding
read returns, so it cannot carry a different fingerprint.

**One deliberate exception.** `LocatedTableRow` is new, because no existing type
fits: `TableRowTargetRef` is a mutation-*result* type whose `kind` field is a
mutation discriminator. Its guard token is named `table_etag`, not `etag`,
because the table-row mutations are guarded by the whole table block's bytes
rather than the row's — the name states the fact that would otherwise surprise
the reader.

## Consequences

Positive:

- A click drives a guarded edit with no second reading of the source.
- Every consumer gets the same resolution; none reimplements it.
- Locs stay cheap to re-derive, so the read → mutate → re-query cycle is
  unchanged.

Negative:

- `SectionIndex::new` is rebuilt on every `locate` call, O(blocks). Acceptable
  for click-driven use; a caller resolving many positions at once would need a
  batch variant.
- Position resolution is now a public contract the library must keep stable,
  including the blank-line rule and which of the two out-of-range errors a
  caller gets.
- `line_count()` counts the empty position after a trailing newline as a line,
  but its first byte is the end of the source — so `locate_line(doc,
  line_count)` errors `ByteOffsetOutOfRange`, not `LineOutOfRange`. That seam
  between the two error kinds is now load-bearing.

## Revisit Triggers

- A consumer measures `SectionIndex::new` rebuild cost as a real problem →
  add `locate_with(&SectionIndex, …)`.
- A consumer needs inline (sub-block) targets — a link or span inside a
  paragraph — which this decision explicitly leaves out of scope.
- The reading UI confirms the record shape in use → the deferred
  `md locate <LINE|BYTE> <FILE> --json` adapter becomes worth adding, and the
  two new `CoreError` variants would then need their own diagnostic codes
  rather than borrowing `block_index_out_of_range`.

## References

- Implementation: `src/locate.rs`, `src/section.rs` (`section_for_block`,
  `section_for_byte`), `src/document.rs` (`line_to_byte`)
- Tests: `tests/library_locate.rs`, `tests/library_section.rs`
- Boundary this rests on: `docs/decisions/2026-08-23-reusable-library-boundary.md`
