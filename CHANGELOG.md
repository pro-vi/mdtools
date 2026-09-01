# Changelog

Releases are git tags on this repository. There is no crates.io package — see
`docs/decisions/2026-08-25-git-tags-are-the-release-boundary.md`.

Consume a release by pinning an existing tag from `git tag --list`:

```toml
mdtools = { git = "https://github.com/pro-vi/mdtools", tag = "<released-tag>", default-features = false }
```

Versions follow semver over the **library** surface — the public items under
`src/lib.rs`. The `md` CLI's generated JSON contract is `mdtools.v3`, and schema
and error-envelope outputs identify that version explicitly. It moves
independently of the crate version. Before 1.0, a breaking library change bumps
the minor.

## v0.4.0 — Unreleased

This is a breaking library and wire release.

- `DocumentIndex` is the sole long-lived source and structural representation;
  parser facts are consumed during construction and never retained.
- A disjoint source-region ledger preserves every byte, including parser-
  unrepresented reference definitions and unreferenced footnotes.
- Search can return targetless `SourceEvidenceRange` values without creating
  mutation authority.
- **Breaking Rust:** `TargetQuery::Search` requires `include_source_gaps` and
  `max_results`; exceeding the budget returns `SearchResultLimitExceeded`
  without partial results.
- **Breaking Rust:** `EvidenceRange` gains required `revision`, and
  `QueryResult` gains the exhaustive `SourceEvidence` variant.
- **Breaking Rust:** public `CoreError` gains `InvalidSourcePosition`,
  `InvalidSourceCoverage`, and `SearchResultLimitExceeded` variants.
- **Breaking Rust:** `ProtocolSchemaVersion::V2` is replaced by `V3`, and the
  exhaustive `DiagnosticCode` enum gains `ResultLimit`.
- **Breaking wire:** the generated contract is `mdtools.v3`, error envelopes
  gain `result_limit`, and no v2 compatibility shim is included.

## v0.3.0 — 2026-08-30

This is a complete breaking replacement of the v0.2 public API and CLI.

- One immutable indexed `Document`, exact `TargetAddress`, typed reads, and
  guarded `Patch` transactions replace the command-specific operation types.
- The former public `parser` module and `ParsedDocument` API were removed; use
  `Document` and its indexed target/read surface.
- Search returns `EvidenceRange` and cannot authorize mutation.
- The feature-gated file adapter verifies revision and file identity before an
  atomic commit.
- The CLI is exactly `map`, `read`, `query`, `patch`, and `schema`.
- The generated wire contract is `mdtools.v2`.

## v0.2.0 — 2026-08-26

Search results now carry the exact-byte evidence needed to verify a hit without
fetching its enclosing block.

### Library

- `search::search` returns each `SearchMatch` with a typed `TargetEtag` over the
  exact original-source bytes covered by `match_span`.
- **Breaking:** `SearchMatch` has a new required public field. This breaks
  downstream Rust struct literals and exhaustive destructuring, so the pre-1.0
  library version moves from `0.1.0` to `0.2.0`.
- `TargetEtag` now serializes to its established lowercase 16-character string.
  String-to-etag conversion still runs through the validating `FromStr`
  implementation; no unchecked deserialization path was added.

### CLI

- `md search --json` and multi-file JSONL output include the library etag on
  every match. The token covers `match_span`, not the lossy preview or the
  enclosing block, and byte-identical matches share a token even at distinct
  spans.
- `md schema --json` advertises the additive field through the append-only
  `search_match_etag` capability.
- The JSON schema remains `mdtools.v1`, and plain tab-separated search output is
  unchanged.

### Decisions recorded in this release

- `docs/decisions/2026-08-26-search-match-etag-boundary.md` — why search keeps
  one shared library/JSON record and fingerprints only the exact matched bytes.


## v0.1.0 — 2026-08-25

First tagged release. It marks the point where mdtools became consumable from
another repository rather than only from a sibling checkout; the `md` CLI and
its behaviour long predate it.

### Library

- `src/lib.rs` exposes the source-in / source-out Markdown surface: `document`,
  `block`, `block_edit`, `section`, `section_edit`, `task`, `table`,
  `frontmatter`, `link`, `search`, `stats`, `locate`, `parser`, `model`,
  `edit`, `fingerprint`, `revision`, `errors`, `core_error`.
- `Document` is the immutable source snapshot operations read from. Its source,
  spans, parsed structure, and `DocumentRevision` cannot drift apart.
- Edit candidates carry new source text and perform no I/O. A persistence owner
  compares the candidate's `base_revision` against the current source
  immediately before replacing the file.
- `locate` and `locate_line` resolve a byte offset or line to the block,
  section, task item, and table row containing it, each with its own etag.
  Library-only; there is no `md locate` command.

### Packaging

- The crate builds as a library without CLI dependencies:
  `default-features = false` drops `clap` and `walkdir` and omits the `md`
  binary. `default = ["cli"]` remains on, so `cargo install` is unaffected.
- `Cargo.toml` carries the metadata a package needs — `description`, `readme`,
  `license`, `repository` — and an `include` allowlist that keeps
  `cargo package` to 120 files. Git dependencies ignore `include`; it is
  maintained so packaging stays correct.

### Decisions recorded in this release

- `docs/decisions/2026-08-23-reusable-library-boundary.md` — what belongs to the
  library and what stays in the binary.
- `docs/decisions/2026-08-25-position-to-target-in-the-library.md` — position
  resolution lives in the library.
- `docs/decisions/2026-08-25-git-tags-are-the-release-boundary.md` — tags, not a
  registry.
