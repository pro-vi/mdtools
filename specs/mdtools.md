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
- A file change after preparation refuses commit. No-change patches verify but
  do not replace the file.
- A leading `---` or `+++` line is treated as frontmatter intent. Frontmatter
  mutation refuses when that intent does not form a valid mutable block; callers
  must disambiguate a leading thematic break before adding frontmatter.

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
