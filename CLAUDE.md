# mdtools contributor notes

## Architecture

- `Document` owns immutable source, parse policy, revision, and one
  source-ordered `DocumentIndex`.
- `TargetQuery` is fuzzy discovery. `TargetAddress` is exact identity.
- `TargetSnapshot` separates selection from `GuardAuthority`.
- `ResolvedTarget` is bound to one document-index instance.
- Reads stay typed by Markdown domain.
- Search returns target-backed `EvidenceRange` or targetless
  `SourceEvidenceRange`; neither is mutation authority.
- `Patch` is one-base, fully preflighted, non-overlapping, applied once, and
  reparsed once before receipts are finalized.
- Core code performs no filesystem I/O. The `file` feature owns verified atomic
  commit safety.
- Rust protocol types generate JSON Schema and shared five-command metadata.

## CLI

The public binary has exactly five commands:

```text
md map <FILE>
md read <FILE> --address <JSON> | --from <PATH|-> | --query <JSON>
md query <FILE> --query <JSON> | --from <PATH|->
md patch <FILE> --patch <JSON> | --from <PATH|-> [--in-place]
md schema
```

CLI code may decode inputs, call library operations, and render outputs. It may
not contain Markdown traversal or mutation semantics.

Default `map` and `query` output is compact discovery JSON. Default `read`
output is original content (or `{present, value}` for frontmatter fields).
`--json` selects full protocol results, including snapshots and guards, for
patch preparation and programmatic consumers. Keep both output shapes in the
generated protocol schema; do not make agents read patch evidence for discovery.
Query reads use `Document::query_one` and the selected document's existing parse
policy. Measurement stays in the binary, is opt-in, records no payloads, and
must preserve output and exit behavior when counting or logging fails.

## Parser boundary

`comrak = "=0.51.0"` is exact-pinned. Comrak types stay inside `src/parser.rs`.
When upgrading, run all fixtures, especially CRLF, setext, footnotes,
multiline code spans, frontmatter, tables, and multibyte source spans.

## Verification

```sh
cargo test --all-targets
cargo check --no-default-features
cargo check --features file
cargo fmt --all -- --check
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --document-private-items
cargo clippy --all-targets -- -D warnings -A clippy::enum_variant_names -A clippy::large_enum_variant
git diff --check
```

Do not add a second selector, guard, patch, receipt, command inventory, or
schema authority.
