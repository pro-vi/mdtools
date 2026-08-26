# Search match etags fingerprint exact spans

**Date:** 2026-08-26
**Status:** accepted 2026-08-26
**Deciders:** Provi, Codex

## Context

Search results exposed an exact source span and a lossy preview but no content
fingerprint. A CLI or process consumer therefore had to fetch another structural
target to obtain a producer-issued fingerprint. A same-process library caller
could fingerprint its immutable `Document` slice directly, but search did not
return that token with the hit. Search already used one public `SearchMatch`
type for both the Rust library result and the CLI's JSON record, while newer
operations used separate typed library and string-valued wire records.

## Decision

Keep one shared `SearchMatch` and add a typed `TargetEtag` field to it. The etag
fingerprints only the non-empty original-source bytes covered by `match_span`.
It does not fingerprint the preview or enclosing block, and it does not identify
one occurrence among byte-identical matches.

`TargetEtag` serializes to its established lowercase 16-character string but
does not gain an unchecked deserialization path. `md search --json` serializes
the same record returned by `search::search`, and `md schema --json` advertises
the additive field through `search_match_etag`. The JSON protocol remains
`mdtools.v1`; the required Rust field ships at the pre-1.0 `0.2.0` boundary.

## Rationale

A separate core and wire search record would duplicate the complete hit shape
solely to convert one field. Keeping the existing shared record makes library
and JSON parity structural. Hashing `match_span` preserves the smallest exact
source boundary the search result owns; adding a whole-block etag would describe
a different target and would not remove the exact-hit verification gap.

## Consequences

Positive:

- Search returns the same exact-span evidence directly to library and JSON
  consumers.
- Unicode case mapping, repeated text, and multifile output share one etag
  construction path.
- Plain tab-separated search output remains unchanged.

Negative:

- Adding the public field breaks downstream Rust struct literals and exhaustive
  destructuring.
- Byte-identical matches share an etag, so consumers must address an occurrence
  by source/document identity plus span; CLI consumers normally use file plus
  span.
- The shared library/wire record keeps serialization in the domain type's public
  contract.

## Revisit Triggers

- A consumer needs to deserialize `SearchMatch` through mdtools itself: add a
  validating `TargetEtag` deserializer or establish a separate wire record.
- The CLI protocol needs a representation that no longer matches the library
  result without translation.
- A second verified consumer needs a different search-evidence granularity.

## References

- `src/fingerprint.rs`
- `src/model.rs`
- `src/search.rs`
- `src/commands/schema.rs`
- `tests/library_search.rs`
- `tests/cli_search.rs`
- `tests/cli_multifile.rs`
- `CHANGELOG.md` (`v0.2.0`)
