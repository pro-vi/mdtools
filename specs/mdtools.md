# mdtools structural protocol

The executable protocol is generated from Rust types:

```sh
md schema
```

This document records semantic rules only. It does not copy Rust declarations
or command inventories.

## Document and targets

- `Document` owns immutable source, revision, parse policy, and one
  source-ordered `DocumentIndex`.
- AST depth is checked iteratively before recursive semantic projection.
- `TargetQuery` performs discovery. `TargetAddress` contains exact identity
  only.
- `TargetSnapshot` separates its optional selected span from
  `GuardAuthority`.
- `ResolvedTarget` is bound to one document index instance.
- Reads remain typed by Markdown domain.
- Search returns `EvidenceRange`; evidence cannot enter a patch as mutation
  authority.

## Patches

- A `Patch` has one base revision and one or more closed `PatchOp` variants.
- Every operation supplies observed target or insertion evidence.
- Planning completes guards, semantic claims, byte edits, result expectations,
  and receipt drafts before any edit is applied.
- Claims and byte edits must not overlap. Operations cannot depend on targets
  created earlier in the same patch.
- Edits apply in descending byte order, followed by one same-policy reparse and
  operation-specific closure verification.
- Receipts carry distinct before and after identities bound to their respective
  revisions.

## Section fragments

- Semantic fragments contain one relative rooted section and render with
  library-owned boundaries and destination heading depth.
- Literal fragments preserve supplied non-empty bytes exactly.
- Semantic unchanged replacement preserves original ATX or setext bytes and
  returns `NoChange`.
- Preamble replacement is separately typed and literal.

## Files

- Core remains source-in/source-out.
- On Unix, the `file` feature resolves a regular canonical referent, captures
  file identity, preserves ownership and permission bits, owns its temp inode,
  syncs staged bytes, rechecks identity and revision, and atomically renames.
  The feature fails to compile on non-Unix targets until equivalent locking and
  identity primitives exist there.
- Writers that honor the same advisory lock serialize through verification and
  rename. A detected file change after preparation refuses commit; a writer
  that ignores the lock can still race the final verification. No-change
  patches verify but do not replace the file.
- A leading `---` or `+++` line is treated as frontmatter intent. Frontmatter
  mutation refuses when that intent does not form a valid mutable block; callers
  must disambiguate a leading thematic break before adding frontmatter. An empty
  delimiter pair is not a mutable frontmatter block under the pinned parser.
- `map` uses the lenient structural policy so malformed frontmatter remains
  visible as Markdown. An exact frontmatter `read` uses the strict semantic
  policy and may reject the same bytes as invalid frontmatter.

## CLI

The public CLI is exactly:

```text
map
read
query
patch
schema
```

CLI code only decodes protocol JSON, calls library operations, and renders
typed results or candidate source.

`map` and `query` default to compact JSON views derived from the same targets:
addresses and summaries for discovery, previews and spans for search evidence.
These views carry no revision, etag, or guard. Source evidence remains tagged
and targetless. `read` defaults to original selected content, preserving source
bytes without a wrapper or duplicate section fragment. Frontmatter fields emit
`{present, value}` to distinguish an absent field from JSON null.

`--json` selects the full `TargetSnapshot`, `QueryResult`, and `TargetRead`
protocol outputs used for patch preparation. Existing scripts that parse full
results must select this mode explicitly. The generated schema describes both
the compact views and full protocol types. Input addresses, query semantics,
patch guards, patch previews, and receipts are unchanged.

`read --query` selects exactly one target using `Document::query_one`.
`--address`, `--from`, and `--query` are mutually exclusive; `--from` remains
address-only. Search evidence is not a read selector. Frontmatter-directed
queries use the same strict read policy as exact frontmatter addresses, before
resolution; other queries keep the existing lenient structural policy. Query
enumeration may return no missing-field target even when its exact address can
represent an absent value. Resolution and reading use one document instance.

Invalid inputs retain the authoritative typed-deserialization error and exit
status. Short examples are serialized from existing protocol types; the raw
input's discriminator may choose guidance only after decoding fails. Examples
are not automatically executed repairs. Patch guidance requires observed guards.

Optional CLI usage measurements describe output streams, not protocol authority
or provider billing. Measurement failures cannot change command results, and a
committed patch remains successful if writing its receipt fails. No document
payloads or file identities are persisted in CLI measurements.
