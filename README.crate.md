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
