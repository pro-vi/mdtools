# CLI tokenizer dependency audit

The CLI adds `tiktoken-rs = "=0.12.0"` as an optional dependency activated only
by the `cli` feature. Core and `file`-only consumers do not acquire it.
No existing locked package version changed. The lockfile adds the tokenizer
and seven transitive packages: anyhow, base64, bit-set, bit-vec, bstr,
fancy-regex, and lazy_static.

No tokenizer existed in this repository, the Pi adapter's installed dependency
tree, or the local Rust registry cache before this addition. The standard
library does not implement the required encoding; a handwritten tokenizer or
per-command Python subprocess would add a separate implementation or runtime
dependency. The pinned Rust crate supplies the embedded `o200k_base` vocabulary.

The [0.12.0 release notes](https://github.com/zurawiki/tiktoken-rs/releases/tag/v0.12.0)
describe the update to the upstream 0.13.0 core and the Rust 1.85 minimum.
The local compiler is Rust 1.94.1. The lower-level `encode`, `encode_as`, and
`count` APIs are fallible in this release. No old tokenizer call sites require
migration because this is a new dependency.

The implementation uses the
[fallible count API](https://docs.rs/tiktoken-rs/0.12.0/tiktoken_rs/struct.CoreBPE.html#method.count)
with an empty allowed-special-token set. Initialization and counting failures
produce unavailable counts, not zero. Source inspection of the downloaded
crate confirms that `o200k_base()` uses `include_str!` for its vocabulary,
so enabled measurements require no runtime asset download.

Validation obligations: core feature checks, exact stream tests, ordinary
special-looking text, cross-write token boundaries, unavailable-count cases,
write failures, and enabled-versus-disabled release timings. The task's final
verification records the results; this dependency audit does not predict them.
