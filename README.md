# mdtools

`mdtools` provides one immutable indexed Markdown document, exact target
addresses, typed reads, and guarded patch transactions. The `md` binary is a
thin adapter over the same Rust protocol, with compact output for agent reads.

## Install

```sh
cargo install --path .
```

The binary exposes five commands:

```text
md map <FILE>
md read <FILE> --address <TARGET_ADDRESS_JSON>
md read <FILE> --query <TARGET_QUERY_JSON>
md query <FILE> --query <TARGET_QUERY_JSON>
md patch <FILE> --from <PATCH_JSON_FILE> [--in-place]
md schema
```

`map` and `query` return compact JSON for discovery: addresses and summaries,
or search previews and spans. `read` returns the selected original Markdown
once, without a JSON wrapper. Frontmatter field reads return `{present, value}`
so a missing field remains distinct from a present `null`.

Use `--json` for full protocol results, including snapshots, guards, and typed
reads. Scripts that previously parsed the default full results must add this
flag. Agent discovery and content reads should omit it; request a full snapshot
when preparing a patch.

In-place patch receipts emit JSON. A patch without `--in-place` writes the
candidate Markdown to stdout and does not modify the file. `md --json patch`
emits a structured preview containing source and receipts.

Use `-` with `--from` to read JSON from stdin. No command prompts.

`read --query` uses the existing query type and requires exactly one target.
Zero matches fail with exit 1; multiple matches fail with exit 4. Search evidence
cannot be read this way. `read --from` remains address-only. A missing frontmatter
field query has no match, while its exact address can report `present: false`.

## Examples

```sh
md query README.md --query '{"type":"kind","kind":"section"}'

md read README.md --address '{"kind":"preamble"}'

md read README.md --query \
  '{"type":"section","text":"Install","match_mode":"exact"}'

md read README.md --address '{"kind":"preamble"}' --json  # patch evidence

md query README.md --query \
  '{"type":"search","text":"guard","match_mode":"literal","block_kinds":[],"include_source_gaps":true,"max_results":100}'

md schema | jq '.patch'

md patch README.md --from patch.json          # candidate to stdout
md patch README.md --from patch.json --in-place
```

Search returns target-backed `evidence` for indexed Markdown and targetless
`source_evidence` for parser-unrepresented ranges when `include_source_gaps` is
true. With `--json`, source evidence carries revision, span, etag, and preview, but no address
or guard and cannot authorize a patch. Inclusion is region-granular: separator
whitespace inside a parser-unrepresented region is searched with that region;
standalone boundary regions are not searched. Every search supplies
`max_results`; exceeding that budget returns an error and no partial result.

Every mutation carries a document revision and target guard. The file adapter
accepts regular files only, rechecks the canonical referent, source revision,
and Unix device/inode before
staging and immediately before atomic rename. Permissions are preserved and
no-change patches verify the file without replacing it.

`map --json` emits a tagged `GuardAuthority` because it describes every target kind.
Patch operations use narrower evidence types from `md schema`; construct that
operation-specific target from the mapped address, revision, span, and etag
rather than copying the tagged `guard` object verbatim. Rust callers use the
provided `TryFrom<&TargetSnapshot>` conversions.

On macOS, extended attributes such as Finder tags are copied to the replacement
file. POSIX ACL cloning and hard-link topology are not yet preserved; see
[`docs/follow-ups/file-metadata.md`](docs/follow-ups/file-metadata.md).

CLI measurement is opt-in with `MD_USAGE_LOG=/path/to/usage.jsonl`. It records
metadata and bytes accepted by stdout/stderr writers plus `o200k_base` counts from
`tiktoken-rs/0.12.0`. Example:

```sh
MD_USAGE_LOG=/tmp/md-usage.jsonl md read README.md --address '{"kind":"preamble"}'
```

The log contains no document text, raw arguments, stdin, output payloads,
headings, or file paths. `MD_USAGE_CALLER` accepts `codex`, `claude`, `pi`,
`human`, or `benchmark`; otherwise the caller is `unknown`. Optional
`MD_USAGE_SESSION` and `MD_USAGE_RUN` accept UUID-shaped identifiers. These are
explicit environment claims, not verified caller identity. A process ID is
recorded separately and is not treated as a session.

Counts describe each complete captured output stream, not model billing or
the agent's full context. At most 16 MiB of each stream is retained in memory
for counting; larger outputs still have byte counts but null token counts with a
reason. Write/flush errors, invalid UTF-8, or tokenizer failures also produce
null counts. Byte counts on a write failure are not proof of delivery. Known
empty streams count as zero. Clap help, version, and argument-validation exits
retain Clap's own printing and are not logged: failed argument parsing provides
no authoritative input-file set to protect from a colliding log sink.

`command_ms` includes command execution, output, and capture; `counting_ms`
measures subsequent tokenization and record preparation, excluding log I/O.
Tokenizer initialization happens only for enabled, nonempty measurements.
Logging failure does not change output or exit status. On Unix, newly created
log directories/files use modes 0700/0600; a symlink, non-regular file, shared
file permissions, or multiple hard links cause the append to be skipped.
Measurement is enabled only on Unix, where file identity checks are available.
The sink must differ from document and protocol inputs and regular stdin,
stdout, and stderr destinations. Input identities are captured before execution
and paths are rechecked after it, including atomic document replacement. An
alias causes logging to be skipped. These checks do not lock out unrelated
processes that rename files concurrently.
If any document or protocol input cannot be resolved to a regular-file identity,
logging is skipped entirely. This includes missing inputs and dangling symlinks;
creating a log file must not create an input's missing referent. Usage records
therefore do not represent all file-open failures.

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
