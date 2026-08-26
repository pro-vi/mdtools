# mdtools

`mdtools` provides the `md` structural Markdown CLI and a reusable Rust library.

Install the CLI:

```sh
cargo install mdtools
```

Use the library without CLI-only dependencies:

```toml
[dependencies]
mdtools = { version = "0.1", default-features = false }
```

The library parses exact Markdown source, exposes source-backed structural
queries, and produces pure edit candidates. Filesystem persistence and the
versioned JSON process protocol belong to the `md` binary.

Repository: <https://github.com/pro-vi/mdtools>

## Library surface

`Document` is the immutable source snapshot used by operations. Its source,
spans, parsed structure, and `DocumentRevision` cannot drift independently.

The library exposes:

- block, section, task, link, frontmatter, table, search, and statistics reads;
- search matches carry a typed `TargetEtag` for the exact original-source bytes
  covered by `match_span`. The etag does not cover the lossy preview or identify
  one occurrence among byte-identical matches; a consumer addresses a hit by
  file plus span;
- position-to-target resolution: `locate` and `locate_line` turn a byte offset
  or a line into the block, section, task item, and table row containing it,
  each with the etag its own read path produces. The block, task, and table-row
  records drive their guarded mutations directly; a section edit still needs one
  `SectionIndex::resolve` round-trip, for which the located entry carries its
  heading's occurrence. A position between blocks is `Ok` with `block: None`,
  not an error; only a byte offset outside the document errors. Library-only —
  there is no `md locate` command;
- block, section, task, frontmatter, and table edit candidates;
- block and section relocation;
- validated `SectionTarget` selectors, document-bound `ResolvedSection` handles,
  and distinct `TargetEtag` fingerprints;
- prepared payload edits, so guards can be checked before an adapter reads a
  file or stdin.

Edit candidates contain new source text but perform no I/O. A persistence owner
must compare the candidate's `base_revision` with the current source immediately
before replacing the file. The `md` binary binds that verification to the same
filesystem object it atomically replaces.

```rust
use mdtools::block_edit;
use mdtools::document::Document;

let document = Document::parse("# Title\n\nold\n")?;
let prepared = block_edit::prepare_replace(&document, 1, None)?;
let candidate = prepared.apply("new\n");
assert_eq!(candidate.content, "# Title\n\nnew\n");
# Ok::<(), mdtools::core_error::CoreError>(())
```
