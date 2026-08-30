# mdtools

`mdtools` provides one immutable indexed Markdown document, exact target
addresses, typed reads, and guarded patch transactions. The `md` binary is a
thin JSON adapter over the same Rust protocol.

## Install

```sh
cargo install --path .
```

The binary exposes five commands:

```text
md map <FILE>
md read <FILE> --address <TARGET_ADDRESS_JSON>
md query <FILE> --query <TARGET_QUERY_JSON>
md patch <FILE> --from <PATCH_JSON_FILE> [--in-place]
md schema
```

`map`, `read`, `query`, and in-place patch receipts emit JSON. A patch without
`--in-place` writes the candidate Markdown to stdout and does not modify the
file. `md --json patch` emits a structured preview containing source and
receipts.

Use `-` with `--from` to read JSON from stdin. No command prompts.

## Examples

```sh
md map README.md | jq '.[] | {kind, address}'

md read README.md --address '{"kind":"preamble"}'

md query README.md --query \
  '{"type":"search","text":"guard","match_mode":"literal","block_kinds":[]}'

md schema | jq '.patch'

md patch README.md --from patch.json          # candidate to stdout
md patch README.md --from patch.json --in-place
```

Every mutation carries a document revision and target guard. The file adapter
rechecks the canonical referent, source revision, and Unix device/inode before
staging and immediately before atomic rename. Permissions are preserved and
no-change patches verify the file without replacing it.

## Rust library

```toml
[dependencies]
mdtools = { path = ".", default-features = false }
```

Core parsing and patching are source-in/source-out. Enable `file` for verified
filesystem commits or `cli` for the binary:

```toml
mdtools = { path = ".", features = ["file"] }
```

The authoritative wire shapes and command metadata come from
`mdtools::protocol::protocol_schema()`.
