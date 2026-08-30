# Git tags are the release boundary, not a registry

**Date:** 2026-08-25
**Status:** accepted 2026-08-25 — landed with tag `v0.1.0`
**Deciders:** Provi (repo owner)

## Context

mdtools now ships a reusable Rust library alongside the `md` binary
(`docs/decisions/2026-08-23-reusable-library-boundary.md`). That makes it a
dependency, and a dependency needs a boundary a consumer in a different
repository can resolve.

Until now the only boundary was a filesystem path. A consumer wrote
`mdtools = { path = "../mdtools" }`, which means its build depends on a sibling
checkout existing, being on the right commit, and not being mid-edit. A clean
clone of such a consumer cannot resolve Cargo at all. The recorded escape from
that — vendoring mdtools' sources into the consumer — was measured and rejected:
preserving the proof boundary meant importing the Python benchmark suite and
probe fixtures too, and it created a second source home for this repository's
code.

So the question was not whether to have a release boundary but which one:
crates.io, or git tags on this repository.

## Decision

**Git tags are the release boundary.** Consumers depend on this repository by
URL and pin a tag:

```toml
mdtools = { git = "https://github.com/pro-vi/mdtools", tag = "v0.1.0", default-features = false }
```

- **Tags are the release artifact.** Every release is an annotated `vMAJOR.MINOR.PATCH`
  tag whose name matches `package.version` in `Cargo.toml` at that commit. `CHANGELOG.md`
  carries one section per tag.
- **Semver applies to the library surface only** — the public items under
  `src/lib.rs`. The `md` CLI's JSON protocol keeps its own `mdtools.v2` schema
  version and is not what the crate version tracks. Pre-1.0, a breaking library
  change bumps the minor.
- **The library is the default consumption mode.** A consumer takes
  `default-features = false` and gets no `clap`, no `walkdir`, and no `md`
  binary. `default = ["cli"]` stays on so `cargo install` and this repository's
  own tests keep working unchanged.
- **The `include` allowlist in `Cargo.toml` stays maintained** even though git
  dependencies ignore it. It is what keeps `cargo package` correct, and
  publishing later must not require rediscovering which files the crate needs.
- **Not published to crates.io.** No name is claimed there and no version is
  uploaded.

## Rationale

**A git tag answers the whole requirement.** The requirement is that a clean
consumer checkout resolves and builds without a sibling directory. A git
dependency on a public repository does exactly that: Cargo clones into
`~/.cargo/git`, and the consumer's lockfile records the resolved commit SHA
whether the manifest names a `tag`, a `branch`, or a `rev`. Builds are
reproducible and no registry is involved. Verified before this decision: a
scratch crate pinning `rev = d4c6b9c` with `default-features = false` compiled
and ran against `model::SourceSpan`, `model::TaskStatus`, and
`parser::ParsedDocument`.

**crates.io costs more than it currently buys.** Publishing is close to
irreversible — the name is claimed permanently, and a published version can be
yanked but never edited or withdrawn. In exchange it offers docs.rs, `cargo add`,
`cargo install mdtools`, and discoverability by strangers. Every one of those is
worth having *when there are strangers*. With the consumer set at one, in a
repository the same person owns, they buy nothing today and commit the public
library API permanently. A tag commits nothing: switching to a registry later is
one line in the consumer's manifest, and this decision is reversible in the
direction that matters. The reverse — unpublishing — is not.

**The library surface is young.** `src/lib.rs` exposes 21 modules, most of them
extracted three days ago, and `locate` was added yesterday and has no consumer
confirming its shape. Publishing that to a registry invites API expectations
this repository is not ready to hold stable. A tag lets the surface settle
against a real consumer first, which is the same sequencing
`2026-08-25-position-to-target-in-the-library.md` used when it deliberately
withheld a `md locate` CLI command.

**Rejected: a floating branch dependency.** `branch = "master"` also resolves,
and it drifts. The consumer's lockfile pins a SHA, but `cargo update` silently
moves it to whatever master then holds, with no changelog entry and no version
number in the diff. A tag makes the upgrade an explicit, reviewable line change.

## Consequences

Positive:

- A consumer in any repository builds from a clean clone, cold, with no sibling
  checkout and no authentication — this repository is public.
- No permanent public commitment is made while the library surface is settling.
- Releases become reviewable: a tag, a changelog section, and a one-line
  manifest change in the consumer.
- `cargo install --git https://github.com/pro-vi/mdtools --tag v0.1.0` remains
  available for the CLI, so the binary distribution story does not regress.

Negative:

- **`include` does not apply to git dependencies.** Cargo clones the whole
  repository, so a consumer fetches the benchmark corpus it will never use.
  Measured: a 2.3 MB bare clone plus roughly 13 MB per pinned revision checked
  out, against 932 KB for the packaged crate. Small in absolute terms, and it
  grows with each distinct revision a consumer has pinned over time.
- **No docs.rs.** Library documentation is `README.crate.md` and `cargo doc`
  locally. There is no hosted rendered API reference.
- **A consumer with a git dependency can never itself be published to
  crates.io.** crates.io rejects any crate carrying one. This constrains every
  downstream consumer, not just the current one, and it is the sharpest edge of
  this decision.
- **Discovery is manual.** Nobody finds `md` by searching a registry.
- Two version-like identifiers now coexist: the crate version, and the
  `mdtools.v2` JSON schema version. They move independently and the changelog
  has to say which one a change touched.

## Revisit Triggers

- A second consumer appears, particularly one outside this owner's repositories
  → the registry's discoverability and `cargo add` start paying for themselves.
- Any consumer needs to publish itself to crates.io → the git dependency becomes
  a hard blocker and this decision must reverse.
- The library surface stops changing across several releases and a consumer
  depends on its stability → the API commitment publishing implies is no longer
  premature.
- Clone size becomes a measured problem in a consumer's CI → the fix is either
  publishing, or splitting the benchmark corpus out of this repository.

## References

- Boundary this rests on: `docs/decisions/2026-08-23-reusable-library-boundary.md`
- Consumption contract: `README.crate.md`, `src/lib.rs`
- Packaging metadata: `Cargo.toml` (`include`, `[features]`, `[[bin]] md`
  behind `required-features = ["cli"]`)
- Release log: `CHANGELOG.md`
