# Changelog

Releases are git tags on this repository. There is no crates.io package — see
`docs/decisions/2026-08-25-git-tags-are-the-release-boundary.md`.

Consume a release by pinning its tag:

```toml
mdtools = { git = "https://github.com/pro-vi/mdtools", tag = "v0.1.0", default-features = false }
```

Versions follow semver over the **library** surface — the public items under
`src/lib.rs`. The `md` CLI's JSON output carries its own `mdtools.v1` schema
version, which moves independently of the crate version. Before 1.0, a breaking
library change bumps the minor.

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
