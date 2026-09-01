# One lossless index separates source coverage from mutation authority

**Date:** 2026-08-31
**Status:** accepted 2026-08-31 — landed with the v0.4 implementation
**Deciders:** Provi, Codex

## Context

`DocumentIndex` previously retained a second parsed-document projection and
used parser indices to reconnect source bytes with reads, search, fragments,
tables, sections, and patch closure. The parser can also omit valid source such
as unreferenced footnote and link-reference definitions. Patch planning
preserved those bytes through local gap reconstruction, while search had no
honest evidence type for them.

Semantic Markdown structure and exact source coverage have different shapes.
Sections, blocks, tasks, rows, and links overlap. Exact source ownership must be
disjoint. Treating parser omissions as mutable targets would claim structure
the parser did not establish.

## Decision

`DocumentIndex` is the sole long-lived representation of one document. It owns:

- one exact `DocumentSource` with line coordinates, revision, parse policy, and
  line-ending state;
- an overlapping semantic node graph for addresses, reads, and patches;
- a disjoint `SourceRegion` ledger covering every source byte exactly once.

Parser facts are ephemeral construction input and are dropped before a
`Document` returns. Parser traversal order and parser indices are not runtime
identity or ordering authority.

Parser-unrepresented source remains non-mutable. Search may return it only as a
targetless `SourceEvidenceRange` carrying revision, exact span, etag, and
preview. It has no address, guard, read variant, or patch conversion. The wire
contract is `mdtools.v3` with no v2 compatibility layer.

Every search query also supplies `max_results`. Exceeding that caller-owned
budget returns an error and no partial result.

## Rationale

Two coordinated views inside one index preserve both truths: semantic nodes can
overlap, while lexical regions form a byte partition. A second retained parser
projection would keep competing authorities. Synthetic source-gap targets were
rejected because parser absence cannot authorize mutation. Silent result
truncation was rejected because the existing result array has no completeness
field; an explicit fail-closed budget keeps the response honest.

## Consequences

Positive:

- Every source byte has one preservation owner and reconstructs exactly.
- Reads, search, statistics, fragments, and patch transactions use index nodes
  rather than parser vectors.
- Parser omissions become searchable without expanding mutation authority.
- Search result-vector allocation is bounded by a caller-selected result
  budget.

Negative:

- Parsing stores source-region metadata and performs coverage validation.
- `mdtools.v3` breaks v2 search-query and evidence consumers.
- Region-granular search includes separator whitespace absorbed by a parser-
  unrepresented region; standalone boundary regions remain unsearched.
- Search returns an error rather than partial results when `max_results` is
  exceeded.
- Case-insensitive search still allocates a folded copy proportional to each
  scanned block or source region; `max_results` bounds retained evidence, not
  scan working memory.

## Revisit Triggers

- A parser upgrade retains currently omitted source as stable semantic nodes.
- A consumer needs incremental parsing or editor-session mutation.
- A consumer needs paginated or explicitly truncated search results rather than
  fail-closed bounded queries.
- Source-region metadata causes a measured regression beyond the accepted
  parsing budget.

## References

- Source ownership: `src/source.rs`, `src/index.rs`
- Parser boundary: `src/parser.rs`
- Search evidence: `src/search.rs`, `src/target.rs`
- Transaction preservation: `src/patch/planner.rs`
- Contract tests: `tests/source_evidence.rs`, `tests/transaction_invariants.rs`
- Residue enforcement: `tests/architecture_residue.rs`
- Release record: `CHANGELOG.md` (`v0.4.0`)
