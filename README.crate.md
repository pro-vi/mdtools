# mdtools

`mdtools` is a source-preserving Markdown library with one immutable
`DocumentIndex`, exact `TargetAddress` values, typed reads, and guarded `Patch`
transactions. The index retains a disjoint lexical ledger so every source byte
can be preserved and inspected without turning parser omissions into mutation
authority.

```rust
use mdtools::document::Document;
use mdtools::target::{TargetAddress, TargetKind};

let document = Document::parse("# Work\n\nbody\n")?;
let section = document
    .map()?
    .into_iter()
    .find(|target| target.kind == TargetKind::Section)
    .unwrap();
let read = document.resolve(&section.address)?.read_section(&document)?;
assert_eq!(read.markdown, "# Work\n\nbody\n");
assert!(matches!(section.address, TargetAddress::Section { .. }));
# Ok::<(), mdtools::core_error::CoreError>(())
```

Core operations perform no filesystem I/O. The optional `file` feature adds a
verified atomic commit adapter. The `cli` feature adds the five-command `md`
binary. JSON Schema and CLI metadata derive from the Rust protocol types under
`mdtools.v3`. Search can return targetless `SourceEvidenceRange` values and
requires an explicit `max_results` budget.
