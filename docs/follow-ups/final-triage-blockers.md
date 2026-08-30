# Final triage blockers

These items survived review of PR #43 and remain merge-blocking.

## Lossless frontmatter field edits

Whole-block YAML/TOML serialization can rewrite mapping-key syntax outside the
target field. A complete fix must preserve non-target bytes while supporting
quoted and empty key paths; refusing every quoted key would break valid exact
addresses. Acceptance: mutate one field without changing comments, quoting,
flow style, anchors, empty keys, line endings, or unrelated bytes.

## Closed heading-link parent addresses

The public schema permits `LinkParentAddress::Heading` to contain the preamble,
while decoding rejects it. Introduce a heading-only section-address type shared
by the address, decoder, and generated schema. Acceptance: every schema-valid
link address decodes, and preamble heading parents are schema-invalid.

## Race-free regular-file opening

The file adapter checks path metadata before opening the path, so a special file
can replace the regular file between those operations. Open with nonblocking
Unix flags, validate the opened handle with `fstat`, and use that handle for
identity capture. Acceptance: a FIFO swap cannot block load or commit, and
regular files and symlink referents retain current behavior.
